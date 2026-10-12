//! The sheoldred deck (decklists/JD/sheoldred-mono-black.md): mono-black drain around Sheoldred, the Apocalypse. Its
//! card code and AI plan; the brief is rust/agents/sheoldred-cards.md, the verdicts are in data/card_overrides.toml and
//! the tests are tests/cards_sheoldred.rs.
//!
//! Several cards here came with the retired candidate decks (glissa, gix, yawgmoth), whose card code was deleted while
//! their `Full` verdicts stayed in the overrides; they are modeled here again, keyed by card name.

use crate::ai::pool::generic_prio;
use crate::cards::{CardDb, Types};
use crate::engine::cast::{cast_card, castable, discard_worst};
use crate::engine::life::{gain, lose_life};
use crate::engine::mana::{can_pay, cost_of, pay};
use crate::engine::removal::{apply_removal, legal_targets};
use crate::engine::stack::{ability_window, trigger_window};
use crate::engine::tutors::{card_worth, shuffle_library};
use crate::engine::values::{epow, etgh, pval, untargetable};
use crate::engine::zones::{
    Enter, Zone, discard_cards, draw, enter, exile_perm, land_to_hand, leave, make_artifact_tokens, max_by, min_by,
    returns_left, to_zone_card,
};
use crate::flow::Res;
use crate::hooks::{Action, Call, Event, Opt, Registry, Sacrificed, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, DataKey, Game, Perm, Val};
use crate::sym::{Sym, intern};
use crate::tag::Tag;

use super::bites::crime;
pub use super::bites::gained_life;
use super::partials::remove_card;

// ------------------------------------------------------------------ helpers
fn name_of(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

fn human(g: &Game, p: PlayerId) -> bool {
    crate::human::is_human(g, p)
}

fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn on_bf(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    x.on_bf && !x.phased
}

fn live_creature(g: &Game, m: PermId) -> bool {
    on_bf(g, m) && g.is_creature(m)
}

fn is_walker(g: &Game, m: PermId) -> bool {
    g.perm(m).loyalty.is_some()
}

fn affordable(g: &Game, p: PlayerId, c: CardId) -> bool {
    let (cg, cp) = cost_of(g, p, c);
    can_pay(g, p, cg, &cp, false) && castable(g, p, c, "hand")
}

fn zero_prio(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    0
}

fn plus_counter(g: &mut Game, m: PermId, n: i32) {
    crate::cardcode::add_counters(g, m, n);
}

fn flipped(g: &Game, m: PermId) -> bool {
    g.perm(m).data.truthy(DataKey::Flipped)
}

fn my_creatures(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.is_creature(m) && !g.perm(m).phased).collect()
}

fn opp_creatures(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| g.is_creature(m) && !g.perm(m).phased)
        .collect()
}

fn gy_creatures(g: &Game, p: PlayerId) -> Vec<CardId> {
    g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).creature).collect()
}

fn selfpt_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        g.selfpt = true;
    }
    Ok(())
}

fn selfpt_as(g: &mut Game, _p: PlayerId, _m: PermId) -> Res {
    g.selfpt = true;
    Ok(())
}

/// p's Sheoldred, the Apocalypse is out
pub fn sheoldred_out(g: &Game, p: PlayerId) -> bool {
    crate::engine::values::has(g, p, Tag::SheoA)
}

// ------------------------------------------------------------------ Melancholic Poet (repartee)
/// the spell being cast is an instant or sorcery of p's that targets a creature
fn repartee(g: &Game, src: Src, caster: PlayerId, c: CardId) -> bool {
    let d = g.db.get(c);
    if caster != owner(g, src) || !on_bf(g, src) || !(d.instant || d.sorcery) {
        return false;
    }
    let Some((cc, ctx, _)) = &g.cur_cast else { return false };
    *cc == c && [ctx.target, ctx.host].into_iter().flatten().any(|t| live_creature(g, t))
}

/// Melancholic Poet: repartee, each opponent loses 1 life and you gain 1
fn poet_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    if repartee(g, src, caster, c) && trigger_window(g, caster, Some(src), "repartee: drain 1", None)? {
        for o in g.opps(caster).collect::<Vec<_>>() {
            lose_life(g, o, 1, Some(caster), "drain", None)?;
        }
        gain(g, caster, 1)?;
    }
    Ok(())
}

// ------------------------------------------------------------------ Cost of Brilliance
/// Cost of Brilliance: the AI draws two (and loses 2 life) itself and puts the counter on its best creature
fn cost_resolve(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let q = ctx.face.filter(|&q| g.player(q).alive).unwrap_or(p);
    draw(g, q, 2, false)?;
    lose_life(g, q, 2, Some(p), "other", None)?;
    let host = ctx.host.filter(|&m| live_creature(g, m)).or_else(|| best_creature(g, p));
    if let Some(m) = host {
        plus_counter(g, m, 1);
    }
    Ok("gy")
}

fn best_creature(g: &Game, p: PlayerId) -> Option<PermId> {
    let cs: Vec<PermId> = g.player(p).perms.iter().copied().filter(|&m| live_creature(g, m)).collect();
    max_by(&cs, |m| pval(g, m))
}

/// with Sheoldred out, the opponent a draw spell should hand cards to: the lowest life total (the drain can kill
/// them first), who has cards left to draw
fn drain_target(g: &Game, p: PlayerId, cards: usize) -> Option<PlayerId> {
    if !sheoldred_out(g, p) {
        return None;
    }
    let opps: Vec<PlayerId> = g.opps(p).filter(|&q| g.player(q).alive && g.player(q).library.len() > cards).collect();
    min_by(&opps, |q| g.player(q).life as f64)
}

fn cost_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if human(g, p) || post.is_none() || g.active != Some(p) || !affordable(g, p, c) {
        return Ok(vec![]);
    }
    let host = best_creature(g, p);
    if let Some(q) = drain_target(g, p, 2) {
        // each card they draw costs them 2 life, and the spell 2 more: six in all
        let u = 5.0 + if g.player(q).life <= 6 { 3.0 } else { 0.0 } + if host.is_some() { 0.3 } else { 0.0 };
        return Ok(vec![Opt {
            utility: u,
            label: format!("Cost of Brilliance ({} draws two)", g.player(q).name),
            act: Some(Action::Plan { f: cost_plan, arg: c.0 as i64 | (((q.0 as i64) + 1) << 16) }),
        }]);
    }
    let pl = g.player(p);
    if pl.life <= 6 || pl.library.len() < 6 {
        return Ok(vec![]);
    }
    let mut u = 3.2 + if pl.hand.len() <= 2 { 1.2 } else { 0.0 } - if pl.life < 12 { 1.0 } else { 0.0 };
    if pl.hand.len() >= 6 {
        u -= 1.5;
    }
    u += if host.is_some() { 0.3 } else { 0.0 };
    Ok(vec![Opt {
        utility: u,
        label: "Cost of Brilliance (draw two)".into(),
        act: Some(Action::Plan { f: cost_plan, arg: c.0 as i64 }),
    }])
}

fn cost_plan(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId((arg & 0xffff) as u16);
    let face = match arg >> 16 {
        0 => p,
        k => PlayerId((k - 1) as u8),
    };
    let host = best_creature(g, p);
    crate::ai::brain::do_cast_with(g, p, c, None, Ctx { face: Some(face), host, ..Ctx::default() })
}

