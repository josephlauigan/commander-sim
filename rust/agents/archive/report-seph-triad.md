# Report: seph-triad
Sephiroth, Fabled SOLDIER // One-Winged Angel (impls/sheoldred.rs, `register_seph` via `register_upgrade`): enters/attacks
sacrifice (front: one, back: all fodder; the AI sacrifices only creatures with sac_worth < 2.2, never a commander or
Sheoldred) and draw; another creature dying drains the weakest opponent; the 4th resolution in a turn (DataKey::SephTimes,
stamp + count) sets Flipped (5/5 flying via self_pt/grant_kw) and adds a "seph" emblem; the emblem drain runs from
zones::die_triggers through cardcode::emblem_dies. Overrides: dsl = [] (the auto DSL would double the drain), kws = [],
tags without flying. Name stays the front's. Triple Triad (impls/mine.rs `triad_upkeep`): all players exile their top card;
the controller plays its own card and each lesser-MV card free, at once in the upkeep, highest first (lands only with a land
drop left, no Auras or counterspells); ownership kept (Enter.orig / Ctx.owner); unplayed cards stay in exile.
Simplification: sorcery-speed cards are played in the upkeep, not the main phase.
Tests: tests/cards_seph_triad.rs (7). Shared-file changes: state.rs (DataKey::SephTimes), cardcode.rs (emblem_dies),
engine/zones.rs (3 lines in die_triggers), data/card_overrides.toml, data/cards.json, data/scryfall_cache.json.
Measurement: audit/sheoldred/sephiroth/README.md (Dark Deal cut: +0.0 / +0.3 / -0.7 / +3.1 / -0.1, all tiers +0.5, no clear effect).
Open: cardgen audit could not be run on the swapped list (offline deck replace failed), so Full rests on the overrides.
