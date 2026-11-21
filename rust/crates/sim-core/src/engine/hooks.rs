//! Calling card code from the engine: Python's `cardimpl.hooked`, `fire`, `total` and `allowed`.
//!
//! `hooked` lists the permanents on the battlefield whose card code handles an event, in entry order (`g.hooks`),
//! skipping those phased out, with abilities off, or whose controller is gone. Static events are asked at once
//! (`any_*`, `total_*`). Triggered events go through `fire_trigger`: each hook becomes a `Trigger` on the queue
//! (engine/stack.rs), unless its card marks it as running at once.

use crate::flow::Res;
use crate::hooks::{Call, CardImpl, Event, Sacrificed, TrigAct, Trigger};
use crate::ids::{PermId, PlayerId};
use crate::state::{DataKey, Game, Val};
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
    if g.hooks.is_empty() && g.left_copiers.is_empty() {
        return 0;
    }
    let live: i32 = hooked(g, Event::TriggerCopies)
        .iter()
        .map(|(src, imp)| (imp.trigger_copies.unwrap())(g, *src, p, kind, x))
        .sum();
    live + x.map_or(0, |x| left_copies(g, p, kind, x))
}

/// A leaving looks back in time (CR 603.10a): a trigger copier that left at the same time as `x` (it is `x`, or it left
/// earlier in the wipe `x` leaves in) still adds its additional times for what `x` leaving causes. Only for "dies" and
/// "leaves" of a permanent that is gone; the copiers on the battlefield are counted by `hooked`.
fn left_copies(g: &Game, p: PlayerId, kind: crate::sym::Sym, x: PermId) -> i32 {
    if g.left_copiers.is_empty() || !matches!(kind, "dies" | "leaves") || g.perm(x).on_bf {
        return 0;
    }
    let together = |v: PermId| {
        v == x
            || matches!(g.perm(v).data.get(DataKey::LkiCopier), Some(Val::Int(b)) if *b != 0 && g.cur_batch == Some(*b as u32))
    };
    g.left_copiers
        .iter()
        .filter(|&&v| !g.perm(v).on_bf && together(v))
        .filter_map(|&v| {
            let imp = g.registry.get(g.perm(v).cd?)?;
            imp.trigger_copies.map(|f| f(g, v, p, kind, Some(x)))
        })
        .sum()
}

/// The event that m's leaving is for the trigger copiers: a death ("dies": Teysa Karlov, Drivnod, Gandalf the White) or
/// any other leaving ("leaves": Gandalf only). `Game::dying` is set around `zones::leave` for a death.
pub fn leave_kind(g: &Game, m: PermId) -> crate::sym::Sym {
    if g.dying == Some(m) { "dies" } else { "leaves" }
}

/// How many times a triggered ability of `controller`'s permanent caused by m leaving the battlefield happens: once, and
/// once more for each Teysa Karlov, Drivnod or Gandalf the White (CR 603.2d). Call it from `zones::leave` and its helpers;
/// the first instance is the original, the rest are repeats.
pub fn leave_trigger_times(g: &Game, controller: PlayerId, m: PermId) -> u32 {
    1 + total_trigger_copies(g, controller, leave_kind(g, m), Some(m)).max(0) as u32
}

/// What a leave event is, for the trigger copiers: (the kind, the permanent that left). A land card put into a graveyard
/// and a land sacrificed carry no permanent: it is the `leaving` that `fire_trigger_for_land` set for this event. None: no
/// permanent (the copiers add nothing), or the event is not a leaving.
fn leave_cause(e: Event, call: &Call, leaving: Option<PermId>) -> Option<(crate::sym::Sym, Option<PermId>)> {
    match (e, call) {
        (Event::Dies, Call::Dies { m, .. }) | (Event::CreatureToGy, Call::Leaves { m }) => Some(("dies", Some(*m))),
        (Event::ExiledFromBf, Call::Leaves { m }) => Some(("leaves", Some(*m))),
        (Event::LandGy, _) => leaving.map(|l| ("dies", Some(l))),
        (Event::Sacrifice, Call::Sacrifice { what, .. }) => match what {
            Sacrificed::Perm(m) => Some(("dies", Some(*m))),
            Sacrificed::Token(_) => Some(("dies", None)),
            Sacrificed::Card(_) => leaving.map(|l| ("dies", Some(l))),
        },
        _ => None,
    }
}

