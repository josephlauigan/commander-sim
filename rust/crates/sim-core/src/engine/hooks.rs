//! Calling card code from the engine: Python's `cardimpl.hooked`, `fire`, `total` and `allowed`.
//!
//! `hooked` lists the permanents on the battlefield whose card code handles an event, in entry order (`g.hooks`),
//! skipping those phased out, with abilities off, or whose controller is gone. Static events are asked at once
//! (`any_*`, `total_*`). Triggered events go through `fire_trigger`: each hook becomes a `Trigger` on the queue
//! (engine/stack.rs), unless its card marks it as running at once.

use crate::flow::Res;
use crate::hooks::{Call, CardImpl, Event, TrigAct, Trigger};
use crate::ids::{PermId, PlayerId};
use crate::state::Game;

/// (source permanent, its card code) for every live permanent handling event e
pub fn hooked(g: &Game, e: Event) -> Vec<(PermId, CardImpl)> {
    let mut out = vec![];
    for &m in &g.hooks {
        let x = g.perm(m);
        if x.phased || x.neutered || !x.on_bf || !g.player(x.owner).alive {
            continue;
        }
        let Some(c) = x.cd else { continue };
        if let Some(imp) = g.registry.get(c)
            && imp.handles(e)
        {
            out.push((m, *imp));
        }
    }
    out
}

/// Shalai, Voice of Plenty and kin: q can't be targeted by opponents
pub fn any_player_hexproof(g: &Game, q: PlayerId) -> bool {
    hooked(g, Event::PlayerHexproof).iter().any(|(src, imp)| (imp.player_hexproof.unwrap())(g, *src, q))
}

/// Solitary Confinement and kin: damage to p is prevented
pub fn any_prevent_damage(g: &Game, p: PlayerId) -> bool {
    hooked(g, Event::PreventDamage).iter().any(|(src, imp)| (imp.prevent_damage.unwrap())(g, *src, p))
}

/// Erebos-style statics from card code: p can't gain life
pub fn any_no_lifegain(g: &Game, p: PlayerId) -> bool {
    hooked(g, Event::NoLifegain).iter().any(|(src, imp)| (imp.no_lifegain.unwrap())(g, *src, p))
}

/// the sum of a count hook over the hooked permanents (Python's `CI.total(g, event, p)`)
pub fn total_count(g: &Game, e: Event, p: PlayerId) -> i32 {
    hooked(g, e)
        .iter()
        .map(|(src, imp)| {
            let f = match e {
                Event::ManaLock => imp.mana_lock,
                Event::NoCreatureMana => imp.no_creature_mana,
                Event::NoArtifactMana => imp.no_artifact_mana,
                Event::NonlandManaBonus => imp.nonland_mana_bonus,
                Event::TreasureBonus => imp.treasure_bonus,
                Event::ExtraLands => imp.extra_lands,
                Event::LandsFromGy => imp.lands_from_gy,
                Event::LandsFromTop => imp.lands_from_top,
                _ => panic!("{e:?} is not a count event"),
            };
            (f.unwrap())(g, *src, p)
        })
        .sum()
}

/// cost(g, src, caster, c) summed: generic mana added to (+) or removed from (-) c's cost
pub fn total_cost(g: &Game, p: PlayerId, c: crate::ids::CardId) -> i32 {
    hooked(g, Event::Cost).iter().map(|(src, imp)| (imp.cost.unwrap())(g, *src, p, c)).sum()
}

/// the highest min_cost floor (Trinisphere), 0 without one
pub fn max_min_cost(g: &Game, p: PlayerId, c: crate::ids::CardId) -> i32 {
    hooked(g, Event::MinCost).iter().map(|(src, imp)| (imp.min_cost.unwrap())(g, *src, p, c)).max().unwrap_or(0).max(0)
}

/// land_mana(g, src, p, land) summed
pub fn total_land_mana(g: &Game, p: PlayerId, l: crate::ids::LandId) -> i32 {
    hooked(g, Event::LandMana).iter().map(|(src, imp)| (imp.land_mana.unwrap())(g, *src, p, l)).sum()
}

/// trigger_copies summed for p's triggered abilities of this kind (Panharmonicon, Teysa, Ancient Greenwarden)
pub fn total_trigger_copies(g: &Game, p: PlayerId, kind: crate::sym::Sym, x: Option<PermId>) -> i32 {
    if g.hooks.is_empty() {
        return 0;
    }
    hooked(g, Event::TriggerCopies).iter().map(|(src, imp)| (imp.trigger_copies.unwrap())(g, *src, p, kind, x)).sum()
}

/// a player-count static summed (no_graveyard)
pub fn total_player(g: &Game, e: Event, p: PlayerId) -> i32 {
    hooked(g, e)
        .iter()
        .map(|(src, imp)| {
            let f = match e {
                Event::NoGraveyard => imp.no_graveyard,
                _ => panic!("{e:?} is not a player count event"),
            };
            (f.unwrap())(g, *src, p)
        })
        .sum()
}

/// the smallest search limit on p (Aven Mindcensor: the top four), if any
pub fn min_search_limit(g: &Game, p: PlayerId) -> Option<u32> {
    if g.hooks.is_empty() {
        return None;
    }
    hooked(g, Event::SearchLimit)
        .iter()
        .filter_map(|(src, imp)| (imp.search_limit.unwrap())(g, *src, p))
        .filter(|&n| n > 0)
        .min()
}

/// c can't be countered (Allosaurus Shepherd, Cavern of Souls ...)
pub fn any_uncounterable(g: &Game, p: PlayerId, c: crate::ids::CardId) -> bool {
    !g.hooks.is_empty()
        && hooked(g, Event::Uncounterable).iter().any(|(src, imp)| (imp.uncounterable.unwrap())(g, *src, p, c))
}

