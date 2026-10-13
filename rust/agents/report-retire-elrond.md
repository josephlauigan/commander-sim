# Report: retire-elrond

Removed the Elrond deck (key `elrond`); card code kept, module renamed `impls/elrond.rs` -> `impls/blue_cards.rs`.

## Removed
- decklists/JD/elrond-ring-tempts.md, `[elrond]` in data/my_decks.toml, fixture entry, `cardgen export` rerun
  (data/cards.json 3 lines: Hope-Ender Coatl loses its `soft=1` tag, see below; data/decks.json -108 lines).
- Key from sim-cli decks.rs/args.rs/report.rs/poolmode.rs, ai/brain.rs, ai/decks.rs, ai/mod.rs, engine/cast.rs,
  impls/rules.rs, settings.rs, cardcode.rs (hold block), impls/mine.rs (ring_pick branch), practice catalog.rs, and the
  counts in practice tests, sim-core/tests/game.rs (72 -> 66). No `"elrond"` string left in rust/crates.
- Deck-only AI in blue_cards.rs: ring_pick, elrond_prio, flash_hold (and the prio registration of the flash creatures),
  elrond_special / exile_options / exile_go (recasting warped creatures from exile), elrond_protect, fear_prio,
  entry planner (entries_of, entry_set, tempt_set), the `elrond` milestone. ai/search.rs had nothing keyed to elrond.

## Kept
All card models (Elrond, Gandalf's draw, Defiler discount, Mouth of the Storm, warp mechanic via warp_hand/warp_end,
the rest). Ring-bearer now the generic Army-first pick for every deck. Doc comments in blue_cards.rs, bulk_blue.rs,
bulk_white.rs rewritten. state.rs comment already card-specific.

## Tests
tests/cards_elrond.rs -> cards_blue.rs (deck stand-in `sheoldred`); deleted ring-bearer pick, Elrond bounce, plan
(turn-three Elrond, flash holds, creatures-first), entry planner, Fear prio and warp-from-exile recast. Gandalf test uses
Glorybringer as the other creature (generic pick prefers the bigger one). cards_bulk_blue.rs re-keyed.

## Results
- `cargo test --release --no-fail-fast`: all pass except practice tests that fail without my change too:
  `http_lan::test_a_saved_two_player_game_waits_for_the_friend` (fails identically on the base commit, 120 s timeout) and
  an intermittent `http_play` test (different one each run; passes on rerun).
- Build with --all-targets: no warnings. `all_decks adaptive 3` and `lookahead 1`: no panic (66 games each).
- `cardgen audit --mine`: no elrond; other decks list the same Partial/Approximate cards as before (0 new).
- cloud and sauron 240-game t3 loose lookahead seed 500000: output identical to base (progress/timing lines aside).

## Notes for the lead
- cardgen applies override `add_tags` only to cards some deck plays, so Hope-Ender Coatl (bulk only now) lost
  `soft=1` in data/cards.json; the override entry stays and returns if a deck plays it. Its Soft assertion was dropped from the test.
- The AI never recasts warped creatures from exile now (that was elrond_special); warp itself still works.
