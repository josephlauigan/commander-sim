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
use crate::tag::Tag;

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
                Event::ProliferateExtra => imp.proliferate_extra,
                Event::SkipDraw => imp.skip_draw,
                Event::LoyaltyExtra => imp.loyalty_extra,
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
pub fn total_land_mana(g: &Game, p: PlayerId, l: crate::ids::PermId) -> i32 {
    hooked(g, Event::LandMana).iter().map(|(src, imp)| (imp.land_mana.unwrap())(g, *src, p, l)).sum()
}

/// land_colorless(g, src, p, land, n): does a land making n mana make one {C} instead (Damping Sphere)
pub fn land_colorless(g: &Game, p: PlayerId, l: crate::ids::PermId, n: u32) -> bool {
    hooked(g, Event::LandColorless).iter().any(|(src, imp)| (imp.land_colorless.unwrap())(g, *src, p, l, n))
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

/// a battlefield card counters c as p casts it (Vexing Bauble, Chalice of the Void, Void Mirror)
pub fn cast_countered(g: &Game, p: PlayerId, c: crate::ids::CardId, ctx: &crate::state::Ctx) -> bool {
    !g.hooks.is_empty()
        && hooked(g, Event::CastCounter).iter().any(|(src, imp)| (imp.cast_counter.unwrap())(g, *src, p, c, ctx))
}

/// Would a battlefield cast-counter counter c as p casts it having spent `spent` mana, `colored` of it coloured? The AI
/// asks before it casts (`cast::castable`): it never means to cast a Mox into a Vexing Bauble.
pub fn would_be_cast_countered(g: &Game, p: PlayerId, c: crate::ids::CardId, spent: u32, colored: bool) -> bool {
    let ctx = crate::state::Ctx { mana_spent: spent, mana_colored: colored, ..Default::default() };
    cast_countered(g, p, c, &ctx)
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
    fire_trigger_where(g, e, call, &|_, _| true, false)
}

/// `fire_trigger` for a pass of a loop that fires the whole event once more for each additional time (Ancient
/// Greenwarden's landfall, Isshin's attacks): with `repeat` its triggers are repeats (`Trigger::repeat`), not firsts
pub fn fire_trigger_pass(g: &mut Game, e: Event, call: Call, repeat: bool) -> Res {
    fire_trigger_where(g, e, call, &|_, _| true, repeat)
}

/// Elesh Norn, Mother of Machines: "Permanents entering the battlefield don't cause abilities of permanents your
/// opponents control to trigger." True when an opponent of `controller` has one on the battlefield (not phased out,
/// abilities on): a triggered ability of one of `controller`'s permanents is not caused by `entering` (None: a land
/// or tokens, whichever the caller has no single permanent for). Only enters-caused triggered abilities are blanked:
/// what happens as a permanent enters (a name chosen, enters tapped or with counters or prepared, an Aura attaching,
/// devour, a Saga's lore counter), static and state registrations, and the abilities of cards in other zones (a
/// graveyard landfall) still happen. A hook is a trigger or setup by `enter_hook_is_trigger`; a hook that mixes the
/// two asks `enters_trigger_times`. (Hushbringer is checked apart, in `zones::enter` and at the head of
/// `fire_trigger_where`.)
pub fn enter_blanked(g: &Game, controller: PlayerId, _entering: Option<PermId>) -> bool {
    g.opps(controller).any(|q| {
        g.player(q).perms.iter().any(|&m| {
            let x = g.perm(m);
            x.on_bf && !x.phased && !x.neutered && x.cd.is_some_and(|c| g.db.get(c).tag(Tag::Mother))
        })
    })
}

/// Is the hook of `imp` for event `e` an enters-caused triggered ability (so an opponent's Elesh Norn blanks it and the
/// event doublers copy it)? A hook of `Etb`, `LandEnters` or `Landfall` that goes on the stack is, unless it is marked
/// `enter_setup` (a triggered ability that the entering did not cause: a Saga's chapter I); a hook that runs now
/// (marked at once, or an event outside `TRIGGER_EVENTS`: every `TokensEnter`, `TokenCreated` and `LandEtb` hook) is
/// not, unless it is marked `enter_trigger`. So setup, which is no triggered ability, never waits in the queue.
pub fn enter_hook_is_trigger(imp: &CardImpl, e: Event) -> bool {
    let now = imp.runs_at_once(e) || !e.is_trigger();
    imp.is_enter_trigger(e)
        || (matches!(e, Event::Etb | Event::LandEnters | Event::Landfall) && !now && !imp.is_enter_setup(e))
}

/// How many times a triggered ability of `controller`'s permanent, caused by `entering` (None: a land, or tokens),
/// happens: for a hook that mixes an enters-caused trigger with what happens as the permanent enters
/// (`enter_setup`), which loops only its trigger part this many times. 0 under an opponent's live Elesh Norn, else 1.
pub fn enters_trigger_times(g: &Game, controller: PlayerId, entering: Option<PermId>) -> u32 {
    if enter_blanked(g, controller, entering) { 0 } else { 1 }
}

/// `fire_trigger`, for the hooked permanents `keep` accepts (a blink fires after the creature is back, and what entered
/// with it never saw it leave: `zones::exiled_from_bf`)
pub fn fire_trigger_where(
    g: &mut Game,
    e: Event,
    call: Call,
    keep: &dyn Fn(&Game, PermId) -> bool,
    repeat: bool,
) -> Res {
    // Hushbringer: a creature entering (a land that is also a creature) sets off nothing
    if let (Event::LandEnters, Call::Etb { m, .. }) = (e, &call)
        && g.is_creature(*m)
        && crate::engine::repl::hushed(g)
    {
        return Ok(());
    }
    let enters =
        matches!(e, Event::Etb | Event::LandEnters | Event::Landfall | Event::TokensEnter | Event::TokenCreated);
    let entering = if let Call::Etb { m, .. } = &call { Some(*m) } else { None };
    let mut entries = vec![];
    for (src, imp) in hooked(g, e) {
        if !keep(g, src) {
            continue;
        }
        let owner = g.perm(src).owner;
        let now = imp.runs_at_once(e) || !e.is_trigger();
        let trig = enter_hook_is_trigger(&imp, e);
        // an opponent's Elesh Norn: no trigger of this permanent is caused by the entering
        if enters && trig && enter_blanked(g, owner, entering) {
            continue;
        }
        let mut reps = 1;
        if let (Event::Dies, Call::Dies { m, .. }) = (e, &call) {
            reps += total_trigger_copies(g, owner, "dies", Some(*m));
        }
        if let (Event::Etb, Call::Etb { m, .. }) = (e, &call)
            && trig
            && !now
        {
            reps += total_trigger_copies(g, owner, "etb", Some(*m));
        }
        if e == Event::TokenCreated && trig {
            // an artifact token (Treasure, Food, Clue) entering: Panharmonicon's additional trigger, Elesh Norn's
            reps += total_trigger_copies(g, owner, "etb", None);
        }
        if let (Event::LandEnters, Call::Etb { m, .. }) = (e, &call)
            && trig
            && !now
        {
            // a land entering: Ancient Greenwarden's additional trigger, and Panharmonicon's if it is also a creature.
            // One source adds one trigger per entering permanent, whichever of the two kinds it answers to (Starfield
            // Vocalist answers to both): the larger of its two counts, summed over the sources
            reps += hooked(g, Event::TriggerCopies)
                .iter()
                .map(|(s, i)| {
                    let f = i.trigger_copies.unwrap();
                    f(g, *s, owner, "landfall", Some(*m)).max(f(g, *s, owner, "etb", Some(*m)))
                })
                .sum::<i32>();
        }
        for k in 0..reps.max(1) {
            // an event outside TRIGGER_EVENTS ('crew', 'monarch') runs at once, as Python's CI.fire does
            if now {
                run_hook(g, src, e, &call)?;
                if g.over {
                    return Ok(());
                }
            } else {
                let act = TrigAct::Hook { event: e, call: call.clone() };
                entries.push(Trigger { repeat: repeat || k > 0, ..Trigger::new(owner, Some(src), act) });
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
        (Event::ExiledFromBf, Call::Leaves { m }) => imp.exiled_from_bf.map_or(Ok(()), |f| f(g, src, *m)),
        (Event::Dies, Call::Dies { m, cause }) => imp.dies.map_or(Ok(()), |f| f(g, src, *m, cause)),
        (Event::SelfDies, Call::Dies { m, cause }) => imp.self_dies.map_or(Ok(()), |f| f(g, src, *m, cause)),
        (Event::SelfExiled, Call::Leaves { m }) => imp.self_exiled.map_or(Ok(()), |f| f(g, src, *m)),
        (Event::Upkeep, Call::Player { p }) => imp.upkeep.map_or(Ok(()), |f| f(g, src, *p)),
        (Event::EndStep, Call::Player { p }) => imp.end_step.map_or(Ok(()), |f| f(g, src, *p)),
        (Event::Main1, Call::Player { p }) => imp.main1.map_or(Ok(()), |f| f(g, src, *p)),
        (Event::Main2, Call::Player { p }) => imp.main2.map_or(Ok(()), |f| f(g, src, *p)),
        (Event::Draw, Call::Player { p }) => imp.draw.map_or(Ok(()), |f| f(g, src, *p)),
        (Event::Landfall, Call::Player { p }) => imp.landfall.map_or(Ok(()), |f| f(g, src, *p)),
        (Event::LandEnters, Call::Etb { p, m }) => imp.land_enters.map_or(Ok(()), |f| f(g, src, *p, *m)),
        (Event::Crew, Call::Player { p }) => imp.crew.map_or(Ok(()), |f| f(g, src, *p)),
        (Event::Monarch, Call::Player { p }) => imp.monarch.map_or(Ok(()), |f| f(g, src, *p)),
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
        (Event::TokensEnter, Call::Tokens { p, toks }) => imp.tokens_enter.map_or(Ok(()), |f| f(g, src, *p, toks)),
        (Event::TokenCreated, Call::ArtifactTokens { p, kinds, n }) => {
            imp.token_created.map_or(Ok(()), |f| f(g, src, *p, kinds, *n))
        }
        (Event::CreatureToGy, Call::Leaves { m }) => imp.creature_to_gy.map_or(Ok(()), |f| f(g, src, *m)),
        (Event::Copycast, Call::Copycast { p, effect }) => imp.copycast.map_or(Ok(()), |f| f(g, src, *p, *effect)),
        (Event::CombatStart, Call::Player { p }) => {
            // the attacking tokens it made join the attack (cardcode::combat_start collects g.new_attackers)
            let Some(f) = imp.combat_start else { return Ok(()) };
            let made = f(g, src, *p)?;
            g.new_attackers.extend(made);
            Ok(())
        }
        (Event::EachCombat, Call::Player { p }) => imp.each_combat.map_or(Ok(()), |f| f(g, src, *p)),
        (Event::CardsToGy, Call::Cards { p, cards }) => imp.cards_to_gy.map_or(Ok(()), |f| f(g, src, *p, cards)),
        (e, c) => panic!("hook call {e:?} with {c:?}: no such pairing"),
    }
}
