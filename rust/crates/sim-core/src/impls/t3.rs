//! Python's `cards/impl/t3.py`: the Tier 3 pool decks' cards (Korvold, Marwyn, Atraxa, Aurelia, Tergrid), and the
//! planeswalkers whose abilities t3.py lists (with the ultimates rules.py appends and the abilities rules.py and
//! rules2.py replace: each walker's final list is registered here, as Python's `WALKERS` holds it once every module
//! has loaded).
//!
//! Hooks that a later module replaces for the same card and event aren't ported here: Esika's Chariot (rules2),
//! Joraga Treespeaker's options and mana (rules2), Legion Loyalist's attack (rules), Painful Quandary's cast trigger
//! (rules), Finale of Devastation (rules2), Embercleave's cast priority (rules2: 0).

use super::common::{WalkerAb, best_opp_creature, best_opp_nonland, death_value, outlets, sac_food, walker};
use super::partials::{at_once, eot_kw, first_max, name_of, remove_card};
use crate::cards::{CardDb, Colors, Types};
use crate::engine::cast::{discard_worst, on_cast};
use crate::engine::hooks::fire_trigger;
use crate::engine::life::{gain, lose_life};
use crate::engine::mana::{can_pay, pay, total_mana};
use crate::engine::removal::apply_removal;
use crate::engine::stack::{ability_window, ability_window_card, counter_window, trigger_window};
use crate::engine::tutors::{card_worth, shuffle_library};
use crate::engine::values::{epow, etgh, has_type, once_per_turn, pval, threat, untargetable};
use crate::engine::zones::{
    Enter, Tokens, die, discard_cards, discard_index, draw, edict, enter, enter_token_copy, exile_perm,
    make_artifact_tokens, make_tokens, max_by, min_by, searchable,
};
use crate::flow::Res;
use crate::hooks::{Action, Assign, Call, Event, Opt, Registry, Sacrificed, Src};
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::pysum::PySum;
use crate::state::{Ctx, DataKey, Game, PermData, TurnStamp, Val};
use crate::sym::{Sym, intern};

// ------------------------------------------------------------------ helpers (shared with rules.rs and rules2.rs)
pub(crate) fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

/// t3._fresh: once_per_turn(key) would pass, without using it up (checked before a trigger window)
pub(crate) fn fresh(g: &Game, p: PlayerId, key: Sym) -> bool {
    g.player(p).flag_turn.get(key) != Some(&g.turn_stamp())
}

/// cardimpl._eot: +dp/+dt until end of turn
pub(crate) fn eot(g: &mut Game, m: PermId, dp: i32, dt: i32) {
    let x = &mut g.perm_mut(m).eot_pt;
    *x = (x.0 + dp, x.1 + dt);
}

/// `m in p.perms`
pub(crate) fn controls(g: &Game, p: PlayerId, m: PermId) -> bool {
    g.perm(m).on_bf && g.perm(m).owner == p
}

/// p's creatures (phased ones too, as Python's `m.creature for m in p.perms`)
pub(crate) fn creatures(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.is_creature(m)).collect()
}

/// p's creatures that aren't phased out
pub(crate) fn live_creatures(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.is_creature(m) && !g.perm(m).phased).collect()
}

/// the opponents' creatures (phased ones too)
pub(crate) fn opp_creatures(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.opps(p).flat_map(|q| g.player(q).perms.iter().copied()).filter(|&m| g.is_creature(m)).collect()
}

/// the card's types include t (Python's `m.cd is not None and 'A' in m.cd.types`)
pub(crate) fn card_is(g: &Game, m: PermId, t: Types) -> bool {
    g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(t))
}

/// a spare body: a token or a creature worth little (pval below `under`)
pub(crate) fn spare(g: &Game, m: PermId, under: f64) -> bool {
    g.perm(m).token || pval(g, m) < under
}

/// a stable sort, highest key first (Python's `sorted(xs, key=lambda m: -v(m))`)
pub(crate) fn sort_desc<T: Copy>(xs: &mut [T], key: impl Fn(T) -> f64) {
    xs.sort_by(|&a, &b| key(b).partial_cmp(&key(a)).unwrap_or(std::cmp::Ordering::Equal));
}

/// a stable sort, lowest key first (Python's `sorted(xs, key=v)`)
pub(crate) fn sort_asc<T: Copy>(xs: &mut [T], key: impl Fn(T) -> f64) {
    xs.sort_by(|&a, &b| key(a).partial_cmp(&key(b)).unwrap_or(std::cmp::Ordering::Equal));
}

/// make_tokens(g, p, n, pw, tg, color=.., types=..)
pub(crate) fn tokens(
    g: &mut Game,
    p: PlayerId,
    n: u32,
    pw: i32,
    tg: i32,
    color: &str,
    types: &[Sym],
) -> Res<Vec<PermId>> {
    let spec =
        Tokens { tgh: Some(tg), color: Some(Colors::from_letters(color)), types: types.to_vec(), ..Tokens::new(n, pw) };
    make_tokens(g, p, spec)
}

/// an option of src's activated ability
pub(crate) fn ability(utility: f64, label: String, src: PermId, f: crate::hooks::AbilityFn, arg: i64) -> Opt {
    Opt { utility, label, act: Some(Action::Ability { src, f, arg }) }
}

/// a play from hand, graveyard or a land (its function and argument)
pub(crate) fn plan(utility: f64, label: String, f: crate::hooks::PlanFn, arg: i64) -> Opt {
    Opt { utility, label, act: Some(Action::Plan { f, arg }) }
}

/// the opponent p threatens most (Python's `max(g.opps(p), key=lambda q: threat(g, p, q))`)
pub(crate) fn top_threat(g: &Game, p: PlayerId) -> Option<PlayerId> {
    let opps: Vec<PlayerId> = g.opps(p).collect();
    max_by(&opps, |q| threat(g, p, q))
}

/// t2.best_target_any: 'deals n damage to any target': kill the best creature it kills, else the most threatening
/// opponent's face. A copy until t2.rs has it (Mayhem Devil and Ugin's +2 are t3's).
pub(crate) fn best_target_any(g: &mut Game, p: PlayerId, n: i32) -> Res {
    let opps: Vec<PlayerId> = g.opps(p).collect();
    if opps.is_empty() {
        return Ok(());
    }
    // HUMAN(phase 9): a person picks the target (hc.deal_damage)
    if let Some(&q) = opps.iter().find(|&&q| g.player(q).life <= n) {
        return lose_life(g, q, n, Some(p), "triggers", None);
    }
    let tg: Vec<PermId> = opps
        .iter()
        .flat_map(|&q| g.player(q).perms.iter().copied())
        .filter(|&m| g.is_creature(m) && !untargetable(g, m) && etgh(g, m) <= n)
        .collect();
    match max_by(&tg, |m| pval(g, m)) {
        Some(best) if pval(g, best) >= 3.0 => apply_removal(g, Some(p), best, &format!("dmg{n}"), None),
        _ => {
            let q = max_by(&opps, |q| threat(g, p, q)).unwrap();
            lose_life(g, q, n, Some(p), "triggers", None)
        }
    }
}

// ======================================================== Korvold, Fae-Cursed King (sacrifice value)
/// t3.sac_worst_permanent: sacrifice the least valuable permanent (Food / Clue / Treasure first); returns what was
/// sacrificed
pub fn sac_worst_permanent(g: &mut Game, p: PlayerId, exclude: Option<PermId>) -> Res<Option<Sacrificed>> {
    if g.player(p).foods > 0 {
        sac_food(g, p, 1)?;
        return Ok(Some(Sacrificed::Token("Food")));
    }
    for (kind, have) in [("Clue", g.player(p).clues), ("Treasure", g.player(p).treasures)] {
        if have > 0 {
            let pl = g.player_mut(p);
            if kind == "Clue" {
                pl.clues -= 1;
            } else {
                pl.treasures -= 1;
            }
            if !g.hooks.is_empty() {
                fire_trigger(g, Event::Sacrifice, Call::Sacrifice { p, what: Sacrificed::Token(kind) })?;
            }
            return Ok(Some(Sacrificed::Token(kind)));
        }
    }
    let cands: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| Some(m) != exclude && !g.perm(m).is_cmd && !g.perm(m).phased)
        .collect();
    if let Some(m) = min_by(&cands, |m| pval(g, m)) {
        die(g, m, "sac")?;
        return Ok(Some(Sacrificed::Perm(m)));
    }
    let lands = g.player(p).lands.clone();
    let colours = |l: LandId| g.db.get(g.land(l).cd).tags.str(crate::tag::Tag::C).map_or(0, |s| s.len());
    if let Some(l) = first_max(&lands, |l| (g.land(l).tapped, std::cmp::Reverse(colours(l)))) {
        // Python: min(p.lands, key=lambda L: (not L.tapped, len(L.cd.tags.get('c', ''))))
        crate::engine::turn::remove_land(g, p, l);
        let cd = g.land(l).cd;
        g.player_mut(p).gy.push(cd);
        if !g.hooks.is_empty() {
            fire_trigger(g, Event::Sacrifice, Call::Sacrifice { p, what: Sacrificed::Card(cd) })?;
            fire_trigger(g, Event::LandGy, Call::Cards { p, cards: vec![cd] })?;
        }
        return Ok(Some(Sacrificed::Card(cd)));
    }
    Ok(None)
}