// ------------------------------------------------------------------ Foolish Fate
/// Foolish Fate: destroy target creature; infusion: if you gained life this turn, its controller loses 3 life
fn fate_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let Some(t) = ctx.target.filter(|&t| on_bf(g, t)) else { return Ok("gy") };
    let ctrl = owner(g, t);
    let infused = gained_life(g, p);
    apply_removal(g, Some(p), t, "destroy", Some(c))?;
    if infused {
        lose_life(g, ctrl, 3, Some(p), "other", None)?;
    }
    Ok("gy")
}

// ------------------------------------------------------------------ Canyon Crawler
/// Canyon Crawler: a Food token as it enters
fn crawler_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src && trigger_window(g, owner(g, src), Some(src), "a Food token", None)? {
        make_artifact_tokens(g, owner(g, src), "Food", 1)?;
    }
    let _ = p;
    Ok(())
}

/// swampcycling {2}: when the hand needs lands, discard it and fetch a Swamp
fn crawler_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if human(g, p) || !can_pay(g, p, 2, "", false) || post == Some(true) {
        return Ok(vec![]);
    }
    let pl = g.player(p);
    let lands = pl.lands.len() + pl.hand.iter().filter(|&&x| g.db.get(x).land).count();
    if !pl.library.iter().any(|&x| name_of(g, x) == "Swamp") || lands >= 7 {
        return Ok(vec![]);
    }
    let hand_lands = pl.hand.iter().filter(|&&x| g.db.get(x).land).count();
    let u = if lands <= 4 && hand_lands == 0 {
        4.5
    } else if lands <= 5 && hand_lands == 0 {
        2.5
    } else {
        return Ok(vec![]);
    };
    Ok(vec![Opt {
        utility: u,
        label: "Canyon Crawler (swampcycling)".into(),
        act: Some(Action::Plan { f: crawler_cycle, arg: c.0 as i64 }),
    }])
}

fn crawler_cycle(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 2, "", false) {
        return Ok(false);
    }
    crate::engine::mana::pay(g, p, 2, "", false)?;
    crate::glog!(g, "  {} swampcycles Canyon Crawler", g.player(p).name);
    discard_cards(g, p, &[c])?;
    if let Some(s) = g.player(p).library.iter().rev().copied().find(|&x| name_of(g, x) == "Swamp") {
        remove_card(&mut g.player_mut(p).library, s);
        g.player_mut(p).hand.push(s);
    }
    shuffle_library(g, p);
    Ok(true)
}

// ------------------------------------------------------------------ Mabel, Vault Plunderer, Seymour, Bugenhagen
/// Mabel, Bitter Recluse: remove up to three counters from another target creature or planeswalker: +1/+1 counters
/// off the best opposing creature, -1/-1 counters off one of mine, loyalty off an opposing planeswalker
fn mabel_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let p = owner(g, src);
    let mut best: Option<(f64, PermId)> = None;
    let consider = |v: f64, t: PermId, best: &mut Option<(f64, PermId)>| {
        if v >= 1.0 && best.is_none_or(|b| v > b.0) {
            *best = Some((v, t));
        }
    };
    for q in g.players.iter().filter(|x| x.alive).map(|x| x.id).collect::<Vec<_>>() {
        for &t in &g.player(q).perms {
            if t == src || !on_bf(g, t) || (q != p && untargetable(g, t)) {
                continue;
            }
            if q != p && is_walker(g, t) {
                consider(g.perm(t).loyalty.unwrap_or(0).min(3) as f64, t, &mut best);
            } else if q != p && g.is_creature(t) && g.perm(t).plus > 0 {
                consider(g.perm(t).plus.min(3) as f64 * (1.0 + pval(g, t) / 8.0), t, &mut best);
            } else if q == p && g.is_creature(t) && g.perm(t).plus < 0 {
                consider((-g.perm(t).plus).min(3) as f64, t, &mut best);
            }
        }
    }
    let Some((_, t)) = best else { return Ok(()) };
    if !trigger_window(g, p, Some(src), "remove up to three counters from another target", Some(3.0))? || !on_bf(g, t) {
        return Ok(());
    }
    if owner(g, t) != p {
        crime(g, p)?;
    }
    if is_walker(g, t) {
        let l = g.perm(t).loyalty.unwrap_or(0) - 3;
        g.perm_mut(t).loyalty = Some(l);
        if l <= 0 {
            let o = owner(g, t);
            leave(g, t)?;
            to_zone_card(g, t, Zone::Gy);
            let nm = g.perm(t).name;
            *g.player_mut(o).lost_names.entry(nm).or_insert(0) += 1;
        }
    } else {
        let x = g.perm_mut(t);
        x.plus = if x.plus > 0 { x.plus - x.plus.min(3) } else { x.plus + (-x.plus).min(3) };
    }
    Ok(())
}

/// Vault Plunderer: target player draws a card and loses 1 life (you, unless you are nearly dead; the opponent with the
/// lowest life while Sheoldred is out)
fn plunderer_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let p = owner(g, src);
    let opps: Vec<PlayerId> = g.opps(p).collect();
    let target = if let Some(q) = drain_target(g, p, 1) {
        q // Sheoldred is out: an opponent draws and loses 3 in all
    } else if g.player(p).life > 3 && g.player(p).library.len() > 2 {
        p
    } else if let Some(q) = min_by(&opps, |q| g.player(q).life as f64) {
        q
    } else {
        return Ok(());
    };
    if trigger_window(g, p, Some(src), "target player draws a card and loses 1 life", Some(3.0))? {
        if target != p {
            crime(g, p)?;
        }
        draw(g, target, 1, false)?;
        lose_life(g, target, 1, Some(p), "other", None)?;
    }
    Ok(())
}

/// Seymour Flux: at the beginning of your upkeep, you may pay 1 life: draw a card and a +1/+1 counter on it
fn seymour_upkeep(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != owner(g, src) || g.player(p).life <= 8 || g.player(p).library.len() < 3 {
        return Ok(());
    }
    if trigger_window(g, p, Some(src), "pay 1 life: draw a card and a +1/+1 counter", Some(4.0))? && on_bf(g, src) {
        lose_life(g, p, 1, Some(p), "other", None)?;
        draw(g, p, 1, false)?;
        plus_counter(g, src, 1);
    }
    Ok(())
}

// ------------------------------------------------------------------ Al Bhed Salvagers
/// the artifacts p controls: the permanents (cards and artifact tokens) plus the Treasure, Food, Clue and Blood tokens
pub fn artifact_count(g: &Game, p: PlayerId) -> i32 {
    let pl = g.player(p);
    let perms = pl
        .perms
        .iter()
        .filter(|&&m| {
            let x = g.perm(m);
            !x.phased
                && (x.cd.is_some_and(|c| g.db.get(c).types.has(Types::ARTIFACT)) || x.ttypes.contains(&"artifact"))
        })
        .count();
    (perms as u32 + pl.treasures + pl.foods + pl.clues + pl.bloods) as i32
}

/// a creature or artifact (a permanent that has just left the battlefield keeps its card)
fn creature_or_artifact(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    g.is_creature(m) || x.cd.is_some_and(|c| g.db.get(c).types.has(Types::ARTIFACT)) || x.ttypes.contains(&"artifact")
}

