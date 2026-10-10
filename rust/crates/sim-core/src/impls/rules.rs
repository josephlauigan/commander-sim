//! Python's `cards/impl/rules.py`: rules the pool cards were approximating, made exact (mana sources, counter
//! interactions, removal restrictions and taxes, "becomes a 3/3" effects, Council's Judgment's vote, Fact or Fiction's
//! split, ability locks, emblems, planeswalker ultimates, and single-card clauses).
//!
//! The ultimates rules.py appends to other modules' walkers are `WalkerAb` constants here: t3.rs's walkers list
//! theirs; the walkers of common.py (Teferi, Hero of Dominaria), t4.py (Chandra, Torch of Defiance; Kaito Shizuki)
//! and t5.py (Tezzeret the Seeker; Jace, Wielder of Mysteries) end their lists with `TEFERI_ULT`, `CHANDRA_ULT`,
//! `KAITO_ULT`, `TEZZ_ULT` and `JWOM_ULT` once those modules are ported.

use super::common::WalkerAb;
use super::partials::{
    at_once, best_opp_creature, best_opp_nonland, eot_kw, first_max, first_min, name_of, of_type, on, pack,
    remove_card, unpack,
};
use super::t3::{card_is, controls, eot, live_creatures, sort_desc, top_threat};
use crate::cards::{CardDb, Colors, Types};
use crate::engine::cast::discard_worst;
use crate::engine::cast::{castable, on_cast};
use crate::engine::life::{check_state, gain, lose_life};
use crate::engine::mana::{can_pay, pay, total_mana};
use crate::engine::removal::apply_removal;
use crate::engine::stack::{counter_window, trigger_window};
use crate::engine::tutors::{card_worth, shuffle_library};
use crate::engine::values::{colors_of, epow, equipped, etgh, has_type, pval, shielded, threat};
use crate::engine::zones::{
    Enter, Tokens, Zone, add_treasure, die, discard_cards, draw, enter, exile_perm, leave, make_tokens, max_by, min_by,
    searchable, to_zone_card,
};
use crate::flow::Res;
use crate::hooks::{Action, Event, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::pysum::PySum;
use crate::state::{Ctx, DataKey, Game, Val};
use crate::sym::Sym;
use crate::tag::Tag;

fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

// ================================================================== mana sources
/// Springleaf Drum, Jaspera Sentinel: mana only while another untapped creature can be tapped
fn drum(g: &Game, p: PlayerId, m: PermId) -> u32 {
    g.player(p).perms.iter().any(|&x| x != m && g.is_creature(x) && !g.perm(x).tapped && !g.perm(x).phased) as u32
}

/// tap the least useful untapped creature (a summoning-sick one first)
fn drum_tap(g: &mut Game, p: PlayerId, _m: PermId, _used: u32) -> Res {
    let cs: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&x| g.is_creature(x) && !g.perm(x).tapped && !g.perm(x).phased)
        .collect();
    if let Some(x) = first_min(&cs, |x| (!g.perm(x).sick, pval(g, x))) {
        g.perm_mut(x).tapped = true;
    }
    Ok(())
}

/// Lotus Petal: sacrificed as it's tapped for mana
fn petal_sac(g: &mut Game, p: PlayerId, m: PermId, _used: u32) -> Res {
    if on(g, p, m) {
        leave(g, m)?;
        to_zone_card(g, m, Zone::Gy);
    }
    Ok(())
}

/// rules.halfling_colors: Delighted Halfling's coloured mana only pays for legendary spells (None: colourless)
pub fn halfling_colors(g: &Game, p: PlayerId) -> Option<Colors> {
    let c = g.pay_for?;
    g.db.get(c).tag(Tag::Leg).then(|| g.player(p).ident).filter(|x| !x.is_empty())
}

/// rules.dryad_colors: Dryad of the Ilysian Grove: p's lands tap for any colour in its identity
pub fn dryad_colors(g: &Game, p: PlayerId) -> bool {
    g.player(p).perms.iter().any(|&m| !g.perm(m).phased && name_of(g, m) == "Dryad of the Ilysian Grove")
}

// ================================================================== tax payments
/// rules.REPLACED_TAG_ENGINES: engines whose card code replaces the generic 'eng' upkeep draw
pub fn replaced_tag_engine(name: &str) -> bool {
    matches!(name, "Mystic Remora" | "Sylvan Library")
}

// ================================================================== counters
/// your decks' keys (engine.CTHRESH): they never answer with Veil of Summer
const CTHRESH: [&str; 8] = ["seph", "veyran", "sauron", "galadriel", "yshtola", "alela", "jodah", "najeela"];

/// rules.veil_response: q is about to counter p's spell with a blue or black counterspell: p answers with Veil of
/// Summer (its spells can't be countered this turn, it draws a card)
pub fn veil_response(g: &mut Game, p: PlayerId, _q: PlayerId, ctr: CardId) -> Res<bool> {
    if g.humans.get(p).is_some() {
        return Ok(false); // HUMAN(phase 9): a person casts their own Veil
    }
    let d = g.db.get(ctr);
    let blue_black =
        d.pips.contains('U') || d.pips.contains('B') || matches!(&*d.name, "Force of Will" | "Force of Negation");
    if CTHRESH.contains(&g.player(p).key) || !blue_black {
        return Ok(false);
    }
    let Some(v) = g.player(p).hand.iter().copied().find(|&c| &*g.db.get(c).name == "Veil of Summer") else {
        return Ok(false);
    };
    if !can_pay(g, p, 0, "G", false) || !castable(g, p, v, "hand") {
        return Ok(false);
    }
    pay(g, p, 0, "G", false)?;
    remove_card(&mut g.player_mut(p).hand, v);
    g.player_mut(p).gy.push(v);
    on_cast(g, p, v)?;
    crate::glog!(g, "    {} responds with Veil of Summer: the spell can't be countered", pname(g, p));
    draw(g, p, 1, false)?;
    Ok(true)
}

// ================================================================== removal restrictions and taxes
/// rules.removal_taxes: targeting costs: Terror of the Peaks (3 life); damage to Phyrexian Obliterator costs
/// permanents. False would stop the removal (nothing here does).
pub fn removal_taxes(g: &mut Game, actor: Option<PlayerId>, m: PermId, kind: Sym) -> Res<bool> {
    let name = name_of(g, m).to_string();
    let Some(a) = actor else { return Ok(true) };
    if name == "Terror of the Peaks" {
        lose_life(g, a, 3, Some(a), "other", None)?;
    }
    if name == "Phyrexian Obliterator" && kind.starts_with("dmg") {
        let n: i32 = kind[3..].parse().unwrap_or(0);
        for _ in 0..n {
            let ps: Vec<PermId> = g.player(a).perms.iter().copied().filter(|&x| !g.perm(x).phased).collect();
            if let Some(x) = min_by(&ps, |x| pval(g, x)) {
                die(g, x, "sac")?;
            }
        }
    }
    Ok(true)
}

