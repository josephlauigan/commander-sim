//! Python's `cards/impl/alela.py`: Alela, Artful Provocateur (Avery's Esper Faerie Flyers deck, key `alela`).
//!
//! - Alela: flying, deathtouch, lifelink; other creatures you control with flying get +1/+0 (a compiled anthem);
//!   whenever you cast an artifact or enchantment spell, create a 1/1 blue Faerie creature token with flying.
//! - The AI: the outside decks' generic priorities (it shares their generic plays), plus the Faerie each artifact or
//!   enchantment makes while Alela is out (more with Wispdrinker Vampire draining and Tetsuko Umezawa unblocking them).
//! - The 99's cards that need code: the Jace token (empower Jace: Keeper of the Quiet Hour, Plan for All Outcomes,
//!   Fatehold Charm), Opposition, Karmic Justice, Static Prison and Aether Hub (energy), Spite // Malice, Muddle the
//!   Mixture's transmute, Ray of Command, the flier payoffs (Plumecreed Mentor, Pileated Provisioner, Jackdaw Savior,
//!   Stirring Hopesinger, Desperate Futurescribe), Malcator, Emry, Airlift Chaplain, Reconstructed Thopter, Strixhaven
//!   Skycoach, Helping Hand, Daydream, Feather of Flight, Ajani Fells the Godsire and Venser, the Sojourner (Multiply
//!   by Zero, Prophesied End, Sphinx's Approach, Page and Initiates of the Ebon Hand are tags: nothing to port).
//!
//! The person's choices (practice mode) are `HUMAN(phase 9)`. Daydream, Venser and Ray of Command use the stand-ins
//! for t2.blink / blink_value and marchesa.best_steal / steal in `galadriel.rs`.

use super::common::{
    AURA, AuraSpec, WalkerAb, allowed, aura, best_opp_creature, best_opp_nonland, oring_exile, oring_return,
    round_stamp, uses, walker,
};
use super::galadriel::{best_steal, blink, blink_value, steal};
use super::partials::at_once;
use crate::ai::decks::protect_response;
use crate::cards::{CardDb, Colors, Types};
use crate::engine::cast::{cast_card, castable, on_cast};
use crate::engine::combat::has_haste;
use crate::engine::mana::{can_pay, cost_of, land_colors_now, pay};
use crate::engine::removal::{apply_removal, legal_targets};
use crate::engine::stack::{ability_window, ability_window_card, counter_window, trigger_window};
use crate::engine::tutors::{card_worth, shuffle_library};
use crate::engine::values::{card_name, colors_of, epow, pval, untargetable};
use crate::engine::zones::{Enter, Tokens, Zone, die, draw, enter, land_to_hand, leave, make_tokens, max_by, mill};
use crate::engine::zones::{searchable, to_zone_card};
use crate::flow::Res;
use crate::hooks::{Action, Event, Opt, Registry, Src};
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::state::{Ctx, DataKey, Game, PermData, Val};
use crate::sym::Sym;
use crate::tag::Tag;

pub const ALELA: &str = "Alela, Artful Provocateur";
pub const JACE: &str = "Jace Token";

// ------------------------------------------------------------------ helpers
/// the first of xs with the highest key (Python's `max(xs, key=..)` for tuple keys)
fn first_max<T: Copy, K: PartialOrd>(xs: &[T], key: impl Fn(T) -> K) -> Option<T> {
    let mut best: Option<(T, K)> = None;
    for &x in xs {
        let k = key(x);
        if best.as_ref().is_none_or(|b| k > b.1) {
            best = Some((x, k));
        }
    }
    best.map(|b| b.0)
}

/// a stable ascending sort by a tuple key (Python's `sorted(xs, key=..)`)
fn sort_by_key<T: Copy, K: PartialOrd>(xs: &mut [T], key: impl Fn(T) -> K) {
    let mut keyed: Vec<(K, T)> = xs.iter().map(|&x| (key(x), x)).collect();
    keyed.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    for (i, (_, x)) in keyed.into_iter().enumerate() {
        xs[i] = x;
    }
}

fn controls(g: &Game, p: PlayerId, m: PermId) -> bool {
    g.perm(m).on_bf && g.perm(m).owner == p
}

fn on_bf(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf
}

fn name_of(g: &Game, m: PermId) -> Sym {
    g.perm(m).name
}

fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

fn remove_first(v: &mut Vec<CardId>, c: CardId) -> bool {
    match v.iter().position(|&x| x == c) {
        Some(i) => {
            v.remove(i);
            true
        }
        None => false,
    }
}

fn add_kw(g: &mut Game, m: PermId, kw: Sym) {
    let k = &mut g.perm_mut(m).eot_kw;
    if !k.contains(&kw) {
        k.push(kw);
    }
}

fn ability(utility: f64, label: String, src: PermId, f: crate::hooks::AbilityFn, arg: i64) -> Opt {
    Opt { utility, label, act: Some(Action::Ability { src, f, arg }) }
}

fn plan(utility: f64, label: String, f: crate::hooks::PlanFn, arg: i64) -> Opt {
    Opt { utility, label, act: Some(Action::Plan { f, arg }) }
}

/// two ids in one option argument
fn pack(a: u32, b: u32) -> i64 {
    ((b as i64) << 32) | a as i64
}

fn unpack(arg: i64) -> (u32, u32) {
    ((arg & 0xffff_ffff) as u32, (arg >> 32) as u32)
}

/// alela.alela_perm: p's Alela on the battlefield
pub fn alela_perm(g: &Game, p: PlayerId) -> Option<PermId> {
    named(g, p).into_iter().find(|&m| card_name(g, m) == Some(ALELA))
}

/// p's permanents not phased out (for alela._named)
fn named(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| !g.perm(m).phased).collect()
}

/// alela._named: p's permanents of this name, not phased out
fn named_as(g: &Game, p: PlayerId, name: &str) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| !g.perm(m).phased && card_name(g, m) == Some(name)).collect()
}

/// alela._flier
fn flier(g: &Game, m: PermId) -> bool {
    g.perm(m).fly || g.perm(m).eot_kw.contains(&"flying")
}

fn creatures(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.is_creature(m) && !g.perm(m).phased).collect()
}

/// engine.casts_this_turn with a predicate: spells p cast this turn that match
fn casts_this_turn(g: &Game, p: PlayerId, pred: impl Fn(CardId) -> bool) -> i32 {
    match &g.player(p).turn_casts {
        Some((st, cs)) if *st == g.turn_stamp() => cs.iter().filter(|&&c| pred(c)).count() as i32,
        _ => 0,
    }
}

// ================================================================== Alela
/// whenever you cast an artifact or enchantment spell: a 1/1 blue Faerie with flying
fn alela_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let o = g.perm(src).owner;
    let d = g.db.get(c);
    if caster != o || d.land || !(d.types.has(Types::ARTIFACT) || d.types.has(Types::ENCHANTMENT)) {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "a 1/1 blue Faerie with flying", Some(3.0))? {
        return Ok(());
    }
    let turns = g.player(o).turns;
    g.player_mut(o).milestone.entry("faerie").or_insert(turns);
    g.player_mut(o).stat("alela_faeries", 1);
    let spec = Tokens {
        tgh: Some(1),
        fly: true,
        color: Some(Colors::from_letters("U")),
        types: vec!["faerie"],
        ..Tokens::new(1, 1)
    };
    make_tokens(g, o, spec)?;
    crate::glog!(g, "    Alela: {} creates a 1/1 Faerie with flying", pname(g, o));
    Ok(())
}