fn korvold_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m == src && trigger_window(g, o, Some(src), "sacrifice another permanent", None)? {
        sac_worst_permanent(g, o, Some(src))?;
    }
    Ok(())
}

fn korvold_atk(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if atk.contains(&src) && trigger_window(g, p, Some(src), "sacrifice another permanent", None)? {
        sac_worst_permanent(g, p, Some(src))?;
    }
    Ok(vec![])
}

/// enters/attacks: sacrifice the least valuable permanent; every sacrifice (Treasure / Food / Clue spending
/// included): +1/+1 counter and a card
fn korvold_sac(g: &mut Game, src: Src, p: PlayerId, what: Sacrificed) -> Res {
    if p == g.perm(src).owner && what != Sacrificed::Perm(src) {
        if !trigger_window(g, p, Some(src), "a +1/+1 counter and draw a card", None)? {
            return Ok(());
        }
        g.perm_mut(src).plus += 1;
        if g.player(p).library.len() > 10 {
            draw(g, p, 1, false)?;
        }
    }
    Ok(())
}

/// any player's sacrifice (Treasures included): 1 damage
fn mayhem(g: &mut Game, src: Src, _p: PlayerId, _what: Sacrificed) -> Res {
    let o = g.perm(src).owner;
    if trigger_window(g, o, Some(src), "1 damage to any target", None)? {
        best_target_any(g, o, 1)?;
    }
    Ok(())
}

/// extra Squirrels for artifact tokens made by hooks (a token loop, Squirrels -> Treasures, stops at depth 10)
fn chatter_art(g: &mut Game, src: Src, p: PlayerId, _kinds: &[Sym], n: i32) -> Res {
    if p != g.perm(src).owner || g.chatter_depth >= 10 {
        return Ok(());
    }
    g.chatter_depth += 1;
    let r = tokens(g, p, n.max(0) as u32, 1, 1, "G", &["squirrel"]);
    g.chatter_depth -= 1;
    r.map(|_| ())
}

fn goose(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m == src && trigger_window(g, o, Some(src), "create a Food", Some(2.0))? {
        make_artifact_tokens(g, o, "Food", 1)?;
    }
    Ok(())
}

/// Gilded Goose taps (and sacrifices a Food) for mana
fn goose_mana(g: &Game, p: PlayerId, _m: PermId) -> u32 {
    (g.player(p).foods > 0) as u32
}

fn goose_tap(g: &mut Game, p: PlayerId, _m: PermId, _used: u32) -> Res {
    sac_food(g, p, 1).map(|_| ())
}

fn trail(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m == src && trigger_window(g, o, Some(src), "create a Food", Some(2.0))? {
        make_artifact_tokens(g, o, "Food", 1)?;
    }
    Ok(())
}

/// a Food sacrificed: pay {1} to look at the top two, a permanent card to hand, the rest to the bottom
fn trail_sac(g: &mut Game, src: Src, p: PlayerId, what: Sacrificed) -> Res {
    if p != g.perm(src).owner
        || what != Sacrificed::Token("Food")
        || !can_pay(g, p, 1, "", false)
        || g.player(p).library.len() <= 5
    {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "pay 1: look at the top two", Some(2.0))? || !can_pay(g, p, 1, "", false) {
        return Ok(());
    }
    pay(g, p, 1, "", false)?;
    let mut top = vec![];
    for _ in 0..g.player(p).library.len().min(2) {
        top.push(g.player_mut(p).library.pop().unwrap());
    }
    let perm: Vec<CardId> = top.iter().copied().filter(|&c| g.db.get(c).perm || g.db.get(c).land).collect();
    if let Some(c) = max_by(&perm, |c| card_worth(g, p, c, false)) {
        remove_card(&mut top, c);
        g.player_mut(p).hand.push(c);
    }
    g.player_mut(p).library.splice(0..0, top);
    Ok(())
}

/// sacrifice spare creatures for Food when the death is worth it
fn oven(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if g.perm(src).tapped || post.is_none() {
        return Ok(vec![]);
    }
    let fod: Vec<PermId> = creatures(g, p).into_iter().filter(|&m| !g.perm(m).is_cmd && spare(g, m, 2.0)).collect();
    let Some(m) = min_by(&fod, |x| pval(g, x)) else { return Ok(vec![]) };
    let v = death_value(g, p, None) + 1.0 - 1.2 * pval(g, m);
    if v < 1.0 {
        return Ok(vec![]);
    }
    Ok(vec![ability(v, format!("Witch's Oven ({})", g.perm(m).name), src, oven_go, m.0 as i64)])
}

fn oven_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let m = PermId(arg as u32);
    if g.perm(src).tapped || !controls(g, p, m) {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    let n = if etgh(g, m) >= 4 { 2 } else { 1 };
    die(g, m, "sac")?;
    if ability_window(g, p, Some(src), &format!("{n} Food"), None, None)? {
        make_artifact_tokens(g, p, "Food", n)?;
    }
    Ok(true)
}

fn savvy(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if atk.contains(&src) && trigger_window(g, p, Some(src), "create a Food", Some(2.0))? {
        make_artifact_tokens(g, p, "Food", 1)?;
    }
    Ok(vec![])
}

fn savvy_draw(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if g.player(p).foods < 2 || post.is_none() {
        return Ok(vec![]);
    }
    Ok(vec![ability(2.0, "Savvy Hunter: two Foods for a card".into(), src, savvy_go, 0)])
}

fn savvy_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if g.player(p).foods < 2 {
        return Ok(false);
    }
    sac_food(g, p, 2)?;
    if ability_window(g, p, Some(src), "draw a card", None, None)? {
        draw(g, p, 1, false)?;
    }
    Ok(true)
}

/// two Treasures per player hit; the -X/-X ability is not used
fn hireling(g: &mut Game, src: Src, _p: PlayerId, a: PermId, d: PlayerId, _dmg: i32) -> Res {
    let o = g.perm(src).owner;
    let key = intern(&format!("hire{}_{}", src.0, d.0));
    if g.perm(a).owner == o && fresh(g, o, key) {
        if !trigger_window(g, o, Some(src), "create two Treasures", None)? || !once_per_turn(g, o, key) {
            return Ok(());
        }
        make_artifact_tokens(g, o, "Treasure", 2)?;
    }
    Ok(())
}

fn gnawbone(g: &mut Game, src: Src, _p: PlayerId, a: PermId, _d: PlayerId, dmg: i32) -> Res {
    let o = g.perm(src).owner;
    if g.perm(a).owner == o && trigger_window(g, o, Some(src), &format!("create {dmg} Treasures"), None)? {
        make_artifact_tokens(g, o, "Treasure", dmg)?;
    }
    Ok(())
}

/// a 0/1 Spawn each upkeep (sacrifice fodder; the mana ability is not used)
fn azone(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p == g.perm(src).owner && trigger_window(g, p, Some(src), "create a 0/1 Eldrazi Spawn", Some(2.0))? {
        tokens(g, p, 1, 0, 1, "", &["eldrazi", "spawn"])?;
    }
    Ok(())
}

/// returns on landfall
fn bloodghast_lf(g: &mut Game, c: CardId, p: PlayerId) -> Res {
    if remove_card(&mut g.player_mut(p).gy, c) {
        enter(g, p, c, Enter::default())?;
    }
    Ok(())
}

/// recurs itself when there is a sacrifice outlet
fn skeleton(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || !can_pay(g, p, 1, "B", false) || outlets(g, p).is_empty() {
        return Ok(vec![]);
    }
    let u = 0.5 + death_value(g, p, None) / 2.0;
    Ok(vec![plan(u, "return Reassembling Skeleton".into(), skeleton_go, c.0 as i64)])
}

fn skeleton_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).gy.contains(&c) || !can_pay(g, p, 1, "B", false) {
        return Ok(false);
    }
    pay(g, p, 1, "B", false)?;
    if ability_window_card(g, p, c, "return to the battlefield", None, None)? && remove_card(&mut g.player_mut(p).gy, c)
    {
        let m = enter(g, p, c, Enter::default())?;
        g.perm_mut(m).tapped = true;
    }
    Ok(true)
}

/// castable from the graveyard with a Zombie; the Phyrexian Altar loop is a combo (see combos)
fn gravecrawler(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || !can_pay(g, p, 0, "B", false) || !g.player(p).perms.iter().any(|&m| has_type(g, m, "zombie")) {
        return Ok(vec![]);
    }
    let altar = g.player(p).perms.iter().any(|&m| g.perm(m).cd.is_some() && name_of(g, m) == "Phyrexian Altar");
    if outlets(g, p).is_empty() && !altar {
        return Ok(vec![]);
    }
    let u = 0.5 + death_value(g, p, None) / 2.0;
    Ok(vec![plan(u, "cast Gravecrawler from the graveyard".into(), gravecrawler_go, c.0 as i64)])
}

fn gravecrawler_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).gy.contains(&c) || !can_pay(g, p, 0, "B", false) {
        return Ok(false);
    }
    remove_card(&mut g.player_mut(p).gy, c);
    pay(g, p, 0, "B", false)?;
    g.player_mut(p).spells_this_turn += 1;
    on_cast(g, p, c)?;
    if counter_window(g, p, c, 3.0, vec![])? {
        enter(g, p, c, Enter::default())?;
    } else {
        g.player_mut(p).gy.push(c);
    }
    Ok(true)
}