/// rules.transform_away: 'becomes a 3/3 Elk' (Kenrith's Transformation, Oko), 'becomes a 0/1 indestructible Insect'
/// (Darksteel Mutation), 'becomes a Forest' (Song of the Dryads)
pub fn transform_away(g: &mut Game, m: PermId, kind: Sym) -> Res {
    if !g.perm(m).on_bf {
        return Ok(());
    }
    if kind == "forest" {
        let (o, tapped) = (g.perm(m).owner, g.perm(m).tapped);
        leave(g, m)?;
        let forest = g.db.id("Forest").expect("Forest is in the card database");
        g.add_land(o, forest, tapped);
        crate::glog!(g, "    {} becomes a Forest", g.perm(m).name);
        return Ok(());
    }
    let x = g.perm_mut(m);
    x.neutered = true;
    if kind == "elk" {
        (x.pow, x.tgh) = (3, 3);
        x.data.set(DataKey::Elk, Val::Bool(true));
    } else {
        (x.pow, x.tgh) = (0, 1);
        x.data.set(DataKey::Indestr, Val::Bool(true));
    }
    (x.fly, x.dt, x.lifelink) = (false, false, false);
    if kind == "mutate" {
        x.plus = x.plus.min(0);
    }
    g.bf_ver += 1;
    crate::glog!(
        g,
        "    {} becomes a {} with no abilities",
        g.perm(m).name,
        if kind == "elk" { "3/3 Elk" } else { "0/1 Insect" }
    );
    Ok(())
}

// ================================================================== Council's Judgment: the vote
/// cast when the best opposing nonland permanent is worth exiling
fn judgment_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    match best_opp_nonland(g, p, |_, _| true) {
        Some(t) if pval(g, t) >= 4.0 => 58,
        _ => 0,
    }
}

/// will of the council: each player votes for a nonland permanent they don't control (their biggest threat); every
/// permanent with the most votes is exiled
fn judgment(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let mut votes: Vec<(PermId, u32)> = vec![];
    let voters: Vec<PlayerId> = g.players.iter().filter(|q| q.alive).map(|q| q.id).collect();
    for &v in &voters {
        let mut cands: Vec<PermId> = voters
            .iter()
            .filter(|&&q| q != v)
            .flat_map(|&q| g.player(q).perms.iter().copied())
            .filter(|&m| !g.perm(m).phased)
            .collect();
        if v == p {
            cands.retain(|&m| g.perm(m).owner != p);
        }
        let Some(pick) = max_by(&cands, |m| {
            let o = g.perm(m).owner;
            pval(g, m) * (1.0 + 0.3 * if o != v { threat(g, v, o) } else { 0.0 })
        }) else {
            continue;
        };
        match votes.iter_mut().find(|x| x.0 == pick) {
            Some(x) => x.1 += 1,
            None => votes.push((pick, 1)),
        }
    }
    let Some(top) = votes.iter().map(|x| x.1).max() else { return Ok("gy") };
    for (m, n) in votes {
        if n == top && g.perm(m).on_bf {
            crate::glog!(g, "    Council's Judgment exiles {} ({} votes)", g.perm(m).name, n);
            exile_perm(g, m)?;
        }
    }
    Ok("gy")
}

// ================================================================== Fact or Fiction: the split
fn fof_prio(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    45
}

/// the most threatened opponent splits the five into the piles that leave you least; you take the better pile
fn fof(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let mut top = vec![];
    for _ in 0..g.player(p).library.len().min(5) {
        top.push(g.player_mut(p).library.pop().unwrap());
    }
    if top.is_empty() {
        return Ok("gy");
    }
    let w: Vec<f64> = top.iter().map(|&x| card_worth(g, p, x, false).max(0.0)).collect();
    let all = w.iter().copied().psum();
    let mut best: Option<(f64, usize, bool)> = None;
    for mask in 0..(1usize << top.len()) {
        let a = (0..top.len()).filter(|i| mask >> i & 1 == 1).map(|i| w[i]).psum();
        let b = all - a;
        if best.is_none_or(|x| a.max(b) < x.0) {
            best = Some((a.max(b), mask, a >= b));
        }
    }
    let (_, mask, take_a) = best.unwrap();
    let pile: Vec<CardId> = (0..top.len()).filter(|&i| (mask >> i & 1 == 1) == take_a).map(|i| top[i]).collect();
    for x in top {
        let pl = g.player_mut(p);
        if pile.contains(&x) {
            pl.hand.push(x);
            pl.seen.insert(x);
        } else {
            pl.gy.push(x);
        }
    }
    Ok("gy")
}

// ================================================================== Boros Charm
/// 4 damage to a player at 4 life or less (the indestructible mode is the protection AI's)
fn boros_charm(g: &mut Game, c: CardId, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    if !can_pay(g, p, 0, "RW", false) || !castable(g, p, c, "hand") {
        return Ok(vec![]);
    }
    let lethal = g.opps(p).find(|&q| g.player(q).life <= 4 && !shielded(g, q));
    let Some(q) = lethal else { return Ok(vec![]) };
    Ok(vec![Opt {
        utility: 9.0,
        label: "Boros Charm (4 damage, lethal)".into(),
        act: Some(Action::Plan { f: boros_go, arg: pack(c.0 as u32, q.0 as u32) }),
    }])
}

fn boros_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (c, q) = unpack(arg);
    let (c, q) = (CardId(c as u16), PlayerId(q as u8));
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 0, "RW", false) {
        return Ok(false);
    }
    remove_card(&mut g.player_mut(p).hand, c);
    pay(g, p, 0, "RW", false)?;
    on_cast(g, p, c)?;
    if counter_window(g, p, c, 6.0, vec![])? {
        lose_life(g, q, 4, Some(p), "burn", Some(true))?;
    }
    g.player_mut(p).gy.push(c);
    Ok(true)
}