/// may p cast c from zone (no lock forbids it): Python's `CI.allowed`
pub fn allowed(g: &Game, p: PlayerId, c: crate::ids::CardId, zone: &str) -> bool {
    let zone = crate::sym::intern(zone);
    hooked(g, Event::CanCast).iter().all(|(src, imp)| (imp.can_cast.unwrap())(g, *src, p, c, zone))
}

/// state checks of hooked cards (Python's `CI.fire(g, 'sba')`, not a triggered event: run at once)
pub fn fire_sba(g: &mut Game) -> Res {
    for (src, imp) in hooked(g, Event::Sba) {
        (imp.sba.unwrap())(g, src)?;
        if g.over {
            break;
        }
    }
    Ok(())
}

/// A triggered event happened: each hooked permanent's trigger is queued, or (marked at once) runs now. Python's
/// `CI.fire` for an event in `TRIGGER_EVENTS`, with its copies: a dies trigger once more per Teysa Karlov, an enters
/// trigger that goes on the stack once more per Panharmonicon.
pub fn fire_trigger(g: &mut Game, e: Event, call: Call) -> Res {
    let mut entries = vec![];
    for (src, imp) in hooked(g, e) {
        let owner = g.perm(src).owner;
        let mut reps = 1;
        if let (Event::Dies, Call::Dies { m, .. }) = (e, &call) {
            reps += total_trigger_copies(g, owner, "dies", Some(*m));
        }
        if let (Event::Etb, Call::Etb { m, .. }) = (e, &call)
            && !imp.runs_at_once(e)
        {
            reps += total_trigger_copies(g, owner, "etb", Some(*m));
        }
        for _ in 0..reps.max(1) {
            if imp.runs_at_once(e) {
                run_hook(g, src, e, &call)?;
                if g.over {
                    return Ok(());
                }
            } else {
                entries.push(Trigger {
                    controller: owner,
                    src: Some(src),
                    act: TrigAct::Hook { event: e, call: call.clone() },
                    name: None,
                    known: false,
                    imp: None,
                    cast_etb: false,
                });
            }
        }
    }
    crate::engine::stack::queue_triggers(g, entries)
}

/// Run src's hook for event e with these arguments (a trigger resolving, or a hook marked at once).
pub fn run_hook(g: &mut Game, src: PermId, e: Event, call: &Call) -> Res {
    let Some(c) = g.perm(src).cd else { return Ok(()) };
    let Some(imp) = g.registry.get(c).copied() else { return Ok(()) };
    match (e, call) {
        (Event::Etb, Call::Etb { p, m }) => imp.etb.map_or(Ok(()), |f| f(g, src, *p, *m)),
        (Event::Leaves, Call::Leaves { m }) => imp.leaves.map_or(Ok(()), |f| f(g, src, *m)),
        (Event::Dies, Call::Dies { m, cause }) => imp.dies.map_or(Ok(()), |f| f(g, src, *m, cause)),
        (Event::SelfDies, Call::Dies { m, cause }) => imp.self_dies.map_or(Ok(()), |f| f(g, src, *m, cause)),
        (Event::Upkeep, Call::Player { p }) => imp.upkeep.map_or(Ok(()), |f| f(g, src, *p)),
        (Event::EndStep, Call::Player { p }) => imp.end_step.map_or(Ok(()), |f| f(g, src, *p)),
        (Event::Main1, Call::Player { p }) => imp.main1.map_or(Ok(()), |f| f(g, src, *p)),
        (Event::Main2, Call::Player { p }) => imp.main2.map_or(Ok(()), |f| f(g, src, *p)),
        (Event::Draw, Call::Player { p }) => imp.draw.map_or(Ok(()), |f| f(g, src, *p)),
        (Event::Landfall, Call::Player { p }) => imp.landfall.map_or(Ok(()), |f| f(g, src, *p)),
        (Event::Cast, Call::Cast { caster, c }) => imp.cast.map_or(Ok(()), |f| f(g, src, *caster, *c)),
        (Event::LoseLife, Call::Life { p, n }) => imp.lose_life.map_or(Ok(()), |f| f(g, src, *p, *n)),
        (Event::GainLife, Call::Life { p, n }) => imp.gain_life.map_or(Ok(()), |f| f(g, src, *p, *n)),
        (Event::CombatDamage, Call::CombatDamage { p, a, d, dmg }) => {
            imp.combat_damage.map_or(Ok(()), |f| f(g, src, *p, *a, *d, *dmg))
        }
        (Event::Attack, Call::Attack { p, atk, d }) => {
            // the attacking tokens it made join the attack (combat.rs collects g.new_attackers)
            let Some(f) = imp.attack else { return Ok(()) };
            let made = f(g, src, *p, atk, *d)?;
            g.new_attackers.extend(made);
            Ok(())
        }
        (Event::Sacrifice, Call::Sacrifice { p, what }) => imp.sacrifice.map_or(Ok(()), |f| f(g, src, *p, *what)),
        (Event::LandPlay, Call::Cards { p, cards }) => imp.land_play.map_or(Ok(()), |f| f(g, src, *p, cards[0])),
        (Event::LandGy, Call::Cards { p, cards }) => imp.land_gy.map_or(Ok(()), |f| f(g, src, *p, cards[0])),
        (Event::Discard, Call::Discard { p, c }) => imp.discard.map_or(Ok(()), |f| f(g, src, *p, *c)),
        (Event::TokenCreated, Call::ArtifactTokens { p, kinds, n }) => {
            imp.token_created.map_or(Ok(()), |f| f(g, src, *p, kinds, *n))
        }
        (e, c) => panic!("hook call {e:?} with {c:?}: no such pairing"),
    }
}
