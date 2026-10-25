//! Tier 6 batch `legends2`: Sisay's legends and interaction (Magda, The Queen of Dale, Tataru Taru, Teferi, Who Slows
//! the Sunset, Spider-Woman, Ertai Resurrected, Legolas's Quick Reflexes, Colossal Skyturtle, Wan Shi Tong). Cards only:
//! no engine file is touched; what needs the engine is listed in rust/agents/engine-needs-legends2.md.

use super::common::{WalkerAb, walker};
use super::partials::{at_once, best_opp_creature, best_opp_nonland, eot_kw, first_max, on};
use super::t5::is_artifact;
use crate::cards::{CardDb, Colors, Types};
use crate::engine::cast::discard_worst;
use crate::engine::cast::{castable, on_cast};
use crate::engine::life::gain;
use crate::engine::mana::{can_pay, cost_of, pay, total_mana};
use crate::engine::removal::apply_removal;
use crate::engine::stack::{ability_window, ability_window_card, counter_window, trigger_window};
use crate::engine::tutors::{card_worth, shuffle_library};
use crate::engine::values::{card_name, once_per_turn, pval, threat, untargetable};
use crate::engine::zones::{
    Enter, Tokens, add_treasure, agent_for, agent_take, draw, enter, make_tokens, max_by, min_by, searchable,
};
use crate::flow::Res;
use crate::hooks::{Action, Call, Event, Opt, Registry, Src};
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::state::StackKind;
use crate::state::{Ctx, Game};
use crate::sym::{Sym, intern};
use crate::tag::Tag;

fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

fn ability(utility: f64, label: &str, src: PermId, f: crate::hooks::AbilityFn, arg: i64) -> Opt {
    Opt { utility, label: label.into(), act: Some(Action::Ability { src, f, arg }) }
}

fn remove_card(v: &mut Vec<CardId>, c: CardId) {
    if let Some(i) = v.iter().position(|&x| x == c) {
        v.remove(i);
    }
}

// ================================================================== Magda, Brazen Outlaw
const MAGDA: &str = "Magda, Brazen Outlaw";

/// a Dwarf creature permanent (a changeling counts)
fn is_dwarf(g: &Game, m: PermId) -> bool {
    g.is_creature(m)
        && g.perm(m).cd.is_some_and(|c| {
            let d = g.db.get(c);
            d.has_subtype("dwarf") || d.has_kw("changeling")
        })
}

/// the Magdas p controls (not phased out), other than `except`
fn magdas(g: &Game, p: PlayerId, except: Option<PermId>) -> usize {
    g.player(p).perms.iter().filter(|&&y| Some(y) != except && live(g, y) && card_name(g, y) == Some(MAGDA)).count()
}

/// other Dwarves you control get +1/+0 (one per Magda), read from `creature_pt` while `g.selfpt` is on
fn magda_pt(g: &Game, m: PermId) -> (i32, i32) {
    if !is_dwarf(g, m) {
        return (0, 0);
    }
    (magdas(g, owner(g, m), Some(m)) as i32, 0)
}

fn magda_as(g: &mut Game, _p: PlayerId, _m: PermId) -> Res {
    g.selfpt = true;
    Ok(())
}

/// whenever a Dwarf you control becomes tapped for mana (the mana-tapped event): a Treasure
fn magda_mana(g: &mut Game, src: Src, p: PlayerId, m: PermId, _n: u32) -> Res {
    if live(g, src) && owner(g, src) == p && owner(g, m) == p && is_dwarf(g, m) {
        add_treasure(g, p, 1)?;
        crate::glog!(g, "    {} makes a Treasure ({} tapped)", g.perm(src).name, g.perm(m).name);
    }
    Ok(())
}

/// whenever a Dwarf you control becomes tapped by attacking: a Treasure each (one trigger window for all of them)
fn magda_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if !live(g, src) || owner(g, src) != p {
        return Ok(vec![]);
    }
    let k = atk.iter().filter(|&&m| owner(g, m) == p && g.perm(m).tapped && is_dwarf(g, m)).count();
    if k > 0 && trigger_window(g, p, Some(src), &format!("create {k} Treasure"), None)? {
        add_treasure(g, p, k as i32)?;
    }
    Ok(vec![])
}

