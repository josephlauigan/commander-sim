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
//! The person's choices (practice mode) are in `human/abilities.rs`. Daydream, Venser and Ray of Command use the stand-ins
//! for t2.blink / blink_value and marchesa.best_steal / steal in `galadriel.rs`.

use super::common::{
    AURA, AuraSpec, WalkerAb, allowed, aura, aura_spec, best_opp_creature, best_opp_nonland, oring_exile, oring_return,
    own_host, round_stamp, uses, walker,
};
use super::galadriel::{best_steal, blink, blink_value, steal};
use super::partials::at_once;
use crate::engine::repl;
use crate::ai::decks::protect_response;
use crate::cards::{CardDb, Colors, Types};
use crate::engine::cast::{cast_card, castable, on_cast};
use crate::engine::combat::{can_block, first_strike, has_haste, kw};
use crate::engine::life::gain;
use crate::engine::mana::{can_pay, cost_of, land_colors_now, pay};
use crate::engine::removal::{apply_removal, legal_targets};
use crate::engine::stack::{ability_window, ability_window_card, counter_window, may_window, trigger_window};
use crate::engine::tutors::{card_worth, shuffle_library};
use crate::engine::values::{card_name, colors_of, epow, etgh, no_sac, pval, untargetable};
use crate::engine::zones::{
    Enter, Tokens, Zone, bounce, die, draw, enter, land_to_hand, leave, make_tokens, max_by, mill,
};
use crate::engine::zones::{searchable, to_zone_card};
use crate::flow::Res;
use crate::hooks::{Action, Assign, Event, Opt, Registry, Src};
use crate::human::human_of;
use crate::ids::{CardId, PermId, PlayerId};
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
    repl::put_plus(g, t, 1);
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
    if !may_window(g, o, Some(src), &format!("destroy {}", name_of(g, t)), Some(4.0))? {
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
    Land(PermId),
}

/// alela._opp_tap
fn opp_tap(g: &mut Game, q: PlayerId, tapper: PermId, what: Tapped, name: &str) {
    g.perm_mut(tapper).tapped = true;
    match what {
        Tapped::Perm(m) => g.perm_mut(m).tapped = true,
        Tapped::Land(l) => g.perm_mut(l).tapped = true,
    }
    crate::glog!(g, "    Opposition: {} taps {} to tap {}", pname(g, q), name_of(g, tapper), name);
}