// ================================================================== ability locks
/// rules.ability_locked: can src's activated abilities not be used right now? (Arrest, Encrust; Collector Ouphe /
/// Karn: artifacts; Cursed Totem: creatures; Grand Abolisher: nothing of yours during its controller's turn)
pub fn ability_locked(g: &Game, src: PermId, p: PlayerId) -> bool {
    if !g.auras.is_empty() && crate::cardcode::locked(g, src, "noact") {
        return true;
    }
    let x = g.perm(src);
    if g.hooks.is_empty() || x.cd.is_none() && !g.is_creature(src) {
        return false;
    }
    let art = of_type(g, src, Types::ARTIFACT) || (x.token && x.ttypes.contains(&"artifact"));
    for q in g.players.iter().filter(|q| q.alive) {
        for &m in &q.perms {
            let y = g.perm(m);
            if y.cd.is_none() || y.phased || y.neutered {
                continue;
            }
            match name_of(g, m) {
                "Collector Ouphe" if art => return true,
                "Karn, the Great Creator" if art && p != q.id => return true,
                "Cursed Totem" if g.is_creature(src) => return true,
                "Grand Abolisher" if p != q.id && g.active == Some(q.id) => return true,
                _ => {}
            }
        }
    }
    false
}

// ================================================================== emblems
/// rules.emblem_cast: Chandra's emblem (5 damage per spell), Venser's (exile a permanent per spell)
pub fn emblem_cast(g: &mut Game, p: PlayerId, _c: CardId) -> Res {
    if g.player(p).emblems.contains(&"chandra") {
        let t = best_opp_creature(g, p, |g, m| etgh(g, m) <= 5);
        let lethal = g.opps(p).find(|&q| g.player(q).life <= 5);
        if let Some(q) = lethal {
            lose_life(g, q, 5, Some(p), "burn", Some(true))?;
        } else if let Some(t) = t.filter(|&t| pval(g, t) >= 4.0) {
            apply_removal(g, Some(p), t, "dmg5", None)?;
        } else {
            let opps: Vec<PlayerId> = g.opps(p).collect();
            if let Some(q) = first_min(&opps, |q| g.player(q).life) {
                lose_life(g, q, 5, Some(p), "burn", Some(true))?;
            }
        }
    }
    if g.player(p).emblems.contains(&"venser") {
        if let Some(t) = best_opp_nonland(g, p, |_, _| true).filter(|&t| pval(g, t) >= 1.0) {
            apply_removal(g, Some(p), t, "exile", None)?;
        }
    }
    Ok(())
}

/// rules.emblem_draw: Teferi's emblem exiles an opposing permanent whenever you draw
pub fn emblem_draw(g: &mut Game, p: PlayerId) -> Res {
    if g.player(p).emblems.contains(&"teferi")
        && let Some(t) = best_opp_nonland(g, p, |_, _| true)
    {
        apply_removal(g, Some(p), t, "exile", None)?;
    }
    Ok(())
}

/// rules.emblem_combat: Vraska's emblem (combat damage makes a player lose), Kaito's (a creature from the library)
pub fn emblem_combat(g: &mut Game, p: PlayerId, a: PermId, d: PlayerId, dmg: i32) -> Res {
    if dmg <= 0 || !g.is_creature(a) {
        return Ok(());
    }
    if g.player(p).emblems.contains(&"vraska") && g.player(d).alive {
        crate::glog!(g, "    Vraska's emblem: {} loses the game", pname(g, d));
        let pl = g.player_mut(d);
        pl.life = 0;
        pl.last_src = Some(p);
        check_state(g)?;
    }
    if g.player(p).emblems.contains(&"kaito") {
        let cs: Vec<CardId> = searchable(g, p)
            .into_iter()
            .filter(|&c| {
                let x = g.db.get(c);
                x.creature && (x.pips.contains('U') || x.pips.contains('B'))
            })
            .collect();
        if let Some(c) = first_max(&cs, |c| (g.db.get(c).bomb, card_worth(g, p, c, false))) {
            remove_card(&mut g.player_mut(p).library, c);
            shuffle_library(g, p);
            enter(g, p, c, Enter::default())?;
        }
    }
    Ok(())
}

/// rules.dovin_blocked: Dovin, Hand of Control's -1: damage to and from m is prevented until its controller's next
/// turn
pub fn dovin_blocked(g: &Game, m: PermId) -> bool {
    match g.perm(m).data.get(DataKey::Dovin) {
        Some(Val::List(v)) => match v.as_slice() {
            [Val::Player(q), Val::Int(t)] => g.player(*q).alive && g.player(*q).turns as i64 == *t,
            _ => false,
        },
        _ => false,
    }
}

// ================================================================== single-card clauses
/// Nature's Claim: its controller gains 4 life
fn natures_claim(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let t = ctx
        .target
        .or_else(|| best_opp_nonland(g, p, |g, m| of_type(g, m, Types::ARTIFACT) || of_type(g, m, Types::ENCHANTMENT)));
    if let Some(t) = t.filter(|&t| g.perm(t).on_bf) {
        let q = g.perm(t).owner;
        apply_removal(g, Some(p), t, "destroy", Some(c))?;
        gain(g, q, 4)?;
    }
    Ok("gy")
}

/// Tithe Taker: afterlife 1
fn tithe_afterlife(g: &mut Game, _src: Src, m: PermId, _cause: Sym) -> Res {
    let o = g.perm(m).owner;
    if trigger_window(g, o, Some(m), "create a 1/1 flying Spirit", None)? {
        let spec =
            Tokens { fly: true, color: Some(Colors::from_letters("W")), types: vec!["spirit"], ..Tokens::new(1, 1) };
        make_tokens(g, o, spec)?;
    }
    Ok(())
}

/// rules.evasion_blocked: extra blocking restrictions: fear, Amrou Seekers, Signal Pest, Legion Loyalist's battalion
/// (tokens can't block), intimidate, protection from creature types
pub fn evasion_blocked(g: &Game, b: PermId, a: PermId) -> bool {
    if g.perm(a).cd.is_none() {
        return false;
    }
    let art_or = |col: char| of_type(g, b, Types::ARTIFACT) || colors_of(g, b).has(col);
    match name_of(g, a) {
        "Shriekmaw" if !art_or('B') => return true,
        "Amrou Seekers" if !art_or('W') => return true,
        "Signal Pest" => {
            let y = g.perm(b);
            let reach = y.cd.is_some_and(|c| g.db.get(c).tag(Tag::Reach));
            if !(y.fly || reach || crate::dsl::has_kw(g, b, "flying") || crate::dsl::has_kw(g, b, "reach")) {
                return true;
            }
        }
        _ => {}
    }
    if g.perm(b).token && g.player(g.perm(a).owner).loyalist_turn == Some(g.turn_stamp()) {
        return true; // Legion Loyalist
    }
    if equipped(g, a, Tag::Nim) && !art_or('B') {
        return true; // intimidate
    }
    super::rules2::prot_unblockable(g, b, a)
}

