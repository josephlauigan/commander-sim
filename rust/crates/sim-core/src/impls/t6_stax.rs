//! Tier 6 "stax" batch: the stax and Silence-effect cards of JD's Dack, Kraum & Tymna and Sisay decks that the compiled
//! Oracle text and the engine's tags don't cover (rust/agents/t6-batches.md). Cards only: no engine file is touched, so
//! what needs the shared engine (a player-level "can't cast spells" check, split second, a kicker choice, "can't lose",
//! enters tapped for opponents, a counter-on-cast hook) is written as a plain `pub fn` the engine can call later and
//! listed in rust/agents/engine-needs-stax.md. Each card's doc comment says how it is modeled and what is left out.
//!
//! - Silence, Orim's Chant, Mandate of Peace: the target player(s) get a turn flag (`cast_lock`) the engine reads, and
//!   today a stand-in the engine already reads: `Player::hope_lock` (no noncreature spells that turn).
//! - Orim's Chant also sets `stax_no_attack` when kicked (`attack_lock`).
//! - Angel's Grace: the turn flag `grace_active`.

use crate::cards::CardDb;
use crate::engine::zones::max_by;
use crate::flow::Res;
use crate::hooks::Registry;
use crate::ids::{CardId, PlayerId};
use crate::state::{Ctx, Game};
use crate::sym::{Sym, intern};

fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

fn set_flag(g: &mut Game, p: PlayerId, key: &str) {
    let stamp = g.turn_stamp();
    g.player_mut(p).flag_turn.insert(intern(key), stamp);
}

fn flag_now(g: &Game, p: PlayerId, key: &str) -> bool {
    g.player(p).flag_turn.get(intern(key)) == Some(&g.turn_stamp())
}

fn never_prio(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    0
}

// ================================================================== Silence effects
/// the turn flag a Silence effect sets on the player who can't cast spells this turn: the engine's cast check should
/// refuse every spell (creatures and flash included) from a flagged player until the turn ends
pub fn cast_lock(g: &Game, p: PlayerId) -> bool {
    flag_now(g, p, "stax_cast_lock")
}

/// p can't cast spells this turn: the flag above, plus the stand-in the engine reads today (`Player::hope_lock`, as
/// Hope of Ghirapur sets it: no noncreature spells until the active player's turn count moves on)
fn lock_player(g: &mut Game, p: PlayerId) {
    set_flag(g, p, "stax_cast_lock");
    if let Some(a) = g.active {
        let t = g.player(a).turns;
        g.player_mut(p).hope_lock = Some((a, t));
    }
}

/// **Silence** ({W} instant): your opponents can't cast spells this turn. Stand-in: opponents can't cast noncreature
/// spells (a flash creature is still castable). Partial: the full lock needs the engine (engine-needs-stax.md). The AI
/// never casts it (priority 0): when it wins a combo turn is the deck plan's call.
fn silence_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    for q in g.opps(p).collect::<Vec<_>>() {
        lock_player(g, q);
    }
    crate::glog!(g, "    {}'s opponents can't cast spells this turn (Silence)", pname(g, p));
    Ok("gy")
}


/// the turn flag a kicked Orim's Chant sets on its caster: creatures can't attack this turn (the engine's attack
/// declaration should return no attackers for anyone while it is set)
pub fn attack_lock(g: &Game) -> bool {
    g.players.iter().any(|q| flag_now(g, q.id, "stax_no_attack"))
}

/// **Orim's Chant** ({W} instant, kicker {W}): target player can't cast spells this turn; if kicked, creatures can't
/// attack this turn. The target is the face the spell was aimed at, else the opponent with the most cards in hand.
/// Partial: like Silence the lock is a noncreature stand-in; the kicker needs the engine to offer and pay it (a
/// kicked cast sets `ctx.kicks`, which nothing does yet), and the no-attack rule needs an attack check
/// (engine-needs-stax.md). Kicked, the flag `attack_lock` is set. The AI never casts it (priority 0).
fn chant_resolve(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let opps: Vec<PlayerId> = g.opps(p).collect();
    let target = ctx.face.or_else(|| max_by(&opps, |o| g.player(o).hand.len() as f64));
    if let Some(t) = target {
        lock_player(g, t);
        crate::glog!(g, "    {} can't cast spells this turn (Orim's Chant)", pname(g, t));
    }
    if ctx.kicks > 0 {
        set_flag(g, p, "stax_no_attack");
        crate::glog!(g, "    creatures can't attack this turn (kicked Orim's Chant)");
    }
    Ok("gy")
}


// ================================================================== Mandate of Peace
/// is it combat (creatures are attacking), the only time Mandate of Peace may be cast; for the engine's cast check
/// and the AI once either offers it
pub fn in_combat_now(g: &Game) -> bool {
    !g.in_combat.is_empty()
}

/// **Mandate of Peace** ({1}{W} instant, only during combat): your opponents can't cast spells this turn; end the
/// combat phase. Partial: the lock is the Silence stand-in; ending the combat phase (removing attackers and blockers,
/// no later combat steps, exiling the stack) is stood in for by a fog this turn (`Game::fog`: no more combat damage,
/// though the creatures stay in combat and extra combats are fogged too); "cast only during combat" is not enforced
/// by the engine for a card in hand (`in_combat_now` says when). The AI never casts it (priority 0).
fn mandate_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    for q in g.opps(p).collect::<Vec<_>>() {
        lock_player(g, q);
    }
    g.fog = Some(g.turn_stamp());
    crate::glog!(g, "    {}'s opponents can't cast spells and combat ends (Mandate of Peace)", pname(g, p));
    Ok("gy")
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, "Silence")?;
    c.resolve = Some(silence_resolve);
    c.prio = Some(never_prio);
    let c = r.card(db, "Orim's Chant")?;
    c.resolve = Some(chant_resolve);
    c.prio = Some(never_prio);
    let c = r.card(db, "Mandate of Peace")?;
    c.resolve = Some(mandate_resolve);
    c.prio = Some(never_prio);
    Ok(())
}