/// Opposition, an opponent's upkeep: tap their mana (best rocks and lands), keeping a share of tappers for their
/// attackers. A person taps their own (abilities.rs), so nothing here for their Opposition.
fn opposition_upkeep(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let q = g.perm(src).owner;
    if p == q
        || !g.opps(q).any(|x| x == p)
        || human_of(g, q).is_some()
        || named_as(g, q, "Opposition").first() != Some(&src)
    {
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
        if g.perm(l).tapped {
            continue;
        }
        let d = g.db.get(g.perm(l).card());
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
/// A person taps their own (abilities.rs).
fn opposition_mine(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != g.perm(src).owner || human_of(g, p).is_some() || named_as(g, p, "Opposition").first() != Some(&src) {
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
fn hub_etb(g: &mut Game, p: PlayerId, _l: PermId) -> Res {
    g.player_mut(p).energy += 1;
    Ok(())
}

/// alela._hub_spare: energy beyond what Static Prison's upkeep needs
fn hub_spare(g: &Game, p: PlayerId) -> i32 {
    g.player(p).energy - 2 * named_as(g, p, "Static Prison").len() as i32
}

/// {T}: {C}, or pay {E} for any colour (energy that Static Prison needs is kept)
fn hub_cols(g: &Game, p: PlayerId, _l: PermId) -> Colors {
    if hub_spare(g, p) > 0 { g.player(p).ident } else { Colors::NONE }
}

fn hub_tap(g: &mut Game, p: PlayerId, _l: PermId, _used: u32) -> Res {
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
    // "you may put a Plains or a cheap creature card into your hand": a person picks the card, or declines for the counter
    let pick = if crate::human::is_human(g, o) {
        let labels: Vec<String> = cs.iter().map(|&c| crate::human::choices::card_label(g, c)).collect();
        if labels.is_empty() {
            None
        } else {
            crate::human::choices::choose(
                g,
                o,
                "choose",
                "Airlift Chaplain: put a milled card into your hand? (otherwise it gets a +1/+1 counter)",
                &labels,
                Some("no (a +1/+1 counter)"),
                None,
            )?
            .map(|k| cs[k])
        }
    } else {
        max_by(&cs, |c| card_worth(g, o, c, false))
    };
    if let Some(c) = pick {
        remove_first(&mut g.player_mut(o).gy, c);
        g.player_mut(o).hand.push(c);
        crate::glog!(g, "    Airlift Chaplain: {} puts {} into hand", pname(g, o), g.db.get(c).name);
    } else if controls(g, o, src) {
        repl::put_plus(g, src, 1);
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
        && matches!(t.str(Tag::Tgt).unwrap_or("c"), "c" | "cp" | "cap" | "ce" | "cep" | "ac" | "cna"))
        || matches!(
            &*d.name,
            "Daydream" | "Ray of Command" | "Spite // Malice" | "Multiply by Zero" | "Fatehold Charm"
        );
    if !targets_creature || !trigger_window(g, o, Some(src), "+1/+1 counter on each creature you control", None)? {
        return Ok(());
    }
    for m in creatures(g, o) {
        repl::put_plus(g, m, 1);
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
        repl::put_plus(g, t, 1);
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
pub fn helping_hand(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let pick = if crate::human::is_human(g, p) {
        // the card the person chose as they cast it
        ctx.cards.first().copied().filter(|c| g.player(p).gy.contains(c))
    } else {
        pick_reanimate(g, p, |x| g.db.get(x).cmc <= 3)
    };
    let Some(x) = pick else { return Ok("gy") };
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
pub fn daydream(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let t = if crate::human::is_human(g, p) {
        ctx.target.filter(|&t| g.perm(t).on_bf && g.perm(t).cd.is_some())
    } else {
        daydream_target(g, p)
    };
    let Some(t) = t else { return Ok("gy") };
    if let Some(n) = blink(g, p, t)? {
        repl::put_plus(g, n, 1);
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

/// -1 just before combat (it's a main-phase ability: used as the attack is about to start). A person uses it in their
/// own main phase.
fn venser_minus(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let l = g.perm(src).loyalty;
    if p != g.perm(src).owner || human_of(g, p).is_some() || l.is_none_or(|l| l < 1) {
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

// ================================================================== the seven cards added with the 10/15 list
// Credit Voucher, Mirage Mirror, Scrapheap, Angelic Page, Cabal Patriarch, Serra the Benevolent, Wall of Junk.

/// cards p would rather not hold (lands past what it needs, dead cards): the AI's own worth for them is low
fn junk_in_hand(g: &Game, p: PlayerId) -> Vec<CardId> {
    let cmd = g.player(p).cmd;
    g.player(p).hand.iter().copied().filter(|&c| c != cmd && card_worth(g, p, c, false) < 15.0).collect()
}

// ------------------------------------------------------------------ Credit Voucher
/// {2}, {T}, sacrifice: shuffle any number of cards from your hand into your library, then draw that many. The AI
/// puts back only the cards it rates low (extra lands late, dead cards): two or more in a main phase, one at the end
/// of the turn or after combat.
fn voucher_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if x.owner != p || !x.on_bf || x.tapped || x.phased || !can_pay(g, p, 2, "", false) {
        return Ok(vec![]);
    }
    let junk = junk_in_hand(g, p).len();
    let need = if post == Some(false) { 2 } else { 1 };
    if junk < need || g.player(p).library.len() <= junk {
        return Ok(vec![]);
    }
    Ok(vec![ability(0.8 + 0.3 * junk as f64, format!("Credit Voucher: put back {junk} card(s)"), src, voucher_go, 0)])
}

fn voucher_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    let junk = junk_in_hand(g, p);
    if !controls(g, p, src) || g.perm(src).tapped || junk.is_empty() || !can_pay(g, p, 2, "", false) {
        return Ok(false);
    }
    pay(g, p, 2, "", false)?;
    g.perm_mut(src).tapped = true;
    die(g, src, "sac")?; // the sacrifice is a cost
    if !ability_window(g, p, Some(src), "shuffle cards from hand into the library, draw that many", None, None)? {
        return Ok(true);
    }
    let junk: Vec<CardId> = junk.into_iter().filter(|c| g.player(p).hand.contains(c)).collect();
    for &c in &junk {
        remove_first(&mut g.player_mut(p).hand, c);
        g.player_mut(p).library.push(c);
    }
    shuffle_library(g, p);
    crate::glog!(g, "  {} Credit Voucher: shuffles {} card(s) away and draws {}", pname(g, p), junk.len(), junk.len());
    draw(g, p, junk.len() as u32, false)?;
    Ok(true)
}

// ------------------------------------------------------------------ Scrapheap
/// whenever an artifact or enchantment is put into your graveyard from the battlefield (Scrapheap itself too): gain 1
/// life. Tokens never reach the graveyard; a card owned by another player goes to its owner's graveyard, not yours.
/// (Covers the permanents that die by `die`: destroyed or sacrificed. Planeswalker-style direct moves don't matter
/// here: it is artifacts and enchantments only.)
fn scrap_fire(g: &mut Game, src: PermId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    let x = g.perm(m);
    if x.token || x.orig != o {
        return Ok(());
    }
    let Some(cd) = x.cd else { return Ok(()) };
    let d = g.db.get(cd);
    if d.land || !(d.types.has(Types::ARTIFACT) || d.types.has(Types::ENCHANTMENT)) {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "gain 1 life", Some(2.0))? {
        return Ok(());
    }
    crate::glog!(g, "    Scrapheap: {} gains 1 life ({} went to the graveyard)", pname(g, o), name_of(g, m));
    gain(g, o, 1)
}

fn scrap_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    scrap_fire(g, src, m)
}

fn scrap_self(g: &mut Game, _src: Src, m: PermId, _cause: Sym) -> Res {
    scrap_fire(g, m, m)
}

// ------------------------------------------------------------------ Mirage Mirror
/// the permanent's face becomes card `cd` (name, body, keywords, hooks), as a copy; no enter effects
pub(crate) fn set_face(g: &mut Game, m: PermId, cd: CardId) {
    // --- scion: also Scion of the Ur-Dragon's "becomes a copy" (impls/t6_scion.rs)
    let (name, pow, tgh, fly, dt, vig, ll, war, noatk, dsl) = {
        let d = g.db.get(cd);
        let t = &d.tags;
        (
            crate::sym::intern(&d.name),
            d.pow,
            d.tgh,
            t.has(Tag::Fly),
            t.has(Tag::Dt),
            t.has(Tag::Vig),
            t.has(Tag::Lifelink),
            t.has(Tag::Warrior),
            t.has(Tag::Noatk),
            d.has_dsl(),
        )
    };
    if let Some(i) = g.hooks.iter().position(|&x| x == m) {
        g.hooks.remove(i);
    }
    let x = g.perm_mut(m);
    x.cd = Some(cd);
    x.name = name;
    x.pow = pow;
    x.tgh = tgh;
    x.fly = fly;
    x.dt = dt;
    x.vig = vig;
    x.lifelink = ll;
    x.warrior = war;
    x.noatk = noatk;
    if dsl {
        g.dsl_on = true;
    }
    if g.registry.get(cd).is_some_and(|i| i.live()) {
        g.hooks.push(m);
    }
    g.bf_ver += 1;
}

/// the permanents Mirage Mirror could copy for p: nonland permanents with a card (a token has no card to copy, a land
/// is never worth {2} here), not itself, and not a legendary permanent p already controls (the legend rule)
fn mirror_targets(g: &Game, p: PlayerId, src: PermId) -> Vec<PermId> {
    let mut out = vec![];
    for q in g.players.iter().filter(|q| q.alive) {
        for &t in &q.perms {
            let x = g.perm(t);
            let Some(cd) = x.cd else { continue };
            if t == src || x.phased || x.token || (x.owner != p && untargetable(g, t)) {
                continue;
            }
            if g.db.get(cd).land || x.data.truthy(DataKey::MirageMirror) {
                continue;
            }
            if x.owner == p && crate::cardcode::is_legendary(g, t) {
                continue;
            }
            if g.db.get(cd).has_subtype("aura") {
                continue; // a copy of an Aura would have nothing to enchant
            }
            out.push(t);
        }
    }
    out
}

/// pay {2} and become a copy of t until end of turn
fn mirror_become(g: &mut Game, src: PermId, p: PlayerId, t: PermId) -> Res<bool> {
    let Some(tcd) = g.perm(t).cd else { return Ok(false) };
    if !controls(g, p, src) || !on_bf(g, t) || !can_pay(g, p, 2, "", false) {
        return Ok(false);
    }
    pay(g, p, 2, "", false)?;
    if !ability_window(g, p, Some(src), &format!("becomes a copy of {}", name_of(g, t)), None, Some(t))? {
        return Ok(true);
    }
    if !controls(g, p, src) || !on_bf(g, t) {
        return Ok(true); // the target left: the ability fizzles
    }
    let own = g.perm(src).cd;
    crate::glog!(g, "  {} Mirage Mirror becomes a copy of {}", pname(g, p), name_of(g, t));
    set_face(g, src, tcd);
    let x = g.perm_mut(src);
    x.phys = own;
    x.data.set(DataKey::MirageMirror, Val::Bool(true));
    Ok(true)
}

/// engine/turn.rs, at the start of every turn: copies made by Mirage Mirror end ("until end of turn")
pub fn mirror_revert(g: &mut Game) -> Res {
    for i in 0..g.perms.len() {
        let m = PermId(i as u32);
        if !g.perm(m).data.truthy(DataKey::MirageMirror) {
            continue;
        }
        g.perm_mut(m).data.remove(DataKey::MirageMirror);
        if !g.perm(m).on_bf {
            continue;
        }
        if let Some(own) = g.perm(m).phys {
            set_face(g, m, own);
            g.perm_mut(m).phys = None;
        }
    }
    Ok(())
}

/// how much the copy is worth as an attacker or a blocker: its printed body (a defender never attacks)
fn mirror_body(g: &Game, t: PermId) -> Option<(i32, i32, bool)> {
    let cd = g.perm(t).cd?;
    let d = g.db.get(cd);
    d.creature.then(|| (d.pow, d.tgh, d.tag(Tag::Fly)))
}

/// {2}: becomes a copy of target artifact, creature, enchantment or land until end of turn. Before attacks (main phase
/// one) the AI copies the best creature on the battlefield when that makes a 3+ power attacker; on the opponent's turn
/// `mirror_blocks` copies a good blocker.
fn mirror_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if x.owner != p
        || post != Some(false)
        || g.active != Some(p)
        || !x.on_bf
        || x.sick
        || x.tapped
        || x.data.truthy(DataKey::MirageMirror)
        || !can_pay(g, p, 2, "", false)
    {
        return Ok(vec![]);
    }
    let best = max_by(&mirror_targets(g, p, src), |t| match mirror_body(g, t) {
        Some((pow, _, fly)) if pow >= 3 && !g.db.get(g.perm(t).cd.unwrap()).has_kw("defender") => {
            pow as f64 + if fly { 2.0 } else { 0.0 } + 0.2 * pval(g, t)
        }
        _ => -1.0,
    });
    let Some(t) = best.filter(|&t| mirror_body(g, t).is_some_and(|b| b.0 >= 3)) else { return Ok(vec![]) };
    if g.db.get(g.perm(t).cd.unwrap()).has_kw("defender") {
        return Ok(vec![]);
    }
    let pow = mirror_body(g, t).unwrap().0;
    let label = format!("Mirage Mirror: copy {}", name_of(g, t));
    Ok(vec![ability(0.6 + 0.35 * pow as f64, label, src, mirror_go, t.0 as i64)])
}

fn mirror_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    mirror_become(g, src, p, PermId(arg as u32))
}

/// On the other player's attack: an untapped Mirror with {2} open becomes a copy of the creature that blocks the biggest
/// unblocked attacker and survives it, and blocks.
fn mirror_blocks(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], d: PlayerId, assign: &mut Assign) -> Res {
    let x = g.perm(src);
    if x.owner != d
        || d == p
        || !x.on_bf
        || x.tapped
        || x.data.truthy(DataKey::MirageMirror)
        || human_of(g, d).is_some()
        || !can_pay(g, d, 2, "", false)
        || assign.iter().any(|y| y.1 == src)
    {
        return Ok(());
    }
    let free: Vec<PermId> =
        atk.iter().copied().filter(|&a| on_bf(g, a) && !assign.iter().any(|y| y.0 == a) && epow(g, a) >= 2).collect();
    let Some(a) = max_by(&free, |a| epow(g, a) as f64) else { return Ok(()) };
    if kw(g, a, "menace") {
        return Ok(());
    }
    let cands: Vec<PermId> = mirror_targets(g, d, src)
        .into_iter()
        .filter(|&t| {
            mirror_body(g, t).is_some_and(|(pow, tgh, fly)| {
                tgh > epow(g, a) && !g.perm(a).dt && (pow >= 1 || tgh >= 5) && (fly || !g.perm(a).fly)
            })
        })
        .collect();
    let Some(t) = max_by(&cands, |t| {
        let (pow, tgh, _) = mirror_body(g, t).unwrap();
        (if pow >= etgh(g, a) { 10 } else { 0 } + tgh) as f64
    }) else {
        return Ok(());
    };
    if mirror_become(g, src, d, t)? && g.perm(src).data.truthy(DataKey::MirageMirror) && can_block(g, src, a) {
        assign.push((a, src));
    }
    Ok(())
}

// ------------------------------------------------------------------ Angelic Page
/// {T}: target attacking or blocking creature gets +1/+1 until end of turn. Used once blocks are declared (this hook
/// runs for both players' Pages): where the +1/+1 wins a fight or saves a creature, else on an unblocked attacker
/// (+1 damage). Not on the turn it came in (summoning sick).
fn page_blocks(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], d: PlayerId, assign: &mut Assign) -> Res {
    let o = g.perm(src).owner;
    let x = g.perm(src);
    if (o != p && o != d) || !x.on_bf || x.tapped || x.sick || x.phased || human_of(g, o).is_some() {
        return Ok(());
    }
    let atk_side = o == p;
    // (own creature, the creature it fights) for each fight on o's side
    let fights: Vec<(PermId, PermId)> = assign
        .iter()
        .copied()
        .filter(|&(a, b)| on_bf(g, a) && on_bf(g, b))
        .map(|(a, b)| if atk_side { (a, b) } else { (b, a) })
        .collect();
    let first = |m: PermId| first_strike(g, m);
    let decisive = |mine: PermId, other: PermId| {
        let (mp, mt) = (epow(g, mine), etgh(g, mine));
        let (op, ot) = (epow(g, other), etgh(g, other));
        let dies = (op >= mt || g.perm(other).dt) && !(first(mine) && !first(other) && mp >= ot);
        let dies_after = (op >= mt + 1 || g.perm(other).dt) && !(first(mine) && !first(other) && mp + 1 >= ot);
        let kills = mp >= ot || g.perm(mine).dt;
        let kills_after = mp + 1 >= ot || g.perm(mine).dt;
        (dies && !dies_after) || (!kills && kills_after && !dies_after)
    };
    let mut pick = fights.iter().copied().find(|&(mine, other)| decisive(mine, other)).map(|f| f.0);
    if pick.is_none() && atk_side {
        let unblocked: Vec<PermId> = atk
            .iter()
            .copied()
            .filter(|&a| on_bf(g, a) && g.perm(a).owner == o && !assign.iter().any(|y| y.0 == a))
            .collect();
        pick = max_by(&unblocked, |a| epow(g, a) as f64);
    }
    let Some(t) = pick else { return Ok(()) };
    g.perm_mut(src).tapped = true;
    if ability_window(g, o, Some(src), &format!("+1/+1 on {}", name_of(g, t)), None, Some(t))? && on_bf(g, t) {
        let y = g.perm_mut(t);
        y.eot_pt = (y.eot_pt.0 + 1, y.eot_pt.1 + 1);
        crate::glog!(g, "    Angelic Page gives {} +1/+1", g.perm(t).name);
    }
    Ok(())
}

// ------------------------------------------------------------------ Wall of Junk
/// 0/7 defender. When it blocks it returns to its owner's hand at end of combat (`wall_return`, run by the combat code
/// after damage). The AI blocks freely with it: the biggest attacker nobody blocks that it survives (it is an
/// artifact spell each time it is recast, so Alela makes a Faerie each time).
fn wall_blocks(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], d: PlayerId, assign: &mut Assign) -> Res {
    let x = g.perm(src);
    if x.owner != d || d == p || !x.on_bf || x.phased {
        return Ok(());
    }
    if !x.tapped && !assign.iter().any(|y| y.1 == src) && human_of(g, d).is_none() {
        let free: Vec<PermId> = atk
            .iter()
            .copied()
            .filter(|&a| {
                on_bf(g, a)
                    && !assign.iter().any(|y| y.0 == a)
                    && epow(g, a) >= 1
                    && epow(g, a) < etgh(g, src)
                    && !g.perm(a).dt
                    && !kw(g, a, "menace")
                    && can_block(g, src, a)
            })
            .collect();
        if let Some(a) = max_by(&free, |a| epow(g, a) as f64) {
            assign.push((a, src));
        }
    }
    if assign.iter().any(|y| y.1 == src) {
        g.perm_mut(src).data.set(DataKey::WallBlocked, Val::Bool(true));
    }
    Ok(())
}

/// end of combat: each Wall of Junk that blocked and is still on the battlefield returns to its owner's hand
pub fn wall_return(g: &mut Game, assign: &[(PermId, PermId)]) -> Res {
    for &(_, b) in assign {
        if !g.perm(b).data.truthy(DataKey::WallBlocked) {
            continue;
        }
        g.perm_mut(b).data.remove(DataKey::WallBlocked);
        if g.perm(b).on_bf {
            crate::glog!(g, "    {} returns to its owner's hand", g.perm(b).name);
            bounce(g, b)?;
        }
    }
    Ok(())
}

// ------------------------------------------------------------------ Cabal Patriarch
/// what one activation of Cabal Patriarch pays besides {2}{B}
#[derive(Clone, Copy, PartialEq)]
enum Pay {
    Sac(PermId),
    Gy(CardId),
}

/// the cheapest costs for n activations: Faerie tokens and other spare bodies first (never Alela or Patriarch), then
/// creature cards in the graveyard
fn patriarch_costs(g: &Game, p: PlayerId, src: PermId, n: usize) -> Option<Vec<Pay>> {
    let mut bodies: Vec<PermId> = creatures(g, p)
        .into_iter()
        .filter(|&m| m != src && !g.perm(m).is_cmd && card_name(g, m) != Some(ALELA) && !no_sac(g, m))
        .filter(|&m| g.perm(m).token || pval(g, m) <= 2.5)
        .collect();
    sort_by_key(&mut bodies, |m| (!g.perm(m).token as i32, pval(g, m)));
    let gy: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).creature).collect();
    let mut out: Vec<Pay> = vec![];
    // a token first, a graveyard creature before a real card, the rest by value
    let mut bi = bodies.into_iter().peekable();
    let mut gi = gy.into_iter();
    while out.len() < n {
        match bi.peek().copied() {
            Some(b) if g.perm(b).token => {
                out.push(Pay::Sac(b));
                bi.next();
            }
            _ => {
                if let Some(c) = gi.next() {
                    out.push(Pay::Gy(c));
                } else if let Some(b) = bi.next() {
                    out.push(Pay::Sac(b));
                } else {
                    return None;
                }
            }
        }
    }
    Some(out)
}

fn pay_value(g: &Game, c: Pay) -> f64 {
    match c {
        Pay::Sac(m) => pval(g, m),
        Pay::Gy(_) => 0.4,
    }
}

/// {2}{B}, sacrifice a creature (or exile a creature card from your graveyard): target creature gets -2/-2 until end of
/// turn. Aimed at opposing creatures the -2/-2 kills (toughness 2 or less: one activation; 3 or 4: two, when the target
/// is worth much more than the costs and the mana is there).
fn patriarch_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if g.perm(src).owner != p || post.is_none() || !on_bf(g, src) || !can_pay(g, p, 2, "B", false) {
        return Ok(vec![]);
    }
    let mut out = vec![];
    for t in legal_targets(g, p, "destroy", "c", false, None) {
        let th = etgh(g, t);
        if th > 4 {
            continue;
        }
        let n = if th <= 2 { 1 } else { 2 };
        if n == 2 && !can_pay(g, p, 4, "BB", false) {
            continue;
        }
        let Some(costs) = patriarch_costs(g, p, src, n) else { continue };
        let spent: f64 = costs.iter().map(|&c| pay_value(g, c)).sum::<f64>() + 0.7 * n as f64;
        let gain_v = pval(g, t);
        if gain_v < spent + 0.5 {
            continue;
        }
        out.push(ability(
            gain_v - spent,
            format!("Cabal Patriarch: {} x -2/-2 on {}", n, name_of(g, t)),
            src,
            patriarch_go,
            pack(t.0, n as u32),
        ));
    }
    Ok(out)
}