// ================================================================== the AI
/// alela.faerie_bonus: what one more Faerie is worth to p now (on the 0-90 priority scale): a 2/1 flier with Alela
/// out, more with Wispdrinker Vampire (each one drains the table) and Tetsuko Umezawa (each one is unblockable)
pub fn faerie_bonus(g: &Game, p: PlayerId) -> i32 {
    if alela_perm(g, p).is_none() {
        return 0;
    }
    let has = |n: &str| named(g, p).iter().any(|&m| card_name(g, m) == Some(n));
    10 + if has("Wispdrinker Vampire") { 6 } else { 0 } + if has("Tetsuko Umezawa, Fugitive") { 4 } else { 0 }
}

/// alela.alela_prio: the outside decks' generic priority, plus a Faerie for each artifact or enchantment spell while
/// Alela is out
pub fn alela_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let mut v = crate::ai::pool::generic_prio(g, p, c);
    let d = g.db.get(c);
    if c == g.player(p).cmd {
        return v.max(85);
    }
    if v == 0 && d.has_dsl() {
        v = (crate::dsl::card_value(g, p, c) * 10.0) as i32; // untagged: its value
    }
    if v == 0 && d.tag(Tag::Landfall2) {
        v = if g.player(p).turns <= 6 { 50 } else { 30 }; // Felidar Retreat: a 2/2 Cat for each land still to come
    }
    if v != 0 && !d.land && (d.types.has(Types::ARTIFACT) || d.types.has(Types::ENCHANTMENT)) {
        v += faerie_bonus(g, p);
    }
    v.min(90)
}

// ================================================================== the 99
/// alela._counter_nonflier: +1/+1 counter on target creature you control without flying (Plumecreed Mentor,
/// Pileated Provisioner)
fn counter_nonflier(g: &mut Game, p: PlayerId, src: PermId) {
    let cs: Vec<PermId> = creatures(g, p).into_iter().filter(|&m| !flier(g, m)).collect();
    let Some(t) = first_max(&cs, |m| (g.perm(m).is_cmd, !g.perm(m).noatk, pval(g, m))) else { return };
    g.perm_mut(t).plus += 1;
    crate::glog!(g, "    {}: +1/+1 counter on {}", name_of(g, src), name_of(g, t));
}

// ------------------------------------------------------------------ the Jace token (empower Jace)
/// alela.jace_token: p's Jace planeswalker token
pub fn jace_token(g: &Game, p: PlayerId) -> Option<PermId> {
    named_as(g, p, JACE).into_iter().next()
}

/// alela.empower: put n loyalty counters on your Jace token, creating it first if you don't control one (a blue Jace
/// planeswalker token with "-1: Surveil 1" and "-3: Draw a card"). The token has no card: it ceases to exist when it
/// leaves.
pub fn empower(g: &mut Game, p: PlayerId, n: i32) {
    let j = match jace_token(g, p) {
        Some(j) => j,
        None => {
            let cd = g.db.id(JACE).expect("the Jace token is in the card database");
            let j = g.new_perm(p, Some(cd), "Token", 0, 0);
            g.enter_no += 1;
            let born = g.enter_no;
            let x = g.perm_mut(j);
            x.loyalty = Some(0);
            x.colors = Colors::from_letters("U");
            x.born = born;
            x.on_bf = true;
            g.player_mut(p).perms.push(j);
            g.hooks.push(j);
            g.bf_ver += 1;
            crate::glog!(g, "    {} creates a Jace planeswalker token", pname(g, p));
            j
        }
    };
    let l = g.perm(j).loyalty.unwrap_or(0) + n;
    g.perm_mut(j).loyalty = Some(l);
    crate::glog!(g, "    empower Jace {n}: loyalty {l}");
}

/// -1 surveil 1: only while the token can't reach the -3 soon (no Plan for All Outcomes growing it)
fn jace_surveil(g: &Game, p: PlayerId, src: PermId) -> Option<f64> {
    let l = g.perm(src).loyalty.unwrap_or(0);
    if l >= 3 || (l == 2 && !named_as(g, p, "Plan for All Outcomes").is_empty()) {
        return None;
    }
    Some(0.7)
}

fn jace_surveil_go(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    crate::cardcode::scry(g, p, 1, true)
}

fn jace_draw_v(_g: &Game, _p: PlayerId, _src: PermId) -> Option<f64> {
    Some(3.0)
}

fn jace_draw(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    draw(g, p, 1, false)
}

/// empower Jace: a Jace planeswalker token (-1 surveil 1, -3 draw a card) that Fatehold Charm, Keeper of the Quiet
/// Hour and Plan for All Outcomes grow
static JACE_ABILITIES: [WalkerAb; 2] = [
    WalkerAb { delta: -1, label: "surveil 1", val: jace_surveil, eff: jace_surveil_go },
    WalkerAb { delta: -3, label: "draw a card", val: jace_draw_v, eff: jace_draw },
];

/// Keeper of the Quiet Hour: 3/2; empower Jace 2 on entry
fn keeper(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m == src && trigger_window(g, o, Some(src), "empower Jace 2", None)? {
        empower(g, o, 2);
    }
    Ok(())
}

/// Plan for All Outcomes: the best opposing nonland permanent goes on top of its owner's library (tokens are gone) ...
fn plan_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = g.perm(src).owner;
    let Some(t) = best_opp_nonland(g, o, |_| true) else { return Ok(()) };
    if pval(g, t) < 2.0 {
        return Ok(());
    }
    let text = format!("{} to the top or bottom of its library", name_of(g, t));
    if !trigger_window(g, o, Some(src), &text, Some(5.0))? {
        return Ok(());
    }
    if on_bf(g, t) && !untargetable(g, t) {
        apply_removal(g, Some(o), t, "top", None)?; // its owner keeps it on top
    }
    Ok(())
}

/// ... empower Jace 1 on your first noncreature spell each turn
fn plan_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let o = g.perm(src).owner;
    let d = g.db.get(c);
    if caster != o || d.creature || d.land {
        return Ok(());
    }
    if casts_this_turn(g, o, |x| !g.db.get(x).creature && !g.db.get(x).land) != 1 {
        return Ok(()); // first noncreature spell each turn
    }
    if trigger_window(g, o, Some(src), "empower Jace 1", None)? {
        empower(g, o, 1);
    }
    Ok(())
}

fn plan_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let v = best_opp_nonland(g, p, |_| true).map_or(0.0, |t| pval(g, t));
    if v >= 2.0 { 40 + 25.min((5.0 * v) as i32) } else { 28 }
}

/// Fatehold Charm (Approximate): draw a card and empower Jace 2 at the end of the turn before yours, or held to return
/// a spell (to its owner's hand) or an attacking creature; the +1/+2 team mode is not used
fn fatehold_draw(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_some() || g.active == Some(p) || !g.player(p).hand.contains(&c) || !can_pay(g, p, 0, "WU", false) {
        return Ok(vec![]);
    }
    Ok(vec![plan(1.4, "Fatehold Charm (draw, empower Jace 2)".into(), fatehold_go, c.0 as i64)])
}