// ------------------------------------------------------------------ Grist, the Hunger Tide
fn grist_minus_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let t = best_opp_creature(g, p, |_| true)?;
    let fod = creatures(g, p).into_iter().any(|m| spare(g, m, 2.0));
    (pval(g, t) >= 4.0 && fod).then(|| pval(g, t) - 2.0)
}

/// sacrifice a spare creature: destroy their best creature
fn grist_minus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let fod: Vec<PermId> = creatures(g, p).into_iter().filter(|&m| spare(g, m, 2.0)).collect();
    let t = best_opp_creature(g, p, |_| true);
    if let (Some(f), Some(t)) = (min_by(&fod, |m| pval(g, m)), t) {
        die(g, f, "sac")?;
        apply_removal(g, Some(p), t, "destroy", None)?;
    }
    Ok(())
}

fn gy_creatures(g: &Game, p: PlayerId) -> i32 {
    g.player(p).gy.iter().filter(|&&c| g.db.get(c).creature).count() as i32
}

/// -5: each opponent loses a life per creature card in your graveyard
fn grist_drain(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    for q in g.opps(p).collect::<Vec<_>>() {
        let n = gy_creatures(g, p);
        lose_life(g, q, n, Some(p), "drain", None)?;
    }
    Ok(())
}

/// +1 Insect and mill (repeats on a milled Insect: rules2), -2 sacrifice to destroy, -5 drain per creature card in the
/// graveyard
static GRIST: [WalkerAb; 3] = [
    WalkerAb { delta: 1, label: "Insect and mill", val: |_, _, _| Some(2.5), eff: super::rules2::grist_plus },
    WalkerAb { delta: -2, label: "sacrifice: destroy", val: grist_minus_val, eff: grist_minus },
    WalkerAb {
        delta: -5,
        label: "drain creature cards",
        val: |g, p, _| Some(1.5 * gy_creatures(g, p) as f64 - 3.0),
        eff: grist_drain,
    },
];

// ------------------------------------------------------------------ Fable of the Mirror-Breaker
/// chapter I Goblin (Treasure on attack), II rummage two, III flips into Reflection (copy a nonlegendary creature each
/// turn; Kiki combos in combos)
fn fable(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let mut d = PermData::default();
    d.set(DataKey::Lore, Val::Int(1)); // enters with a lore counter (not the trigger)
    g.perm_mut(src).data = d;
    let o = g.perm(src).owner;
    if !trigger_window(g, o, Some(src), "chapter I: create a 2/2 Goblin Shaman", None)? {
        return Ok(());
    }
    for t in tokens(g, o, 1, 2, 2, "R", &["goblin", "shaman"])? {
        let mut d = PermData::default();
        d.set(DataKey::FableGoblin, Val::Bool(true));
        g.perm_mut(t).data = d;
    }
    Ok(())
}

fn fable_lore(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != g.perm(src).owner || g.perm(src).data.get(DataKey::Lore).is_none() {
        return Ok(());
    }
    let n = g.perm(src).data.int(DataKey::Lore) + 1; // the lore counter is not part of the trigger
    let name = if n == 2 { "chapter II: discard up to two, draw that many" } else { "chapter III: transform" };
    let ok = trigger_window(g, p, Some(src), name, Some(4.0))?;
    g.perm_mut(src).data.set(DataKey::Lore, Val::Int(n));
    if !ok {
        if n >= 3 && controls(g, p, src) {
            die(g, src, "sac")?; // final chapter gone: the Saga is sacrificed
        }
        return Ok(());
    }
    let lore = g.perm(src).data.int(DataKey::Lore);
    if lore == 2 {
        let k = g.player(p).hand.len().min(2);
        let mut hand = g.player(p).hand.clone();
        sort_asc(&mut hand, |c| card_worth(g, p, c, false));
        let worst: Vec<CardId> = hand.into_iter().take(k).collect();
        if !worst.is_empty() {
            discard_cards(g, p, &worst)?;
            draw(g, p, worst.len() as u32, false)?;
        }
    } else if lore >= 3 {
        let turns = g.player(p).turns as i64;
        let mut d = PermData::default();
        d.set(DataKey::Reflection, Val::Bool(true));
        d.set(DataKey::Flipped, Val::Int(turns));
        g.perm_mut(src).data = d;
        crate::glog!(g, "    Fable flips into Reflection of Kiki-Jiki");
    }
    Ok(())
}

fn fable_goblin(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if g.perm(src).owner == p {
        let k = atk.iter().filter(|&&m| g.perm(m).data.truthy(DataKey::FableGoblin)).count() as i32;
        if k > 0 && trigger_window(g, p, Some(src), &format!("create {k} Treasure"), None)? {
            make_artifact_tokens(g, p, "Treasure", k)?;
        }
    }
    Ok(vec![])
}

// ======================================================== Marwyn, the Nurturer (Elves, Craterhoof)
/// +1/+1 counter per Elf; taps for G equal to its power
fn marwyn(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m != src
        && g.perm(m).owner == o
        && has_type(g, m, "elf")
        && trigger_window(g, o, Some(src), "a +1/+1 counter", Some(1.0))?
    {
        g.perm_mut(src).plus += 1;
    }
    Ok(())
}

/// haste; creatures +X/+X and trample (X = creatures you control)
fn hoof(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = g.perm(src).owner;
    if !trigger_window(g, o, Some(src), "creatures get +X/+X and trample", Some(8.0))? {
        return Ok(());
    }
    let x = live_creatures(g, o).len() as i32;
    for c in creatures(g, o) {
        eot(g, c, x, x);
        eot_kw(g, c, "trample");
    }
    g.player_mut(o).trample = true;
    crate::glog!(g, "    Craterhoof: creatures get +{x}/+{x} and trample");
    Ok(())
}

/// t3.hoof_prio: cast Craterhoof (or tutor into it) only when the board makes it lethal-ish; before combat
fn hoof_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let n = live_creatures(g, p).len();
    if n >= 6 {
        88
    } else if n >= 4 {
        30
    } else {
        0
    }
}

/// t3._put_creature: a creature card from the library onto the battlefield: the deck's wish list first, Craterhoof
/// with a wide board, else the most useful
pub fn put_creature(
    g: &mut Game,
    p: PlayerId,
    pred: impl Fn(&Game, CardId) -> bool,
    prefer_hoof: bool,
) -> Res<Option<CardId>> {
    let cs: Vec<CardId> = searchable(g, p).into_iter().filter(|&c| g.db.get(c).creature && pred(g, c)).collect();
    if cs.is_empty() {
        return Ok(None);
    }
    let wish = crate::ai::plans::wish_list(g, p);
    let c = match cs.iter().copied().find(|c| wish.contains(c)) {
        Some(c) => c,
        None => {
            let hoof = cs.iter().copied().find(|&c| &*g.db.get(c).name == "Craterhoof Behemoth");
            let n = live_creatures(g, p).len();
            match hoof {
                Some(h) if prefer_hoof && n >= 5 => h,
                _ => first_max(&cs, |c| (card_worth(g, p, c, false), g.db.get(c).cmc)).unwrap(),
            }
        }
    };
    remove_card(&mut g.player_mut(p).library, c);
    shuffle_library(g, p);
    g.player_mut(p).stat("tutored", 1);
    enter(g, p, c, Enter::default())?;
    Ok(Some(c))
}

/// t3.natural_order_prio: Natural Order sacrifices a green creature first: Craterhoof only when the board is still
/// wide after that (five or more creatures besides the one sacrificed), otherwise only a spare body (a token or a
/// cheap creature) for a real threat (a bomb or the deck's wish list); never a real creature for a sidegrade
fn natural_order_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let green: Vec<PermId> = live_creatures(g, p)
        .into_iter()
        .filter(|&m| g.perm(m).cd.is_some_and(|c| g.db.get(c).pips.contains('G')))
        .collect();
    if green.is_empty() {
        return 0;
    }
    let n = live_creatures(g, p).len();
    let lib: Vec<CardId> = g
        .player(p)
        .library
        .iter()
        .copied()
        .filter(|&x| g.db.get(x).creature && g.db.get(x).pips.contains('G'))
        .collect();
    if n >= 6 && lib.iter().any(|&x| &*g.db.get(x).name == "Craterhoof Behemoth") {
        return 85;
    }
    if !green.iter().any(|&m| spare(g, m, 2.0)) {
        return 0;
    }
    let wish = crate::ai::plans::wish_list(g, p);
    let best = lib.iter().map(|&x| if wish.contains(&x) { 6 } else { g.db.get(x).bomb as i64 }).max().unwrap_or(0);
    if best >= 5 { 55 } else { 0 }
}

/// sacrifices a spare green creature for the best green creature (Craterhoof on a wide board)
fn natural_order(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    put_creature(g, p, |g, x| g.db.get(x).pips.contains('G'), true)?;
    Ok("gy")
}

/// t3._x_tutor's cast priority: with 3 + (its coloured pips) mana or more
fn x_tutor_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    if total_mana(g, p, false) >= 3 + g.db.get(c).pips.len() as u32 { 60 } else { 0 }
}