// ------------------------------------------------------------------ Al Bhed Salvagers
/// the weakest opponent (lowest life, the first of equals)
fn weakest_opp(g: &Game, p: PlayerId) -> Option<PlayerId> {
    g.opps(p).min_by_key(|&q| g.player(q).life)
}

fn salvagers_drain(g: &mut Game, src: Src) -> Res {
    let o = owner(g, src);
    if !trigger_window(g, o, Some(src), "drain 1", None)? {
        return Ok(());
    }
    if let Some(q) = weakest_opp(g, o) {
        lose_life(g, q, 1, Some(o), "drain", Some(false))?;
    }
    gain(g, o, 1)
}

/// another creature or artifact you control dies
fn salvagers_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    if m != src && owner(g, m) == owner(g, src) && creature_or_artifact(g, m) {
        salvagers_drain(g, src)?;
    }
    Ok(())
}

/// the Salvagers themselves die
fn salvagers_self(g: &mut Game, src: Src, _m: PermId, _cause: Sym) -> Res {
    salvagers_drain(g, src)
}

/// a Clue, Treasure, Food or Blood token of its controller is sacrificed
fn salvagers_token(g: &mut Game, src: Src, p: PlayerId, what: Sacrificed) -> Res {
    if p == owner(g, src) && matches!(what, Sacrificed::Token(_)) && on_bf(g, src) {
        salvagers_drain(g, src)?;
    }
    Ok(())
}

// ------------------------------------------------------------------ Necrosquito
fn oil(g: &Game, m: PermId) -> i32 {
    g.perm(m).data.int(DataKey::Oil) as i32
}

/// as it enters (before the state check sees a 0/0): two oil counters, and the size rule on
fn necro_as_enters(g: &mut Game, _p: PlayerId, m: PermId) -> Res {
    g.selfpt = true;
    g.perm_mut(m).data.set(DataKey::Oil, Val::Int(2));
    Ok(())
}

fn necro_oil(g: &mut Game, src: Src) -> Res {
    let o = owner(g, src);
    if trigger_window(g, o, Some(src), "put an oil counter on it", Some(1.0))? && on_bf(g, src) {
        let n = oil(g, src) + 1;
        g.perm_mut(src).data.set(DataKey::Oil, Val::Int(n as i64));
    }
    Ok(())
}

fn necro_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    if m != src && owner(g, m) == owner(g, src) && creature_or_artifact(g, m) {
        necro_oil(g, src)?;
    }
    Ok(())
}

fn necro_token(g: &mut Game, src: Src, p: PlayerId, what: Sacrificed) -> Res {
    if p == owner(g, src) && matches!(what, Sacrificed::Token(_)) && on_bf(g, src) {
        necro_oil(g, src)?;
    }
    Ok(())
}

fn necro_pt(g: &Game, _p: PlayerId, m: PermId) -> (i32, i32) {
    let k = oil(g, m);
    (k, k)
}

// ------------------------------------------------------------------ Callous Inspector
/// when it dies it deals 1 damage to you and you create a Clue
fn inspector_dies(g: &mut Game, _src: Src, m: PermId, _cause: Sym) -> Res {
    let o = owner(g, m);
    if trigger_window(g, o, Some(m), "1 damage to you and a Clue", None)? {
        lose_life(g, o, 1, Some(o), "burn", None)?;
        crate::engine::zones::make_artifact_tokens(g, o, "Clue", 1)?;
    }
    Ok(())
}

// ------------------------------------------------------------------ Apex Witchstalker
fn apex_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src && trigger_window(g, p, Some(src), "gain 2 life", None)? {
        gain(g, p, 2)?;
    }
    Ok(())
}

fn apex_dies(g: &mut Game, _src: Src, m: PermId, _cause: Sym) -> Res {
    let o = owner(g, m);
    if trigger_window(g, o, Some(m), "gain 2 life", None)? {
        gain(g, o, 2)?;
    }
    Ok(())
}

/// basic landcycling {2}: when the hand has no land to play and the Witchstalker can't be cast soon
fn apex_hand(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if !can_pay(g, p, 2, "", false) || g.player(p).library.is_empty() {
        return Ok(vec![]);
    }
    let lands_in_hand = g.player(p).hand.iter().filter(|&&x| g.db.get(x).land).count();
    let on_bf = g.player(p).lands.len();
    let u = if lands_in_hand == 0 && on_bf < 6 {
        2.4
    } else if on_bf < 5 && lands_in_hand < 2 {
        1.2
    } else if post.is_none() && on_bf < 8 {
        0.6
    } else {
        return Ok(vec![]);
    };
    Ok(vec![Opt {
        utility: u,
        label: "Apex Witchstalker (basic landcycling)".into(),
        act: Some(Action::Plan { f: apex_cycle, arg: c.0 as i64 }),
    }])
}

fn apex_cycle(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 2, "", false) {
        return Ok(false);
    }
    pay(g, p, 2, "", false)?;
    crate::glog!(g, "  {} basic landcycles Apex Witchstalker", pname(g, p));
    discard_cards(g, p, &[c])?;
    land_to_hand(g, p)?;
    Ok(true)
}
// ------------------------------------------------------------------ size rules: Gravblade Heavy, Ulna Alley Shopkeep, Bloodseeker
fn gravblade_pt(g: &Game, p: PlayerId, _m: PermId) -> (i32, i32) {
    (i32::from(artifact_count(g, p) > 0), 0)
}

/// deathtouch is read from the permanent's flag in combat: keep it in step with the artifact count
fn gravblade_sba(g: &mut Game, src: Src) -> Res {
    let want = artifact_count(g, owner(g, src)) > 0;
    if g.perm(src).dt != want {
        g.perm_mut(src).dt = want;
    }
    Ok(())
}

fn gravblade_kw(g: &Game, src: Src, m: PermId, k: Sym) -> bool {
    m == src && k == "deathtouch" && artifact_count(g, owner(g, src)) > 0
}

fn ulna_pt(g: &Game, p: PlayerId, _m: PermId) -> (i32, i32) {
    (if gained_life(g, p) { 2 } else { 0 }, 0)
}

fn bloodseeker_pt(g: &Game, _p: PlayerId, m: PermId) -> (i32, i32) {
    (if flipped(g, m) { 2 } else { 0 }, 0)
}

// ------------------------------------------------------------------ Restless Bloodseeker and Blood tokens
/// at the beginning of your end step, if you gained life this turn: a Blood token
fn seeker_end(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p == owner(g, src)
        && on_bf(g, src)
        && gained_life(g, p)
        && trigger_window(g, p, Some(src), "create a Blood token", Some(1.0))?
    {
        make_blood(g, p, 1)?;
    }
    Ok(())
}

/// n Blood tokens (a count on the player, like Clues)
pub fn make_blood(g: &mut Game, p: PlayerId, n: i32) -> Res {
    let n = n * crate::dsl::token_mult(g, p) as i32;
    if n <= 0 {
        return Ok(());
    }
    g.player_mut(p).bloods += n as u32;
    if !g.hooks.is_empty() {
        crate::engine::hooks::fire_trigger(
            g,
            Event::TokenCreated,
            Call::ArtifactTokens { p, kinds: vec!["Blood"], n },
        )?;
    }
    Ok(())
}

