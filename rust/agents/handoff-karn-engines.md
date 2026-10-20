# Handoff: karn-engines

## Done
- Part 1 (6 new cards) in `rust/crates/sim-core/src/impls/karn_engines.rs` (registered last in impls/mod.rs), tests in
  `tests/cards_karn_engines.rs` (18 pass). `Player::silver` (state.rs) holds Karn's silver-counter cards.
  Mystic Forge's and Urza's Tome's DSL set to [] in card_overrides.toml (hand code replaces it). `cardgen export` run.

## Next (Part 2, in order)
Land abilities are NOT driven by DSL (dsl::ability_options only walks perms): Urza's Cave's compiled search never runs
and its tag `c:"A"` is wrong (it taps for {C}); Inventors' Fair has only c=C; Rogue's Passage: check combat.rs:1088.
Then the artifact DSL cards (Batterskull living weapon/{3} return, Hangarback X, Endless Atlas restriction, Expedition
Map / Cave need "land card" (any land: land_ramp only fetches basics), Liquimetal Torque 2nd ability, Pithing Needle
(wire `t6_stax::needle_stops` into `rules::ability_locked`), Staff of Domination infinite-mana draw-down.

## Gotchas
- `perm()` in tests flushes triggers; walker options come from the registry's `options` fn.
- Library top is the LAST element.