fn patriarch_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let (t, n) = unpack(arg);
    let (t, n) = (PermId(t), n as usize);
    let mut did = false;
    for _ in 0..n {
        if !controls(g, p, src) || !on_bf(g, t) || !can_pay(g, p, 2, "B", false) {
            break;
        }
        let Some(cost) = patriarch_costs(g, p, src, 1).and_then(|v| v.first().copied()) else { break };
        pay(g, p, 2, "B", false)?;
        match cost {
            Pay::Sac(m) => die(g, m, "sac")?,
            Pay::Gy(c) => {
                if remove_first(&mut g.player_mut(p).gy, c) {
                    g.player_mut(p).exile.push(c);
                }
            }
        }
        did = true;
        if ability_window(g, p, Some(src), "-2/-2", None, Some(t))? {
            shrink(g, p, t, 2)?;
        }
    }
    Ok(did)
}

/// -n/-n until end of turn on m (a lethal one goes through the removal code)
fn shrink(g: &mut Game, p: PlayerId, m: PermId, n: i32) -> Res {
    if !g.perm(m).on_bf {
        return Ok(());
    }
    if etgh(g, m) <= n {
        return apply_removal(g, Some(p), m, &format!("shrink{n}"), None);
    }
    let x = g.perm_mut(m);
    x.eot_pt = (x.eot_pt.0 - n, x.eot_pt.1 - n);
    Ok(())
}