fn sac_blood(g: &mut Game, p: PlayerId, n: u32) -> Res<bool> {
    if g.player(p).bloods < n {
        return Ok(false);
    }
    g.player_mut(p).bloods -= n;
    for _ in 0..n {
        if !g.hooks.is_empty() {
            crate::engine::hooks::fire_trigger(
                g,
                Event::Sacrifice,
                Call::Sacrifice { p, what: Sacrificed::Token("Blood") },
            )?;
        }
    }
    Ok(true)
}

/// the Bloodseeker's plays: sacrifice two Blood tokens to transform (sorcery speed); once Bloodsoaked Reveler,
/// {4}{B}: each opponent loses 2 life and you gain 2; and a Blood's own ability ({1}, {T}, discard a card, sacrifice:
/// draw a card) when the hand has a card to spare
fn seeker_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let mut out = vec![];
    if owner(g, src) != p || !on_bf(g, src) || g.perm(src).neutered {
        return Ok(out);
    }
    if !flipped(g, src) {
        if post == Some(false) && g.active == Some(p) && g.player(p).bloods >= 2 {
            out.push(Opt {
                utility: 2.2,
                label: "Restless Bloodseeker: transform".into(),
                act: Some(Action::Plan { f: seeker_flip, arg: src.0 as i64 }),
            });
        }
    } else if post.is_some() && can_pay(g, p, 4, "B", false) {
        if let Some(q) = weakest_opp(g, p) {
            out.push(Opt {
                utility: if g.player(q).life <= 2 { 9.0 } else { 1.0 },
                label: "Bloodsoaked Reveler: drain 2".into(),
                act: Some(Action::Plan { f: reveler_drain, arg: src.0 as i64 }),
            });
        }
    }
    // a Blood's rummage: discard a card of no use to draw a new one
    if g.player(p).bloods > 0
        && can_pay(g, p, 1, "", false)
        && !g.player(p).hand.is_empty()
        && !g.player(p).library.is_empty()
    {
        let hand = g.player(p).hand.clone();
        let worst = min_by(&hand, |c| card_worth(g, p, c, false));
        if worst.is_some_and(|c| card_worth(g, p, c, false) < 15.0) {
            out.push(Opt {
                utility: 1.1,
                label: "rummage with a Blood".into(),
                act: Some(Action::Plan { f: blood_rummage, arg: 0 }),
            });
        }
    }
    Ok(out)
}

fn seeker_flip(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let m = PermId(arg as u32);
    if !on_bf(g, m) || flipped(g, m) || g.player(p).bloods < 2 {
        return Ok(false);
    }
    sac_blood(g, p, 2)?;
    if ability_window(g, p, Some(m), "transform", Some(2.0), None)? && on_bf(g, m) {
        g.perm_mut(m).data.set(DataKey::Flipped, Val::Bool(true));
        g.selfpt = true;
        crate::glog!(g, "  Restless Bloodseeker transforms into Bloodsoaked Reveler");
    }
    Ok(true)
}

fn reveler_drain(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let m = PermId(arg as u32);
    if !on_bf(g, m) || !flipped(g, m) || !can_pay(g, p, 4, "B", false) {
        return Ok(false);
    }
    pay(g, p, 4, "B", false)?;
    if ability_window(g, p, Some(m), "each opponent loses 2 life and you gain 2", Some(3.0), None)? {
        let opps: Vec<PlayerId> = g.opps(p).collect();
        for q in opps {
            lose_life(g, q, 2, Some(p), "drain", Some(false))?;
        }
        gain(g, p, 2)?;
    }
    Ok(true)
}

fn blood_rummage(g: &mut Game, p: PlayerId, _arg: i64) -> Res<bool> {
    if g.player(p).bloods == 0 || g.player(p).hand.is_empty() || !can_pay(g, p, 1, "", false) {
        return Ok(false);
    }
    pay(g, p, 1, "", false)?;
    discard_worst(g, p, 1)?;
    sac_blood(g, p, 1)?;
    draw(g, p, 1, false)?;
    Ok(true)
}
// ------------------------------------------------------------------ End of the Hunt
/// the opponent whose greatest-mana-value creature or planeswalker is worth the most loses it; they exile the least
/// valuable among those tied for the greatest mana value
fn hunt_pick(g: &Game, p: PlayerId) -> Option<(PlayerId, PermId)> {
    let mut best: Option<(PlayerId, PermId, f64)> = None;
    for q in g.opps(p) {
        let cands: Vec<PermId> = g
            .player(q)
            .perms
            .iter()
            .copied()
            .filter(|&m| {
                on_bf(g, m)
                    && (g.is_creature(m) || g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(Types::PLANESWALKER)))
            })
            .collect();
        let mv = |m: PermId| g.perm(m).cd.map_or(0, |c| g.db.get(c).cmc);
        let Some(top) = cands.iter().map(|&m| mv(m)).max() else { continue };
        let tied: Vec<PermId> = cands.into_iter().filter(|&m| mv(m) == top).collect();
        let Some(m) = min_by(&tied, |m| pval(g, m)) else { continue };
        let v = pval(g, m);
        if best.is_none_or(|b| v > b.2) {
            best = Some((q, m, v));
        }
    }
    best.map(|b| (b.0, b.1))
}

fn hunt_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &crate::state::Ctx) -> Res<Sym> {
    if let Some((q, m)) = hunt_pick(g, p) {
        crate::glog!(g, "    {} exiles {}", pname(g, q), g.perm(m).name);
        exile_perm(g, m)?;
    }
    Ok("gy")
}

fn hunt_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    match hunt_pick(g, p) {
        Some((_, m)) if pval(g, m) >= 2.0 => (45.0 + 4.0 * pval(g, m)).min(75.0) as i32,
        _ => 0,
    }
}

// ------------------------------------------------------------------ Phyrexian Delver, Soul Enervation, Perigee Beckoner
/// what a creature card in the graveyard is worth to bring back: the table's bomb value or its size, its mana value,
/// and the value of its own enter / death triggers
fn cv(g: &Game, c: CardId) -> f64 {
    let d = g.db.get(c);
    let mut v = if d.bomb != 0 { d.bomb as f64 } else { 1.0 + 0.5 * d.pow as f64 };
    v += 0.15 * d.cmc as f64;
    v += match &*d.name {
        DELVER => 2.5,
        "Dread Drone" => 1.8,
        "Blood Artist" => 1.2,
        "Accursed Marauder" => 1.0,
        BECKONER => 0.8,
        "Phyrexian Rager" | "Burglar Rat" | "Undercity Dire Rat" | "Death Cultist" | "Festering Goblin" => 0.7,
        "Carrion Feeder" | "Sangrophage" => 0.4,
        _ => 0.0,
    };
    v
}

/// the best creature card of p's graveyard that `pred` accepts
fn best_gy(g: &Game, p: PlayerId, pred: impl Fn(&Game, CardId) -> bool) -> Option<CardId> {
    let cs: Vec<CardId> = gy_creatures(g, p).into_iter().filter(|&c| pred(g, c)).collect();
    max_by(&cs, |c| cv(g, c))
}

/// a graveyard card enters the battlefield under p's control. Returns the permanent.
fn reanimate(g: &mut Game, p: PlayerId, c: CardId) -> Res<Option<PermId>> {
    let Some(i) = g.player(p).gy.iter().position(|&x| x == c) else { return Ok(None) };
    g.player_mut(p).gy.remove(i);
    g.player_mut(p).stat("rean_resolved", 1);
    let m = enter(g, p, c, Enter::default())?;
    crate::glog!(g, "    {} returns to the battlefield", name_of(g, c));
    Ok(Some(m))
}

