//! Model gaps, batch A4 (rust/agents/model-gaps-a4.md): the activated abilities of five pool creatures that had no
//! ability code (not for their own controller, so not for Marvin, Murderous Mimic either). Each is card code with the
//! cost paid first and the effect on the stack (`stack::ability_run_w`), so Peter Parker's Camera and Rings of
//! Brighthearth can copy it, and each is an `options` hook, so the AI uses it and Marvin borrows it with Marvin as the
//! source. The person's versions are `human/model_gaps_a4.rs`.
//!
//! - **Siege-Gang Commander**: {1}{R}, sacrifice a Goblin: 2 damage to any target. The AI uses it to finish a player
//!   (all the activations the mana and the Goblins allow reach the life), a planeswalker or a creature it values.
//! - **Syr Konrad, the Grim**: {1}{B}: each player mills a card (Konrad's own triggers do the damage). The AI uses it
//!   with spare mana when the creature cards it can mill are worth the damage.
//! - **Professional Face-Breaker**: sacrifice a Treasure: exile the top card of your library, you may play it this
//!   turn (it waits in the hand marked `impulse`). The AI uses it when the card is likely to be playable.
//! - **Tinybones, Trinket Thief**: {4}{B}{B}: each opponent with no cards in hand loses 10 life. The AI uses it when
//!   an opponent it empties is at 10 or less (or at 20 or less, as the first half of two).
//! - **Ezuri, Renegade Leader**: {2}{G}{G}{G}: Elf creatures you control get +3/+3 and trample until end of turn. The
//!   AI's overrun (`impls::t1`'s, dead code since the Python port replaced it with an empty hook, probably because it
//!   fired whenever four Elves were ready) is used here only before combat, when it makes the attack lethal through
//!   the blockers or close to it.

use crate::cards::CardDb;
use crate::engine::combat::{can_block, kw};
use crate::engine::life::lose_life;
use crate::engine::mana::{can_pay, pay, total_mana};
use crate::engine::stack::ability_run_w;
use crate::engine::values::{
    colors_of, epow, etgh, hand_count, has_type, indestructible, no_sac, protected_from, pval, untargetable,
};
use crate::engine::zones::{die, mill};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::human::Target;
use crate::ids::{PermId, PlayerId};
use crate::state::Game;

pub const SIEGE: &str = "Siege-Gang Commander";
pub const KONRAD: &str = "Syr Konrad, the Grim";
pub const FACE_BREAKER: &str = "Professional Face-Breaker";
pub const TINYBONES: &str = "Tinybones, Trinket Thief";
pub const EZURI: &str = "Ezuri, Renegade Leader";

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

fn ability(utility: f64, label: String, src: PermId, f: crate::hooks::AbilityFn, arg: i64) -> Opt {
    Opt { utility, label, act: Some(Action::Ability { src, f, arg }) }
}

/// the permanent `src` is p's and can act (Marvin included: it borrows the abilities with itself as the source)
fn usable(g: &Game, src: PermId, p: PlayerId) -> bool {
    g.perm(src).owner == p && live(g, src)
}

/// the fraction of the cards in q's library that satisfy `f`
fn lib_frac(g: &Game, q: PlayerId, f: impl Fn(&Game, crate::ids::CardId) -> bool) -> f64 {
    let lib = &g.player(q).library;
    if lib.is_empty() {
        return 0.0;
    }
    lib.iter().filter(|&&c| f(g, c)).count() as f64 / lib.len() as f64
}

// ================================================================== Siege-Gang Commander
/// what a ping can hit: a player or a permanent (a creature or a planeswalker)
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tgt {
    Player(PlayerId),
    Perm(PermId),
}

impl Tgt {
    fn pack(self) -> i64 {
        match self {
            Tgt::Player(q) => q.0 as i64,
            Tgt::Perm(m) => (1 << 32) | m.0 as i64,
        }
    }