fn fatehold_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 0, "WU", false) {
        return Ok(false);
    }
    pay(g, p, 0, "WU", false)?;
    remove_first(&mut g.player_mut(p).hand, c);
    g.player_mut(p).spells_this_turn += 1;
    on_cast(g, p, c)?;
    let ok = counter_window(g, p, c, 2.0, vec![])?;
    g.player_mut(p).gy.push(c);
    if ok {
        draw(g, p, 1, false)?;
        empower(g, p, 2);
    }
    crate::glog!(g, "  {} casts Fatehold Charm: draw a card, empower Jace 2", pname(g, p));
    Ok(true)
}

// ------------------------------------------------------------------ removal and control
/// Spite // Malice: Spite is held to counter a noncreature spell; Malice ({3}{B}) destroys a nonblack creature (no
/// regeneration) when the creature is worth more than holding the counter
fn malice(g: &mut Game, c: CardId, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 3, "B", false) {
        return Ok(vec![]);
    }
    let tg: Vec<PermId> = legal_targets(g, p, "destroy", "c", false, Some(c))
        .into_iter()
        .filter(|&m| !colors_of(g, m).has('B'))
        .collect();
    let Some(t) = max_by(&tg, |m| pval(g, m)) else { return Ok(vec![]) };
    if pval(g, t) < 4.0 {
        return Ok(vec![]);
    }
    Ok(vec![plan(pval(g, t) * 0.5 - 1.0, format!("Malice on {}", name_of(g, t)), malice_go, pack(c.0 as u32, t.0))])
}

fn malice_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (c, t) = unpack(arg);
    let (c, t) = (CardId(c as u16), PermId(t));
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 3, "B", false) || !on_bf(g, t) {
        return Ok(false);
    }
    pay(g, p, 3, "B", false)?;
    remove_first(&mut g.player_mut(p).hand, c);
    g.player_mut(p).spells_this_turn += 1;
    on_cast(g, p, c)?;
    let ok = counter_window(g, p, c, 4.0, vec![])?;
    g.player_mut(p).gy.push(c);
    crate::glog!(g, "  {} casts Malice on {}", pname(g, p), name_of(g, t));
    if ok && on_bf(g, t) && !untargetable(g, t) {
        apply_removal(g, Some(p), t, "destroy", Some(c))?; // noregen tag
    }
    Ok(true)
}

/// Muddle the Mixture: transmute {1}{U}{U} (sorcery speed): the best two-mana card in the library, when it is worth
/// clearly more than holding the counter
fn transmute(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || g.active != Some(p) || !g.player(p).hand.contains(&c) || !can_pay(g, p, 1, "UU", false) {
        return Ok(vec![]);
    }
    let cs: Vec<CardId> = searchable(g, p).into_iter().filter(|&x| g.db.get(x).cmc == 2).collect();
    let Some(best) = max_by(&cs, |x| card_worth(g, p, x, false)) else { return Ok(vec![]) };
    let gain = card_worth(g, p, best, false) - card_worth(g, p, c, false);
    if gain < 8.0 {
        return Ok(vec![]);
    }
    let label = format!("transmute Muddle the Mixture ({})", g.db.get(best).name);
    Ok(vec![plan(gain / 12.0, label, transmute_go, pack(c.0 as u32, best.0 as u32))])
}

fn transmute_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (c, best) = unpack(arg);
    let (c, best) = (CardId(c as u16), CardId(best as u16));
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 1, "UU", false) || !g.player(p).library.contains(&best) {
        return Ok(false);
    }
    pay(g, p, 1, "UU", false)?;
    remove_first(&mut g.player_mut(p).hand, c);
    g.player_mut(p).gy.push(c);
    if !ability_window_card(g, p, c, "transmute", None, None)? {
        return Ok(true);
    }
    remove_first(&mut g.player_mut(p).library, best);
    shuffle_library(g, p);
    g.player_mut(p).hand.push(best);
    crate::glog!(g, "  {} transmutes Muddle the Mixture for {}", pname(g, p), g.db.get(best).name);
    Ok(true)
}

/// Ray of Command: before combat on your turn, take the best opposing creature until end of turn (untapped, haste)
/// and attack with it; it is tapped when it goes back
fn ray(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || g.active != Some(p) || !g.player(p).hand.contains(&c) || !can_pay(g, p, 3, "U", false) {
        return Ok(vec![]);
    }
    let Some(t) = best_steal(g, p, Some(c)) else { return Ok(vec![]) };
    if epow(g, t) < 3 {
        return Ok(vec![]);
    }
    let u = 0.3 * epow(g, t) as f64 + 0.2 * pval(g, t) - 1.2;
    Ok(vec![plan(u, format!("Ray of Command on {}", name_of(g, t)), ray_go, pack(c.0 as u32, t.0))])
}

fn ray_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (c, t) = unpack(arg);
    let (c, t) = (CardId(c as u16), PermId(t));
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 3, "U", false) || !on_bf(g, t) {
        return Ok(false);
    }
    pay(g, p, 3, "U", false)?;
    remove_first(&mut g.player_mut(p).hand, c);
    g.player_mut(p).spells_this_turn += 1;
    on_cast(g, p, c)?;
    let ok = counter_window(g, p, c, 4.0, vec![])?;
    g.player_mut(p).gy.push(c);
    if !ok || !on_bf(g, t) || g.perm(t).owner == p || untargetable(g, t) {
        return Ok(true);
    }
    let to = g.perm(t).owner;
    if protect_response(g, to, t, "steal", Some(p), Some(c))? || !on_bf(g, t) {
        return Ok(true);
    }
    steal(g, p, t, true);
    g.perm_mut(t).data.set(DataKey::TapOnReturn, Val::Bool(true));
    Ok(true)
}

/// Karmic Justice: when an opponent's spell or ability destroys a noncreature permanent you control (Karmic Justice
/// itself too), destroy their best permanent
fn karmic(g: &mut Game, src: Src, m: PermId, cause: Sym) -> Res {
    karmic_fire(g, src, m, cause)
}

fn karmic_self(g: &mut Game, _src: Src, m: PermId, cause: Sym) -> Res {
    karmic_fire(g, m, m, cause)
}

fn karmic_fire(g: &mut Game, src: PermId, m: PermId, cause: Sym) -> Res {
    let o = g.perm(src).owner;
    let Some(d) = g.destroyer else { return Ok(()) };
    if cause != "destroy" || g.is_creature(m) || g.perm(m).owner != o || d == o || !g.opps(o).any(|q| q == d) {
        return Ok(());
    }
    let cands: Vec<PermId> =
        g.player(d).perms.iter().copied().filter(|&x| !g.perm(x).phased && !untargetable(g, x)).collect();
    let Some(t) = max_by(&cands, |x| pval(g, x)) else { return Ok(()) };
    if !trigger_window(g, o, Some(src), &format!("destroy {}", name_of(g, t)), Some(4.0))? {
        return Ok(());
    }
    if controls(g, d, t) {
        apply_removal(g, Some(o), t, "destroy", None)?;
    }
    Ok(())
}

