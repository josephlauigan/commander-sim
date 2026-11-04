# Report: Garth One-Eye

## Cards
- **Garth One-Eye** (status Full): `impls/garth.rs`. `{T}`: choose an unchosen name of the six, create a copy, cast it
  paying `cost_of` (The Ur-Dragon's eminence takes {1} off Shivan Dragon) through `cast_card` with the zone "copy" (a
  real cast: cast triggers, counterspells, stack; no card moves; a permanent copy enters as a token). A copy that
  cannot be cast, whose target left, or that is countered ceases to exist (`garth::cast_copy` removes a countered
  copy from the graveyard). Chosen names: `DataKey::Garth`, a bit mask on the Garth permanent, so a new Garth
  (after it changed zones) chooses all six again.
- **Black Lotus**: `rock=3:A` token artifact; `rules::petal_sac` (made `pub`) sacrifices it when tapped.
- **Braingeyser**: the compiled "target player draws X" is an opponent in the ability language, so the override
  writes it as "you draw X" (the AI targets itself). X is paid by Garth's cast.
- **Regrowth**, **Shivan Dragon**: compiled as before, status Full.
- **Terror** (pool cards, shared): was `destroy target creature`; now `rem=destroy tgt=cna nonblack, dsl = []`,
  like Disenchant, so it is nonartifact and nonblack as printed. This also changes Terror in the pool decks that
  play it (more accurate; no longer hits black or artifact creatures).

## AI (`garth::options`, Event::Options)
Garth is offered only for a copy that can be cast now and is worth casting: Terror / Disenchant at an opposing
target with pval >= 2.5 / 2.0 (also in the end-of-turn window); Shivan Dragon when affordable (card_utility + 1);
Regrowth when the best graveyard card is a nonland with bomb >= 3 or mana value >= 3; Braingeyser when X >= 3 (UU
paid, library keeps 8); Black Lotus at utility 6 when its three mana make a hand card or the commander castable this
turn, else 0.5 (last resort). Sorcery-speed names only in a main phase. `data/plans/urdragon.toml` untouched.

## Shared files touched
- `rust/crates/cardgen/src/export.rs`: the four cards added to the always-exported `extra` list.
- `data/card_overrides.toml`, `data/cards.json`, `data/scryfall_cache.json` (four cards fetched), via `cardgen export`.
- `rust/crates/sim-core/src/state.rs`: `DataKey::Garth`.
- `rust/crates/sim-core/src/impls/rules.rs`: `petal_sac` made `pub`.
- `rust/crates/sim-core/src/impls/mod.rs`: module and registration.

## Simplifications
- Braingeyser always targets its caster; Terror's "can't be regenerated" is not modeled.
- A person's Garth (practice mode) gets no choice prompt; the AI hook is skipped for humans.
- Black Lotus's colour choice is the engine's `rock=3:A`.

## Checks
`cargo test --release --no-fail-fast`: all pass except `plans::every_plan_file_fits_its_deck` (known, urdragon) and
two `play_urdragon_human` tests (`godo_lets_you_search_for_any_equipment`, `the_camera_copies_godos_enter_trigger`:
"no Embercleave / Hammer in the search choices"), which fail identically on base e6243581. `cargo build --release
--all-targets`: no warnings. `tests/cards_garth.rs`: 16 tests pass. No simulations run.