/// Brimaz: a 1/1 Cat blocking with it
fn brimaz_block(
    g: &mut Game,
    src: Src,
    _p: PlayerId,
    _atk: &[PermId],
    d: PlayerId,
    assign: &mut Vec<(PermId, PermId)>,
) -> Res {
    if g.perm(src).owner == d
        && assign.iter().any(|x| x.1 == src)
        && trigger_window(g, d, Some(src), "create a blocking 1/1 Cat", None)?
    {
        let spec = Tokens { color: Some(Colors::from_letters("W")), types: vec!["cat"], ..Tokens::new(1, 1) };
        make_tokens(g, d, spec)?;
    }
    Ok(())
}

/// Allosaurus Shepherd: your green spells can't be countered
fn shepherd(g: &Game, src: Src, p: PlayerId, c: CardId) -> bool {
    p == g.perm(src).owner && g.db.get(c).pips.contains('G')
}

/// Ezuri: rules.py replaces t1's overrun with no activated abilities (its regeneration is ezuri_regen)
fn ezuri_none(_g: &mut Game, _src: Src, _p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    Ok(vec![])
}

/// rules.ezuri_regen: {G}: regenerate another Elf (saves valuable Elves)
pub fn ezuri_regen(g: &mut Game, m: PermId) -> Res<bool> {
    let p = g.perm(m).owner;
    if !has_type(g, m, "elf") || name_of(g, m) == "Ezuri, Renegade Leader" {
        return Ok(false);
    }
    if !g.player(p).perms.iter().any(|&x| !g.perm(x).phased && name_of(g, x) == "Ezuri, Renegade Leader") {
        return Ok(false);
    }
    if pval(g, m) < 3.0 || !can_pay(g, p, 0, "G", false) {
        return Ok(false);
    }
    pay(g, p, 0, "G", false)?;
    g.perm_mut(m).tapped = true;
    crate::glog!(g, "    Ezuri regenerates {}", g.perm(m).name);
    Ok(true)
}

/// Mirkwood Bats: creating tokens drains each opponent
fn bats_make(g: &mut Game, src: Src, p: PlayerId, _kinds: &[Sym], n: i32) -> Res {
    if g.perm(src).owner == p && trigger_window(g, p, Some(src), &format!("each opponent loses {n} life"), None)? {
        for q in g.opps(p).collect::<Vec<_>>() {
            lose_life(g, q, n, Some(p), "drain", None)?;
        }
    }
    Ok(())
}

/// Bloodchief's Thirst as it's cast (a SELF_CAST hook): is it kicked? Bloodchief's Thirst is {B} with kicker {2}{B};
/// the engine stores it at the cost the AI always pays, {2}{B}{B}: the {B} and the kicker. The removal code aims it
/// at a creature or planeswalker with mana value 3 or more only when that mana is there (engine/removal.rs
/// legal_targets), and the AI pays it whatever the target, so the kick is taken exactly when the target needs it:
/// recorded here for the resolve. HUMAN(phase 9): a person pays the printed {B}, and the kicker only for such a
/// target (play/legal.target_extra_cost).
fn thirst_cast(g: &mut Game, p: PlayerId, c: CardId) -> Res {
    let target = g.cur_cast.as_ref().filter(|x| x.0 == c).and_then(|x| x.1.target);
    let needs = target.is_some_and(|t| g.perm(t).cd.is_some_and(|x| g.db.get(x).cmc > 2));
    g.player_mut(p).thirst_kicked = needs;
    Ok(())
}

/// Bloodchief's Thirst: destroys a creature or planeswalker with mana value 2 or less, or anything when kicked
/// (kicked when the target needs it and the mana is there).
/// Fix (Rust only): Python reads `getattr(p, 'thirst_kicked', False)`, which nothing sets, so a target with mana value
/// 3 or more (chosen only when the kicker mana is there, and paid for) was never destroyed. The kick is recorded by
/// `thirst_cast` as it's cast.
fn thirst(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let Some(t) = ctx.target.filter(|&t| g.perm(t).on_bf) else { return Ok("gy") };
    let mv = g.perm(t).cd.map_or(0, |x| g.db.get(x).cmc);
    let kicked = g.player(p).thirst_kicked;
    if mv > 2 && !kicked {
        return Ok("gy");
    }
    apply_removal(g, Some(p), t, "destroy", Some(c))?;
    Ok("gy")
}

// ================================================================== more mana sources
/// Mox Amber: mana only while you control a legendary creature or planeswalker
fn moxamber(g: &Game, p: PlayerId, _m: PermId) -> u32 {
    g.player(p).perms.iter().any(|&x| {
        let y = g.perm(x);
        y.cd.is_some_and(|c| g.db.get(c).tag(Tag::Leg))
            && (g.is_creature(x) || card_is(g, x, Types::PLANESWALKER))
            && !y.phased
    }) as u32
}

/// Mox Opal: metalcraft: mana only with three or more artifacts
fn moxopal(g: &Game, p: PlayerId, _m: PermId) -> u32 {
    let n = g
        .player(p)
        .perms
        .iter()
        .filter(|&&x| card_is(g, x, Types::ARTIFACT) || (g.perm(x).token && g.perm(x).ttypes.contains(&"artifact")))
        .count();
    (n >= 3) as u32
}

/// Sanctum Weaver: {T}: X mana of one colour, X = enchantments you control
fn sanctum_weaver(g: &Game, p: PlayerId, _m: PermId) -> u32 {
    g.player(p).perms.iter().filter(|&&x| card_is(g, x, Types::ENCHANTMENT) && !g.perm(x).phased).count() as u32
}

/// Goldspan Dragon: a Treasure when it attacks
fn goldspan_atk(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if p == g.perm(src).owner && atk.contains(&src) && trigger_window(g, p, Some(src), "create a Treasure", None)? {
        add_treasure(g, p, 1)?;
    }
    Ok(vec![])
}

/// Goldspan Dragon: your Treasures tap for two mana
fn goldspan_bonus(g: &Game, src: Src, p: PlayerId) -> i32 {
    (p == g.perm(src).owner) as i32
}

// ================================================================== Mystic Remora
/// draws when an opponent casts a noncreature spell unless they can spare {4}
fn remora(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let o = g.perm(src).owner;
    let d = g.db.get(c);
    if caster == o || d.creature || d.land {
        return Ok(());
    }
    let name = format!("draw a card unless {} pays {{4}}", pname(g, caster));
    if !trigger_window(g, o, Some(src), &name, None)? {
        return Ok(());
    }
    // HUMAN(phase 9): a person decides whether to pay (hc.pay_tax)
    if crate::cardcode::spare_after(g, caster, 4) {
        pay(g, caster, 4, "", false)?;
        return Ok(());
    }
    draw(g, o, 1, false)
}