fn karmic_prio(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    30
}

// ------------------------------------------------------------------ Opposition
/// alela._tappers: q's untapped creatures to tap for Opposition, cheapest first (no {T} in the cost: summoning
/// sickness is fine)
fn tappers(g: &Game, q: PlayerId) -> Vec<PermId> {
    let mut t: Vec<PermId> = g
        .player(q)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            g.is_creature(m) && !x.tapped && !x.phased && !x.is_cmd
        })
        .collect();
    sort_by_key(&mut t, |m| (!g.perm(m).token, pval(g, m)));
    t
}

/// alela._turns_left: opponents' turns from a's (this one) until q's next turn
fn turns_left(g: &Game, q: PlayerId, a: PlayerId) -> usize {
    let mut n = 1;
    for x in g.after(a) {
        if x == q {
            break;
        }
        if g.player(x).alive {
            n += 1;
        }
    }
    n
}

/// alela._share: the tappers q spends this turn
fn share(g: &Game, q: PlayerId, a: PlayerId) -> Vec<PermId> {
    let t = tappers(g, q);
    let k = t.len().div_ceil(turns_left(g, q, a));
    t[..k].to_vec()
}

/// what Opposition taps: a permanent or a land
#[derive(Clone, Copy)]
enum Tapped {
    Perm(PermId),
    Land(LandId),
}

/// alela._opp_tap
fn opp_tap(g: &mut Game, q: PlayerId, tapper: PermId, what: Tapped, name: &str) {
    g.perm_mut(tapper).tapped = true;
    match what {
        Tapped::Perm(m) => g.perm_mut(m).tapped = true,
        Tapped::Land(l) => g.land_mut(l).tapped = true,
    }
    crate::glog!(g, "    Opposition: {} taps {} to tap {}", pname(g, q), name_of(g, tapper), name);
}

/// Opposition, an opponent's upkeep: tap their mana (best rocks and lands), keeping a share of tappers for their
/// attackers. HUMAN(phase 9): a person taps their own.
fn opposition_upkeep(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let q = g.perm(src).owner;
    if p == q || !g.opps(q).any(|x| x == p) || named_as(g, q, "Opposition").first() != Some(&src) {
        return Ok(());
    }
    let mut use_ = share(g, q, p);
    let atk = g.player(p).perms.iter().filter(|&&m| g.is_creature(m) && !g.perm(m).phased && epow(g, m) >= 3).count();
    use_.truncate(use_.len().saturating_sub(atk));
    let mut srcs: Vec<(f64, Tapped, String)> = vec![];
    for &m in &g.player(p).perms {
        let x = g.perm(m);
        let Some(cd) = x.cd else { continue };
        let Some(rock) = g.db.get(cd).tags.str(Tag::Rock) else { continue };
        if x.tapped || x.phased || untargetable(g, m) {
            continue;
        }
        let n: i32 = rock.split(':').next().unwrap_or("0").parse().unwrap_or(0);
        srcs.push((n as f64 + 0.5, Tapped::Perm(m), x.name.to_string()));
    }
    for &l in &g.player(p).lands {
        if g.land(l).tapped {
            continue;
        }
        let d = g.db.get(g.land(l).cd);
        let amt = d.tags.int(Tag::Amt).filter(|&v| v != 0).unwrap_or(1);
        srcs.push((amt as f64 + 0.1 * land_colors_now(g, p, l).count() as f64, Tapped::Land(l), d.name.to_string()));
    }
    srcs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal)); // stable: Python's key=-x[0]
    for (tapper, (_, what, name)) in use_.into_iter().zip(srcs) {
        if !trigger_window(g, q, Some(src), &format!("tap {name}"), Some(1.0))? {
            break;
        }
        opp_tap(g, q, tapper, what, &name);
    }
    Ok(())
}

/// alela.opposition_precombat (CI.opposition_precombat): q at the beginning of another player's combat: tap their
/// attackers with Opposition
pub fn opposition_precombat(g: &mut Game, q: PlayerId) -> Res {
    let Some(&src) = named_as(g, q, "Opposition").first() else { return Ok(()) };
    let Some(a) = g.active else { return Ok(()) };
    let _ = src;
    let mut cands: Vec<PermId> = g
        .player(a)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            g.is_creature(m)
                && !x.tapped
                && !x.phased
                && !untargetable(g, m)
                && epow(g, m) >= 2
                && (!x.sick || has_haste(g, m))
        })
        .collect();
    sort_by_key(&mut cands, |m| -epow(g, m));
    for (tapper, t) in share(g, q, a).into_iter().zip(cands) {
        if epow(g, t) <= epow(g, tapper) && !g.perm(t).is_cmd {
            break; // the tapper blocks it as well
        }
        let n = name_of(g, t);
        opp_tap(g, q, tapper, Tapped::Perm(t), n);
    }
    Ok(())
}

/// Opposition, your beginning of combat: creatures that can't attack tap the opponents' fliers that could block.
/// HUMAN(phase 9): a person taps their own.
fn opposition_mine(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != g.perm(src).owner || named_as(g, p, "Opposition").first() != Some(&src) {
        return Ok(());
    }
    let idle: Vec<PermId> = tappers(g, p)
        .into_iter()
        .filter(|&m| {
            let x = g.perm(m);
            (x.sick && !has_haste(g, m)) || x.noatk || epow(g, m) == 0
        })
        .collect();
    let mut blockers: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| {
            let x = g.perm(m);
            g.is_creature(m)
                && !x.tapped
                && !x.phased
                && !untargetable(g, m)
                && (flier(g, m) || x.cd.is_some_and(|c| g.db.get(c).tag(Tag::Reach)))
        })
        .collect();
    sort_by_key(&mut blockers, |m| -pval(g, m));
    for (tapper, t) in idle.into_iter().zip(blockers) {
        let n = name_of(g, t);
        opp_tap(g, p, tapper, Tapped::Perm(t), n);
    }
    Ok(())
}

fn opposition_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let n = creatures(g, p).len() as i32;
    65.min(35 + 4 * n)
}

// ------------------------------------------------------------------ Static Prison, Aether Hub (energy)
/// Static Prison: exiles the best opposing nonland permanent until it leaves; {E}{E}
fn prison(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = g.perm(src).owner;
    if trigger_window(g, o, Some(src), "exile a permanent; you get {E}{E}", Some(5.0))? && controls(g, o, src) {
        oring_exile(g, src, o, |_, _| true, false, false)?;
    }
    g.player_mut(o).energy += 2;
    Ok(())
}

fn prison_leaves(g: &mut Game, src: Src, _m: PermId) -> Res {
    oring_return(g, src)
}

/// each turn pay {E} or sacrifice it (Aether Hub energy keeps it longer)
fn prison_upkeep(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != g.perm(src).owner {
        return Ok(());
    }
    if g.player(p).energy >= 1 && g.perm(src).data.truthy(DataKey::Oring) {
        g.player_mut(p).energy -= 1;
        crate::glog!(g, "    Static Prison: {} pays {{E}} ({} left)", pname(g, p), g.player(p).energy);
    } else if trigger_window(g, p, Some(src), "sacrifice it (no energy)", Some(2.0))? && controls(g, p, src) {
        die(g, src, "sac")?;
    }
    Ok(())
}