/// Green Sun's Zenith: X = spare mana; a green creature with mana value X or less onto the battlefield
fn gsz(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let x = ctx.x;
    put_creature(g, p, |g, y| g.db.get(y).cmc as i32 <= x && g.db.get(y).pips.contains('G'), true)?;
    Ok("gy")
}

/// Chord of Calling ({X}{G}{G}{G}, convoke: t3._x_tutor): X = spare mana and creatures; a creature with mana value X
/// or less onto the battlefield
fn chord(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let x = ctx.x;
    put_creature(g, p, |g, y| (g.db.get(y).cmc as i32) <= x, true)?;
    Ok("gy")
}

fn eldritch_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if creatures(g, p).iter().any(|&m| !g.perm(m).is_cmd) { 55 } else { 0 }
}

/// sacrifice the lowest creature, fetch one with mana value up to two more onto the battlefield
fn eldritch(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let mv = g.sac_snapshot.map_or(1, |s| s.0);
    put_creature(g, p, |g, x| g.db.get(x).cmc <= mv + 2, false)?;
    g.sac_snapshot = None;
    Ok("exile")
}

/// fights on entry (as 7 damage)
fn kogla(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = g.perm(src).owner;
    if best_opp_creature(g, o, |x| etgh(g, x) <= 7).is_none() {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "fight a creature", Some(5.0))? {
        return Ok(());
    }
    if let Some(t) = best_opp_creature(g, o, |x| etgh(g, x) <= 7) {
        apply_removal(g, Some(o), t, "dmg7", None)?;
    }
    Ok(())
}

/// destroys an artifact or enchantment on attack
fn kogla_atk(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], d: PlayerId) -> Res<Vec<PermId>> {
    if !atk.contains(&src) {
        return Ok(vec![]);
    }
    let ok = |g: &Game, m: PermId| {
        (card_is(g, m, Types::ARTIFACT) || card_is(g, m, Types::ENCHANTMENT)) && !untargetable(g, m)
    };
    if !g.player(d).perms.iter().any(|&m| ok(g, m))
        || !trigger_window(g, p, Some(src), "destroy an artifact or enchantment", Some(5.0))?
    {
        return Ok(vec![]);
    }
    let ts: Vec<PermId> = g.player(d).perms.iter().copied().filter(|&m| ok(g, m)).collect();
    if let Some(t) = max_by(&ts, |m| pval(g, m)) {
        apply_removal(g, Some(p), t, "destroy", None)?;
    }
    Ok(vec![])
}

/// X pump and fight: used as removal (its resolve does nothing more)
fn primal(_g: &mut Game, _p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    Ok("gy")
}

fn zero_prio(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    0
}

// ------------------------------------------------------------------ Freyalise, Nissa
fn ench_or_art(g: &Game, m: PermId) -> bool {
    card_is(g, m, Types::ARTIFACT) || card_is(g, m, Types::ENCHANTMENT)
}

fn freyalise_minus_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let t = best_opp_nonland(g, p, |m| ench_or_art(g, m))?;
    (pval(g, t) >= 4.0).then(|| pval(g, t) - 2.0)
}

fn freyalise_minus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    if let Some(t) = best_opp_nonland(g, p, |m| ench_or_art(g, m)) {
        apply_removal(g, Some(p), t, "destroy", None)?;
    }
    Ok(())
}

/// +2 two 1/1 Elf Druids that tap for {G} (rules2); -2 destroy an artifact or enchantment; -6 draw per green creature
static FREYALISE: [WalkerAb; 3] = [
    WalkerAb { delta: 2, label: "two Elf Druids", val: |_, _, _| Some(3.0), eff: super::rules2::freyalise_plus },
    WalkerAb { delta: -2, label: "destroy artifact/enchantment", val: freyalise_minus_val, eff: freyalise_minus },
    WalkerAb {
        delta: -6,
        label: "draw per green creature",
        val: |g, p, _| Some(0.8 * creatures(g, p).len() as f64),
        eff: |g, p, _| {
            let n = creatures(g, p).len() as u32;
            draw(g, p, n, false)
        },
    },
];

/// Forests tap for an extra G
fn nissa_wstw(g: &Game, src: Src, p: PlayerId, l: LandId) -> i32 {
    let d = g.db.get(g.land(l).cd);
    (p == g.perm(src).owner && (d.has_subtype("forest") || &*d.name == "Forest")) as i32
}

fn nissa_plus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let spec = Tokens {
        tgh: Some(3),
        sick: false,
        color: Some(Colors::from_letters("G")),
        types: vec!["elemental"],
        ..Tokens::new(1, 3)
    };
    make_tokens(g, p, spec).map(|_| ())
}

/// +1 makes a 3/3 haste (as a token, not a land); -8 every Forest from the library (rules)
static NISSA: [WalkerAb; 2] = [
    WalkerAb { delta: 1, label: "land becomes a 3/3 haste", val: |_, _, _| Some(3.0), eff: nissa_plus },
    super::rules::NISSA_ULT,
];

// ======================================================== Aurelia (extra combats)
/// exerts (not two turns running): untap the others, additional combat
fn celebrant(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    let turns = g.player(p).turns as i64;
    let exerted = g.perm(src).data.get(DataKey::Exerted).cloned();
    if !atk.contains(&src) || exerted.is_some_and(|v| v == Val::Int(turns) || v == Val::Int(turns - 1)) {
        return Ok(vec![]);
    }
    let ok = trigger_window(g, p, Some(src), "untap creatures; an additional combat", Some(6.0))?;
    g.perm_mut(src).data.set(DataKey::Exerted, Val::Int(turns)); // exerted as it attacked
    if !ok {
        return Ok(vec![]);
    }
    for m in creatures(g, p) {
        if m != src {
            g.perm_mut(m).tapped = false;
        }
    }
    g.player_mut(p).extra_combats += 1;
    crate::glog!(g, "    Combat Celebrant exerts: untap, additional combat");
    Ok(vec![])
}

/// pays {5}{R}{R} for another combat when the attack is big enough
fn charger(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    let key = intern(&format!("charger{}_{}", src.0, g.player(p).combat_no));
    let power: i32 = atk.iter().map(|&m| epow(g, m)).sum();
    if atk.contains(&src) && can_pay(g, p, 5, "RR", false) && power >= 8 && fresh(g, p, key) {
        if !trigger_window(g, p, Some(src), "pay 5RR: untap attackers, an additional combat", Some(6.0))? {
            return Ok(vec![]);
        }
        if !can_pay(g, p, 5, "RR", false) || !once_per_turn(g, p, key) {
            return Ok(vec![]);
        }
        pay(g, p, 5, "RR", false)?;
        for &m in atk {
            g.perm_mut(m).tapped = false;
        }
        g.player_mut(p).extra_combats += 1;
    }
    Ok(vec![])
}

/// combat damage: untap all, additional combat (once per player hit)
fn razer(g: &mut Game, src: Src, p: PlayerId, a: PermId, d: PlayerId, _dmg: i32) -> Res {
    let key = intern(&format!("razer{}_{}", src.0, d.0));
    if a == src && fresh(g, p, key) {
        if !trigger_window(g, p, Some(src), "untap creatures; an additional combat", Some(6.0))?
            || !once_per_turn(g, p, key)
        {
            return Ok(());
        }
        for m in creatures(g, p) {
            g.perm_mut(m).tapped = false;
        }
        g.player_mut(p).extra_combats += 1;
    }
    Ok(())
}

/// t3._combat_spell's cast priority: before the first combat, with two or more ready attackers
fn combat_spell_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let ready = creatures(g, p).iter().filter(|&&m| !g.perm(m).sick && !g.perm(m).noatk).count();
    if g.active == Some(p) && g.player(p).combat_no == 0 && ready >= 2 { 70 } else { 0 }
}

/// an additional combat (Relentless Assault, Seize the Day; Savage Beating entwined: double strike)
fn combat_spell(g: &mut Game, p: PlayerId, c: CardId, _ctx: &Ctx) -> Res<Sym> {
    g.player_mut(p).extra_combats += 1;
    match &*g.db.get(c).name {
        "Savage Beating" if can_pay(g, p, 1, "R", false) => {
            pay(g, p, 1, "R", false)?;
            for m in creatures(g, p) {
                eot_kw(g, m, "double strike");
            }
        }
        "World at War" => {
            g.player_mut(p).rebound.push(c);
            return Ok("exile");
        }
        _ => {}
    }
    Ok("gy")
}

/// World at War's rebound: another additional combat
fn war_rebound(g: &mut Game, p: PlayerId, c: CardId) -> Res {
    if remove_card(&mut g.player_mut(p).exile, c) {
        let pl = g.player_mut(p);
        pl.gy.push(c);
        pl.extra_combats += 1;
    }
    Ok(())
}

/// cast in the main phase onto the best attacker
fn embercleave(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = g.perm(src).owner;
    let ready = |g: &Game| creatures(g, o).into_iter().filter(|&x| !g.perm(x).noatk).collect::<Vec<_>>();
    if ready(g).is_empty() {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "attach to a creature", Some(4.0))? || !controls(g, o, src) {
        return Ok(());
    }
    let cr = ready(g);
    if let Some(x) = first_max(&cr, |x| (!g.perm(x).sick, epow(g, x))) {
        g.perm_mut(src).attached = Some(x);
    }
    Ok(())
}