// ------------------------------------------------------------------ Serra the Benevolent
fn serra_fliers(g: &Game, p: PlayerId) -> Vec<PermId> {
    creatures(g, p).into_iter().filter(|&m| flier(g, m) || kw(g, m, "flying")).collect()
}

fn serra_plus_v(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let ready =
        serra_fliers(g, p).into_iter().filter(|&m| !g.perm(m).tapped && (!g.perm(m).sick || has_haste(g, m))).count();
    Some(if ready > 0 { 1.2 + 0.5 * ready as f64 } else { 0.5 })
}

/// +2: your fliers get +1/+1 until end of turn
fn serra_plus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    for m in serra_fliers(g, p) {
        let x = g.perm_mut(m);
        x.eot_pt = (x.eot_pt.0 + 1, x.eot_pt.1 + 1);
    }
    Ok(())
}

/// -3 when the board needs a body or the opponents' attackers would kill Serra at 4 loyalty
fn serra_angel_v(g: &Game, p: PlayerId, src: PermId) -> Option<f64> {
    let mine = creatures(g, p).len();
    let loy = g.perm(src).loyalty.unwrap_or(0);
    let threat: i32 = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| g.is_creature(m) && !g.perm(m).phased)
        .map(|m| epow(g, m))
        .sum();
    let v = if mine <= 1 {
        4.0
    } else if mine <= 3 || threat >= loy {
        2.5
    } else {
        1.0
    };
    Some(v)
}