    fn unpack(x: i64) -> Tgt {
        if x >> 32 == 1 { Tgt::Perm(PermId((x & 0xffff_ffff) as u32)) } else { Tgt::Player(PlayerId(x as u8)) }
    }

    pub fn target(self) -> Target {
        match self {
            Tgt::Player(q) => Target::Player(q),
            Tgt::Perm(m) => Target::Perm(m),
        }
    }
}

/// the Goblins p controls (what "sacrifice a Goblin" may take)
pub fn goblins(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| live(g, m) && g.is_creature(m) && has_type(g, m, "goblin") && !no_sac(g, m))
        .collect()
}

/// the Goblin the AI gives up: the least valuable, never a commander, and (unless it must, `any`) a token or a
/// creature worth less than 3; the ability's own source last
fn siege_fodder(g: &Game, p: PlayerId, src: PermId, any: bool) -> Option<PermId> {
    let mut all: Vec<PermId> = goblins(g, p).into_iter().filter(|&m| !g.perm(m).is_cmd).collect();
    if !any {
        all.retain(|&m| g.perm(m).token || pval(g, m) < 3.0);
    }
    let rest: Vec<PermId> = all.iter().copied().filter(|&m| m != src).collect();
    let pool = if rest.is_empty() { all } else { rest };
    pool.into_iter().min_by(|&a, &b| pval(g, a).total_cmp(&pval(g, b)))
}

/// can `t` be hit by 2 damage from `src` (p's): alive, on the battlefield, not hexproof / shrouded from p, not
/// protected from the source's colours
pub fn ping_legal(g: &Game, p: PlayerId, src: PermId, t: Tgt) -> bool {
    match t {
        Tgt::Player(q) => {
            g.player(q).alive && (q == p || !crate::engine::values::player_hexproof_from(g, q, colors_of(g, src)))
        }
        Tgt::Perm(m) => {
            live(g, m)
                && (g.is_creature(m) || g.perm(m).loyalty.is_some())
                && !(g.perm(m).owner != p && untargetable(g, m))
                && !protected_from(g, m, colors_of(g, src))
        }
    }
}

/// the effect: n damage to the target (a creature dies if that reaches its toughness, a planeswalker loses loyalty, a
/// player loses life)
pub fn ping(g: &mut Game, p: PlayerId, src: PermId, t: Tgt, n: i32) -> Res {
    if !ping_legal(g, p, src, t) {
        return Ok(());
    }
    let name = g.perm(src).name.to_string();
    crate::human::choices::apply_damage(g, p, t.target(), n, &name, "burn")
}

/// the best ping the AI has now: (target, utility, lethal)
fn siege_plan(g: &Game, p: PlayerId, src: PermId) -> Option<(Tgt, f64)> {
    if !can_pay(g, p, 1, "R", false) {
        return None;
    }
    let all = goblins(g, p).len();
    let spare = goblins(g, p).into_iter().filter(|&m| !g.perm(m).is_cmd).count().min(all);
    if spare == 0 {
        return None;
    }
    // how many activations the mana and the Goblins allow now
    let mut k = spare.min(total_mana(g, p, false) as usize / 2);
    while k > 0 && !can_pay(g, p, k as u32, &"R".repeat(k), false) {
        k -= 1;
    }
    if k == 0 {
        return None;
    }
    // a player the pings finish: the one with the least life
    let mut best: Option<(Tgt, f64)> = None;
    let mut lethal = g.opps(p).filter(|&q| ping_legal(g, p, src, Tgt::Player(q)) && g.player(q).life <= 2 * k as i32);
    if let Some(q) =
        lethal.next().map(|first| lethal.fold(first, |a, b| if g.player(b).life < g.player(a).life { b } else { a }))
    {
        best = Some((Tgt::Player(q), 8.5));
    }
    // a planeswalker 2 damage finishes, a creature it kills that is worth a Goblin
    for q in g.opps(p) {
        for &m in &g.player(q).perms {
            if !ping_legal(g, p, src, Tgt::Perm(m)) {
                continue;
            }
            let u = if g.perm(m).loyalty.is_some() && !g.is_creature(m) {
                if g.perm(m).loyalty.unwrap_or(9) > 2 {
                    continue;
                }
                3.0 + (pval(g, m) / 2.0).min(3.0)
            } else {
                if etgh(g, m) > 2 || indestructible(g, m) || crate::engine::repl::no_damage(g, m) || pval(g, m) < 3.5 {
                    continue;
                }
                pval(g, m) - 2.0
            };
            if u > best.map_or(0.8, |b| b.1) {
                best = Some((Tgt::Perm(m), u));
            }
        }
    }
    best
}

