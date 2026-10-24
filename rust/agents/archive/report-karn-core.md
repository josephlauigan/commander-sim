# Report: karn-core (branch karn-core)

## Done
- **Wiring** for key `karn` (colourless, `ident = ""`): practice (`catalog.rs` MY_DECKS, `session/setup.rs`, five test files),
  sim-cli (`decks.rs` FILES/MINE, `args.rs`, `poolmode.rs` test table, `report.rs` plan text + milestone `karn`),
  ai (`mod.rs` MAIN_STYLES + `jd_deck`, `decks.rs` `deck_prio`, `brain.rs` GENERIC_PLAYS), engine (`cast.rs` commander
  importance, `rules.rs` CTHRESH, `settings.rs` cthresh), deck guard fixture (`cardgen deck karn`), counts in tests
  (14 decks, 98 games). A colourless deck needed no fix: 98 games (every tier) and 100 t3 games run without a panic.
- **Artifact-only mana, one mechanism**: `Floating::art` + `Source::FloatArt` (AI engine, `engine/mana.rs`, spent before any
  land; cleared when a turn starts, `engine/turn.rs`) and `ManaPool::art` (practice, `human/mana.rs`; `empty()` leaves it,
  gathered from `floating.art`). `mana::art_mana_ok(g)`: a spell being paid for (`Game::pay_for`) must be an artifact; abilities
  and taxes may use it. Powerstone tokens and The Mightstone and Weakstone (tag `pstone`) use the same test (before: never for
  abilities). Mishra's Workshop stays artifact-spells-only (exact as printed). `brain::cast_removal` / `cast_wipe` now set `pay_for`.
- **`impls/karn.rs`**: Karn (P/T = greatest MV among artifacts, upkeep mana, milestone `karn`), The Mightstone and Weakstone
  ETB (AI: -5/-5 on a threat worth 5+, else draw two; a person chooses), Tron (2/2/3) and Urza's Workshop (metalcraft) as
  `dyn_mana_land`, Mishra's Factory/Foundry (animate to attack when open; animate to block a dangerous attack; artifact while
  animated), `karn_prio` (Karn first, cheap artifacts, Greaves/Boots after Karn, O-Stone/Disk when behind, All Is Dust/Ugin
  vs a coloured board), `hold_ability` (Baubles stay out while Karn is out/coming and the hand has 2+ cards), `land_adjust`
  (Tron piece), `search_pick` (Expedition Map / Urza's Cave take the missing Tron piece, then Workshop, then Wastes), `search_bonus`
  (look-ahead: casting Karn +10, cheap artifacts with Karn out +1.5).
- Overrides (`data/card_overrides.toml`): Karn Full, Mightstone Full, Gilded Lotus (`rock=3:A`), Tron lands + Urza's Workshop +
  Mishra's Workshop Full, Urza's Factory (token is an artifact) Full, Darksteel Citadel Approximate (indestructible not
  modeled), Mishra's Factory/Foundry Approximate (pump abilities not modeled). `cardgen export` run.
- `data/printings/karn.txt`: Scryfall's default printing of every card (placeholders), `# proxy` line above the 8 proxies.
- Tests: `tests/cards_karn.rs` (13), `tests/play_karn_mana.rs` (3: a person pays with Karn's mana, refused for a planeswalker).
- 100 games vs t3: win rate 17% then 20% (CI 13-29%), Karn online (first upkeep mana) in 71% then 77% of games, average win
  turn ~16. Traces show Karn's mana paying Cityscape Leveler, Duplicant, Obelisk, etc. Not a measurement of the deck: many cards
  are still unmodeled until karn-threats / karn-engines merge.

## Shared-file edits (all small and additive)
`state.rs` (Floating.art), `engine/mana.rs`, `engine/turn.rs` (clear art; land_adjust), `human/mana.rs`, `ai/brain.rs`
(`can_pay_for`/`pay_for`, karn block response), `ai/decks.rs`, `ai/mod.rs` (dsl_search_pick), `ai/search.rs` (karn bonus),
`dsl/mod.rs` (hold Baubles; nonbasic land searches of karn skip the basic-only shortcut), `impls/mod.rs`.

## Notes for the merge
- `dsl/mod.rs search`: for key `karn` a nonbasic land search (Expedition Map, Urza's Cave) goes through the general path and
  `karn::search_pick`; karn-engines' "land-fetchers find Wastes" work (basic-land searches, `zones::basics_in`) is separate and
  untouched. Solemn / Burnished Hart / Wayfarer's Bauble (basic land) still need Wastes counted as a basic there.
- Not modeled (left to the other agents or later): Mishra's Factory/Foundry pump, Darksteel Citadel indestructible; a
  nonartifact spell cast through a path that does not set `pay_for` could still use Karn's / a Powerstone's mana (the main
  paths, removal and wipes set it).
- Python bugs found: none (no Python).