fn prison_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let v = best_opp_nonland(g, p, |_| true).map_or(0.0, |t| pval(g, t));
    if v >= 2.0 { 45 + 30.min((6.0 * v) as i32) } else { 0 }
}

/// Aether Hub: {E} on entry
fn hub_etb(g: &mut Game, p: PlayerId, _l: LandId) -> Res {
    g.player_mut(p).energy += 1;
    Ok(())
}

/// alela._hub_spare: energy beyond what Static Prison's upkeep needs
fn hub_spare(g: &Game, p: PlayerId) -> i32 {
    g.player(p).energy - 2 * named_as(g, p, "Static Prison").len() as i32
}

/// {T}: {C}, or pay {E} for any colour (energy that Static Prison needs is kept)
fn hub_cols(g: &Game, p: PlayerId, _l: LandId) -> Colors {
    if hub_spare(g, p) > 0 { g.player(p).ident } else { Colors::NONE }
}

fn hub_tap(g: &mut Game, p: PlayerId, _l: LandId, _used: u32) -> Res {
    if "WUBRG".chars().any(|x| g.tap_cols.has(x)) {
        let e = g.player(p).energy;
        g.player_mut(p).energy = (e - 1).max(0);
    }
    Ok(())
}

// ------------------------------------------------------------------ creatures
/// Airlift Chaplain: flying; mill three on entry, keep a Plains or a creature card with MV 3 or less (else a +1/+1
/// counter)
fn chaplain(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m != src || !trigger_window(g, o, Some(src), "mill three", None)? {
        return Ok(());
    }
    let lib = &g.player(o).library;
    let top: Vec<CardId> = lib[lib.len().saturating_sub(3)..].to_vec();
    mill(g, o, 3)?;
    let cs: Vec<CardId> = top
        .into_iter()
        .filter(|&c| {
            let d = g.db.get(c);
            g.player(o).gy.contains(&c) && (&*d.name == "Plains" || (d.creature && d.cmc <= 3))
        })
        .collect();
    if let Some(c) = max_by(&cs, |c| card_worth(g, o, c, false)) {
        remove_first(&mut g.player_mut(o).gy, c);
        g.player_mut(o).hand.push(c);
        crate::glog!(g, "    Airlift Chaplain: {} puts {} into hand", pname(g, o), g.db.get(c).name);
    } else if controls(g, o, src) {
        g.perm_mut(src).plus += 1;
    }
    Ok(())
}

/// Jackdaw Savior: flying; when it or another flier of yours dies, return a creature card with lesser mana value from
/// your graveyard to the battlefield
fn jackdaw(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    let o = g.perm(src).owner;
    if g.perm(m).owner == o && m != src && g.is_creature(m) && flier(g, m) {
        jackdaw_return(g, o, src, m)?;
    }
    Ok(())
}

fn jackdaw_self(g: &mut Game, _src: Src, m: PermId, _cause: Sym) -> Res {
    let o = g.perm(m).owner;
    jackdaw_return(g, o, m, m)
}

/// alela._jackdaw_return
fn jackdaw_return(g: &mut Game, o: PlayerId, src: PermId, dead: PermId) -> Res {
    let Some(dcd) = g.perm(dead).cd else { return Ok(()) };
    if g.perm(dead).token {
        return Ok(());
    }
    let mv = g.db.get(dcd).cmc;
    let cs: Vec<CardId> =
        g.player(o).gy.iter().copied().filter(|&c| g.db.get(c).creature && g.db.get(c).cmc < mv && c != dcd).collect();
    let Some(c) = max_by(&cs, |c| {
        let d = g.db.get(c);
        crate::ai::card_etb_value(g, o, c) + d.cmc as f64 + d.pow as f64
    }) else {
        return Ok(());
    };
    if !trigger_window(g, o, Some(src), &format!("return {}", g.db.get(c).name), Some(3.0))?
        || !g.player(o).gy.contains(&c)
    {
        return Ok(());
    }
    remove_first(&mut g.player_mut(o).gy, c);
    enter(g, o, c, Enter::default())?;
    crate::glog!(g, "    Jackdaw Savior returns {}", g.db.get(c).name);
    Ok(())
}

/// Plumecreed Mentor: flying; whenever it or another flier of yours enters (Faerie tokens too), a +1/+1 counter on a
/// creature of yours without flying
fn plume(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if g.perm(m).owner == o
        && g.is_creature(m)
        && flier(g, m)
        && trigger_window(g, o, Some(src), "+1/+1 counter", None)?
    {
        counter_nonflier(g, o, src);
    }
    Ok(())
}

fn plume_tokens(g: &mut Game, src: Src, p: PlayerId, toks: &[PermId]) -> Res {
    if p != g.perm(src).owner {
        return Ok(());
    }
    for &t in toks {
        if g.perm(t).fly && controls(g, p, t) && trigger_window(g, p, Some(src), "+1/+1 counter", None)? {
            counter_nonflier(g, p, src);
        }
    }
    Ok(())
}

/// Pileated Provisioner: flying; a +1/+1 counter on a creature of yours without flying on entry
fn provisioner(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m == src && trigger_window(g, o, Some(src), "+1/+1 counter", None)? {
        counter_nonflier(g, o, src);
    }
    Ok(())
}

/// Stirring Hopesinger: flying, lifelink; your instants and sorceries that target a creature (removal, blink, steal)
/// put a +1/+1 counter on each creature you control
fn hopesinger(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let o = g.perm(src).owner;
    let d = g.db.get(c);
    if caster != o || !(d.instant || d.sorcery) {
        return Ok(());
    }
    let t = &d.tags;
    let targets_creature = (t.has(Tag::Rem)
        && matches!(t.str(Tag::Tgt).unwrap_or("c"), "c" | "cp" | "cap" | "ce" | "ac" | "cna"))
        || matches!(
            &*d.name,
            "Daydream" | "Ray of Command" | "Spite // Malice" | "Multiply by Zero" | "Fatehold Charm"
        );
    if !targets_creature || !trigger_window(g, o, Some(src), "+1/+1 counter on each creature you control", None)? {
        return Ok(());
    }
    for m in creatures(g, o) {
        g.perm_mut(m).plus += 1;
    }
    Ok(())
}

/// Desperate Futurescribe: flying; each combat on your turn another creature gets +1/+1 (a +1/+1 counter if you
/// scried or surveilled this turn)
fn futurescribe(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != g.perm(src).owner {
        return Ok(());
    }
    let cs: Vec<PermId> = creatures(g, p).into_iter().filter(|&m| m != src).collect();
    let Some(t) = first_max(&cs, |m| {
        let x = g.perm(m);
        (!x.tapped && !x.sick, flier(g, m), epow(g, m))
    }) else {
        return Ok(());
    };
    if !trigger_window(g, p, Some(src), &format!("+1/+1 to {}", name_of(g, t)), None)? || !controls(g, p, t) {
        return Ok(());
    }
    if g.player(p).scry_turn == Some(g.turn_stamp()) {
        g.perm_mut(t).plus += 1;
    } else {
        g.perm_mut(t).eot_pt.0 += 1;
        g.perm_mut(t).eot_pt.1 += 1;
        g.dsl_on = true;
    }
    Ok(())
}