fn siege_options(g: &mut Game, src: Src, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    if !usable(g, src, p) {
        return Ok(vec![]);
    }
    let Some((t, u)) = siege_plan(g, p, src) else { return Ok(vec![]) };
    let who = match t {
        Tgt::Player(q) => g.player(q).name.to_string(),
        Tgt::Perm(m) => g.perm(m).name.to_string(),
    };
    Ok(vec![ability(u, format!("{} -> {}", g.perm(src).name, who), src, siege_go, t.pack())])
}

fn siege_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let t = Tgt::unpack(arg);
    let lethal = matches!(t, Tgt::Player(_));
    if !usable(g, src, p) || !ping_legal(g, p, src, t) || !can_pay(g, p, 1, "R", false) {
        return Ok(false);
    }
    let Some(f) = siege_fodder(g, p, src, lethal) else { return Ok(false) };
    if !pay(g, p, 1, "R", false)? {
        return Ok(false);
    }
    die(g, f, "sac")?;
    siege_resolve(g, p, src, t, lethal)?;
    Ok(true)
}

/// the ability once its cost is paid and the Goblin sacrificed: on the stack, then 2 damage to the target
pub fn siege_resolve(g: &mut Game, p: PlayerId, src: PermId, t: Tgt, worth_a_copy: bool) -> Res {
    let what = match t {
        Tgt::Player(q) => g.player(q).name.to_string(),
        Tgt::Perm(m) => g.perm(m).name.to_string(),
    };
    let tp = if let Tgt::Perm(m) = t { Some(m) } else { None };
    let human = crate::human::is_human(g, p);
    ability_run_w(
        g,
        p,
        Some(src),
        None,
        &format!("2 damage to {what}"),
        Some(6.0),
        tp,
        if worth_a_copy { 3.0 } else { 0.0 },
        |g, is_copy| {
            let mut t = t;
            if is_copy && human {
                // a copy chooses its target anew
                let cands = crate::human::choices::damage_targets(g, p, colors_of(g, src));
                let labels: Vec<String> =
                    cands.iter().map(|&x| crate::human::legal::describe_target(g, p, x)).collect();
                let prompt = format!("The copy of {}'s ability: 2 damage to which target?", g.perm(src).name);
                if let Some(k) =
                    crate::human::choices::choose(g, p, "target", &prompt, &labels, Some("keep the target"), None)?
                {
                    t = match cands[k] {
                        Target::Player(q) => Tgt::Player(q),
                        Target::Perm(m) => Tgt::Perm(m),
                    };
                }
            }
            ping(g, p, src, t, 2)
        },
    )?;
    Ok(())
}

// ================================================================== Syr Konrad, the Grim
/// the AI's value of one mill: the creature cards it expects to put into a graveyard, each 1 damage to every opponent
/// (Konrad's trigger), and None when milling would hurt (its own library nearly empty)
fn konrad_value(g: &Game, p: PlayerId) -> Option<f64> {
    if g.player(p).library.len() < 8 {
        return None;
    }
    let opps = g.opps(p).count() as f64;
    let creature = |g: &Game, c: crate::ids::CardId| g.db.get(c).creature;
    let mut cards = lib_frac(g, p, creature);
    for q in g.opps(p) {
        cards += lib_frac(g, q, creature);
    }
    Some(0.8 + 0.3 * cards * opps)
}

