# Report: cloud-ramp-cards (claude-sonnet-5-5)

All ten cards are Full in the scratch audit (`cardgen --root <scratch> audit --mine`: "== cloud: 80 unique cards, 0 not Full" with the ten
appended under 31 Plains; the real deck list is unchanged). Overrides are at the end of data/card_overrides.toml; `cardgen export`
re-run (only data/cards.json changes; the old Map/Explorer DSL searches and the Keeper/Warhound/Gift regex tags are gone).

## Modeling
- Catch-up ramp (impls/bulk_white.rs, section "the catch-up ramp"): lands = `lands.len()` on the battlefield. Each "if" is checked on
  trigger and again on resolution (`trigger_window` in between), like Knight of the White Orchid.
  - Loyal Warhound: ETB, behind -> one basic Plains onto the battlefield tapped (with landfall).
  - Oreskos Explorer: ETB, X = opponents with more lands counted at resolution; up to X Plains cards to hand.
  - Gift of Estates: `resolve` hook; behind -> up to three Plains cards to hand; goes to the graveyard either way.
  - Keeper of the Accord: `end_step` hook on each opponent's end step: Soldier token if they have more creatures, then (separate
    trigger) basic Plains tapped if they have more lands.
  - Archaeomancer's Map: ETB two basic Plains to hand; `landfall` hook (fires for every player's land): opponent ahead on lands ->
    put a land from hand onto the battlefield (Plains first, else one that enters untapped).
  - Shared `fetch_plains` (searchable, so Aven Mindcensor limits apply, shuffles).
- Rocks: Thran Dynamo `rock=3:C` and Prismatic Lens `rock=1:C` already worked in mana.rs; only the verdict was added. The Lens filter
  ability ({1},{T}: any color) is NOT modeled (documented in the note): it only converts one mana into a coloured one and the deck
  needs only {W}; arguably this should be Approximate, JD's call.
- Equipment (impls/cloud.rs): Kor Outfitter ETB (`outfitter_etb`, best loose / worse-worn Equipment onto the best holder = Cloud);
  Brass Squire (`squire_options`, `squire_go`: {T}, before combat on your turn, ignores itself as target, not while sick);
  Auriok Steelshaper: `equip_cost` -1 per Steelshaper (generic only: Veteran's Powerblade's {W} for a Soldier is untouched), and
  `steelshaper_bonus` (+1/+1 per equipped Steelshaper for Soldiers and Knights).
- AI priorities for the ten in `cloud_prio` (Gift 0 unless behind; Warhound/Explorer 60 behind, else 36; Map 58; Keeper 56;
  Outfitter 66 with Cloud + Equipment out; Squire 60; Steelshaper 58 with Equipment out).

## Shared-file changes
- `dsl/mod.rs` `pt()`: 3 lines adding `cloud::steelshaper_bonus` next to the `base_pt` call.
- No other shared file touched. impls/elrond.rs untouched.

## Checks
- New tests: tests/cards_cloud_ramp.rs (17 tests, every card at least once).
- `cargo test --release -j 4 --no-fail-fast`: all pass except practice `http_lan::test_a_saved_two_player_game_waits_for_the_friend`
  (120 s timeout). It fails identically on the base commit (checked in a scratch worktree), so it is pre-existing, not from this change.
- `cargo build --release --all-targets`: no warnings. all_decks adaptive 3 and lookahead 1: no panic.
- Smoke (48 games lookahead): fine. `--deck cloud --pool t3 --profile loose --games 240 --seed 500000 --ai lookahead --jobs 8`:
  output identical before and after (diff of the report minus progress/timing lines).