/// a hasty token copy of the equipped creature at the start of each combat
fn helm(g: &mut Game, src: Src, p: PlayerId) -> Res<Vec<PermId>> {
    let host = |g: &Game| g.perm(src).attached.filter(|&a| controls(g, p, a) && g.perm(a).cd.is_some());
    if g.perm(src).owner != p {
        return Ok(vec![]);
    }
    let Some(a) = host(g) else { return Ok(vec![]) };
    let name = format!("create a token copy of {}", g.perm(a).name);
    if !trigger_window(g, p, Some(src), &name, Some(5.0))? {
        return Ok(vec![]);
    }
    let Some(a) = host(g) else { return Ok(vec![]) };
    let name = g.perm(a).name; // the copy can make the legend rule remove the equipped original
    let Some(t) = enter_token_copy(g, p, g.perm(a).cd.unwrap())? else { return Ok(vec![]) };
    g.perm_mut(t).sick = false;
    crate::glog!(g, "    Helm of the Host copies {name}");
    Ok(vec![t])
}

/// fetches an Equipment
fn sfm(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m == src && trigger_window(g, o, Some(src), "search for an Equipment", None)? {
        super::t1::tutor_named(g, o, &|g, c| g.db.get(c).has_subtype("equipment"), 1, "hand")?;
    }
    Ok(())
}

/// Khans: an impulse card each upkeep
fn siege(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p == g.perm(src).owner
        && !g.player(p).library.is_empty()
        && trigger_window(g, p, Some(src), "exile the top card; you may play it", None)?
        && let Some(c) = g.player_mut(p).library.pop()
    {
        let pl = g.player_mut(p);
        pl.hand.push(c);
        pl.impulse.push(c);
        pl.seen.insert(c);
    }
    Ok(())
}

/// proliferates at your end step
fn atraxa(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p == g.perm(src).owner && trigger_window(g, p, Some(src), "proliferate", None)? {
        crate::impls::common::proliferate(g, p, 1)?;
    }
    Ok(())
}

/// landfall: proliferate
fn evosage(g: &mut Game, src: Src, p: PlayerId) -> Res {
    atraxa(g, src, p)
}

/// noncreature spell: proliferate
fn flux(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    if caster == g.perm(src).owner
        && !g.db.get(c).creature
        && trigger_window(g, caster, Some(src), "proliferate", None)?
    {
        crate::impls::common::proliferate(g, caster, 1)?;
    }
    Ok(())
}

/// every spell: proliferate
fn tide(g: &mut Game, src: Src, caster: PlayerId, _c: CardId) -> Res {
    if caster == g.perm(src).owner && trigger_window(g, caster, Some(src), "proliferate", None)? {
        crate::impls::common::proliferate(g, caster, 1)?;
    }
    Ok(())
}

fn thrum(g: &mut Game, src: Src, p: PlayerId, a: PermId, _d: PlayerId, _dmg: i32) -> Res {
    if a == src && trigger_window(g, p, Some(src), "proliferate", None)? {
        crate::impls::common::proliferate(g, p, 1)?;
    }
    Ok(())
}

/// proliferate twice; its indestructible ability is not used
fn tekuthal(g: &Game, src: Src, p: PlayerId) -> i32 {
    (p == g.perm(src).owner) as i32
}

/// planeswalkers enter with double loyalty (tokens and counters are its compiled replacements)
fn doubling(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src
        && g.perm(m).owner == g.perm(src).owner
        && g.perm(m).loyalty.is_some()
        && card_is(g, m, Types::PLANESWALKER)
    {
        let x = g.perm_mut(m);
        x.loyalty = x.loyalty.map(|l| l * 2);
    }
    Ok(())
}

fn plan_prio(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    35
}

fn contentious_plan(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    crate::impls::common::proliferate(g, p, 1)?;
    draw(g, p, 1, false)?;
    Ok("gy")
}

// ------------------------------------------------------------------ the planeswalkers
fn nothing(_g: &mut Game, _p: PlayerId, _src: PermId) -> Res {
    Ok(())
}

fn jace_minus_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let t = best_opp_creature(g, p, |_| true)?;
    (pval(g, t) >= 5.0).then(|| pval(g, t) - 3.0)
}

fn jace_minus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    if let Some(t) = best_opp_creature(g, p, |_| true) {
        apply_removal(g, Some(p), t, "bounce", None)?;
    }
    Ok(())
}

/// Brainstorm, fateseal as loyalty only, bounce; -12 exiles a library (rules)
static JTMS: [WalkerAb; 4] = [
    WalkerAb {
        delta: 0,
        label: "Brainstorm",
        val: |_, _, _| Some(3.5),
        eff: |g, p, _| super::topdeck::brainstorm_effect(g, p, None),
    },
    WalkerAb { delta: 2, label: "fateseal", val: |_, _, _| Some(2.0), eff: nothing },
    WalkerAb { delta: -1, label: "bounce a creature", val: jace_minus_val, eff: jace_minus },
    super::rules::JTMS_ULT,
];

/// +4: the opponent with the most cards loses its least useful one (Python removes it from the hand without putting
/// it in exile)
fn karn_plus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let opps: Vec<PlayerId> = g.opps(p).collect();
    let Some(q) = first_max(&opps, |q| g.player(q).hand.len()) else { return Ok(()) };
    let hand = g.player(q).hand.clone();
    if let Some(c) = min_by(&hand, |c| card_worth(g, q, c, false)) {
        remove_card(&mut g.player_mut(q).hand, c);
    }
    Ok(())
}

fn karn_minus_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let t = best_opp_nonland(g, p, |_| true)?;
    (pval(g, t) >= 4.0).then(|| pval(g, t) - 2.0)
}

fn karn_minus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    if let Some(t) = best_opp_nonland(g, p, |_| true) {
        apply_removal(g, Some(p), t, "exile", None)?;
    }
    Ok(())
}

/// +4 takes the opponent's worst card; -3 exiles a permanent; -14 restarts the game (rules: read as a win)
static KARN: [WalkerAb; 3] = [
    WalkerAb { delta: 4, label: "opponent exiles a card from hand", val: |_, _, _| Some(2.5), eff: karn_plus },
    WalkerAb { delta: -3, label: "exile a permanent", val: karn_minus_val, eff: karn_minus },
    super::rules::KARN_ULT,
];

/// +2 Food, +1 the best opposing creature becomes a vanilla 3/3 Elk (rules), -5 exchange control (rules)
static OKO: [WalkerAb; 3] = [
    WalkerAb {
        delta: 2,
        label: "Food",
        val: |_, _, _| Some(2.0),
        eff: |g, p, _| make_artifact_tokens(g, p, "Food", 1),
    },
    super::rules::OKO_ELK,
    super::rules::OKO_ULT,
];

fn ttr_target(g: &Game, p: PlayerId) -> Option<PermId> {
    best_opp_nonland(g, p, |m| g.is_creature(m) || ench_or_art(g, m))
}

fn ttr_minus_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let t = ttr_target(g, p)?;
    (pval(g, t) >= 3.0).then(|| pval(g, t) - 1.5)
}

fn ttr_minus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    if let Some(t) = ttr_target(g, p) {
        apply_removal(g, Some(p), t, "bounce", None)?;
    }
    draw(g, p, 1, false)
}

/// opponents cast spells only at sorcery speed (their counterspells and instant removal are off on other turns); +1
/// has no effect here
static TTR: [WalkerAb; 2] = [
    WalkerAb { delta: 1, label: "sorceries at instant speed", val: |_, _, _| Some(1.5), eff: nothing },
    WalkerAb { delta: -3, label: "bounce and draw", val: ttr_minus_val, eff: ttr_minus },
];

fn ttr_lock(g: &Game, src: Src, caster: PlayerId, _c: CardId, _zone: Sym) -> bool {
    caster == g.perm(src).owner || g.active == Some(caster)
}

fn garruk_untap(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    for l in g.player(p).lands.iter().take(2).copied().collect::<Vec<_>>() {
        g.land_mut(l).tapped = false;
    }
    Ok(())
}

fn garruk_overrun_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let ready = creatures(g, p).iter().filter(|&&m| !g.perm(m).sick).count();
    (g.active == Some(p)).then(|| 1.5 * ready as f64 - 2.0)
}

fn garruk_overrun(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    for m in creatures(g, p) {
        eot(g, m, 3, 3);
    }
    g.player_mut(p).trample = true;
    Ok(())
}

static GARRUK: [WalkerAb; 3] = [
    WalkerAb { delta: 1, label: "untap two lands", val: |_, _, _| Some(1.5), eff: garruk_untap },
    WalkerAb {
        delta: -1,
        label: "3/3 Beast",
        val: |_, _, _| Some(2.8),
        eff: |g, p, _| tokens(g, p, 1, 3, 3, "G", &["beast"]).map(|_| ()),
    },
    WalkerAb { delta: -4, label: "overrun", val: garruk_overrun_val, eff: garruk_overrun },
];

fn vraska_fodder(g: &Game, p: PlayerId, src: PermId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| m != src && spare(g, m, 1.5)).collect()
}