/// suspect m: menace, and it can't block (until it leaves the battlefield)
fn suspect(g: &mut Game, m: PermId) {
    g.perm_mut(m).data.set(DataKey::Suspect, Val::Bool(true));
    crate::glog!(g, "    {} is suspected", g.perm(m).name);
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

/// `zones::die_rest`: a creature that died with Presumed Dead's or Perigee Beckoner's grant on it returns to the
/// battlefield under its owner's control (tapped, or suspected). `x` is the permanent as it died. True if it returned.
pub fn returns_when_dies(g: &mut Game, x: &Perm, _cause: Sym) -> Res<bool> {
    let tapped = x.eot_kw.contains(&"ret_dies_tapped");
    let susp = x.eot_kw.contains(&"ret_dies_suspect");
    let Some(cd) = x.cd else { return Ok(false) };
    if (!tapped && !susp) || x.token || x.is_cmd || !returns_left(g, cd) {
        return Ok(false);
    }
    if !g.hooks.is_empty() && crate::engine::hooks::total_player(g, Event::NoGraveyard, x.orig) != 0 {
        return Ok(false);
    }
    let n = enter(g, x.orig, cd, Enter::default())?;
    crate::glog!(g, "    {} returns to the battlefield", name_of(g, cd));
    if tapped {
        g.perm_mut(n).tapped = true;
    }
    if susp {
        suspect(g, n);
    }
    Ok(true)
}

fn grant_eot(g: &mut Game, m: PermId, kind: &str) {
    let k = intern(kind);
    let x = g.perm_mut(m);
    if !x.eot_kw.contains(&k) {
        x.eot_kw.push(k);
    }
}
// ------------------------------------------------------------------ Soul Enervation
/// -4/-4 on the opposing creature it kills (the most valuable), else the one it shrinks most
fn enervation_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = g.perm(src).owner;
    snapshot(g, src);
    let tg: Vec<PermId> = opp_creatures(g, o).into_iter().filter(|&x| !untargetable(g, x)).collect();
    let kill: Vec<PermId> = tg.iter().copied().filter(|&x| etgh(g, x) <= 4).collect();
    let pick = max_by(&kill, |x| pval(g, x)).or_else(|| max_by(&tg, |x| pval(g, x)));
    if let Some(x) = pick
        && trigger_window(g, o, Some(src), "-4/-4 until end of turn", Some(4.0))?
    {
        shrink(g, o, x, 4)?;
    }
    Ok(())
}

/// the creature cards in p's graveyard as of the last look
fn snapshot(g: &mut Game, src: PermId) {
    let o = g.perm(src).owner;
    let cur: Vec<Val> = gy_creatures(g, o).into_iter().map(|c| Val::Int(c.0 as i64)).collect();
    g.perm_mut(src).data.set(DataKey::Held, Val::List(cur));
}

/// whenever one or more creature cards leave your graveyard, each opponent loses 1 life and you gain 1 life. Seen as a
/// state check: a creature card of the last look that is no longer in the graveyard left it (reanimated, returned to
/// hand, cast, exiled).
fn enervation_sba(g: &mut Game, src: Src) -> Res {
    let x = g.perm(src);
    if !x.on_bf || x.phased || x.neutered {
        return Ok(());
    }
    let o = x.owner;
    let Some(Val::List(prev)) = x.data.get(DataKey::Held).cloned() else {
        snapshot(g, src);
        return Ok(());
    };
    let left = prev.iter().any(|v| {
        let c = CardId(v.int() as u16);
        !g.player(o).gy.contains(&c)
    });
    snapshot(g, src);
    if left && g.player(o).alive {
        crate::glog!(g, "    Soul Enervation: each opponent loses 1 life, {} gains 1", pname(g, o));
        for q in g.opps(o).collect::<Vec<_>>() {
            lose_life(g, q, 1, Some(o), "drain", None)?;
        }
        gain(g, o, 1)?;
    }
    Ok(())
}

// ------------------------------------------------------------------ the reanimation spells
/// the best graveyard creature card that costs no more than `life` - 6 to bring back with a life loss of its mana value
fn rean_pick(g: &Game, p: PlayerId, max_mv: u32, pay_life: bool) -> Option<CardId> {
    let life = g.player(p).life;
    best_gy(g, p, |g, c| {
        let d = g.db.get(c);
        d.cmc <= max_mv && (!pay_life || life - d.cmc as i32 >= 6)
    })
}

/// Rewrite Regrets: return target creature or planeswalker card with mana value 6 or less from your graveyard to the

/// Phyrexian Delver: when it enters, return target creature card from your graveyard to the battlefield; you lose
/// life equal to its mana value
fn delver_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = g.perm(src).owner;
    let Some(c) = rean_pick(g, o, 99, true) else { return Ok(()) };
    if !trigger_window(g, o, Some(src), "return a creature card from your graveyard", Some(5.0))? {
        return Ok(());
    }
    if reanimate(g, o, c)?.is_some() {
        let mv = g.db.get(c).cmc as i32;
        lose_life(g, o, mv, Some(o), "other", None)?;
    }
    Ok(())
}

/// Send in the Pest: each opponent discards a card (their choice); you create a 1/1 black and green Pest (it gains
/// Perigee Beckoner: when it enters, until end of turn another target creature you control gets +2/+0 and gains "When
/// this creature dies, return it to the battlefield tapped under its owner's control."
fn beckoner_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = g.perm(src).owner;
    let cs: Vec<PermId> = my_creatures(g, o).into_iter().filter(|&x| x != src).collect();
    let Some(t) = max_by(&cs, |x| pval(g, x) + if g.perm(x).sick { 0.0 } else { 1.0 }) else { return Ok(()) };
    if trigger_window(g, o, Some(src), "+2/+0 and returns when it dies", Some(3.0))? && g.perm(t).on_bf {
        g.perm_mut(t).eot_pt.0 += 2;
        grant_eot(g, t, "ret_dies_tapped");
    }
    Ok(())
}

/// the warped Beckoner is exiled at the beginning of the next end step; its card can be cast from exile later
fn beckoner_end(g: &mut Game, src: Src, _p: PlayerId) -> Res {
    if !g.perm(src).on_bf || !g.perm(src).data.truthy(DataKey::Warp) {
        return Ok(());
    }
    let o = g.perm(src).owner;
    let c = g.perm(src).cd.unwrap();
    g.perm_mut(src).data.remove(DataKey::Warp);
    crate::glog!(g, "    {} (warp) is exiled", name_of(g, c));
    crate::engine::zones::exile_perm(g, src)?;
    let st = g.turn_stamp();
    g.player_mut(o).flag_turn.insert(intern("beckoner_warped"), st);
    if !g.player(o).exile.contains(&c) {
        g.player_mut(o).exile.push(c);
    }
    Ok(())
}

fn warp_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 1, "B", false) {
        return Ok(false);
    }
    pay(g, p, 1, "B", false)?;
    cast_card(g, p, c, "hand", Ctx::default())?;
    let found = g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && g.perm(m).cd == Some(c));
    if let Some(m) = found {
        g.perm_mut(m).data.set(DataKey::Warp, Val::Bool(true));
    }
    Ok(true)
}

