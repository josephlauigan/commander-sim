# Report: retire-omnath-godo (rust side)

Commits (branch retire-omnath-godo-rust, not pushed): 9242a8b5 brief; 74fbc0f5 key lists + count tests + testkit;
961bf690 cardgen export / cards.json / fixture / docs; 2c2a08e9 real lists for the retired seats, sim-cli count,
realism test; plus this report.

## Deck-key wiring removed (14 -> 12)
- sim-cli: `decks.rs` FILES and MINE (and the `the_repository_decks_load` test, 12), `args.rs` DECK_KEYS,
  `poolmode.rs` tier test, `report.rs` omnath plan text / plan turn.
- practice: `catalog.rs` MY_DECKS and the results-table `"omnath"` row key, `session/setup.rs` MY_DECKS,
  `advisor/mulligan.rs` Godo branches (dead: practice can no longer seat godo).

## Kept (on purpose)
- All card code: Omnath, Titania, the landfall cards, Godo, Helm of the Host, `impls/t6_godo.rs`, `t6_plan_godo`,
  Tier 6 plan entries.
- Key gates the retired test seats still run, so the tests test the same behaviour: `ai/mod.rs` MAIN_STYLES and
  `jd_deck` (omnath), `settings.rs` cthresh, `engine/cast.rs` commander value, `impls/rules.rs` CTHRESH,
  `impls/t6_plans.rs` and `t6stats.rs` T6_KEYS (godo; comment now says "retired, a test seat"), `ai/brain.rs`,
  `ai/decks.rs`, `impls/titania.rs` omnath AI, `impls/t2.rs` omnath milestone. None is reachable from the CLI or
  practice (the keys are refused). ai/decks.rs, cast.rs, cardcode.rs, combos/common/fixes untouched (PR #142).

## Tests
- testkit `retired_decks`: `omnath` and `godo` seats with their **real lists**, parsed at compile time
  (`include_str!`) from `decklists/pool/retired/omnath-landfall.md` and `godo-cedh-equipment.md` (restored from
  79d92e6f^; checked identical to the old decks.json lists). A commander-plus-basics seat failed 55 tests that
  draw/search their libraries, so the real lists were needed. All ~40 test files that seat omnath/godo are unchanged.
- Count tests -> 12: practice app_entry, http_catalog, http_server, server_fake; sim-core game.rs (84).
- sim-cli realism test: `--deck godo --pool t6` -> `--deck karn --pool t5` (same "drop --deck" check).
- Dropped: `practice/tests/hint_mulligan_godo.rs` (tested the removed practice-only Godo mulligan hint; the Godo
  keep rule itself is still covered by the sim-core Tier 6 keep tests).

## Data fixes (79d92e6f's re-export was incomplete)
- `cardgen export` had dropped Shatterskull Smashing (double-faced; card code in t6_lands.rs, 2318 test panics) and
  lost Karplusan Forest's `pain` and Silhana Starfletcher's `dork=G` tag edits (overrides only apply to played
  cards). `cardgen/src/export.rs` `wanted()` now adds these three to `played`; re-export restores data/cards.json to
  its pre-retirement content exactly. decks.json unchanged.
- `tests/fixtures/my_decks_parsed.json` had a trailing newline the generator doesn't write
  (`deck_guard_fixture_is_current` failed): removed.

## Docs
- `documents/how-it-works.md`: "six Tier 6 pool decks (and the retired godo, kept as a test seat)".
- `decklists/pool/README.md`: one line on the two JD lists kept in retired/ as test decks.
- No other rust/ doc lists JD's decks.

## Checks
- `cargo build --release -j 6 --all-targets`: 0 warnings. `cargo test --release -j 6 --no-fail-fast`: 3297 passed,
  0 failed.
- `commander-sim --deck omnath|godo` refused (invalid choice; 12 keys listed). `--deck cloud --pool t1 --games 50
  --jobs 6` runs (32 s). `cardgen export --offline`: git status clean afterwards.
- Practice catalog lists 12 decks (app_entry / http_catalog / server_fake tests).