fn konrad_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if !usable(g, src, p) || !can_pay(g, p, 1, "B", false) || post == Some(false) {
        return Ok(vec![]);
    }
    let Some(v) = konrad_value(g, p) else { return Ok(vec![]) };
    // the mana is what is left after the turn's plays: the end-of-turn window first
    let u = (v - if post.is_some() { 0.7 } else { 0.0 }).min(5.0);
    if u < 1.0 {
        return Ok(vec![]);
    }
    Ok(vec![ability(u, format!("{} mills each player", g.perm(src).name), src, konrad_go, 0)])
}

fn konrad_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !usable(g, src, p) || !pay(g, p, 1, "B", false)? {
        return Ok(false);
    }
    konrad_resolve(g, p, src)?;
    Ok(true)
}

/// each player mills a card: the ability on the stack, then the mills (Konrad's triggers see each one)
pub fn konrad_resolve(g: &mut Game, p: PlayerId, src: PermId) -> Res {
    ability_run_w(g, p, Some(src), None, "each player mills a card", Some(3.0), None, 0.0, |g, _| {
        let order: Vec<PlayerId> = std::iter::once(p).chain(g.after(p).collect::<Vec<_>>()).collect();
        for q in order {
            if g.player(q).alive && !g.over {
                mill(g, q, 1)?;
            }
        }
        crate::engine::life::check_state(g)
    })?;
    Ok(())
}

// ================================================================== Professional Face-Breaker
/// sacrifice a Treasure as a cost (a tapped one first): its Sacrifice triggers fire
pub fn sac_treasure(g: &mut Game, p: PlayerId) -> Res<bool> {
    if g.player(p).treasures == 0 {
        return Ok(false);
    }
    g.player_mut(p).sac_treasure();
    if !g.hooks.is_empty() {
        crate::engine::hooks::fire_trigger(
            g,
            crate::hooks::Event::Sacrifice,
            crate::hooks::Call::Sacrifice { p, what: crate::hooks::Sacrificed::Token("Treasure") },
        )?;
    }
    Ok(true)
}

/// exile the top card of p's library; p may play it this turn (it waits in the hand, marked `impulse`, until the end
/// of the turn sends it back to exile). The card, if there was one.
pub fn exile_top_playable(g: &mut Game, p: PlayerId) -> Option<crate::ids::CardId> {
    let c = g.player_mut(p).library.pop()?;
    let pl = g.player_mut(p);
    pl.hand.push(c);
    pl.impulse.push(c);
    pl.seen.insert(c);
    crate::glog!(g, "    {} exiles {} (playable this turn)", g.player(p).name, g.db.get(c).name);
    Some(c)
}

/// the ability after its cost: on the stack, then the exile
pub fn face_breaker_resolve(g: &mut Game, p: PlayerId, src: PermId) -> Res {
    ability_run_w(
        g,
        p,
        Some(src),
        None,
        "exile the top card, you may play it this turn",
        Some(4.0),
        None,
        0.0,
        |g, _| {
            exile_top_playable(g, p);
            Ok(())
        },
    )?;
    Ok(())
}

/// how likely the top card is playable this turn if the AI sacrifices a Treasure now: a land with the land drop
/// open, or a nonland card the mana left pays for (colours ignored)
fn face_breaker_chance(g: &Game, p: PlayerId) -> f64 {
    let pl = g.player(p);
    if pl.library.len() < 5 || pl.treasures == 0 {
        return 0.0;
    }
    // a tapped Treasure is free to give up; a live one is a mana less
    let mana = total_mana(g, p, false).saturating_sub(if pl.live_treasures() == pl.treasures { 1 } else { 0 });
    let drop = crate::human::legal::land_drops(g, p) > 0;
    let land = lib_frac(g, p, |g, c| g.db.get(c).land);
    let cheap = lib_frac(g, p, |g, c| !g.db.get(c).land && g.db.get(c).cmc <= mana);
    (if drop { land } else { 0.0 }) + cheap
}