fn exile_cast_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let (cg, cp) = cost_of(g, p, c);
    let Some(i) = g.player(p).exile.iter().position(|&x| x == c) else { return Ok(false) };
    if !can_pay(g, p, cg, &cp, false) {
        return Ok(false);
    }
    pay(g, p, cg, &cp, false)?;
    g.player_mut(p).exile.remove(i);
    cast_card(g, p, c, "exile", Ctx::default())?;
    Ok(true)
}

/// Arnim Zola, Bio-Fanatic: {3}, {T}: create a tapped 2/1 black Villain creature token with menace. Activate only if

// ------------------------------------------------------------------ Locust Spray
/// Locust Spray: target creature gets -1/-1 until end of turn
fn spray_resolve(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    if let Some(t) = ctx.target.filter(|&t| live_creature(g, t)) {
        shrink(g, p, t, 1)?;
    }
    Ok("gy")
}

/// cast at an opposing creature the -1/-1 kills; otherwise cycle it ({B}) once the turn's real plays are made
fn spray_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if human(g, p) || g.active != Some(p) || !affordable(g, p, c) {
        return Ok(vec![]);
    }
    let tg: Vec<PermId> = legal_targets(g, p, "destroy", "c", false, Some(c))
        .into_iter()
        .filter(|&m| etgh(g, m) <= 1 && pval(g, m) >= 1.0)
        .collect();
    if let Some(t) = max_by(&tg, |m| pval(g, m)) {
        return Ok(vec![Opt {
            utility: pval(g, t) - 3.0,
            label: format!("Locust Spray -> {}", g.perm(t).name),
            act: Some(Action::Plan { f: spray_cast, arg: c.0 as i64 | ((t.0 as i64) << 16) }),
        }]);
    }
    if post == Some(true) && !g.player(p).library.is_empty() {
        return Ok(vec![Opt {
            utility: 0.9,
            label: "Locust Spray (cycling)".into(),
            act: Some(Action::Plan { f: spray_cycle, arg: c.0 as i64 }),
        }]);
    }
    Ok(vec![])
}

fn spray_cast(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (c, t) = (CardId((arg & 0xffff) as u16), PermId((arg >> 16) as u32));
    if !g.player(p).hand.contains(&c) || !live_creature(g, t) {
        return Ok(false);
    }
    crate::ai::brain::do_cast_with(g, p, c, None, Ctx { target: Some(t), ..Ctx::default() })
}

fn spray_cycle(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 0, "B", false) {
        return Ok(false);
    }
    pay(g, p, 0, "B", false)?;
    crate::glog!(g, "  {} cycles Locust Spray", pname(g, p));
    discard_cards(g, p, &[c])?;
    draw(g, p, 1, false)?;
    Ok(true)
}

// ------------------------------------------------------------------ Ribskiff
/// Ribskiff: when it enters, draw a card
fn ribskiff_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = owner(g, src);
    if m == src && trigger_window(g, o, Some(src), "draw a card", None)? {
        draw(g, o, 1, false)?;
    }
    Ok(())
}

/// crew 3 with creatures that would not attack anyway (summoning sick, or weaker than the 4/4); never Sheoldred
fn ribskiff_crew(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let x = g.perm(src);
    if p != x.owner || x.tapped || x.sick || !x.on_bf || x.data.truthy(DataKey::Anim) {
        return Ok(());
    }
    let mut cands: Vec<PermId> = my_creatures(g, p)
        .into_iter()
        .filter(|&m| !g.perm(m).tapped && m != src && !g.perm(m).is_cmd && !sheoldred_card(g, m))
        .collect();
    cands.sort_by(|&a, &b| {
        (pval(g, a), -epow(g, a)).partial_cmp(&(pval(g, b), -epow(g, b))).unwrap_or(std::cmp::Ordering::Equal)
    });
    let (mut power, mut pick) = (0, vec![]);
    for m in cands {
        if power >= 3 {
            break;
        }
        if g.perm(m).sick || g.perm(m).noatk || epow(g, m) < 4 {
            pick.push(m);
            power += epow(g, m);
        }
    }
    if power < 3 || !super::lands::open_attack(g, p, 4, false) {
        return Ok(());
    }
    for m in pick {
        g.perm_mut(m).tapped = true;
    }
    let st = g.turn_stamp();
    let s = g.perm_mut(src);
    s.data.set(DataKey::Anim, Val::Bool(true));
    s.data.set(DataKey::Crewed, Val::Stamp(st));
    s.pow = 4;
    s.tgh = 4;
    g.bf_ver += 1;
    crate::glog!(g, "  {} crews Ribskiff", pname(g, p));
    Ok(())
}

fn sheoldred_card(g: &Game, m: PermId) -> bool {
    g.perm(m).cd.is_some_and(|c| name_of(g, c) == "Sheoldred, the Apocalypse")
}

/// toxic 2: two poison counters with the combat damage to a player
fn ribskiff_damage(g: &mut Game, src: Src, _p: PlayerId, a: PermId, d: PlayerId, dmg: i32) -> Res {
    if a == src && dmg > 0 && !crate::engine::values::melira(g, d) {
        g.player_mut(d).poison += 2;
        crate::glog!(g, "    {} gets 2 poison counters (toxic)", g.player(d).name);
    }
    Ok(())
}

// ------------------------------------------------------------------ Tome of Legends and Diary of Dreams
fn pages(g: &Game, m: PermId) -> i32 {
    g.perm(m).data.int(DataKey::Counters) as i32
}

fn add_pages(g: &mut Game, m: PermId, n: i32) {
    let k = g.perm(m).data.int(DataKey::Counters);
    g.perm_mut(m).data.set(DataKey::Counters, Val::Int(k + n as i64));
}

/// Tome of Legends: it enters with a page counter, and gets one whenever your commander enters
fn tome_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        add_pages(g, src, 1);
    } else if g.perm(m).is_cmd && g.perm(m).owner == owner(g, src) && on_bf(g, src) {
        add_pages(g, src, 1);
    }
    Ok(())
}

/// ... or attacks
fn tome_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if p == owner(g, src) && on_bf(g, src) && atk.iter().any(|&m| g.perm(m).is_cmd) {
        add_pages(g, src, 1);
    }
    Ok(vec![])
}

fn tome_prio(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    38
}

/// a draw is worth more with Sheoldred out (it gains 2); a full hand wants nothing
fn draw_utility(g: &Game, p: PlayerId) -> f64 {
    let pl = g.player(p);
    let mut u = 1.8 + if sheoldred_out(g, p) { 1.5 } else { 0.0 };
    if pl.hand.len() >= 6 {
        u -= 1.5;
    }
    u
}

fn tome_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if owner(g, src) != p || !x.on_bf || x.tapped || x.phased || x.neutered || post.is_none() {
        return Ok(vec![]);
    }
    if pages(g, src) < 1 || !can_pay(g, p, 1, "", false) || g.player(p).library.is_empty() {
        return Ok(vec![]);
    }
    let u = draw_utility(g, p);
    if u <= 0.0 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: u,
        label: "Tome of Legends: draw".into(),
        act: Some(Action::Ability { src, f: tome_go, arg: 0 }),
    }])
}

fn tome_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !on_bf(g, src) || g.perm(src).tapped || pages(g, src) < 1 || !can_pay(g, p, 1, "", false) {
        return Ok(false);
    }
    pay(g, p, 1, "", false)?;
    g.perm_mut(src).tapped = true;
    add_pages(g, src, -1);
    if ability_window(g, p, Some(src), "draw a card", Some(2.0), None)? {
        draw(g, p, 1, false)?;
    }
    Ok(true)
}