/// Malcator, Purity Overseer: a 3/3 Golem on entry ...
fn malcator(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m == src && trigger_window(g, o, Some(src), "a 3/3 Golem", None)? {
        golem(g, o)?;
    }
    Ok(())
}

/// alela._golem
fn golem(g: &mut Game, p: PlayerId) -> Res {
    art_entered(g, p, 1);
    let spec =
        Tokens { tgh: Some(3), color: Some(Colors::NONE), types: vec!["golem", "phyrexian"], ..Tokens::new(1, 3) };
    make_tokens(g, p, spec)?;
    Ok(())
}

/// alela._art_entered: artifact tokens made this turn
fn art_entered(g: &mut Game, p: PlayerId, n: i32) {
    let st = g.turn_stamp();
    let pl = g.player_mut(p);
    let have = match pl.art_tok {
        Some((s, k)) if s == st => k,
        _ => 0,
    };
    pl.art_tok = Some((st, have + n));
}

fn malcator_tok(g: &mut Game, src: Src, p: PlayerId, _kinds: &[Sym], n: i32) -> Res {
    if p == g.perm(src).owner {
        art_entered(g, p, n);
    }
    Ok(())
}

/// ... another at your end step if three or more artifacts entered under your control this turn (tokens count)
fn malcator_end(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != g.perm(src).owner {
        return Ok(());
    }
    let st = g.turn_stamp();
    let mut n = g
        .entered
        .iter()
        .filter(|&&(t, m)| {
            t == st && g.perm(m).owner == p && g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(Types::ARTIFACT))
        })
        .count() as i32;
    if let Some((s, k)) = g.player(p).art_tok
        && s == st
    {
        n += k;
    }
    if n >= 3 && trigger_window(g, p, Some(src), "a 3/3 Golem", None)? {
        golem(g, p)?;
    }
    Ok(())
}

/// Emry, Lurker of the Loch: affinity for artifacts (engine.SELF_COST)
pub fn emry_cost(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let pl = g.player(p);
    let arts = pl
        .perms
        .iter()
        .filter(|&&m| !g.perm(m).phased && g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(Types::ARTIFACT)))
        .count() as i32;
    -(arts + pl.treasures as i32)
}

/// mill four on entry
fn emry(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m == src && trigger_window(g, o, Some(src), "mill four", None)? {
        mill(g, o, 4)?;
    }
    Ok(())
}

/// {T}: cast an artifact card from your graveyard this turn (the AI casts it at once)
fn emry_cast(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if x.owner != p || post.is_none() || g.active != Some(p) || x.tapped || x.sick {
        return Ok(vec![]);
    }
    let mut out = vec![];
    let gy: Vec<CardId> = g
        .player(p)
        .gy
        .iter()
        .copied()
        .filter(|&c| g.db.get(c).types.has(Types::ARTIFACT) && !g.db.get(c).land && castable(g, p, c, "gy"))
        .collect();
    for c in gy {
        let (gn, pips) = cost_of(g, p, c);
        if !can_pay(g, p, gn, &pips, false) {
            continue;
        }
        let v = crate::ai::decks::deck_prio(g, p, c);
        if v <= 0 {
            continue;
        }
        let label = format!("Emry: cast {} from the graveyard", g.db.get(c).name);
        out.push(ability(v as f64 / 10.0 - 0.3, label, src, emry_go, c.0 as i64));
    }
    Ok(out)
}

fn emry_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if g.perm(src).tapped || !g.player(p).gy.contains(&c) {
        return Ok(false);
    }
    let (gn, pips) = cost_of(g, p, c);
    g.pay_for = Some(c);
    let r = (|| -> Res<Option<bool>> {
        if !can_pay(g, p, gn, &pips, false) {
            return Ok(Some(false));
        }
        g.perm_mut(src).tapped = true;
        let text = format!("cast {} from the graveyard", g.db.get(c).name);
        if !ability_window(g, p, Some(src), &text, None, None)? {
            return Ok(Some(true));
        }
        pay(g, p, gn, &pips, false)?;
        Ok(None)
    })();
    g.pay_for = None;
    if let Some(done) = r? {
        return Ok(done);
    }
    crate::glog!(g, "  {} casts {} from the graveyard (Emry)", pname(g, p), g.db.get(c).name);
    cast_card(g, p, c, "mgy", Ctx::default())?;
    Ok(true)
}

/// Reconstructed Thopter: unearth {2} (sorcery speed, once: it is exiled after)
fn thopter(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || g.active != Some(p) || !g.player(p).gy.contains(&c) || !can_pay(g, p, 2, "", false) {
        return Ok(vec![]);
    }
    if g.player(p).unearthed.contains(&c) {
        return Ok(vec![]);
    }
    let u = 0.8 + if alela_perm(g, p).is_some() { 1.0 } else { 0.0 };
    Ok(vec![plan(u, "unearth Reconstructed Thopter".into(), thopter_go, c.0 as i64)])
}

fn thopter_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).gy.contains(&c) || !can_pay(g, p, 2, "", false) {
        return Ok(false);
    }
    pay(g, p, 2, "", false)?;
    if !g.player(p).unearthed.contains(&c) {
        g.player_mut(p).unearthed.push(c);
    }
    if !ability_window_card(g, p, c, "unearth", None, None)? || !g.player(p).gy.contains(&c) {
        return Ok(true);
    }
    remove_first(&mut g.player_mut(p).gy, c);
    let m = enter(g, p, c, Enter::default())?;
    g.perm_mut(m).sick = false;
    g.perm_mut(m).data.set(DataKey::Unearth, Val::Bool(true));
    crate::glog!(g, "  {} unearths Reconstructed Thopter", pname(g, p));
    Ok(true)
}

/// flying; unearth {2} once (hasty, exiled at end of turn)
fn thopter_exile(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let o = g.perm(src).owner;
    if g.perm(src).data.truthy(DataKey::Unearth) && on_bf(g, src) && p == o {
        if !trigger_window(g, p, Some(src), "exile it (unearth)", Some(2.0))? || !controls(g, p, src) {
            return Ok(());
        }
        leave(g, src)?;
        let cd = g.perm(src).cd.unwrap();
        g.player_mut(p).exile.push(cd);
    }
    Ok(())
}

/// Strixhaven Skycoach: a basic land to hand on entry ...
fn skycoach(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m == src && trigger_window(g, o, Some(src), "search for a basic land", None)? {
        land_to_hand(g, o)?;
    }
    Ok(())
}