/// Event doublers' additional instances caused by entering. None denotes an artifact token only.
/// For a land, each copier contributes the larger of its landfall and etb counts, counted once.
pub fn enter_copies(g: &Game, controller: PlayerId, entering: Option<PermId>) -> u32 {
    let land = entering.is_some_and(|m| g.player(g.perm(m).owner).lands.contains(&m));
    let n = if land {
        hooked(g, Event::TriggerCopies)
            .iter()
            .map(|(s, i)| {
                let f = i.trigger_copies.unwrap();
                f(g, *s, controller, "landfall", entering).max(f(g, *s, controller, "etb", entering))
            })
            .sum::<i32>()
    } else {
        total_trigger_copies(g, controller, "etb", entering)
    };
    n.max(0) as u32
}

/// Snapshot each listener controller's event copies before any pass runs. Inline payloads contribute their
/// controller too; a copier with no listener never creates a later pass or engages the resolution hold.
pub fn listener_copies(g: &Game, e: Event, kind: crate::sym::Sym, inline: Option<PlayerId>) -> Vec<i32> {
    listener_copies_for(g, e, kind, None, inline)
}

/// `listener_copies`, for the permanent that caused the event (the land that entered: Gandalf the White and Panharmonicon
/// ask what it is). None: no single permanent, as for an attack.
pub fn listener_copies_for(
    g: &Game,
    e: Event,
    kind: crate::sym::Sym,
    x: Option<PermId>,
    inline: Option<PlayerId>,
) -> Vec<i32> {
    if hooked(g, Event::TriggerCopies).is_empty() {
        return vec![0; g.players.len()];
    }
    let listeners = hooked(g, e);
    let compiled_listener = |s| {
        let x = g.perm(s);
        let has_trigger = |c| {
            g.db.get(c)
                .abilities
                .iter()
                .any(|a| matches!(a, crate::dsl::model::Ability::Triggered { event, .. } if event == kind))
        };
        x.on_bf && !x.phased && !x.neutered && x.cd.is_some_and(has_trigger)
    };
    g.players
        .iter()
        .map(|q| {
            let listens = Some(q.id) == inline
                || listeners.iter().any(|(s, _)| g.perm(*s).owner == q.id)
                || (g.dsl_on
                    && q.perms.iter().copied().chain(crate::dsl::land_sources(g, q.id)).any(compiled_listener));
            if q.alive && listens { crate::engine::repl::trigger_copies(g, q.id, kind, x) } else { 0 }
        })
        .collect()
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

/// `fire_trigger` for an event about a land card that left the battlefield from `l` (a land card put into a graveyard, a land
/// sacrificed): the call carries no permanent, so `Game::leaving` names the one whose leaving it is, for the trigger copiers.
pub fn fire_trigger_for_land(g: &mut Game, l: PermId, e: Event, call: Call) -> Res {
    let prev = g.leaving.replace(l);
    let r = fire_trigger(g, e, call);
    g.leaving = prev;
    r
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
/// two asks `enters_trigger_times`. (Hushbringer, for a creature entering, is `enter_silenced`.)
pub fn enter_blanked(g: &Game, controller: PlayerId, _entering: Option<PermId>) -> bool {
    g.opps(controller).any(|q| {
        g.player(q).perms.iter().any(|&m| {
            let x = g.perm(m);
            x.on_bf && !x.phased && !x.neutered && x.cd.is_some_and(|c| g.db.get(c).tag(Tag::Mother))
        })
    })
}

/// A triggered ability of `controller`'s permanent caused by `entering` does not trigger: an opponent's Elesh Norn, Mother of
/// Machines (`enter_blanked`), or Hushbringer for a creature entering ("Creatures entering the battlefield or dying don't
/// cause abilities to trigger": every player's, a land that is a creature too). `entering` None: a land or tokens, or an
/// artifact token that is no creature (only Elesh blanks those).
pub fn enter_silenced(g: &Game, controller: PlayerId, entering: Option<PermId>) -> bool {
    enter_blanked(g, controller, entering)
        || entering.is_some_and(|m| g.is_creature(m)) && crate::engine::repl::hushed(g)
}

/// Is the hook of `imp` for event `e` an enters-caused triggered ability (so an opponent's Elesh Norn blanks it and the
/// event doublers copy it)? A hook of `Etb`, `LandEnters` or `Landfall` that goes on the stack is, unless it is marked
/// `enter_setup` (a triggered ability that the entering did not cause: a Saga's chapter I); a hook that runs now
/// (marked at once, or an event outside `TRIGGER_EVENTS`: every `TokensEnter`, `TokenCreated` and `LandEtb` hook) is
/// not, unless it is marked `enter_trigger` (also implied by `enter_queues`). A queue adapter stays a trigger
/// for blanking, but runs once: its queued triggers receive the copies. Setup never waits in the queue.
pub fn enter_hook_is_trigger(imp: &CardImpl, e: Event) -> bool {
    let now = imp.runs_at_once(e) || !e.is_trigger();
    imp.is_enter_trigger(e)
        || (matches!(e, Event::Etb | Event::LandEnters | Event::Landfall) && !now && !imp.is_enter_setup(e))
}

/// Is the hook of `imp` for leave event `e` (`Leaves`, `Dies`, `SelfDies`, `SelfExiled`, `ExiledFromBf`,
/// `CreatureToGy`, `LandGy`, `Sacrifice`) a leave-caused triggered ability (so Teysa Karlov, Drivnod and Gandalf the
/// White copy it)? A hook that goes on the stack is, unless it is marked `enter_setup` (a duration that ends, "until
/// this leaves the battlefield", that the engine queues); a hook that runs now (marked at once) is not, unless it is
/// marked `enter_trigger` (a trigger with no window: it runs at once, but it is a triggered ability). A `Sacrifice` hook
/// is not either way, unless marked `enter_trigger`: a sacrifice is not a leaving, so "whenever you sacrifice a
/// permanent" is not caused by one, while "whenever an artifact is put into a graveyard from the battlefield" is caused
/// by a Treasure being sacrificed. The bits are the ones the enters events use; for a leave event they say "leave-caused
/// trigger" and "a queued hook the leave did not cause".
pub fn leave_hook_is_trigger(imp: &CardImpl, e: Event) -> bool {
    let now = imp.runs_at_once(e) || !e.is_trigger();
    imp.is_enter_trigger(e) || (e != Event::Sacrifice && !now && !imp.is_enter_setup(e))
}

/// Instances of src's enters trigger: zero under an opponent's Elesh, otherwise original + event + source extras.
/// Only mixed hooks, land paths and inline proxies call this; never dispatcher-multiplied T hooks or repeat passes.
/// None denotes an artifact token only.
pub fn enters_trigger_times(g: &Game, controller: PlayerId, src: Option<PermId>, entering: Option<PermId>) -> u32 {
    if enter_silenced(g, controller, entering) {
        0
    } else {
        1 + enter_copies(g, controller, entering) + src.map_or(0, |s| crate::engine::stack::extra_for(g, s) as u32)
    }
}

/// A `TokensEnter` hook whose ability is one enters trigger per token (Haliya): queue src's `Etb` hook once for each token
/// still on the battlefield, as `fire_trigger_where` does for one permanent entering (Panharmonicon-style additional
/// times as repeats; Cloud, Delney and Roaming Throne through `queue_triggers`). The caller's hook is marked
/// `enter_queues`, so the dispatcher blanks it under an opponent's Elesh Norn and runs it only once.
/// (Hushbringer: a creature token entering triggers nothing.)
pub fn queue_etb_per_token(g: &mut Game, src: PermId, p: PlayerId, toks: &[PermId]) -> Res {
    let owner = g.perm(src).owner;
    let mut entries = vec![];
    let hushed = crate::engine::repl::hushed(g);
    for &t in toks.iter().filter(|&&t| g.perm(t).on_bf && !(hushed && g.is_creature(t))) {
        let reps = 1 + total_trigger_copies(g, owner, "etb", Some(t));
        for k in 0..reps.max(1) {
            let act = TrigAct::Hook { event: Event::Etb, call: Call::Etb { p, m: t } };
            entries.push(Trigger { repeat: k > 0, ..Trigger::new(owner, Some(src), act) });
        }
    }
    crate::engine::stack::queue_triggers(g, entries)
}

/// `fire_trigger`, for the hooked permanents `keep` accepts (a blink fires after the creature is back, and what entered
/// with it never saw it leave: `zones::exiled_from_bf`). A queued enters-caused `TokenCreated` hook (Haliya, Reckless
/// Fireweaver, Weftstalker Ardent) goes on the stack once for each artifact token made, as each is its own triggered
/// ability (CR 603.2c): a batch of n is n entries of one token, up to `zones::TOKEN_CAP` of them with the rest in the
/// last, and each of them has its own additional triggers.
pub fn fire_trigger_where(
    g: &mut Game,
    e: Event,
    call: Call,
    keep: &dyn Fn(&Game, PermId) -> bool,
    repeat: bool,
) -> Res {
    fire_trigger_entering(g, e, call, keep, repeat, None)
}

/// `fire_trigger_where` for an event whose call does not name the permanent that entered (landfall: `entering` is the
/// land), so Elesh Norn and Hushbringer can tell what caused it
pub fn fire_trigger_entering(
    g: &mut Game,
    e: Event,
    call: Call,
    keep: &dyn Fn(&Game, PermId) -> bool,
    repeat: bool,
    entering: Option<PermId>,
) -> Res {
    // the land this event is about belongs to this event alone: what is fired while its triggers are handled is not about it
    let leaving = g.leaving.take();
    let enters =
        matches!(e, Event::Etb | Event::LandEnters | Event::Landfall | Event::TokensEnter | Event::TokenCreated);
    let entering = match &call {
        Call::Etb { m, .. } => Some(*m),
        Call::Tokens { toks, .. } => toks.first().copied(),
        _ => entering,
    };
    let hushed = enters && entering.is_some_and(|m| g.is_creature(m)) && crate::engine::repl::hushed(g);
    let mut entries = vec![];
    for (src, imp) in hooked(g, e) {
        if !keep(g, src) {
            continue;
        }
        let owner = g.perm(src).owner;
        let now = imp.runs_at_once(e) || !e.is_trigger();
        let trig = enter_hook_is_trigger(&imp, e);
        // an opponent's Elesh Norn, or Hushbringer for a creature: no trigger of this permanent is caused by the entering
        // (a hook that is not a trigger, a permanent setting itself up as it enters, still runs)
        if enters && trig && (hushed || enter_blanked(g, owner, entering)) {
            continue;
        }
        let mut reps = 1;
        // a leave-caused trigger (CR 603.6c, 603.10a): Teysa Karlov, Drivnod, Gandalf the White
        if matches!(e, Event::Dies | Event::ExiledFromBf | Event::CreatureToGy | Event::LandGy | Event::Sacrifice)
            && leave_hook_is_trigger(&imp, e)
            && let Some((kind, x)) = leave_cause(e, &call, leaving)
        {
            reps += total_trigger_copies(g, owner, kind, x);
        }
        if let (Event::Etb, Call::Etb { m, .. }) = (e, &call)
            && trig
        {
            reps += enter_copies(g, owner, Some(*m)) as i32;
        }
        if e == Event::TokenCreated && trig {
            // an artifact token (Treasure, Food, Clue) entering: Panharmonicon's additional trigger, Elesh Norn's
            reps += total_trigger_copies(g, owner, "etb", None);
        }
        if let (Event::LandEnters, Call::Etb { m, .. }) = (e, &call)
            && trig
        {
            reps += enter_copies(g, owner, Some(*m)) as i32;
        }
        if let (Event::TokensEnter, Call::Tokens { toks, .. }) = (e, &call)
            && trig
            && let Some(&m) = toks.first()
        {
            reps += enter_copies(g, owner, Some(m)) as i32;
        }
        // Queue adapters run once: the real triggers get event copies and source extras when queued.
        // Landfall's later event-copy passes already repeat the hook, so only the original gains source extras.
        if imp.is_enter_queues(e) {
            reps = 1;
        } else if enters && trig && now && !repeat {
            reps += crate::engine::stack::extra_for(g, src) as i32;
        }
        // one entry per artifact token of a batch made (a hook that runs now sees the batch as it is)
        let parts: Vec<Call> = match &call {
            Call::ArtifactTokens { p, kinds, n } if e == Event::TokenCreated && trig && !now && *n > 1 => {
                let cap = (*n as usize).min(crate::engine::zones::TOKEN_CAP);
                (0..cap)
                    .map(|i| {
                        let n = if i + 1 == cap { *n - (cap as i32 - 1) } else { 1 };
                        Call::ArtifactTokens { p: *p, kinds: kinds.clone(), n }
                    })
                    .collect()
            }
            _ => vec![call.clone()],
        };
        for k in 0..reps.max(1) {
            // an event outside TRIGGER_EVENTS ('crew', 'monarch') runs at once, as Python's CI.fire does
            if now {
                let prev = g.trig_ctl.take(); // it runs now: its source's current controller is its controller
                let r = run_hook(g, src, e, &call);
                g.trig_ctl = prev;
                r?;
                if g.over {
                    return Ok(());
                }
            } else {
                for part in &parts {
                    let act = TrigAct::Hook { event: e, call: part.clone() };
                    entries.push(Trigger { repeat: repeat || k > 0, ..Trigger::new(owner, Some(src), act) });
                }
            }
        }
    }
    crate::engine::stack::queue_triggers(g, entries)
}

/// the controller of src's triggered ability being probed or resolved: whoever controlled src as it triggered (CR 603.3a),
/// even if control of src has changed since; src's current controller for a hook that runs at once
pub fn trigger_controller(g: &Game, src: PermId) -> PlayerId {
    match g.trig_ctl {
        Some((s, q)) if s == src => q,
        _ => g.perm(src).owner,
    }
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