/// cumulative upkeep {1} (kept three turns)
fn remora_age(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != g.perm(src).owner || !trigger_window(g, p, Some(src), "cumulative upkeep {1}", None)? {
        return Ok(());
    }
    let age = g.perm(src).data.int(DataKey::Age) + 1;
    g.perm_mut(src).data.set(DataKey::Age, Val::Int(age));
    // HUMAN(phase 9): a person pays the cumulative upkeep, or sacrifices it
    if age <= 3 && can_pay(g, p, age as u32, "", false) {
        pay(g, p, age as u32, "", false)?;
    } else {
        crate::glog!(g, "    {} sacrifices Mystic Remora (cumulative upkeep {})", pname(g, p), age);
        leave(g, src)?;
        to_zone_card(g, src, Zone::Gy);
    }
    Ok(())
}

// ================================================================== planeswalker ultimates and emblems
/// rules.give_emblem
pub fn give_emblem(g: &mut Game, p: PlayerId, kind: Sym) {
    let e = &mut g.player_mut(p).emblems;
    if !e.contains(&kind) {
        e.push(kind);
    }
}

/// Chandra, Torch of Defiance -7: emblem (5 damage per spell you cast). For t4's walker list.
pub const CHANDRA_ULT: WalkerAb = WalkerAb {
    delta: -7,
    label: "emblem",
    val: |_, _, _| Some(9.0),
    eff: |g, p, _| {
        give_emblem(g, p, "chandra");
        Ok(())
    },
};

/// Vraska, Golgari Queen -9: emblem (combat damage makes a player lose)
pub const VRASKA_ULT: WalkerAb = WalkerAb {
    delta: -9,
    label: "emblem",
    val: |_, _, _| Some(12.0),
    eff: |g, p, _| {
        give_emblem(g, p, "vraska");
        Ok(())
    },
};

/// Teferi, Hero of Dominaria -8: emblem (exile an opposing permanent whenever you draw). For common's walker list.
pub const TEFERI_ULT: WalkerAb = WalkerAb {
    delta: -8,
    label: "emblem",
    val: |_, _, _| Some(10.0),
    eff: |g, p, _| {
        give_emblem(g, p, "teferi");
        Ok(())
    },
};

/// Kaito Shizuki -7: emblem (combat damage puts a creature from your library onto the battlefield). For t4's list.
pub const KAITO_ULT: WalkerAb = WalkerAb {
    delta: -7,
    label: "emblem",
    val: |_, _, _| Some(8.0),
    eff: |g, p, _| {
        give_emblem(g, p, "kaito");
        Ok(())
    },
};

/// Tamiyo, Field Researcher -7: draw three and cast spells from hand for free
fn tamiyo_ult(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    draw(g, p, 3, false)?;
    give_emblem(g, p, "tamiyo");
    Ok(())
}

pub const TAMIYO_ULT: WalkerAb =
    WalkerAb { delta: -7, label: "draw three and emblem", val: |_, _, _| Some(10.0), eff: tamiyo_ult };

/// Liliana of the Veil -6: the opponent with the most on the table splits it into two piles (alternating by value);
/// it keeps the better pile and sacrifices the rest
fn lotv_ult(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let opps: Vec<PlayerId> = g.opps(p).collect();
    let Some(q) = max_by(&opps, |q| g.player(q).perms.iter().map(|&m| pval(g, m)).psum()) else { return Ok(()) };
    let mut ps: Vec<PermId> = g.player(q).perms.iter().copied().filter(|&m| !g.perm(m).phased).collect();
    sort_desc(&mut ps, |m| pval(g, m));
    let a: Vec<PermId> = ps.iter().copied().step_by(2).collect();
    let b: Vec<PermId> = ps.iter().copied().skip(1).step_by(2).collect();
    let keep = if a.iter().map(|&m| pval(g, m)).psum() >= b.iter().map(|&m| pval(g, m)).psum() { a } else { b };
    for m in ps {
        if !keep.contains(&m) && controls(g, q, m) {
            die(g, m, "sac")?;
        }
    }
    Ok(())
}

pub const LOTV_ULT: WalkerAb =
    WalkerAb { delta: -6, label: "split permanents", val: |g, p, _| g.opps(p).next().map(|_| 8.0), eff: lotv_ult };

/// the first letter of a card's type line as the engine stores it (Python's `m.cd.types[0]`): the stored codes put a
/// land first, then an artifact ('LA', 'LE', 'AP'); the creatures are keyed apart
fn first_type(g: &Game, m: PermId) -> char {
    let Some(c) = g.perm(m).cd else { return 'T' };
    let t = g.db.get(c).types;
    [
        (Types::LAND, 'L'),
        (Types::ARTIFACT, 'A'),
        (Types::ENCHANTMENT, 'E'),
        (Types::PLANESWALKER, 'P'),
        (Types::CREATURE, 'C'),
        (Types::INSTANT, 'I'),
        (Types::SORCERY, 'S'),
    ]
    .iter()
    .find(|x| t.has(x.0))
    .map_or('T', |x| x.1)
}

/// Liliana, Dreadhorde General -9: each opponent keeps one permanent of each type (and one land)
fn ldg_ult(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    for q in g.opps(p).collect::<Vec<_>>() {
        let mut ps: Vec<PermId> = g.player(q).perms.iter().copied().filter(|&m| !g.perm(m).phased).collect();
        sort_desc(&mut ps, |m| pval(g, m));
        let mut keep: Vec<(char, PermId)> = vec![];
        for m in ps {
            let k = if g.is_creature(m) { 'C' } else { first_type(g, m) };
            if !keep.iter().any(|x| x.0 == k) {
                keep.push((k, m));
            }
        }
        for m in g.player(q).perms.clone() {
            if !keep.iter().any(|x| x.1 == m) {
                die(g, m, "sac")?;
            }
        }
        let lands = g.player(q).lands.clone();
        if lands.len() > 1 {
            let amt = |l: crate::ids::LandId| g.db.get(g.land(l).cd).tags.int(Tag::Amt).unwrap_or(1);
            let best = first_max(&lands, amt).unwrap();
            for l in lands {
                if l != best {
                    crate::engine::turn::remove_land(g, q, l);
                    let cd = g.land(l).cd;
                    g.player_mut(q).gy.push(cd);
                }
            }
        }
    }
    Ok(())
}

pub const LDG_ULT: WalkerAb =
    WalkerAb { delta: -9, label: "each opponent keeps one of each type", val: |_, _, _| Some(12.0), eff: ldg_ult };

/// Jace, the Mind Sculptor -12: the most threatening opponent's library is exiled; its hand becomes its library
fn jtms_ult(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let Some(q) = top_threat(g, p) else { return Ok(()) };
    let pl = g.player_mut(q);
    let lib = std::mem::take(&mut pl.library);
    pl.exile.extend(lib);
    pl.library = std::mem::take(&mut pl.hand);
    shuffle_library(g, q);
    crate::glog!(g, "    Jace exiles {}'s library", pname(g, q));
    Ok(())
}