/// -3: a 4/4 white Angel with flying and vigilance
fn serra_angel(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let spec = Tokens { fly: true, color: Some(Colors::from_letters("W")), types: vec!["angel"], ..Tokens::new(1, 4) };
    let spec = Tokens { tgh: Some(4), ..spec };
    for t in make_tokens(g, p, spec)? {
        g.perm_mut(t).vig = true;
    }
    Ok(())
}

fn serra_emblem_v(_g: &Game, _p: PlayerId, _src: PermId) -> Option<f64> {
    Some(10.0)
}

/// -6: the emblem "if you control a creature, damage that would reduce your life total to less than 1 reduces it to 1
/// instead" (`serra_floor`, called from `lose_life`)
fn serra_emblem(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    g.player_mut(p).emblems.push("serra");
    Ok(())
}

static SERRA: [WalkerAb; 3] = [
    WalkerAb { delta: 2, label: "fliers get +1/+1", val: serra_plus_v, eff: serra_plus },
    WalkerAb { delta: -3, label: "4/4 Angel", val: serra_angel_v, eff: serra_angel },
    WalkerAb { delta: -6, label: "emblem", val: serra_emblem_v, eff: serra_emblem },
];

/// engine/life.rs: damage n to p; with Serra's emblem and a creature, it can't take p below 1 life. True if nothing is
/// left to deal.
pub fn serra_floor(g: &Game, p: PlayerId, n: &mut i32) -> bool {
    let pl = g.player(p);
    if !pl.emblems.contains(&"serra") || pl.life < 1 || *n < pl.life {
        return *n <= 0;
    }
    if !pl.perms.iter().any(|&m| !g.perm(m).phased && g.is_creature(m)) {
        return false;
    }
    *n = pl.life - 1;
    *n <= 0
}