/// the artifact or Dragon card Magda fetches: the best by worth
fn magda_pick(g: &Game, p: PlayerId) -> Option<CardId> {
    let ok: Vec<CardId> = searchable(g, p)
        .into_iter()
        .filter(|&c| {
            let d = g.db.get(c);
            d.types.has(Types::ARTIFACT) || d.has_subtype("dragon")
        })
        .collect();
    first_max(&ok, |c| card_worth(g, p, c, false))
}

/// Sacrifice five Treasures: search for an artifact or Dragon card, put it onto the battlefield, shuffle
fn magda_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || !live(g, src) || owner(g, src) != p || g.player(p).treasures < 5 {
        return Ok(vec![]);
    }
    let Some(t) = magda_pick(g, p) else { return Ok(vec![]) };
    let u = 8.0f64.min(2.0 + 0.5 * g.db.get(t).cmc as f64);
    Ok(vec![ability(u, &format!("Magda: {}", g.db.get(t).name), src, magda_go, t.0 as i64)])
}

fn magda_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let t = CardId(arg as u16);
    if !on(g, p, src) || g.player(p).treasures < 5 || !g.player(p).library.contains(&t) {
        return Ok(false);
    }
    // the cost: all five at once, then their sacrifice triggers (a trigger between them could spend a Treasure and
    // leave fewer than five to pay with)
    for _ in 0..5 {
        g.player_mut(p).sac_treasure();
    }
    for _ in 0..5 {
        if !g.hooks.is_empty() {
            crate::engine::hooks::fire_trigger(
                g,
                Event::Sacrifice,
                Call::Sacrifice { p, what: crate::hooks::Sacrificed::Token("Treasure") },
            )?;
        }
    }
    crate::glog!(g, "  {} sacrifices five Treasures: Magda searches for {}", g.player(p).name, g.db.get(t).name);
    if !ability_window(g, p, Some(src), &format!("search for {}", g.db.get(t).name), None, None)?
        || !g.player(p).library.contains(&t)
    {
        return Ok(true);
    }
    remove_card(&mut g.player_mut(p).library, t);
    shuffle_library(g, p);
    match agent_for(g, p) {
        Some(a) => agent_take(g, a, p, t),
        None => {
            enter(g, p, t, Enter::default())?;
        }
    }
    Ok(true)
}

// ================================================================== The Queen of Dale
/// Recruit: draw a card, then discard a card (the AI's usual discard); if the discarded card is not a land, create a
/// 1/1 white Human Soldier creature token
pub fn recruit(g: &mut Game, p: PlayerId) -> Res {
    draw(g, p, 1, false)?;
    let before = g.player(p).hand.clone();
    discard_worst(g, p, 1)?;
    let gone: Vec<CardId> = before.into_iter().filter(|c| !g.player(p).hand.contains(c)).collect();
    if gone.first().is_some_and(|&c| !g.db.get(c).land) {
        let spec = Tokens {
            color: Some(Colors::from_letters("W")),
            types: vec!["human".into(), "soldier".into()],
            ..Tokens::new(1, 1)
        };
        make_tokens(g, p, spec)?;
    }
    Ok(())
}

/// **The Queen of Dale** ({1}{W}, 2/1): whenever an opponent casts their first noncreature spell each turn, you recruit
fn queen_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let o = owner(g, src);
    if !live(g, src) || caster == o || g.db.get(c).creature {
        return Ok(());
    }
    let st = g.turn_stamp();
    let earlier = match &g.player(caster).turn_casts {
        Some((s, cs)) if *s == st => cs.iter().filter(|&&x| x != c && !g.db.get(x).creature).count(),
        _ => 0,
    };
    if earlier == 0 && trigger_window(g, o, Some(src), "recruit", None)? {
        recruit(g, o)?;
    }
    Ok(())
}

// ================================================================== Tataru Taru
/// **Tataru Taru** ({1}{W}, 0/3 Dwarf): when it enters, you draw a card (the engine's `draw` tag on the card does
/// that) and target opponent may draw a card (the AI opponent with the least threat draws, if it has a card to draw).
/// Scions' Secretary: whenever an opponent draws a card, if it isn't that player's turn, create a tapped Treasure
/// token (once each turn; `make_artifact_tokens_tapped`, it untaps in its controller's untap step).
fn tataru_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src || !live(g, src) {
        return Ok(());
    }
    let o = owner(g, src);
    let opps: Vec<PlayerId> = g.opps(o).filter(|&q| !g.player(q).library.is_empty()).collect();
    let Some(t) = min_by(&opps, |q| threat(g, o, q)) else { return Ok(()) };
    if trigger_window(g, o, Some(src), "target opponent may draw a card", None)? {
        draw(g, t, 1, false)?;
    }
    Ok(())
}