fn face_breaker_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if !usable(g, src, p) || post.is_none() || g.active != Some(p) || g.player(p).treasures == 0 {
        return Ok(vec![]);
    }
    let chance = face_breaker_chance(g, p);
    if chance < 0.55 {
        return Ok(vec![]);
    }
    let u = 1.0 + 4.0 * chance;
    Ok(vec![ability(u, format!("{}: sacrifice a Treasure", g.perm(src).name), src, face_breaker_go, 0)])
}

fn face_breaker_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !usable(g, src, p) || !sac_treasure(g, p)? {
        return Ok(false);
    }
    face_breaker_resolve(g, p, src)?;
    Ok(true)
}

// ================================================================== Tinybones, Trinket Thief
/// the opponents the ability empties of 10 life now: each with no cards in hand
fn tinybones_victims(g: &Game, p: PlayerId) -> Vec<PlayerId> {
    g.opps(p).filter(|&q| hand_count(g, q) == 0).collect()
}

fn tinybones_options(g: &mut Game, src: Src, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    if !usable(g, src, p) || !can_pay(g, p, 4, "BB", false) {
        return Ok(vec![]);
    }
    let victims = tinybones_victims(g, p);
    let kills = victims.iter().filter(|&&q| g.player(q).life <= 10).count();
    let near = victims.iter().filter(|&&q| g.player(q).life <= 20).count();
    let u = if kills > 0 {
        8.5 + kills as f64 * 0.2
    } else if near > 0 {
        3.0
    } else {
        return Ok(vec![]);
    };
    Ok(vec![ability(u, format!("{}: empty-handed opponents lose 10", g.perm(src).name), src, tinybones_go, 0)])
}

fn tinybones_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !usable(g, src, p) || !pay(g, p, 4, "BB", false)? {
        return Ok(false);
    }
    tinybones_resolve(g, p, src)?;
    Ok(true)
}

/// the ability on the stack, then each opponent with no cards in hand (as it resolves) loses 10 life
pub fn tinybones_resolve(g: &mut Game, p: PlayerId, src: PermId) -> Res {
    let worth = if tinybones_victims(g, p).iter().any(|&q| g.player(q).life <= 10) { 3.0 } else { 0.0 };
    ability_run_w(
        g,
        p,
        Some(src),
        None,
        "each opponent with no cards in hand loses 10 life",
        Some(7.0),
        None,
        worth,
        |g, _| {
            for q in g.opps(p).collect::<Vec<_>>() {
                if g.player(q).alive && hand_count(g, q) == 0 && !g.over {
                    lose_life(g, q, 10, Some(p), "other", None)?;
                }
            }
            crate::engine::life::check_state(g)
        },
    )?;
    Ok(())
}

// ================================================================== Ezuri, Renegade Leader
/// Elf creatures `p` controls
pub fn elves(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m) && has_type(g, m, "elf")).collect()
}