// ------------------------------------------------------------------ Crystal Chimes
/// {3}, {T}, sacrifice: return all enchantment cards from your graveyard to your hand. The AI waits for two or more
/// (Opposition, Karmic Justice, Auras ... all come back at once).
fn chimes_options(g: &mut Game, src: Src, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if x.owner != p || !x.on_bf || x.tapped || x.phased || !can_pay(g, p, 3, "", false) {
        return Ok(vec![]);
    }
    let n = chimes_cards(g, p).len();
    if n < 2 {
        return Ok(vec![]);
    }
    Ok(vec![ability(1.0 + 0.5 * n as f64, format!("Crystal Chimes: {n} enchantments back"), src, chimes_go, 0)])
}

fn chimes_cards(g: &Game, p: PlayerId) -> Vec<CardId> {
    g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).types.has(Types::ENCHANTMENT)).collect()
}

fn chimes_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !controls(g, p, src) || g.perm(src).tapped || !can_pay(g, p, 3, "", false) {
        return Ok(false);
    }
    pay(g, p, 3, "", false)?;
    g.perm_mut(src).tapped = true;
    die(g, src, "sac")?;
    if !ability_window(g, p, Some(src), "return all enchantment cards from the graveyard to hand", None, None)? {
        return Ok(true);
    }
    let back = chimes_cards(g, p);
    for &c in &back {
        remove_first(&mut g.player_mut(p).gy, c);
        g.player_mut(p).hand.push(c);
    }
    crate::glog!(g, "  {} Crystal Chimes returns {} enchantment card(s)", pname(g, p), back.len());
    Ok(true)
}