/// ... crew 2 with creatures that would not attack anyway (or are worse attackers than the 3/2 flier)
fn skycoach_crew(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let x = g.perm(src);
    if p != x.owner || x.tapped || x.sick {
        return Ok(());
    }
    let mut cands: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let y = g.perm(m);
            g.is_creature(m) && !y.tapped && !y.phased && m != src && !y.is_cmd
        })
        .collect();
    sort_by_key(&mut cands, |m| (!(g.perm(m).sick || g.perm(m).noatk), pval(g, m)));
    let (mut power, mut pick) = (0, vec![]);
    for m in cands {
        if power >= 2 {
            break;
        }
        let y = g.perm(m);
        if y.sick || y.noatk || epow(g, m) < 3 {
            pick.push(m);
            power += epow(g, m);
        }
    }
    let ready: i32 = pick.iter().filter(|&&m| !(g.perm(m).sick || g.perm(m).noatk)).map(|&m| epow(g, m)).sum();
    if power < 2 || ready >= 3 {
        return Ok(());
    }
    for m in pick {
        g.perm_mut(m).tapped = true;
    }
    let st = g.turn_stamp();
    let s = g.perm_mut(src);
    s.data.set(DataKey::Anim, Val::Bool(true));
    s.data.set(DataKey::Crewed, Val::Stamp(st));
    s.pow = 3;
    s.tgh = 2;
    g.bf_ver += 1;
    crate::glog!(g, "  {} crews Strixhaven Skycoach", pname(g, p));
    Ok(())
}

// ------------------------------------------------------------------ sorceries, Auras, sagas, Venser
/// alela._pick_reanimate
fn pick_reanimate(g: &Game, p: PlayerId, pred: impl Fn(CardId) -> bool) -> Option<CardId> {
    let cs: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).creature && pred(c)).collect();
    max_by(&cs, |c| {
        let d = g.db.get(c);
        crate::ai::card_etb_value(g, p, c) + d.pow as f64 + d.cmc as f64
    })
}

fn helping_hand_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    match pick_reanimate(g, p, |x| g.db.get(x).cmc <= 3) {
        Some(x) => 35 + 3 * 10.min(g.db.get(x).cmc as i32 + 2),
        None => 0,
    }
}

/// Helping Hand: returns the best creature card with MV 3 or less from your graveyard, tapped
pub fn helping_hand(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let Some(x) = pick_reanimate(g, p, |x| g.db.get(x).cmc <= 3) else { return Ok("gy") };
    remove_first(&mut g.player_mut(p).gy, x);
    let m = enter(g, p, x, Enter::default())?;
    g.perm_mut(m).tapped = true;
    Ok("gy")
}

/// alela._daydream_target
fn daydream_target(g: &Game, p: PlayerId) -> Option<PermId> {
    let cs: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.is_creature(m) && !g.perm(m).token && !g.perm(m).phased && g.perm(m).cd.is_some())
        .collect();
    max_by(&cs, |m| blink_value(g, p, m) + 0.1 * pval(g, m))
}

fn daydream_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let v = daydream_target(g, p).map_or(0.0, |t| blink_value(g, p, t));
    if v >= 2.0 { 30 + 30.min((5.0 * v) as i32) } else { 0 }
}

/// Daydream: blinks your creature with the best entry ability (+1/+1 counter); flashback {2}{W}
pub fn daydream(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let Some(t) = daydream_target(g, p) else { return Ok("gy") };
    if let Some(n) = blink(g, p, t)? {
        g.perm_mut(n).plus += 1;
    }
    Ok("gy")
}

/// Feather of Flight: draw a card as it enters
fn feather_draw(g: &mut Game, p: PlayerId, _a: PermId, _h: PermId) -> Res {
    draw(g, p, 1, false)
}

/// alela._feather_host: a creature without flying first
fn feather_host(g: &mut Game, p: PlayerId, a: PermId) -> Res<Option<PermId>> {
    let cs: Vec<PermId> = creatures(g, p).into_iter().filter(|&m| m != a).collect();
    Ok(max_by(&cs, |m| (!flier(g, m) as i32 * 3 + epow(g, m) + g.perm(m).is_cmd as i32) as f64))
}

/// flash; draw a card; +1/+0 and flying (on a creature without flying)
static FEATHER: AuraSpec =
    AuraSpec { pow: 1, tgh: 0, kws: &["flying"], on_etb: Some(feather_draw), host_pick: Some(feather_host), ..AURA };

/// Ajani Fells the Godsire: I: exile an opposing creature with power 3+ ...
fn ajani(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let mut d = PermData::default();
    d.set(DataKey::Lore, Val::Int(1));
    g.perm_mut(src).data = d;
    let o = g.perm(src).owner;
    let t = best_opp_creature(g, o, |x| epow(g, x) >= 3);
    if let Some(t) = t
        && trigger_window(g, o, Some(src), &format!("chapter I: exile {}", name_of(g, t)), Some(5.0))?
        && on_bf(g, t)
    {
        apply_removal(g, Some(o), t, "exile", None)?;
    }
    Ok(())
}

/// ... II: a 2/1 Cat Warrior and a vigilance counter; III: double strike to your best attacker. The lore counter is
/// added before the trigger window, so the trigger probe adds one too (as in the Python).
fn ajani_lore(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != g.perm(src).owner || g.perm(src).data.is_empty() || g.perm(src).data.get(DataKey::Lore).is_none() {
        return Ok(());
    }
    let n = g.perm(src).data.int(DataKey::Lore) + 1;
    g.perm_mut(src).data.set(DataKey::Lore, Val::Int(n));
    let text = if n == 2 { "chapter II: a 2/1 Cat Warrior, a vigilance counter" } else { "chapter III: double strike" };
    let ok = trigger_window(g, p, Some(src), text, Some(3.0))?;
    if ok && n == 2 {
        let spec = Tokens {
            tgh: Some(1),
            color: Some(Colors::from_letters("W")),
            types: vec!["cat", "warrior"],
            ..Tokens::new(1, 2)
        };
        make_tokens(g, p, spec)?;
        let cs: Vec<PermId> = creatures(g, p).into_iter().filter(|&m| !g.perm(m).vig).collect();
        if let Some(t) = first_max(&cs, |m| (g.perm(m).is_cmd, epow(g, m))) {
            g.perm_mut(t).vig = true;
        }
    } else if ok && n >= 3 {
        let cs: Vec<PermId> = creatures(g, p).into_iter().filter(|&m| !g.perm(m).tapped).collect();
        if let Some(t) = first_max(&cs, |m| (flier(g, m), epow(g, m) + if g.perm(m).dt { 3 } else { 0 })) {
            add_kw(g, t, "double strike");
            g.dsl_on = true;
        }
    }
    if n >= 3 && controls(g, p, src) {
        die(g, src, "sac")?;
    }
    Ok(())
}

fn ajani_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    match best_opp_creature(g, p, |x| epow(g, x) >= 3) {
        Some(t) => 40 + 30.min((5.0 * pval(g, t)) as i32),
        None => 32,
    }
}

/// alela._venser_blink_target
fn venser_blink_target(g: &Game, p: PlayerId) -> Option<PermId> {
    let cs: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            !x.token
                && !x.phased
                && !x.is_cmd
                && x.cd.is_some_and(|c| !g.db.get(c).tag(Tag::Tokpw) && &*g.db.get(c).name != "Venser, the Sojourner")
        })
        .collect();
    max_by(&cs, |m| blink_value(g, p, m))
}