pub const JTMS_ULT: WalkerAb =
    WalkerAb { delta: -12, label: "exile a library", val: |_, _, _| Some(10.0), eff: jtms_ult };

/// Karn Liberated -14: restart the game with Karn's exiled cards (read as a win)
fn karn_ult(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    crate::glog!(g, "    Karn Liberated restarts the game");
    for q in g.opps(p).collect::<Vec<_>>() {
        let pl = g.player_mut(q);
        pl.life = 0;
        pl.last_src = Some(p);
    }
    check_state(g)
}

pub const KARN_ULT: WalkerAb =
    WalkerAb { delta: -14, label: "restart the game", val: |_, _, _| Some(15.0), eff: karn_ult };

/// rules._ugin_ult: Ugin, the Spirit Dragon -10: gain 7, draw 7, then up to seven permanents from hand onto the
/// battlefield, the best first (one an earlier one's enter trigger made you discard is skipped)
pub fn ugin_ult(g: &mut Game, p: PlayerId, _src: Option<PermId>) -> Res {
    gain(g, p, 7)?;
    draw(g, p, 7, false)?;
    let mut cs: Vec<CardId> =
        g.player(p).hand.iter().copied().filter(|&c| g.db.get(c).perm && !g.db.get(c).land).collect();
    sort_desc(&mut cs, |c| card_worth(g, p, c, false));
    for c in cs.into_iter().take(7) {
        if !remove_card(&mut g.player_mut(p).hand, c) {
            continue; // an earlier one's enter trigger made you discard it
        }
        enter(g, p, c, Enter::default())?;
    }
    Ok(())
}

pub const UGIN_ULT: WalkerAb = WalkerAb {
    delta: -10,
    label: "gain 7, draw 7, put 7 permanents",
    val: |_, _, _| Some(12.0),
    eff: |g, p, src| ugin_ult(g, p, Some(src)),
};

/// Nissa, Who Shakes the World -8: every Forest from the library onto the battlefield tapped (the emblem, lands
/// indestructible, is recorded in Python as `p.nissa_emblem`, which nothing reads)
fn nissa_ult(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let forests: Vec<CardId> = searchable(g, p)
        .into_iter()
        .filter(|&c| {
            let d = g.db.get(c);
            d.land && (&*d.name == "Forest" || d.has_subtype("forest"))
        })
        .collect();
    for c in forests {
        remove_card(&mut g.player_mut(p).library, c);
        g.add_land(p, c, true);
    }
    shuffle_library(g, p);
    Ok(())
}

pub const NISSA_ULT: WalkerAb =
    WalkerAb { delta: -8, label: "emblem and Forests", val: |_, _, _| Some(8.0), eff: nissa_ult };

/// Tezzeret the Seeker -5: your artifacts become 5/5 creatures (Python sets `p.tezz_turn`, which nothing reads, so
/// they stay creatures)
fn tezz_ult(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    for m in g.player(p).perms.clone() {
        if card_is(g, m, Types::ARTIFACT) || (g.perm(m).token && g.perm(m).ttypes.contains(&"artifact")) {
            let x = g.perm_mut(m);
            x.data.set(DataKey::Anim, Val::Bool(true));
            (x.pow, x.tgh) = (5, 5);
            x.sick = false;
        }
    }
    Ok(())
}

/// For t5's walker list.
pub const TEZZ_ULT: WalkerAb = WalkerAb {
    delta: -5,
    label: "artifacts become 5/5",
    val: |g, p, src| {
        let n = g.player(p).perms.iter().filter(|&&m| card_is(g, m, Types::ARTIFACT) && !g.is_creature(m)).count();
        (g.perm(src).loyalty.unwrap_or(0) >= 5).then(|| 1.5 * n as f64)
    },
    eff: tezz_ult,
};

/// Jace, Wielder of Mysteries -8: draw seven; an empty library wins
fn jwom_ult(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    draw(g, p, 7, false)?;
    if g.player(p).library.is_empty() {
        crate::ai::win(g, p, "Jace", None)?;
    }
    Ok(())
}

/// For t5's walker list.
pub const JWOM_ULT: WalkerAb = WalkerAb {
    delta: -8,
    label: "draw seven",
    val: |g, p, _| Some(if g.player(p).library.len() <= 7 { 12.0 } else { 4.0 }),
    eff: jwom_ult,
};

/// Oko +1, exact (rules replaces t3's vanilla 3/3): the best opposing creature becomes a 3/3 Elk with no abilities
fn oko_elk_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let t = best_opp_creature(g, p, |_, _| true)?;
    (pval(g, t) >= 4.0).then(|| pval(g, t) - 2.0)
}

fn oko_elk_exact(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    if let Some(t) = best_opp_creature(g, p, |_, _| true) {
        transform_away(g, t, "elk")?;
    }
    Ok(())
}

pub const OKO_ELK: WalkerAb =
    WalkerAb { delta: 1, label: "Elk an opposing creature", val: oko_elk_val, eff: oko_elk_exact };

fn oko_steal_target(g: &Game, p: PlayerId) -> Option<PermId> {
    best_opp_creature(g, p, |g, m| epow(g, m) <= 3)
}

/// Oko -5: exchange a spare creature (or Food) for their best creature with power 3 or less (targeted: it can be
/// answered)
fn oko_exchange(g: &mut Game, p: PlayerId, src: PermId) -> Res {
    let mine: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            (x.token && x.ttypes.contains(&"food")) || (g.is_creature(m) && pval(g, m) < 2.0)
        })
        .collect();
    let Some(t) = oko_steal_target(g, p) else { return Ok(()) };
    let q = g.perm(t).owner;
    let cd = g.perm(src).cd;
    if crate::ai::protect_response(g, q, t, "steal", Some(p), cd)? || !controls(g, q, t) {
        return Ok(());
    }
    g.player_mut(q).perms.retain(|&x| x != t);
    g.perm_mut(t).owner = p;
    g.player_mut(p).perms.push(t);
    g.perm_mut(t).sick = true;
    if let Some(&x) = mine.first() {
        g.player_mut(p).perms.retain(|&y| y != x);
        g.perm_mut(x).owner = q;
        g.player_mut(q).perms.push(x);
    }
    g.bf_ver += 1;
    crate::glog!(g, "    Oko exchanges control: {} takes {}", pname(g, p), g.perm(t).name);
    Ok(())
}

pub const OKO_ULT: WalkerAb = WalkerAb {
    delta: -5,
    label: "exchange control",
    val: |g, p, _| {
        let t = oko_steal_target(g, p)?;
        (pval(g, t) >= 5.0).then(|| pval(g, t) - 1.0)
    },
    eff: oko_exchange,
};