/// Diary of Dreams: a page counter whenever you cast an instant or sorcery spell
fn diary_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let d = g.db.get(c);
    if caster == owner(g, src) && on_bf(g, src) && (d.instant || d.sorcery) {
        add_pages(g, src, 1);
    }
    Ok(())
}

/// {5}, {T}: draw a card; it costs {1} less for each page counter
fn diary_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if owner(g, src) != p || !x.on_bf || x.tapped || x.phased || x.neutered || post.is_none() {
        return Ok(vec![]);
    }
    let cost = (5 - pages(g, src)).max(0) as u32;
    if !can_pay(g, p, cost, "", false) || g.player(p).library.is_empty() {
        return Ok(vec![]);
    }
    let u = draw_utility(g, p) - 0.1 * cost as f64;
    if u <= 0.0 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: u,
        label: "Diary of Dreams: draw".into(),
        act: Some(Action::Ability { src, f: diary_go, arg: 0 }),
    }])
}

fn diary_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    let cost = (5 - pages(g, src)).max(0) as u32;
    if !on_bf(g, src) || g.perm(src).tapped || !can_pay(g, p, cost, "", false) {
        return Ok(false);
    }
    pay(g, p, cost, "", false)?;
    g.perm_mut(src).tapped = true;
    if ability_window(g, p, Some(src), "draw a card", Some(2.0), None)? {
        draw(g, p, 1, false)?;
    }
    Ok(true)
}

// ------------------------------------------------------------------ Leech Collector // Bloodletting
/// the first time each turn you gain life, it becomes prepared
fn leech_gain(g: &mut Game, src: Src, p: PlayerId, n: i32) -> Res {
    if n > 0 && p == owner(g, src) && on_bf(g, src) && crate::engine::values::once_per_turn(g, p, intern("leech_prep"))
    {
        g.perm_mut(src).data.set(DataKey::Prepared, Val::Bool(true));
        crate::glog!(g, "    Leech Collector becomes prepared");
    }
    Ok(())
}

/// Bloodletting ({B}, sorcery): each opponent loses 2 life. Cast as a copy while Leech Collector is prepared.
fn leech_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if owner(g, src) != p
        || !x.on_bf
        || x.phased
        || !x.data.truthy(DataKey::Prepared)
        || post.is_none()
        || g.active != Some(p)
        || !can_pay(g, p, 0, "B", false)
        || human(g, p)
    {
        return Ok(vec![]);
    }
    let opps = g.opps(p).filter(|&q| g.player(q).alive).count() as f64;
    let near = g.opps(p).any(|q| g.player(q).alive && g.player(q).life <= 2);
    Ok(vec![Opt {
        utility: 1.4 + 0.5 * opps + if near { 6.0 } else { 0.0 },
        label: "Bloodletting (prepared copy)".into(),
        act: Some(Action::Ability { src, f: leech_go, arg: 0 }),
    }])
}

fn leech_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !g.perm(src).data.truthy(DataKey::Prepared) || !can_pay(g, p, 0, "B", false) {
        return Ok(false);
    }
    pay(g, p, 0, "B", false)?;
    g.perm_mut(src).data.set(DataKey::Prepared, Val::Bool(false));
    crate::impls::mine::cast_copy_with(g, p, None, None, false, 3.0, &mut |g| {
        crate::glog!(g, "  {} casts a copy of Bloodletting", g.player(p).name);
        for q in g.opps(p).collect::<Vec<_>>() {
            lose_life(g, q, 2, Some(p), "drain", None)?;
        }
        Ok(())
    })?;
    Ok(true)
}

// ------------------------------------------------------------------ Moseo, Vein's New Dean
/// Infusion: at the beginning of your end step, if you gained life this turn, return up to one creature card with
/// mana value X or less from your graveyard to the battlefield, X being the life you gained this turn
fn moseo_end(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != owner(g, src) || !on_bf(g, src) || !gained_life(g, p) {
        return Ok(());
    }
    let x = g.player(p).gained_turn.map_or(0, |t| t.1);
    let Some(c) = best_gy(g, p, |g, c| g.db.get(c).cmc as i32 <= x) else { return Ok(()) };
    if trigger_window(g, p, Some(src), "return a creature card from your graveyard", Some(cv(g, c)))?
        && g.player(p).gy.contains(&c)
    {
        reanimate(g, p, c)?;
    }
    Ok(())
}

// ------------------------------------------------------------------ Lethal Throwdown
/// the creature p would sacrifice to Lethal Throwdown: a modified one is best (the extra card), otherwise the cheapest
fn throwdown_fodder(g: &Game, p: PlayerId, target: PermId) -> Option<PermId> {
    let cands: Vec<PermId> = my_creatures(g, p)
        .into_iter()
        .filter(|&m| m != target && !g.perm(m).is_cmd && !crate::engine::values::no_sac(g, m))
        .collect();
    min_by(&cands, |m| crate::engine::zones::sac_worth(g, m) - if modified(g, p, m) { 1.5 } else { 0.0 })
}

/// a creature with a counter on it, or with Equipment or an Aura of yours attached
fn modified(g: &Game, p: PlayerId, m: PermId) -> bool {
    g.perm(m).plus != 0
        || g.perm(m).data.int(DataKey::Counters) != 0
        || g.player(p).perms.iter().any(|&a| a != m && g.perm(a).attached == Some(m))
}

fn throwdown_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if human(g, p) || post.is_none() || g.active != Some(p) || !affordable(g, p, c) {
        return Ok(vec![]);
    }
    let tg = legal_targets(g, p, "destroy", "cp", false, Some(c));
    let Some(t) = max_by(&tg, |m| pval(g, m)) else { return Ok(vec![]) };
    let Some(f) = throwdown_fodder(g, p, t) else { return Ok(vec![]) };
    let gain = pval(g, t) - crate::engine::zones::sac_worth(g, f) + if modified(g, p, f) { 1.5 } else { 0.0 };
    if gain < 1.5 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: gain - 3.0,
        label: format!("Lethal Throwdown -> {} (sacrifice {})", g.perm(t).name, g.perm(f).name),
        act: Some(Action::Plan { f: throwdown_plan, arg: c.0 as i64 | ((t.0 as i64) << 16) }),
    }])
}

/// pay the additional cost (sacrifice a creature, noting whether it was modified), then cast it at the target
fn throwdown_plan(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (c, t) = (CardId((arg & 0xffff) as u16), PermId((arg >> 16) as u32));
    if !g.player(p).hand.contains(&c) || !on_bf(g, t) || !affordable(g, p, c) {
        return Ok(false);
    }
    let Some(f) = throwdown_fodder(g, p, t) else { return Ok(false) };
    let was_modified = modified(g, p, f);
    crate::glog!(g, "  {} sacrifices {} to Lethal Throwdown", pname(g, p), g.perm(f).name);
    crate::engine::zones::die(g, f, "sac")?;
    crate::ai::brain::do_cast_with(g, p, c, None, Ctx { target: Some(t), x: was_modified as i32, ..Ctx::default() })
}