// ------------------------------------------------------------------ Angelic Shield (the AI's save)
/// Angelic Shield, "sacrifice: return target creature to its owner's hand": in response to removal aimed at one of
/// your valuable creatures (not a token, which would be lost; not the commander, who can go to the command zone), the
/// creature goes back to your hand and the removal fizzles. (Aimed at an opposing threat, the ability-language option
/// of the card does the bouncing.)
pub fn alela_protect(
    g: &mut Game,
    owner: PlayerId,
    m: PermId,
    kind: Sym,
    _actor: Option<PlayerId>,
    _spell: Option<CardId>,
) -> Res<bool> {
    let x = g.perm(m);
    if !x.on_bf || x.token || x.is_cmd || !g.is_creature(m) || kind == "bounce" || pval(g, m) < 3.0 {
        return Ok(false);
    }
    let Some(&shield) = named_as(g, owner, "Angelic Shield").first() else { return Ok(false) };
    crate::glog!(g, "  {} sacrifices Angelic Shield: {} returns to hand", pname(g, owner), name_of(g, m));
    die(g, shield, "sac")?;
    if ability_window(g, owner, Some(shield), "return a creature to its owner's hand", None, Some(m))? && on_bf(g, m) {
        bounce(g, m)?;
    }
    Ok(true)
}