/// the damage p's creatures that can attack get through to q when all of them attack it: each blocker of q chumps one
/// of the biggest attackers it can block (a menace creature needs two), trample passes the excess over the blocker's
/// toughness. `pumped`: the Elves have +3/+3 and trample.
fn damage_through(g: &Game, p: PlayerId, q: PlayerId, pumped: bool) -> i32 {
    let mut atk: Vec<(PermId, i32, bool)> = vec![];
    for &m in &g.player(p).perms {
        let x = g.perm(m);
        if !live(g, m) || !g.is_creature(m) || x.tapped || x.noatk || (x.sick && !kw(g, m, "haste")) {
            continue;
        }
        let elf = pumped && has_type(g, m, "elf");
        let pw = epow(g, m) + if elf { 3 } else { 0 };
        if pw > 0 {
            atk.push((m, pw, elf || kw(g, m, "trample")));
        }
    }
    atk.sort_by_key(|a| -a.1);
    let mut blockers: Vec<PermId> =
        g.player(q).perms.iter().copied().filter(|&b| live(g, b) && g.is_creature(b) && !g.perm(b).tapped).collect();
    let mut total = 0;
    for (a, pw, trample) in atk {
        let mut cand: Vec<PermId> = blockers.iter().copied().filter(|&b| can_block(g, b, a)).collect();
        let need = if kw(g, a, "menace") { 2 } else { 1 };
        if cand.len() < need {
            total += pw;
            continue;
        }
        cand.sort_by_key(|&b| etgh(g, b));
        let used: Vec<PermId> = cand.into_iter().take(need).collect();
        blockers.retain(|b| !used.contains(b));
        if trample {
            let tough: i32 = used.iter().map(|&b| etgh(g, b).max(0)).sum();
            total += (pw - tough).max(0);
        }
    }
    total
}

fn ezuri_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    // before combat only: the overrun is for an attack the AI is about to make
    if post != Some(false) || !usable(g, src, p) || !can_pay(g, p, 2, "GGG", false) {
        return Ok(vec![]);
    }
    let ready = elves(g, p)
        .into_iter()
        .filter(|&m| !g.perm(m).tapped && !g.perm(m).noatk && (!g.perm(m).sick || kw(g, m, "haste")))
        .count();
    if ready == 0 {
        return Ok(vec![]);
    }
    let mut best: Option<(f64, PlayerId)> = None;
    for q in g.opps(p).collect::<Vec<_>>() {
        let life = g.player(q).life;
        if life <= 0 || g.player(q).life_locked {
            continue;
        }
        let (now, pumped) = (damage_through(g, p, q, false), damage_through(g, p, q, true));
        let u = if pumped >= life && now < life {
            9.0 // the pump makes the attack lethal
        } else if now < life && pumped - now >= 4 && pumped * 10 >= life * 7 {
            2.5 + (pumped as f64 / life as f64).min(1.0) * 1.5 // close: the rest is one more attack
        } else {
            continue;
        };
        if best.is_none_or(|b| u > b.0) {
            best = Some((u, q));
        }
    }
    let Some((u, q)) = best else { return Ok(vec![]) };
    let label = format!("{} overrun ({} Elves vs {})", g.perm(src).name, ready, g.player(q).name);
    Ok(vec![ability(u, label, src, ezuri_go, 0)])
}

fn ezuri_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !usable(g, src, p) || !pay(g, p, 2, "GGG", false)? {
        return Ok(false);
    }
    ezuri_resolve(g, p, src)?;
    Ok(true)
}

/// the ability on the stack, then the Elves p controls as it resolves get +3/+3 and trample until end of turn
pub fn ezuri_resolve(g: &mut Game, p: PlayerId, src: PermId) -> Res {
    ability_run_w(g, p, Some(src), None, "Elves get +3/+3 and trample", Some(7.0), None, 3.0, |g, _| {
        for m in elves(g, p) {
            super::t3::eot(g, m, 3, 3);
            super::partials::eot_kw(g, m, "trample");
        }
        g.bf_ver += 1;
        crate::glog!(g, "    {}'s Elves get +3/+3 and trample", g.player(p).name);
        Ok(())
    })?;
    Ok(())
}

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let hooks: [(&str, crate::hooks::OptionsFn); 5] = [
        (SIEGE, siege_options),
        (KONRAD, konrad_options),
        (FACE_BREAKER, face_breaker_options),
        (TINYBONES, tinybones_options),
        (EZURI, ezuri_options),
    ];
    for (name, f) in hooks {
        if db.id(name).is_some() {
            r.card(db, name)?.options = Some(f);
        }
    }
    Ok(())
}