fn vraska_plus_val(g: &Game, p: PlayerId, src: PermId) -> Option<f64> {
    let pl = g.player(p);
    (!vraska_fodder(g, p, src).is_empty() || pl.foods > 0 || pl.treasures > 0 || pl.clues > 0).then_some(2.5)
}

fn vraska_plus(g: &mut Game, p: PlayerId, src: PermId) -> Res {
    let fod = vraska_fodder(g, p, src);
    if let Some(m) = min_by(&fod, |m| pval(g, m)) {
        die(g, m, "sac")?;
        gain(g, p, 1)?;
        draw(g, p, 1, false)?;
    } else if g.player(p).foods > 0 || g.player(p).treasures > 0 || g.player(p).clues > 0 {
        sac_worst_permanent(g, p, Some(src))?;
        gain(g, p, 1)?;
        draw(g, p, 1, false)?;
    }
    Ok(())
}

fn vraska_target(g: &Game, p: PlayerId) -> Option<PermId> {
    best_opp_nonland(g, p, |m| g.perm(m).cd.is_some_and(|c| g.db.get(c).cmc <= 3))
}

fn vraska_minus_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let t = vraska_target(g, p)?;
    (pval(g, t) >= 3.5).then(|| pval(g, t) - 2.0)
}

fn vraska_minus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    if let Some(t) = vraska_target(g, p) {
        apply_removal(g, Some(p), t, "destroy", None)?;
    }
    Ok(())
}

/// +2 sacrifice for a card, -3 destroy MV 3 or less, -9 emblem (rules: combat damage makes a player lose)
static VRASKA: [WalkerAb; 3] = [
    WalkerAb { delta: 2, label: "sacrifice: gain 1, draw", val: vraska_plus_val, eff: vraska_plus },
    WalkerAb { delta: -3, label: "destroy MV 3 or less", val: vraska_minus_val, eff: vraska_minus },
    super::rules::VRASKA_ULT,
];

/// a coloured nonland permanent card with mana value 4 or less (Ugin's -X fixed at 4)
fn ugin_hit(g: &Game, m: PermId) -> bool {
    g.perm(m).cd.is_some_and(|c| {
        let d = g.db.get(c);
        d.pips.chars().any(|ch| "WUBRG".contains(ch)) && d.cmc <= 4
    })
}

fn ugin_minus_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let o = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| ugin_hit(g, m))
        .map(|m| pval(g, m))
        .psum();
    let mine = g.player(p).perms.iter().copied().filter(|&m| ugin_hit(g, m)).map(|m| pval(g, m)).psum();
    (o - mine >= 8.0).then(|| (o - mine) / 2.0 - 1.0)
}

fn ugin_minus(g: &mut Game, _p: PlayerId, _src: PermId) -> Res {
    for i in 0..g.players.len() {
        for m in g.players[i].perms.clone() {
            if ugin_hit(g, m) && g.perm(m).cd.is_some_and(|c| !g.db.get(c).land) {
                exile_perm(g, m)?;
            }
        }
    }
    Ok(())
}

/// +2 3 damage, -X exiles coloured permanents (fixed at 4), -10 gain 7, draw 7, seven permanents (rules)
static UGIN: [WalkerAb; 3] = [
    WalkerAb { delta: 2, label: "3 damage", val: |_, _, _| Some(3.0), eff: |g, p, _| best_target_any(g, p, 3) },
    WalkerAb { delta: -4, label: "exile coloured permanents MV 4 or less", val: ugin_minus_val, eff: ugin_minus },
    super::rules::UGIN_ULT,
];

fn emperor_target(g: &Game, p: PlayerId) -> Option<PermId> {
    best_opp_creature(g, p, |m| g.perm(m).tapped)
}

fn emperor_exile_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let t = emperor_target(g, p)?;
    (pval(g, t) >= 3.0).then(|| pval(g, t) - 1.5)
}

fn emperor_exile(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    if let Some(t) = emperor_target(g, p) {
        apply_removal(g, Some(p), t, "exile", None)?;
        gain(g, p, 2)?;
    }
    Ok(())
}

/// used at sorcery speed (its flash is rules2's hand_defend)
static EMPEROR: [WalkerAb; 3] = [
    WalkerAb { delta: 1, label: "+1/+1 counter, first strike", val: |_, _, _| Some(1.5), eff: nothing },
    WalkerAb {
        delta: -1,
        label: "2/2 Samurai",
        val: |_, _, _| Some(2.8),
        eff: |g, p, _| tokens(g, p, 1, 2, 2, "W", &["samurai"]).map(|_| ()),
    },
    WalkerAb { delta: -2, label: "exile a tapped creature", val: emperor_exile_val, eff: emperor_exile },
];

fn tamiyo_plus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    if creatures(g, p).iter().any(|&m| !g.perm(m).sick) {
        draw(g, p, 1, false)?;
    }
    Ok(())
}

fn tamiyo_tap(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let mut cs = opp_creatures(g, p);
    sort_desc(&mut cs, |m| pval(g, m));
    for m in cs.into_iter().take(2) {
        g.perm_mut(m).tapped = true;
    }
    Ok(())
}

/// +1 approximated as a card, -2 taps two threats, -7 draw three and cast spells from hand for free (rules)
static TAMIYO: [WalkerAb; 3] = [
    WalkerAb { delta: 1, label: "draw on combat damage", val: |_, _, _| Some(2.0), eff: tamiyo_plus },
    WalkerAb {
        delta: -2,
        label: "tap two threats",
        val: |g, p, _| (!opp_creatures(g, p).is_empty()).then_some(1.5),
        eff: tamiyo_tap,
    },
    super::rules::TAMIYO_ULT,
];

/// t3.IC_wipe_gain: what a wipe of every nonland permanent swings (theirs, less 1.3 times yours)
fn wipe_gain(g: &Game, p: PlayerId) -> f64 {
    let o = g.opps(p).flat_map(|q| g.player(q).perms.iter().copied()).map(|m| pval(g, m)).psum();
    let mine = g.player(p).perms.iter().map(|&m| pval(g, m)).psum();
    o - 1.3 * mine
}

fn hour_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if wipe_gain(g, p) >= 10.0 { 70 } else { 0 }
}

/// destroys all nonland permanents when that swings the board
fn hour_rev(g: &mut Game, _p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    for i in 0..g.players.len() {
        for m in g.players[i].perms.clone() {
            if !g.perm(m).phased {
                die(g, m, "destroy")?;
            }
        }
    }
    Ok("gy")
}

/// Ichormoon Gauntlet: a planeswalker may proliferate instead of its own ability. Python marks the walker
/// `loyalty_used = (round, key)`, which the walker framework's own (round, key, n) never equals: a walker that already
/// used its ability can still proliferate, and a proliferate counts as one use. Here the framework's use is the round
/// stamp with its count (LoyaltyN); Ichormoon's is the stamp without one.
fn ichor_used(g: &Game, p: PlayerId, w: PermId) -> bool {
    let st = TurnStamp { round: g.round, active: Some(p) };
    g.perm(w).loyalty_used == Some(st) && g.perm(w).data.get(DataKey::LoyaltyN).is_none()
}

fn ichormoon(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let ws: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| card_is(g, m, Types::PLANESWALKER) && !ichor_used(g, p, m))
        .collect();
    if post.is_none() || ws.is_empty() {
        return Ok(vec![]);
    }
    Ok(vec![ability(1.0, "Ichormoon Gauntlet: proliferate".into(), src, ichormoon_go, ws[0].0 as i64)])
}

fn ichormoon_go(g: &mut Game, _src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let w = PermId(arg as u32);
    if ichor_used(g, p, w) {
        return Ok(false);
    }
    let st = TurnStamp { round: g.round, active: Some(p) };
    let x = g.perm_mut(w);
    x.loyalty_used = Some(st);
    x.data.remove(DataKey::LoyaltyN);
    if ability_window(g, p, Some(w), "proliferate", None, None)? {
        crate::impls::common::proliferate(g, p, 1)?;
    }
    Ok(true)
}

// ======================================================== Tergrid (discard, edicts)
/// t3._on_opp_discard: an opponent discarded: this card's effect
fn opp_discard(g: &mut Game, src: Src, q: PlayerId, c: CardId) -> Res {
    let o = g.perm(src).owner;
    if q == o || !trigger_window(g, o, Some(src), &format!("{} discarded", pname(g, q)), None)? {
        return Ok(());
    }
    match name_of(g, src) {
        "Liliana's Caress" => lose_life(g, q, 2, Some(o), "drain", None),
        "Megrim" => lose_life(g, q, 2, Some(o), "triggers", None),
        "Geth's Grimoire" => {
            if g.player(o).library.len() > 10 {
                draw(g, o, 1, false)?;
            }
            Ok(())
        }
        _ => {
            // Waste Not
            let d = g.db.get(c);
            if d.creature {
                tokens(g, o, 1, 2, 2, "B", &["zombie"]).map(|_| ())
            } else if d.land {
                g.player_mut(o).floating.any += 2;
                Ok(())
            } else {
                draw(g, o, 1, false)
            }
        }
    }
}