// ------------------------------------------------------------------ Lightning Greaves (off Alela)
/// Greaves on Alela gives her shroud, so the deck's own Aura (or Daydream) can't target her. When the hand holds one
/// that wants Alela, the AI first moves the Greaves (equip {0}) to the best other creature.
fn greaves_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if g.perm(src).owner != p || crate::ai::hand_key(g, p) != "alela" || post != Some(false) || g.active != Some(p) {
        return Ok(vec![]);
    }
    let Some(am) = alela_perm(g, p) else { return Ok(vec![]) };
    if g.perm(src).attached != Some(am) {
        return Ok(vec![]);
    }
    let wants = g.player(p).hand.iter().any(|&c| {
        let aura_here = aura_spec(g, c)
            .is_some_and(|s| s.target == "own" && s.host_pick.is_none() && own_host(g, p, s, None) == Some(am));
        let daydream = &*g.db.get(c).name == "Daydream" && daydream_target(g, p) == Some(am);
        (aura_here || daydream) && castable(g, p, c, "hand")
    });
    if !wants {
        return Ok(vec![]);
    }
    let others: Vec<PermId> = creatures(g, p).into_iter().filter(|&m| m != am).collect();
    let Some(to) = max_by(&others, |m| pval(g, m)) else { return Ok(vec![]) };
    Ok(vec![ability(
        3.0,
        format!("Lightning Greaves off Alela, onto {}", name_of(g, to)),
        src,
        greaves_go,
        to.0 as i64,
    )])
}

fn greaves_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let to = PermId(arg as u32);
    if !controls(g, p, src) || !on_bf(g, to) || g.perm(to).owner != p {
        return Ok(false);
    }
    crate::glog!(g, "  {} moves Lightning Greaves to {} (Alela needs to be targetable)", pname(g, p), name_of(g, to));
    if ability_window(g, p, Some(src), "equip {0}", None, Some(to))? && on_bf(g, to) {
        g.perm_mut(src).attached = Some(to);
    }
    Ok(true)
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
    let c = r.card(db, "Credit Voucher")?;
    c.options = Some(voucher_options);
    c.prio = Some(|_, _, _| 22);
    let c = r.card(db, "Scrapheap")?;
    c.dies = Some(scrap_dies);
    c.self_dies = Some(scrap_self);
    at_once(c, Event::Dies, true);
    at_once(c, Event::SelfDies, true);
    c.prio = Some(|_, _, _| 25);
    let c = r.card(db, "Mirage Mirror")?;
    c.options = Some(mirror_options);
    c.blocks = Some(mirror_blocks);
    c.prio = Some(|_, _, _| 28);
    r.card(db, "Angelic Page")?.blocks = Some(page_blocks);
    r.card(db, "Wall of Junk")?.blocks = Some(wall_blocks);
    r.card(db, "Cabal Patriarch")?.options = Some(patriarch_options);
    walker(r, db, "Serra the Benevolent", &SERRA)?;
    r.card(db, "Serra the Benevolent")?.prio = Some(|_, _, _| 65);
    r.card(db, "Crystal Chimes")?.options = Some(chimes_options);
    r.card(db, "Lightning Greaves")?.options = Some(greaves_options);
    Ok(())
}