fn tataru_draw(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let o = owner(g, src);
    if !live(g, src) || p == o || g.active == Some(p) {
        return Ok(());
    }
    if !once_per_turn(g, o, intern(&format!("scions_{}", src.0))) {
        return Ok(());
    }
    if trigger_window(g, o, Some(src), "create a tapped Treasure", None)? {
        crate::engine::zones::make_artifact_tokens_tapped(g, o, "Treasure", 1, true)?;
    }
    Ok(())
}

// ================================================================== Teferi, Who Slows the Sunset
/// the permanent +1 aims at in one category: a tapped one of ours to untap, else an untapped one of theirs to tap
enum Aim {
    Untap(PermId),
    Tap(PermId),
}

fn aim_perm(
    g: &Game,
    p: PlayerId,
    pred: impl Fn(&Game, PermId) -> bool,
    worth: impl Fn(&Game, PermId) -> f64,
) -> Option<(Aim, f64)> {
    let own: Vec<PermId> =
        g.player(p).perms.iter().copied().filter(|&m| live(g, m) && g.perm(m).tapped && pred(g, m)).collect();
    let opp: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied().collect::<Vec<_>>())
        .filter(|&m| live(g, m) && !g.perm(m).tapped && pred(g, m) && !untargetable(g, m))
        .collect();
    let a = max_by(&own, |m| worth(g, m)).map(|m| (Aim::Untap(m), worth(g, m)));
    let b = max_by(&opp, |m| worth(g, m)).map(|m| (Aim::Tap(m), worth(g, m)));
    match (a, b) {
        (Some(x), Some(y)) => Some(if y.1 > x.1 { y } else { x }),
        (x, y) => x.or(y),
    }
}

fn rock_worth(g: &Game, m: PermId) -> f64 {
    let rock = g.perm(m).cd.is_some_and(|c| g.db.get(c).tags.str(Tag::Rock).is_some());
    if rock { 2.0 } else { 0.3 }
}

/// the three choices of +1 (an artifact, a creature, a land) with what each is worth to the AI
fn teferi_aims(g: &Game, p: PlayerId) -> (Option<(Aim, f64)>, Option<(Aim, f64)>, Option<(LandId, bool)>) {
    let art = aim_perm(g, p, |g, m| is_artifact(g, m), rock_worth);
    let cre = aim_perm(g, p, |g, m| g.is_creature(m), |g, m| pval(g, m) / 3.0);
    let own = g.player(p).lands.iter().copied().find(|&l| g.land(l).on_bf && g.land(l).tapped);
    let opp = g.opps(p).flat_map(|q| g.player(q).lands.clone()).find(|&l| g.land(l).on_bf && !g.land(l).tapped);
    let land = own.map(|l| (l, true)).or(opp.map(|l| (l, false)));
    (art, cre, land)
}

fn teferi_plus_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let (a, c, l) = teferi_aims(g, p);
    Some(1.0 + a.map_or(0.0, |x| x.1) + c.map_or(0.0, |x| x.1) + l.map_or(0.0, |(_, own)| if own { 1.0 } else { 0.3 }))
}

/// +1: choose up to one target artifact, creature and land; untap the chosen ones you control, tap the chosen ones you
/// don't; you gain 2 life
fn teferi_plus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let (a, c, l) = teferi_aims(g, p);
    for aim in [a, c].into_iter().flatten() {
        match aim.0 {
            Aim::Untap(m) => g.perm_mut(m).tapped = false,
            Aim::Tap(m) => g.perm_mut(m).tapped = true,
        }
    }
    if let Some((l, own)) = l {
        g.land_mut(l).tapped = !own;
    }
    gain(g, p, 2)
}

/// -2: look at the top three cards, one into your hand, the rest on the bottom (the one the deck wants most)
fn teferi_minus_val(g: &Game, p: PlayerId, src: PermId) -> Option<f64> {
    (g.perm(src).loyalty.unwrap_or(0) >= 4 && g.player(p).library.len() >= 3).then_some(2.2)
}