/// Dovin, Hand of Control -1: damage to and from their best creature is prevented until your next turn
fn dovin_minus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    if let Some(t) = best_opp_creature(g, p, |_, _| true) {
        let turns = g.player(p).turns as i64;
        g.perm_mut(t).data.set(DataKey::Dovin, Val::List(vec![Val::Player(p), Val::Int(turns)]));
        crate::glog!(
            g,
            "    Dovin: damage to and from {} is prevented until {}'s next turn",
            g.perm(t).name,
            pname(g, p)
        );
    }
    Ok(())
}

static DOVIN: [WalkerAb; 1] = [WalkerAb {
    delta: -1,
    label: "prevent damage to and from a creature",
    val: |g, p, _| {
        let t = best_opp_creature(g, p, |_, _| true)?;
        (pval(g, t) >= 4.0).then(|| pval(g, t) - 2.5)
    },
    eff: dovin_minus,
}];

/// Dovin: opponents' artifact, instant and sorcery spells cost {1} more (common._tax_spell)
fn dovin_tax(g: &Game, src: Src, caster: PlayerId, c: CardId) -> i32 {
    let d = g.db.get(c);
    if caster == g.perm(src).owner {
        return 0;
    }
    (d.types.has(Types::ARTIFACT) || d.instant || d.sorcery) as i32
}

// ================================================================== more single-card clauses
/// rules._freeze: Frost Titan taps a creature or artifact; it doesn't untap during its controller's next untap step
fn freeze(g: &mut Game, src: PermId) -> Res {
    let o = g.perm(src).owner;
    let Some(t) = best_opp_nonland(g, o, |g, m| g.is_creature(m) || card_is(g, m, Types::ARTIFACT)) else {
        return Ok(());
    };
    g.perm_mut(t).tapped = true;
    g.perm_mut(t).data.set(DataKey::Frozen, Val::Int(1));
    crate::glog!(g, "    Frost Titan taps {} (it doesn't untap next turn)", g.perm(t).name);
    Ok(())
}

fn frost(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m == src && trigger_window(g, o, Some(src), "tap a permanent", None)? {
        freeze(g, src)?;
    }
    Ok(())
}

fn frost_atk(g: &mut Game, src: Src, _p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    let o = g.perm(src).owner;
    if atk.contains(&src) && trigger_window(g, o, Some(src), "tap a permanent", None)? {
        freeze(g, src)?;
    }
    Ok(vec![])
}

/// Bloodghast: haste while an opponent has 10 or less life
fn bloodghast(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m == src && g.opps(o).any(|q| g.player(q).life <= 10) {
        g.perm_mut(src).sick = false;
    }
    Ok(())
}

/// Mogg War Marshal: echo is never paid (the sacrifice makes a second Goblin)
fn mwm_echo(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p == g.perm(src).owner && g.perm(src).data.get(DataKey::Echo) != Some(&Val::Str("done")) {
        let ok = trigger_window(g, p, Some(src), "echo: sacrifice it unless you pay", Some(1.0))?;
        g.perm_mut(src).data.set(DataKey::Echo, Val::Str("done"));
        if ok && controls(g, p, src) {
            crate::glog!(g, "    {} doesn't pay echo for Mogg War Marshal", pname(g, p));
            die(g, src, "sac")?;
        }
    }
    Ok(())
}

/// Plaguecrafter: each player sacrifices a creature or planeswalker, or discards a card if they can't
fn plague(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = g.perm(src).owner;
    if !trigger_window(g, o, Some(src), "each player sacrifices a creature or planeswalker", Some(5.0))? {
        return Ok(());
    }
    let alive: Vec<PlayerId> = g.players.iter().filter(|q| q.alive).map(|q| q.id).collect();
    for q in alive {
        let mut cs: Vec<PermId> = g
            .player(q)
            .perms
            .iter()
            .copied()
            .filter(|&x| (g.is_creature(x) || card_is(g, x, Types::PLANESWALKER)) && !g.perm(x).phased)
            .collect();
        if q == o {
            let others: Vec<PermId> = cs.iter().copied().filter(|&x| x != src).collect();
            if !others.is_empty() {
                cs = others;
            }
        }
        if let Some(x) = min_by(&cs, |x| pval(g, x)) {
            die(g, x, "sac")?;
        } else if !g.player(q).hand.is_empty() {
            discard_worst(g, q, 1)?;
        }
    }
    Ok(())
}

/// Liliana's Triumph: each opponent sacrifices a creature; with a Liliana out, each discards too
fn triumph(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    for q in g.opps(p).collect::<Vec<_>>() {
        let cs = live_creatures(g, q);
        if let Some(x) = min_by(&cs, |x| pval(g, x)) {
            die(g, x, "sac")?;
        }
    }
    if g.player(p).perms.iter().any(|&m| g.perm(m).cd.is_some() && name_of(g, m).contains("Liliana")) {
        for q in g.opps(p).collect::<Vec<_>>() {
            if !g.player(q).hand.is_empty() {
                discard_worst(g, q, 1)?;
            }
        }
    }
    Ok("gy")
}

/// Painful Quandary: each opponent's spell: they discard a junk card (or any card when low on life), else lose 5
fn quandary(g: &mut Game, src: Src, caster: PlayerId, _c: CardId) -> Res {
    let o = g.perm(src).owner;
    if caster == o {
        return Ok(());
    }
    let name = format!("{} discards or loses 5 life", pname(g, caster));
    if !trigger_window(g, o, Some(src), &name, None)? {
        return Ok(());
    }
    let junk: Vec<CardId> =
        g.player(caster).hand.iter().copied().filter(|&x| card_worth(g, caster, x, false) < 30.0).collect();
    let life = g.player(caster).life;
    if !junk.is_empty() && life > 10 {
        let x = min_by(&junk, |x| card_worth(g, caster, x, false)).unwrap();
        discard_cards(g, caster, &[x])
    } else if !g.player(caster).hand.is_empty() && life <= 10 {
        discard_worst(g, caster, 1)
    } else {
        lose_life(g, caster, 5, Some(o), "triggers", None)
    }
}

/// Sticky Fingers: draw when the enchanted creature dies
fn sticky_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    let o = g.perm(src).owner;
    if g.perm(src).attached == Some(m) && trigger_window(g, o, Some(src), "draw a card", None)? {
        draw(g, o, 1, false)?;
    }
    Ok(())
}