/// alela._venser_unbl_value: Venser's -1 (creatures can't be blocked this turn): what the attack it frees is worth
fn venser_unbl_value(g: &Game, p: PlayerId) -> f64 {
    let dmg: i32 = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            g.is_creature(m) && !x.tapped && !x.phased && !x.noatk && (!x.sick || has_haste(g, m))
        })
        .map(|m| epow(g, m))
        .sum();
    if dmg < 5 {
        return 0.0;
    }
    let low = g.opps(p).map(|q| g.player(q).life).min().unwrap_or(99);
    dmg as f64 * 0.5 + if dmg >= low { 8.0 } else { 0.0 }
}

fn venser_plus(g: &Game, p: PlayerId, src: PermId) -> Option<f64> {
    if venser_unbl_value(g, p) > 4.0 && g.perm(src).loyalty.unwrap_or(0) >= 1 {
        return None; // the -1 at combat is worth more
    }
    let t = venser_blink_target(g, p);
    Some(1.2 + t.map_or(0.0, |t| 0.5 * blink_value(g, p, t)))
}

fn venser_blink(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    if let Some(t) = venser_blink_target(g, p)
        && blink_value(g, p, t) > 0.0
    {
        blink(g, p, t)?;
    }
    Ok(())
}

fn venser_emblem_v(_g: &Game, _p: PlayerId, _src: PermId) -> Option<f64> {
    Some(10.0)
}

/// rules.give_emblem(p, 'venser'): exile a permanent per spell you cast (rules.emblem_cast)
fn venser_emblem(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let e = &mut g.player_mut(p).emblems;
    if !e.contains(&"venser") {
        e.push("venser");
    }
    Ok(())
}

/// +2 blinks your permanent with the best entry ability (back at once, not at end step); -1 makes your creatures
/// unblockable before a big attack (`venser_minus`); -8 emblem (exile a permanent per spell you cast)
static VENSER: [WalkerAb; 2] = [
    WalkerAb { delta: 2, label: "blink a permanent", val: venser_plus, eff: venser_blink },
    WalkerAb { delta: -8, label: "emblem", val: venser_emblem_v, eff: venser_emblem },
];

/// -1 just before combat (it's a main-phase ability: used as the attack is about to start). HUMAN(phase 9): a person
/// uses it in their own main phase.
fn venser_minus(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let l = g.perm(src).loyalty;
    if p != g.perm(src).owner || l.is_none_or(|l| l < 1) {
        return Ok(());
    }
    if uses(g, p, src) >= allowed(g, p) || venser_unbl_value(g, p) <= 4.0 {
        return Ok(());
    }
    let n = uses(g, p, src) + 1;
    let stamp = round_stamp(g, p);
    let x = g.perm_mut(src);
    x.loyalty_used = Some(stamp);
    x.data.set(DataKey::LoyaltyN, Val::Int(n));
    x.loyalty = Some(l.unwrap() - 1);
    crate::glog!(g, "  {} uses Venser, the Sojourner (-1): creatures can't be blocked this turn", pname(g, p));
    if ability_window(g, p, Some(src), "-1", None, None)? {
        g.player_mut(p).unbl_all = Some(g.turn_stamp());
    }
    if controls(g, p, src) && g.perm(src).loyalty.unwrap_or(0) <= 0 {
        leave(g, src)?;
        to_zone_card(g, src, Zone::Gy);
    }
    Ok(())
}

// ================================================================== registry
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, ALELA)?.cast = Some(alela_cast);
    walker(r, db, JACE, &JACE_ABILITIES)?;
    r.card(db, "Keeper of the Quiet Hour")?.etb = Some(keeper);
    let c = r.card(db, "Plan for All Outcomes")?;
    c.etb = Some(plan_etb);
    c.cast = Some(plan_cast);
    c.prio = Some(plan_prio);
    r.card(db, "Fatehold Charm")?.hand_options = Some(fatehold_draw);
    r.card(db, "Spite // Malice")?.hand_options = Some(malice);
    r.card(db, "Muddle the Mixture")?.hand_options = Some(transmute);
    r.card(db, "Ray of Command")?.hand_options = Some(ray);
    let c = r.card(db, "Karmic Justice")?;
    c.dies = Some(karmic);
    c.self_dies = Some(karmic_self);
    at_once(c, Event::Dies, true); // no trigger_window of their own: at once
    at_once(c, Event::SelfDies, true);
    c.prio = Some(karmic_prio);
    let c = r.card(db, "Opposition")?;
    c.upkeep = Some(opposition_upkeep);
    c.crew = Some(opposition_mine);
    at_once(c, Event::Crew, true);
    c.prio = Some(opposition_prio);
    let c = r.card(db, "Static Prison")?;
    c.etb = Some(prison);
    c.leaves = Some(prison_leaves);
    at_once(c, Event::Leaves, true);
    c.upkeep = Some(prison_upkeep);
    c.prio = Some(prison_prio);
    let c = r.card(db, "Aether Hub")?;
    c.land_etb = Some(hub_etb);
    c.land_cols = Some(hub_cols);
    c.on_tap_land = Some(hub_tap);
    r.card(db, "Airlift Chaplain")?.etb = Some(chaplain);
    let c = r.card(db, "Jackdaw Savior")?;
    c.dies = Some(jackdaw);
    c.self_dies = Some(jackdaw_self);
    at_once(c, Event::Dies, true);
    at_once(c, Event::SelfDies, true);
    let c = r.card(db, "Plumecreed Mentor")?;
    c.etb = Some(plume);
    c.tokens_enter = Some(plume_tokens);
    at_once(c, Event::TokensEnter, true);
    r.card(db, "Pileated Provisioner")?.etb = Some(provisioner);
    r.card(db, "Stirring Hopesinger")?.cast = Some(hopesinger);
    let c = r.card(db, "Desperate Futurescribe")?;
    c.crew = Some(futurescribe);
    at_once(c, Event::Crew, true);
    let c = r.card(db, "Malcator, Purity Overseer")?;
    c.etb = Some(malcator);
    c.token_created = Some(malcator_tok);
    at_once(c, Event::TokenCreated, true);
    c.end_step = Some(malcator_end);
    let c = r.card(db, "Emry, Lurker of the Loch")?;
    c.etb = Some(emry);
    c.options = Some(emry_cast);
    let c = r.card(db, "Reconstructed Thopter")?;
    c.gy_options = Some(thopter);
    c.end_step = Some(thopter_exile);
    let c = r.card(db, "Strixhaven Skycoach")?;
    c.etb = Some(skycoach);
    c.crew = Some(skycoach_crew);
    at_once(c, Event::Crew, true);
    let c = r.card(db, "Helping Hand")?;
    c.resolve = Some(helping_hand);
    c.prio = Some(helping_hand_prio);
    let c = r.card(db, "Daydream")?;
    c.resolve = Some(daydream);
    c.prio = Some(daydream_prio);
    aura(r, db, "Feather of Flight", &FEATHER)?;
    let c = r.card(db, "Ajani Fells the Godsire")?;
    c.etb = Some(ajani);
    c.upkeep = Some(ajani_lore);
    c.prio = Some(ajani_prio);
    walker(r, db, "Venser, the Sojourner", &VENSER)?;
    let c = r.card(db, "Venser, the Sojourner")?;
    c.crew = Some(venser_minus);
    at_once(c, Event::Crew, true);
    Ok(())
}