/// t3._discard_random: q discards n cards at random
fn discard_random(g: &mut Game, q: PlayerId, n: i32) -> Res {
    for _ in 0..n.max(0) {
        let k = g.player(q).hand.len();
        if k > 0 {
            let i = g.rng.below(k as u64) as usize;
            discard_index(g, q, i)?;
        }
    }
    Ok(())
}

/// the opponent with the fullest hand (then the most threatening)
fn fullest_hand(g: &Game, p: PlayerId) -> Option<PlayerId> {
    let opps: Vec<PlayerId> = g.opps(p).collect();
    first_max(&opps, |q| (g.player(q).hand.len(), threat(g, p, q)))
}

fn fifty(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    50
}

/// two random discards from the fullest hand
fn hymn(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    if let Some(q) = fullest_hand(g, p) {
        discard_random(g, q, 2)?;
    }
    Ok("gy")
}

fn twist_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if total_mana(g, p, false) >= 4 { 50 } else { 0 }
}

/// X = spare mana, random discards
fn twist(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    if let Some(q) = fullest_hand(g, p) {
        discard_random(g, q, ctx.x)?;
    }
    Ok("gy")
}

fn quandary_prio(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    45
}

/// Painful Quandary as a spell does nothing more (its trigger is rules')
fn quandary_never(_g: &mut Game, _p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    Ok("gy")
}

fn lotv_plus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    for i in 0..g.players.len() {
        let q = g.players[i].id;
        if g.player(q).alive && !g.player(q).hand.is_empty() {
            if q == p {
                discard_worst(g, q, 1)?;
            } else {
                let k = g.player(q).hand.len() as u64;
                let i = g.rng.below(k) as usize;
                discard_index(g, q, i)?;
            }
        }
    }
    Ok(())
}

/// +1 each player discards, -2 edict, -6 two piles (rules)
static LOTV: [WalkerAb; 3] = [
    WalkerAb {
        delta: 1,
        label: "each player discards",
        val: |g, p, _| Some(if g.opps(p).map(|q| g.player(q).hand.len()).sum::<usize>() >= 2 { 2.0 } else { 0.5 }),
        eff: lotv_plus,
    },
    WalkerAb {
        delta: -2,
        label: "edict",
        val: |g, p, _| (!opp_creatures(g, p).is_empty()).then_some(3.0),
        eff: |g, p, _| match top_threat(g, p) {
            Some(q) => edict(g, q, false),
            None => Ok(()),
        },
    },
    super::rules::LOTV_ULT,
];

/// +1 Zombie, -4 each player sacrifices two creatures, -9 opponents keep one permanent of each type (rules)
static LDG: [WalkerAb; 3] = [
    WalkerAb {
        delta: 1,
        label: "2/2 Zombie",
        val: |_, _, _| Some(2.8),
        eff: |g, p, _| tokens(g, p, 1, 2, 2, "B", &["zombie"]).map(|_| ()),
    },
    WalkerAb {
        delta: -4,
        label: "each player sacrifices two",
        val: |g, p, _| (opp_creatures(g, p).len() >= 3).then_some(4.0),
        eff: |g, _p, _| {
            for i in 0..g.players.len() {
                let q = g.players[i].id;
                if g.player(q).alive {
                    edict(g, q, false)?;
                    edict(g, q, false)?;
                }
            }
            Ok(())
        },
    },
    super::rules::LDG_ULT,
];

/// creatures of yours dying draw
fn ldg(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    let o = g.perm(src).owner;
    if g.perm(m).owner == o && g.is_creature(m) && trigger_window(g, o, Some(src), "draw a card", None)? {
        draw(g, o, 1, false)?;
    }
    Ok(())
}

/// t3.discard_worst_for: an opponent chooses what to discard: their least useful card
pub fn discard_worst_for(g: &mut Game, q: PlayerId, n: u32) -> Res {
    for _ in 0..n {
        let hand = g.player(q).hand.clone();
        if let Some(c) = min_by(&hand, |c| card_worth(g, q, c, false)) {
            discard_cards(g, q, &[c])?;
        }
    }
    Ok(())
}

/// t3._each_opp_discard_etb: each opponent discards a card on entry (Burglar Rat, Elderfang Disciple)
fn each_opp_discard(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m == src && trigger_window(g, o, Some(src), "each opponent discards 1", None)? {
        for q in g.opps(o).collect::<Vec<_>>() {
            discard_worst_for(g, q, 1)?;
        }
    }
    Ok(())
}

/// an opponent puts a card from hand on top of their library
fn rats(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m != src || g.opps(o).next().is_none() {
        return Ok(());
    }
    let name = "an opponent puts a card from hand on top of their library";
    if !trigger_window(g, o, Some(src), name, Some(4.0))? {
        return Ok(());
    }
    let Some(q) = top_threat(g, o) else { return Ok(()) };
    let hand = g.player(q).hand.clone();
    if let Some(c) = min_by(&hand, |c| card_worth(g, q, c, false)) {
        remove_card(&mut g.player_mut(q).hand, c);
        g.player_mut(q).library.push(c);
    }
    Ok(())
}

/// t3._each_player_sacrifice: each player sacrifices the creature it values least (nontoken: a nontoken one)
fn each_player_sacrifice(g: &mut Game, nontoken: bool) -> Res {
    for i in 0..g.players.len() {
        let q = g.players[i].id;
        if !g.player(q).alive {
            continue;
        }
        let cr: Vec<PermId> = live_creatures(g, q).into_iter().filter(|&m| !(nontoken && g.perm(m).token)).collect();
        if let Some(m) = min_by(&cr, |m| pval(g, m)) {
            die(g, m, "sac")?;
        }
    }
    Ok(())
}

/// each player sacrifices a nontoken creature (marchesa.py replaces this hook in Python)
fn marauder(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m == src && trigger_window(g, o, Some(src), "each player sacrifices a nontoken creature", Some(5.0))? {
        each_player_sacrifice(g, true)?;
    }
    Ok(())
}

fn executioner(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m == src && trigger_window(g, o, Some(src), "each player sacrifices a creature", Some(5.0))? {
        each_player_sacrifice(g, false)?;
    }
    Ok(())
}

fn innocent_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if opp_creatures(g, p).is_empty() { 0 } else { 45 }
}

fn innocent(g: &mut Game, _p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    each_player_sacrifice(g, false)?;
    Ok("gy")
}

/// opponents keep only two creatures at their end step
fn depravity(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let o = g.perm(src).owner;
    if p == o || live_creatures(g, p).len() <= 2 {
        return Ok(());
    }
    let name = format!("{} keeps two creatures, sacrifices the rest", pname(g, p));
    if !trigger_window(g, o, Some(src), &name, Some(6.0))? {
        return Ok(());
    }
    let mut cr = live_creatures(g, p);
    sort_desc(&mut cr, |m| pval(g, m));
    for m in cr.into_iter().skip(2) {
        die(g, m, "sac")?;
    }
    Ok(())
}

/// sacrifices a spare creature each end step (creature type only): each opponent sacrifices a creature, or loses 2 and
/// you draw
fn braids(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != g.perm(src).owner {
        return Ok(());
    }
    let fodder = |g: &Game| creatures(g, p).into_iter().filter(|&m| m != src && spare(g, m, 2.0)).collect::<Vec<_>>();
    if fodder(g).is_empty() {
        return Ok(());
    }
    let name = "sacrifice a creature: opponents sacrifice or you drain";
    if !trigger_window(g, p, Some(src), name, None)? {
        return Ok(());
    }
    let fod = fodder(g);
    let Some(m) = min_by(&fod, |m| pval(g, m)) else { return Ok(()) };
    die(g, m, "sac")?;
    for q in g.opps(p).collect::<Vec<_>>() {
        let cr = creatures(g, q);
        if let Some(x) = min_by(&cr, |m| pval(g, m)) {
            die(g, x, "sac")?;
        } else {
            lose_life(g, q, 2, Some(p), "drain", None)?;
            draw(g, p, 1, false)?;
        }
    }
    Ok(())
}

/// blocking it costs the attacker that many permanents; damage from spells not counted
fn obliterator(g: &mut Game, src: Src, p: PlayerId, _atk: &[PermId], d: PlayerId, assign: &mut Assign) -> Res {
    if d != g.perm(src).owner {
        return Ok(());
    }
    if !assign.iter().any(|&(a, b)| b == src && epow(g, a) > 0) {
        return Ok(());
    }
    if !trigger_window(g, d, Some(src), &format!("{} sacrifices permanents", pname(g, p)), Some(6.0))? {
        return Ok(());
    }
    for (a, b) in assign.clone() {
        if b == src {
            for _ in 0..epow(g, a) {
                let ps: Vec<PermId> = g.player(p).perms.iter().copied().filter(|&x| !g.perm(x).phased).collect();
                let Some(x) = min_by(&ps, |x| pval(g, x)) else { break };
                die(g, x, "sac")?;
            }
        }
    }
    Ok(())
}