/// Reckless Fireweaver: each artifact (cast or token) entering under your control deals 1 to each opponent
fn fireweaver(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if g.perm(m).owner == o
        && m != src
        && card_is(g, m, Types::ARTIFACT)
        && trigger_window(g, o, Some(src), "1 damage to each opponent", None)?
    {
        for q in g.opps(o).collect::<Vec<_>>() {
            lose_life(g, q, 1, Some(o), "triggers", None)?;
        }
    }
    Ok(())
}

/// Legion Loyalist: battalion: attackers gain first strike and trample and can't be blocked by tokens
fn loyalist(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if atk.contains(&src)
        && atk.len() >= 3
        && trigger_window(g, p, Some(src), "attackers gain first strike and trample", None)?
    {
        for &m in atk {
            eot_kw(g, m, "first strike");
            eot_kw(g, m, "trample");
        }
        g.player_mut(p).loyalist_turn = Some(g.turn_stamp());
    }
    Ok(vec![])
}

/// Moraug: creatures +1/+0 for each time they attacked this turn
fn moraug(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if p != g.perm(src).owner {
        return Ok(vec![]);
    }
    let st = g.turn_stamp();
    for &m in atk {
        let n = match g.perm(m).data.get(DataKey::Attacks) {
            Some(Val::List(v)) if v.first() == Some(&Val::Stamp(st)) => v.get(1).map_or(0, Val::int) + 1,
            _ => 1,
        };
        g.perm_mut(m).data.set(DataKey::Attacks, Val::List(vec![Val::Stamp(st), Val::Int(n)]));
        eot(g, m, 1, 0);
    }
    Ok(vec![])
}

/// Everflowing Chalice: multikicker {2} (as many kicks as the mana allows, up to three); taps for one per kick
fn chalice(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || !castable(g, p, c, "hand") {
        return Ok(vec![]);
    }
    let k = (total_mana(g, p, false) / 2).min(3);
    if k < 1 {
        return Ok(vec![]);
    }
    let u = if g.player(p).turns <= 6 { 3.0 + k as f64 } else { 1.0 + k as f64 };
    Ok(vec![Opt {
        utility: u,
        label: format!("Everflowing Chalice x{k}"),
        act: Some(Action::Plan { f: chalice_go, arg: pack(c.0 as u32, k) }),
    }])
}

fn chalice_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (c, k) = unpack(arg);
    let c = CardId(c as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 2 * k, "", false) {
        return Ok(false);
    }
    remove_card(&mut g.player_mut(p).hand, c);
    pay(g, p, 2 * k, "", false)?;
    crate::glog!(g, "  {} casts Everflowing Chalice kicked {} times", pname(g, p), k);
    on_cast(g, p, c)?;
    if !counter_window(g, p, c, 2.0, vec![])? {
        g.player_mut(p).gy.push(c);
        return Ok(true);
    }
    let m = enter(g, p, c, Enter { was_cast: true, ..Enter::default() })?;
    g.perm_mut(m).data.set(DataKey::Kicks, Val::Int(k as i64));
    Ok(true)
}

fn chalice_mana(g: &Game, _p: PlayerId, m: PermId) -> u32 {
    g.perm(m).data.int(DataKey::Kicks).max(0) as u32
}

fn no_prio(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    0
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, "Jaspera Sentinel")?;
    c.dyn_mana_perm = Some(drum);
    c.on_tap_perm = Some(drum_tap);
    r.card(db, "Lotus Petal")?.on_tap_perm = Some(petal_sac);
    let c = r.card(db, "Council's Judgment")?;
    c.resolve = Some(judgment);
    c.prio = Some(judgment_prio);
    let c = r.card(db, "Fact or Fiction")?;
    c.resolve = Some(fof);
    c.prio = Some(fof_prio);
    r.card(db, "Boros Charm")?.hand_options = Some(boros_charm);
    r.card(db, "Nature's Claim")?.resolve = Some(natures_claim);
    let c = r.card(db, "Tithe Taker")?;
    c.self_dies = Some(tithe_afterlife);
    at_once(c, Event::SelfDies, false);
    let c = r.card(db, "Brimaz, King of Oreskos")?;
    c.blocks = Some(brimaz_block);
    at_once(c, Event::Blocks, false);
    r.card(db, "Allosaurus Shepherd")?.uncounterable = Some(shepherd);
    r.card(db, "Ezuri, Renegade Leader")?.options = Some(ezuri_none);
    let c = r.card(db, "Mirkwood Bats")?;
    c.token_created = Some(bats_make);
    at_once(c, Event::TokenCreated, false);
    let c = r.card(db, "Bloodchief's Thirst")?;
    c.resolve = Some(thirst);
    c.self_cast = Some(thirst_cast);
    // mana sources
    r.card(db, "Mox Amber")?.dyn_mana_perm = Some(moxamber);
    r.card(db, "Mox Opal")?.dyn_mana_perm = Some(moxopal);
    let c = r.card(db, "Springleaf Drum")?;
    c.dyn_mana_perm = Some(drum);
    c.on_tap_perm = Some(drum_tap);
    r.card(db, "Sanctum Weaver")?.dyn_mana_perm = Some(sanctum_weaver);
    let c = r.card(db, "Goldspan Dragon")?;
    c.attack = Some(goldspan_atk);
    c.treasure_bonus = Some(goldspan_bonus);
    let c = r.card(db, "Mystic Remora")?;
    c.cast = Some(remora);
    c.upkeep = Some(remora_age);
    // Dovin
    super::common::walker(r, db, "Dovin, Hand of Control", &DOVIN)?;
    r.card(db, "Dovin, Hand of Control")?.cost = Some(dovin_tax);
    // single-card clauses
    let c = r.card(db, "Frost Titan")?;
    c.etb = Some(frost);
    c.attack = Some(frost_atk);
    let c = r.card(db, "Bloodghast")?;
    c.etb = Some(bloodghast);
    at_once(c, Event::Etb, true);
    r.card(db, "Mogg War Marshal")?.upkeep = Some(mwm_echo);
    r.card(db, "Plaguecrafter")?.etb = Some(plague);
    r.card(db, "Liliana's Triumph")?.resolve = Some(triumph);
    r.card(db, "Painful Quandary")?.cast = Some(quandary);
    r.card(db, "Sticky Fingers")?.dies = Some(sticky_dies);
    r.card(db, "Reckless Fireweaver")?.etb = Some(fireweaver);
    r.card(db, "Legion Loyalist")?.attack = Some(loyalist);
    let c = r.card(db, "Moraug, Fury of Akoum")?;
    c.attack = Some(moraug);
    at_once(c, Event::Attack, true);
    let c = r.card(db, "Everflowing Chalice")?;
    c.hand_options = Some(chalice);
    c.dyn_mana_perm = Some(chalice_mana);
    c.prio = Some(no_prio);
    Ok(())
}