/// destroy target creature or planeswalker; if the sacrificed creature was modified, draw a card
fn throwdown_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    if let Some(t) = ctx.target.filter(|&t| on_bf(g, t)) {
        apply_removal(g, Some(p), t, "destroy", Some(c))?;
    }
    if ctx.x == 1 {
        draw(g, p, 1, false)?;
    }
    Ok("gy")
}

// ------------------------------------------------------------------ the AI plan
/// sheoldred's cast priority: the generic priorities, the recursion creatures only with something worth bringing
/// back, Soul Enervation with a creature to kill
pub fn sheoldred_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let d = g.db.get(c);
    match &*d.name {
        DELVER => {
            let ok = rean_pick(g, p, 99, true).map_or(0.0, |x| cv(g, x)) >= 2.5 && g.player(p).life > 14;
            if ok { 74 } else { 36 }
        }
        "Soul Enervation" => {
            let kill = opp_creatures(g, p).iter().any(|&m| etgh(g, m) <= 4 && pval(g, m) >= 2.0);
            if kill { 66 } else { 34 }
        }
        "Perigee Beckoner" => 52,
        _ => generic_prio(g, p, c),
    }
}

/// sheoldred's own plays (brain.special_options): the Beckoner's warp and its cast from exile
pub fn sheoldred_special(g: &mut Game, p: PlayerId, post: bool) -> Res<Vec<Opt>> {
    beckoner_options(g, p, post)
}

/// the plays from the hand and exile: warp the Beckoner, and cast it from exile on a later turn
fn beckoner_options(g: &mut Game, p: PlayerId, post: bool) -> Res<Vec<Opt>> {
    let mut out = vec![];
    if g.active != Some(p) {
        return Ok(out);
    }
    for c in g.player(p).hand.clone() {
        if name_of(g, c) == BECKONER && !post && can_pay(g, p, 1, "B", false) {
            // warp when the full cost can't be paid this turn and there is a creature to buff
            let (cg, cp) = cost_of(g, p, c);
            if !can_pay(g, p, cg, &cp, false) && !my_creatures(g, p).is_empty() {
                out.push(Opt {
                    utility: 2.2,
                    label: "Perigee Beckoner (warp)".into(),
                    act: Some(Action::Plan { f: warp_go, arg: c.0 as i64 }),
                });
            }
        }
    }
    let st = g.turn_stamp();
    let later = g.player(p).flag_turn.get(&intern("beckoner_warped")).is_none_or(|s| *s != st);
    if later && let Some(c) = g.player(p).exile.iter().copied().find(|&c| name_of(g, c) == BECKONER) {
        let (cg, cp) = cost_of(g, p, c);
        if can_pay(g, p, cg, &cp, false) {
            out.push(Opt {
                utility: 4.5,
                label: "Perigee Beckoner (from exile)".into(),
                act: Some(Action::Plan { f: exile_cast_go, arg: c.0 as i64 }),
            });
        }
    }
    Ok(out)
}

/// sheoldred's responses to removal aimed at its permanents: none yet
pub fn sheoldred_protect(
    _g: &mut Game,
    _p: PlayerId,
    _m: PermId,
    _kind: &str,
    _actor: Option<PlayerId>,
    _spell: Option<CardId>,
) -> Res<bool> {
    Ok(false)
}

const BECKONER: &str = "Perigee Beckoner";
const DELVER: &str = "Phyrexian Delver";

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, "Melancholic Poet")?.cast = Some(poet_cast);
    let c = r.card(db, "Cost of Brilliance")?;
    c.resolve = Some(cost_resolve);
    c.hand_options = Some(cost_options);
    c.prio = Some(zero_prio);
    r.card(db, "Foolish Fate")?.resolve = Some(fate_resolve);
    let c = r.card(db, "Canyon Crawler")?;
    c.etb = Some(crawler_etb);
    c.hand_options = Some(crawler_options);
    r.card(db, "Mabel, Bitter Recluse")?.etb = Some(mabel_etb);
    r.card(db, "Vault Plunderer")?.etb = Some(plunderer_etb);
    r.card(db, "Seymour Flux")?.upkeep = Some(seymour_upkeep);
    let c = r.card(db, "Al Bhed Salvagers")?;
    c.dies = Some(salvagers_dies);
    c.self_dies = Some(salvagers_self);
    c.sacrifice = Some(salvagers_token);
    let c = r.card(db, "Necrosquito")?;
    c.as_enters = Some(necro_as_enters);
    c.etb = Some(selfpt_etb);
    c.self_pt = Some(necro_pt);
    c.dies = Some(necro_dies);
    c.sacrifice = Some(necro_token);
    r.card(db, "Callous Inspector")?.self_dies = Some(inspector_dies);
    let c = r.card(db, "Apex Witchstalker")?;
    c.etb = Some(apex_etb);
    c.self_dies = Some(apex_dies);
    c.hand_options = Some(apex_hand);
    let c = r.card(db, "Gravblade Heavy")?;
    c.etb = Some(selfpt_etb);
    c.as_enters = Some(selfpt_as);
    c.self_pt = Some(gravblade_pt);
    c.sba = Some(gravblade_sba);
    c.grant_kw = Some(gravblade_kw);
    let c = r.card(db, "Ulna Alley Shopkeep")?;
    c.etb = Some(selfpt_etb);
    c.as_enters = Some(selfpt_as);
    c.self_pt = Some(ulna_pt);
    let c = r.card(db, "Restless Bloodseeker // Bloodsoaked Reveler")?;
    c.etb = Some(selfpt_etb);
    c.as_enters = Some(selfpt_as);
    c.self_pt = Some(bloodseeker_pt);
    c.end_step = Some(seeker_end);
    c.options = Some(seeker_options);
    let c = r.card(db, "End of the Hunt")?;
    c.resolve = Some(hunt_resolve);
    c.prio = Some(hunt_prio);
    let c = r.card(db, "Soul Enervation")?;
    c.etb = Some(enervation_etb);
    c.sba = Some(enervation_sba);
    r.card(db, "Phyrexian Delver")?.etb = Some(delver_etb);
    let c = r.card(db, BECKONER)?;
    c.etb = Some(beckoner_etb);
    c.end_step = Some(beckoner_end);
    let c = r.card(db, "Locust Spray")?;
    c.resolve = Some(spray_resolve);
    c.hand_options = Some(spray_options);
    c.prio = Some(zero_prio);
    let c = r.card(db, "Ribskiff")?;
    c.etb = Some(ribskiff_etb);
    c.crew = Some(ribskiff_crew);
    super::partials::at_once(c, Event::Crew, true);
    c.combat_damage = Some(ribskiff_damage);
    let c = r.card(db, "Tome of Legends")?;
    c.etb = Some(tome_etb);
    c.attack = Some(tome_attack);
    c.options = Some(tome_options);
    c.prio = Some(tome_prio);
    let c = r.card(db, "Diary of Dreams")?;
    c.cast = Some(diary_cast);
    c.options = Some(diary_options);
    c.prio = Some(tome_prio);
    let c = r.card(db, "Leech Collector // Bloodletting")?;
    c.gain_life = Some(leech_gain);
    c.options = Some(leech_options);
    r.card(db, "Moseo, Vein's New Dean")?.end_step = Some(moseo_end);
    let c = r.card(db, "Lethal Throwdown")?;
    c.resolve = Some(throwdown_resolve);
    c.hand_options = Some(throwdown_options);
    c.prio = Some(zero_prio);
    Ok(())
}