/// end step after an opponent discards: plays the best cheap permanent from an opponent's graveyard
fn tinybones(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != g.perm(src).owner {
        return Ok(());
    }
    let now = Some(g.turn_stamp());
    if !g.opps(p).any(|q| g.player(q).discarded_turn == now) {
        return Ok(());
    }
    let cands = |g: &Game| -> Vec<CardId> {
        g.opps(p)
            .flat_map(|q| g.player(q).gy.iter().copied())
            .filter(|&c| !g.db.get(c).land && g.db.get(c).cmc <= 4)
            .collect()
    };
    if cands(g).is_empty() || !trigger_window(g, p, Some(src), "play a card from an opponent's graveyard", None)? {
        return Ok(());
    }
    let cs = cands(g);
    if let Some(c) = max_by(&cs, |c| card_worth(g, p, c, false)) {
        for q in g.opps(p).collect::<Vec<_>>() {
            remove_card(&mut g.player_mut(q).gy, c);
        }
        if g.db.get(c).perm {
            enter(g, p, c, Enter::default())?;
        }
    }
    Ok(())
}

fn regisaur(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p == g.perm(src).owner
        && !g.player(p).hand.is_empty()
        && trigger_window(g, p, Some(src), "discard a card", None)?
    {
        discard_worst(g, p, 1)?;
    }
    Ok(())
}

/// draw for combat damage (paying 1 life); the discard-to-play ability is not used
fn gix(g: &mut Game, src: Src, p: PlayerId, a: PermId, d: PlayerId, _dmg: i32) -> Res {
    let o = g.perm(src).owner;
    if g.perm(a).owner == o
        && d != o
        && g.player(p).life > 10
        && g.player(p).library.len() > 10
        && trigger_window(g, p, Some(src), "pay 1 life: draw a card", None)?
    {
        lose_life(g, p, 1, Some(p), "other", None)?;
        draw(g, p, 1, false)?;
    }
    Ok(())
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    // Korvold
    let c = r.card(db, "Korvold, Fae-Cursed King")?;
    c.etb = Some(korvold_etb);
    c.attack = Some(korvold_atk);
    c.sacrifice = Some(korvold_sac);
    let c = r.card(db, "Mayhem Devil")?;
    c.sacrifice = Some(mayhem);
    let c = r.card(db, "Chatterfang, Squirrel General")?;
    c.token_created = Some(chatter_art);
    at_once(c, Event::TokenCreated, true);
    let c = r.card(db, "Gilded Goose")?;
    c.etb = Some(goose);
    c.dyn_mana_perm = Some(goose_mana);
    c.on_tap_perm = Some(goose_tap);
    let c = r.card(db, "Trail of Crumbs")?;
    c.etb = Some(trail);
    c.sacrifice = Some(trail_sac);
    r.card(db, "Witch's Oven")?.options = Some(oven);
    let c = r.card(db, "Savvy Hunter")?;
    c.attack = Some(savvy);
    c.options = Some(savvy_draw);
    r.card(db, "Grim Hireling")?.combat_damage = Some(hireling);
    r.card(db, "Old Gnawbone")?.combat_damage = Some(gnawbone);
    r.card(db, "Awakening Zone")?.upkeep = Some(azone);
    r.card(db, "Bloodghast")?.gy_landfall = Some(bloodghast_lf);
    r.card(db, "Reassembling Skeleton")?.gy_options = Some(skeleton);
    r.card(db, "Gravecrawler")?.gy_options = Some(gravecrawler);
    walker(r, db, "Grist, the Hunger Tide", &GRIST)?;
    let c = r.card(db, "Fable of the Mirror-Breaker // Reflection of Kiki-Jiki")?;
    c.etb = Some(fable);
    c.upkeep = Some(fable_lore);
    c.attack = Some(fable_goblin);
    // Marwyn
    r.card(db, "Marwyn, the Nurturer")?.etb = Some(marwyn);
    let c = r.card(db, "Craterhoof Behemoth")?;
    c.etb = Some(hoof);
    c.prio = Some(hoof_prio);
    let c = r.card(db, "Natural Order")?;
    c.resolve = Some(natural_order);
    c.prio = Some(natural_order_prio);
    for (name, f) in [("Green Sun's Zenith", gsz as crate::hooks::ResolveFn), ("Chord of Calling", chord)] {
        let c = r.card(db, name)?;
        c.resolve = Some(f);
        c.prio = Some(x_tutor_prio);
    }
    let c = r.card(db, "Eldritch Evolution")?;
    c.resolve = Some(eldritch);
    c.prio = Some(eldritch_prio);
    let c = r.card(db, "Kogla, the Titan Ape")?;
    c.etb = Some(kogla);
    c.attack = Some(kogla_atk);
    let c = r.card(db, "Primal Might")?;
    c.resolve = Some(primal);
    c.prio = Some(zero_prio);
    walker(r, db, "Freyalise, Llanowar's Fury", &FREYALISE)?;
    walker(r, db, "Nissa, Who Shakes the World", &NISSA)?;
    r.card(db, "Nissa, Who Shakes the World")?.land_mana = Some(nissa_wstw);
    // Aurelia
    r.card(db, "Combat Celebrant")?.attack = Some(celebrant);
    r.card(db, "Hellkite Charger")?.attack = Some(charger);
    r.card(db, "Port Razer")?.combat_damage = Some(razer);
    for name in ["Relentless Assault", "Seize the Day", "World at War", "Savage Beating"] {
        let c = r.card(db, name)?;
        c.resolve = Some(combat_spell);
        c.prio = Some(combat_spell_prio);
    }
    r.card(db, "World at War")?.rebound = Some(war_rebound);
    r.card(db, "Embercleave")?.etb = Some(embercleave);
    r.card(db, "Helm of the Host")?.combat_start = Some(helm);
    r.card(db, "Stoneforge Mystic")?.etb = Some(sfm);
    r.card(db, "Outpost Siege")?.upkeep = Some(siege);
    // Atraxa
    r.card(db, "Atraxa, Praetors' Voice")?.end_step = Some(atraxa);
    r.card(db, "Evolution Sage")?.landfall = Some(evosage);
    r.card(db, "Flux Channeler")?.cast = Some(flux);
    r.card(db, "Inexorable Tide")?.cast = Some(tide);
    r.card(db, "Thrummingbird")?.combat_damage = Some(thrum);
    r.card(db, "Tekuthal, Inquiry Dominus")?.proliferate_extra = Some(tekuthal);
    let c = r.card(db, "Doubling Season")?;
    c.etb = Some(doubling);
    at_once(c, Event::Etb, true);
    let c = r.card(db, "Contentious Plan")?;
    c.resolve = Some(contentious_plan);
    c.prio = Some(plan_prio);
    walker(r, db, "Jace, the Mind Sculptor", &JTMS)?;
    walker(r, db, "Karn Liberated", &KARN)?;
    walker(r, db, "Oko, Thief of Crowns", &OKO)?;
    walker(r, db, "Teferi, Time Raveler", &TTR)?;
    r.card(db, "Teferi, Time Raveler")?.can_cast = Some(ttr_lock);
    walker(r, db, "Garruk Wildspeaker", &GARRUK)?;
    walker(r, db, "Vraska, Golgari Queen", &VRASKA)?;
    walker(r, db, "Ugin, the Spirit Dragon", &UGIN)?;
    walker(r, db, "The Wandering Emperor", &EMPEROR)?;
    walker(r, db, "Tamiyo, Field Researcher", &TAMIYO)?;
    let c = r.card(db, "Hour of Revelation")?;
    c.resolve = Some(hour_rev);
    c.prio = Some(hour_prio);
    r.card(db, "Ichormoon Gauntlet")?.options = Some(ichormoon);
    // Tergrid
    for name in ["Liliana's Caress", "Megrim", "Geth's Grimoire", "Waste Not"] {
        r.card(db, name)?.discard = Some(opp_discard);
    }
    let c = r.card(db, "Hymn to Tourach")?;
    c.resolve = Some(hymn);
    c.prio = Some(fifty);
    let c = r.card(db, "Mind Twist")?;
    c.resolve = Some(twist);
    c.prio = Some(twist_prio);
    let c = r.card(db, "Painful Quandary")?;
    c.resolve = Some(quandary_never);
    c.prio = Some(quandary_prio);
    walker(r, db, "Liliana of the Veil", &LOTV)?;
    walker(r, db, "Liliana, Dreadhorde General", &LDG)?;
    r.card(db, "Liliana, Dreadhorde General")?.dies = Some(ldg);
    for name in ["Burglar Rat", "Elderfang Disciple"] {
        r.card(db, name)?.etb = Some(each_opp_discard);
    }
    r.card(db, "Chittering Rats")?.etb = Some(rats);
    r.card(db, "Accursed Marauder")?.etb = Some(marauder);
    r.card(db, "Merciless Executioner")?.etb = Some(executioner);
    let c = r.card(db, "Innocent Blood")?;
    c.resolve = Some(innocent);
    c.prio = Some(innocent_prio);
    r.card(db, "Archfiend of Depravity")?.end_step = Some(depravity);
    r.card(db, "Braids, Arisen Nightmare")?.end_step = Some(braids);
    r.card(db, "Phyrexian Obliterator")?.blocks = Some(obliterator);
    r.card(db, "Tinybones, Trinket Thief")?.end_step = Some(tinybones);
    r.card(db, "Rotting Regisaur")?.upkeep = Some(regisaur);
    r.card(db, "Gix, Yawgmoth Praetor")?.combat_damage = Some(gix);
    r.card(db, "Academy Manufactor")?.prio = Some(fifty);
    Ok(())
}