fn teferi_minus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let top = crate::ai::topdeck::look(g, p, 3);
    let w: Vec<f64> = top.iter().map(|&x| -crate::ai::topdeck::desire(g, p, x)).collect();
    let mut idx: Vec<usize> = (0..top.len()).collect();
    idx.sort_by(|&a, &b| w[a].partial_cmp(&w[b]).unwrap_or(std::cmp::Ordering::Equal));
    let top: Vec<CardId> = idx.into_iter().map(|i| top[i]).collect();
    let pl = g.player_mut(p);
    if let Some(&x) = top.first() {
        pl.hand.push(x);
        pl.seen.insert(x);
    }
    pl.library.splice(0..0, top.iter().skip(1).copied());
    Ok(())
}

/// -7: the emblem "untap all permanents you control during each opponent's untap step" and "you draw a card during
/// each opponent's draw step". The emblem is recorded but the engine does not act on it (engine-needs-legends2.md),
/// so the AI does not spend 7 loyalty on it (value None).
const TEFERI_EMBLEM: &str = "teferi_sunset";

fn teferi_ult(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    g.player_mut(p).emblems.push(TEFERI_EMBLEM);
    Ok(())
}

static TEFERI: [WalkerAb; 3] = [
    WalkerAb {
        delta: 1,
        label: "untap / tap an artifact, creature and land; gain 2",
        val: teferi_plus_val,
        eff: teferi_plus,
    },
    WalkerAb { delta: -2, label: "look at three, take one", val: teferi_minus_val, eff: teferi_minus },
    WalkerAb { delta: -7, label: "emblem: untap and draw in their steps", val: |_, _, _| None, eff: teferi_ult },
];

// ================================================================== Spider-Woman, Stunning Savior
const SPIDER_WOMAN: &str = "Spider-Woman, Stunning Savior";

/// Venom Blast: artifacts and creatures your opponents control enter tapped (the `enters_tapped` replacement; a
/// Treasure token comes in tapped too).
fn spider_tapped(g: &Game, src: Src, p: PlayerId, m: Option<PermId>) -> bool {
    live(g, src) && p != owner(g, src) && m.is_none_or(|m| m != src && (g.is_creature(m) || is_artifact(g, m)))
}

// ================================================================== Ertai Resurrected
const ERTAI: &str = "Ertai Resurrected";

/// the spell or ability this Ertai countered as it came in, still on the stack
fn ertai_countered(g: &Game, src: PermId) -> Option<PlayerId> {
    let c = g.perm(src).cd?;
    g.stack.iter().find(|it| it.countered && it.countered_by == Some(c)).map(|it| it.controller)
}

/// the creature or planeswalker the destroy mode picks: the opponents' best, when it is worth a card to them
fn ertai_target(g: &Game, p: PlayerId) -> Option<PermId> {
    let t = best_opp_nonland(g, p, |g, m| {
        g.is_creature(m) || g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(Types::PLANESWALKER))
    })?;
    (pval(g, t) >= 3.0).then_some(t)
}

/// **Ertai Resurrected** ({2}{U}{B}, flash 3/2): when it enters, choose up to one: counter target spell, activated
/// ability or triggered ability, its controller draws a card; or destroy another target creature or planeswalker, its
/// controller draws a card. The counter-a-spell mode is the engine's (the card's `ctr=any` tag makes it a flash
/// counter like Mystic Snake, cast in response; the countered spell is marked `countered_by` Ertai), so on entering
/// from that response this hook gives the spell's controller their card. Otherwise it takes the destroy mode on the
/// opponents' best creature or planeswalker when that is worth 3 or more. Countering an ability is `ertai_answer`.
fn ertai_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src || !live(g, src) {
        return Ok(());
    }
    let o = owner(g, src);
    if let Some(q) = ertai_countered(g, src) {
        crate::glog!(g, "    Ertai Resurrected: {} draws a card", g.player(q).name);
        return draw(g, q, 1, false);
    }
    let Some(t) = ertai_target(g, o) else { return Ok(()) };
    if trigger_window(g, o, Some(src), &format!("destroy {}", g.perm(t).name), Some(5.0))? && live(g, t) {
        let q = owner(g, t);
        apply_removal(g, Some(o), t, "destroy", None)?;
        draw(g, q, 1, false)?;
    }
    Ok(())
}

/// The AI q with Ertai in hand answers an opponent's important ability or trigger on the stack (item `item`): flash
/// in Ertai ({2}{U}{B}) and counter it; the ability's controller draws a card. Written after `partials::answer_ability`
/// (Tishana's Tidebinder); `cardcode::answer_ability` calls it.
/// Returns true if Ertai was cast.
pub fn ertai_answer(g: &mut Game, q: PlayerId, item: u32) -> Res<bool> {
    let Some(it) = g.stack.iter().find(|x| x.id == item).cloned() else { return Ok(false) };
    if it.kind == StackKind::Spell || it.controller == q || it.countered {
        return Ok(false);
    }
    let Some(c) = g.player(q).hand.iter().copied().find(|&x| &*g.db.get(x).name == ERTAI) else { return Ok(false) };
    let (gn, pips) = cost_of(g, q, c);
    if g.humans.get(q).is_some() || !castable(g, q, c, "hand") || !can_pay(g, q, gn, &pips, false) {
        return Ok(false);
    }
    if !pay(g, q, gn, &pips, false)? {
        return Ok(false);
    }
    remove_card(&mut g.player_mut(q).hand, c);
    crate::glog!(g, "    {} flashes in Ertai Resurrected: {} is countered", g.player(q).name, it.name);
    on_cast(g, q, c)?;
    if !counter_window(g, q, c, 4.0, vec![])? {
        g.player_mut(q).gy.push(c);
        return Ok(true);
    }
    g.last_counter = Some(c);
    if let Some(owner) = g.stack.iter().find(|x| x.id == item).map(|x| x.controller) {
        crate::t6stats::counter_hit(g, q, c, owner);
    }
    if let Some(x) = g.stack.iter_mut().find(|x| x.id == item) {
        x.countered = true;
        x.countered_by = Some(c);
    }
    enter(g, q, c, Enter { was_cast: true, ..Enter::default() })?; // the enter hook draws for the ability's controller
    Ok(true)
}

// ================================================================== Legolas's Quick Reflexes
const LEGOLAS: &str = "Legolas's Quick Reflexes";

/// the creature Legolas's Quick Reflexes is aimed at when nothing was chosen: p's own, a tapped one first, the most
/// valuable
fn legolas_pick(g: &Game, p: PlayerId) -> Option<PermId> {
    let cs: Vec<PermId> = g.player(p).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m)).collect();
    let tapped: Vec<PermId> = cs.iter().copied().filter(|&m| g.perm(m).tapped).collect();
    max_by(if tapped.is_empty() { &cs } else { &tapped }, |m| pval(g, m))
}

/// the effect: untap target creature; until end of turn it gains reach and hexproof, and "whenever this creature
/// becomes tapped, it deals damage equal to its power to up to one target creature" (`t3_marwyn::on_tapped`). (Split
/// second is the engine's.)
fn legolas_effect(g: &mut Game, t: PermId) {
    g.perm_mut(t).tapped = false;
    eot_kw(g, t, "reach");
    eot_kw(g, t, "hexproof");
    crate::impls::t3_marwyn::legolas_mark(g, t); // "whenever this creature becomes tapped": t3_marwyn::on_tapped
    crate::glog!(g, "    {} untaps, gains reach and hexproof (Legolas's Quick Reflexes)", g.perm(t).name);
}

/// when it resolves (cast by the engine's usual path, or by a person): the chosen creature, else the best of ours
fn legolas_resolve(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let t = ctx.target.filter(|&m| live(g, m) && g.is_creature(m)).or_else(|| legolas_pick(g, p));
    if let Some(t) = t {
        legolas_effect(g, t);
    }
    Ok("gy")
}

fn zero_prio(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    0
}

/// `who` casts Legolas's Quick Reflexes from the hand on creature `t` as a response (saving it from removal, or
/// untapping it): {G}, countered like any spell. True if the card was cast. For the engine's response code
/// (engine-needs-legends2.md).
pub fn legolas_cast(g: &mut Game, who: PlayerId, c: CardId, t: PermId) -> Res<bool> {
    if !g.player(who).hand.contains(&c) || !castable(g, who, c, "hand") || !live(g, t) {
        return Ok(false);
    }
    let (gn, pips) = cost_of(g, who, c);
    if !can_pay(g, who, gn, &pips, false) || !pay(g, who, gn, &pips, false)? {
        return Ok(false);
    }
    remove_card(&mut g.player_mut(who).hand, c);
    crate::glog!(g, "  {} casts {} on {}", g.player(who).name, LEGOLAS, g.perm(t).name);
    on_cast(g, who, c)?;
    if counter_window(g, who, c, 5.0, vec![])? && live(g, t) {
        legolas_effect(g, t);
    }
    g.player_mut(who).gy.push(c);
    Ok(true)
}

// ================================================================== Colossal Skyturtle
const SKYTURTLE: &str = "Colossal Skyturtle";

fn plan(utility: f64, label: &str, f: crate::hooks::PlanFn, arg: i64) -> Opt {
    Opt { utility, label: label.into(), act: Some(Action::Plan { f, arg }) }
}

/// the card the regrow channel returns: the most wanted nonland card in the graveyard worth keeping (40 or more)
fn turtle_regrow_pick(g: &Game, p: PlayerId, turtle: CardId) -> Option<CardId> {
    let cs: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| c != turtle && !g.db.get(c).land).collect();
    let best = first_max(&cs, |c| card_worth(g, p, c, false))?;
    (card_worth(g, p, best, false) >= 40.0).then_some(best)
}

/// **Colossal Skyturtle** ({4}{G}{G}{U}, flying 6/5, ward {2}; the body is the card's data). Channel from the hand:
/// {2}{G}, discard it: return target card from your graveyard to your hand (when the turtle itself can't be cast and a
/// card worth 40+ is there); {1}{U}, discard it: return target creature to its owner's hand (the opponents' best
/// creature when it is worth 5+). Both in a main phase.
fn turtle_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let mut o = vec![];
    if post.is_none() {
        return Ok(o);
    }
    if can_pay(g, p, 2, "G", false)
        && !can_pay(g, p, 4, "GGU", false)
        && let Some(t) = turtle_regrow_pick(g, p, c)
    {
        let u = (card_worth(g, p, t, false) / 10.0 - 1.5).min(7.0);
        o.push(plan(u, &format!("channel Colossal Skyturtle: {}", g.db.get(t).name), turtle_regrow_go, t.0 as i64));
    }
    if can_pay(g, p, 1, "U", false)
        && let Some(t) = best_opp_creature(g, p, |_, _| true)
        && pval(g, t) >= 5.0
    {
        o.push(plan(pval(g, t) - 2.0, "channel Colossal Skyturtle: bounce", turtle_bounce_go, t.0 as i64));
    }
    Ok(o)
}

/// pay a channel cost and discard the turtle; false when it can't be done
fn turtle_channel(g: &mut Game, p: PlayerId, c: CardId, generic: u32, pips: &str) -> Res<bool> {
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, generic, pips, false) || !pay(g, p, generic, pips, false)? {
        return Ok(false);
    }
    remove_card(&mut g.player_mut(p).hand, c);
    g.player_mut(p).gy.push(c);
    Ok(true)
}

pub fn turtle_regrow_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let t = CardId(arg as u16);
    let Some(c) = g.db.id(SKYTURTLE) else { return Ok(false) };
    if !g.player(p).gy.contains(&t) || !turtle_channel(g, p, c, 2, "G")? {
        return Ok(false);
    }
    crate::glog!(g, "  {} channels Colossal Skyturtle: returns {}", g.player(p).name, g.db.get(t).name);
    if ability_window_card(g, p, c, "return a card from the graveyard", None, None)? && g.player(p).gy.contains(&t) {
        remove_card(&mut g.player_mut(p).gy, t);
        g.player_mut(p).hand.push(t);
    }
    Ok(true)
}

fn turtle_bounce_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let t = PermId(arg as u32);
    let Some(c) = g.db.id(SKYTURTLE) else { return Ok(false) };
    if !live(g, t) || !g.is_creature(t) || !turtle_channel(g, p, c, 1, "U")? {
        return Ok(false);
    }
    crate::glog!(g, "  {} channels Colossal Skyturtle: bounces {}", g.player(p).name, g.perm(t).name);
    if ability_window_card(g, p, c, "return target creature to its owner's hand", None, Some(t))? && live(g, t) {
        apply_removal(g, Some(p), t, "bounce", None)?;
    }
    Ok(true)
}

// ================================================================== Wan Shi Tong, Librarian
const WAN_SHI_TONG: &str = "Wan Shi Tong, Librarian";

/// The most X the AI can pay for Wan Shi Tong ({X}{U}{U}) with the mana it has
fn wan_x(g: &Game, p: PlayerId) -> i32 {
    let mut x = total_mana(g, p, false) as i32 - 2;
    while x >= 1 && !can_pay(g, p, x as u32, "UU", false) {
        x -= 1;
    }
    x.max(0)
}

fn wan_prio(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    0
}

/// **Wan Shi Tong, Librarian** ({X}{U}{U}, flash, flying, vigilance, 1/1): when it enters, X +1/+1 counters, then draw
/// half X rounded down. The engine has no X for creature spells, and a plain cast would be X = 0, so the AI never
/// casts it that way (zero `prio`); at the end of the turn before its own (the flash window) it casts it for all its
/// spare mana, X = mana - 2, when X is 2 or more (`wan_go`, written after Hydroid Krasis).
fn wan_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_some() || !castable(g, p, c, "hand") {
        return Ok(vec![]);
    }
    let x = wan_x(g, p);
    if x < 2 {
        return Ok(vec![]);
    }
    Ok(vec![plan(1.5 + 0.5 * x as f64, &format!("Wan Shi Tong X={x}"), wan_go, c.0 as i64)])
}

pub fn wan_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let x = wan_x(g, p);
    if !g.player(p).hand.contains(&c) || x < 1 || !pay(g, p, x as u32, "UU", false)? {
        return Ok(false);
    }
    remove_card(&mut g.player_mut(p).hand, c);
    let pl = g.player_mut(p);
    pl.cast_names.insert(c);
    pl.spells_this_turn += 1;
    pl.stat("spells_cast", 1);
    g.last_x = x;
    crate::glog!(g, "  {} casts Wan Shi Tong for X={}", g.player(p).name, x);
    on_cast(g, p, c)?;
    if !counter_window(g, p, c, 5.0, vec![])? {
        g.player_mut(p).gy.push(c);
        return Ok(true);
    }
    let m = enter(g, p, c, Enter { was_cast: true, sick: false, ..Enter::default() })?;
    crate::cardcode::put_plus(g, m, x);
    draw(g, p, (x / 2) as u32, false)?;
    Ok(true)
}

/// Whenever an opponent searches their library, put a +1/+1 counter on Wan Shi Tong and draw a card. The engine has no
/// "searches" event: every search by `searcher` has to call this (engine-needs-legends2.md).
pub fn wan_shi_tong_searched(g: &mut Game, searcher: PlayerId) -> Res {
    let tongs: Vec<PermId> = g
        .opps(searcher)
        .flat_map(|q| g.player(q).perms.clone())
        .filter(|&m| live(g, m) && card_name(g, m) == Some(WAN_SHI_TONG))
        .collect();
    for m in tongs {
        let o = owner(g, m);
        if live(g, m) && trigger_window(g, o, Some(m), "+1/+1 counter and draw a card", None)? {
            crate::cardcode::put_plus(g, m, 1);
            draw(g, o, 1, false)?;
        }
    }
    Ok(())
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, "The Queen of Dale")?;
    c.cast = Some(queen_cast);
    at_once(c, Event::Cast, false);
    let c = r.card(db, "Tataru Taru")?;
    c.etb = Some(tataru_etb);
    at_once(c, Event::Etb, false);
    c.draw = Some(tataru_draw);
    at_once(c, Event::Draw, false);
    walker(r, db, "Teferi, Who Slows the Sunset", &TEFERI)?;
    let c = r.card(db, WAN_SHI_TONG)?;
    c.hand_options = Some(wan_options);
    c.prio = Some(wan_prio); // never cast with X = 0
    r.card(db, SKYTURTLE)?.hand_options = Some(turtle_options);
    let c = r.card(db, LEGOLAS)?;
    c.resolve = Some(legolas_resolve);
    c.prio = Some(zero_prio); // the AI holds it: only a response uses it (legolas_cast)
    let c = r.card(db, ERTAI)?;
    c.etb = Some(ertai_etb);
    at_once(c, Event::Etb, true);
    let c = r.card(db, SPIDER_WOMAN)?;
    c.enters_tapped = Some(spider_tapped);
    r.creature_pt.push(magda_pt);
    let c = r.card(db, MAGDA)?;
    c.as_enters = Some(magda_as);
    c.mana_tapped = Some(magda_mana);
    c.attack = Some(magda_attack);
    at_once(c, Event::Attack, false);
    c.options = Some(magda_options);
    Ok(())
}
