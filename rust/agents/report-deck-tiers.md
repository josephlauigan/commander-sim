# Report: deck-tiers

Rule (JD, 2026-10-14): each of JD's decks simulates only against its matched opponent tiers.

## Data
- `data/my_decks.toml`: `tiers = [...]` per deck (yshtola/seph t4,t5; sheoldred t3,t4; sauron/kefka t2-t4; sephfs/veyran t1-t3;
  cloud/titania t1,t2; alela/galadriel t1).
- `cardgen` (`data.rs`): `MyDeck.tiers`, `get_tiers` errors on missing / empty / unknown / duplicate tiers (so `cardgen validate`
  and `export` fail for a deck without them). `export.rs` writes `mine[].tiers` into `data/decks.json` (regenerated).
- `sim-core` `RawDeck.tiers` (serde default empty; pool decks carry none); `testkit.rs` retired decks set `tiers: vec![]`.
- `tests/fixtures/my_decks_parsed.json` holds card lists only: unchanged, still current.

## CLI (sim-cli)
- New `--any-tier` flag. `poolmode.rs`: `deck_tiers`, `select_tiers`, `tier_plan`, `tier_range`.
  - `--deck K --pool all` (plain, `--analyze`, swaps): the deck's own tiers. `--pool tN` outside them: refused with a message naming the deck's tiers; `--any-tier` allows it (and makes `all` mean t0-t5, the old behaviour).
  - `--all-decks`: each deck in its own tiers; with `--pool tN` only the decks that play tN (all with `--any-tier`). Matrix prints `-` where a deck doesn't play a tier and a new column "average over its tiers" (`38.2% over t1-t3`).
  - `--trace` without a tier or with `all`: the deck's first tier (was t1); outside tier refused like above.
  - `--calibrate` unchanged (all six tiers).
- `searchtest` (tools.rs): a deck of JD's is refused outside its tiers unless `--any-tier` is among the arguments. `swaptest` works on pool decks only: unchanged.
- Help text, usage examples updated.

## TIERS loops
- `poolmode.rs` `main` (tier list, work plan, matrix): changed. `matrix`: changed (per-deck tiers).
- `poolmode.rs` `run_calibration` (within / ordering loops over `TIERS`): intentionally unchanged (no deck).
- `trace`: changed. `tools.rs` searchtest: changed.
- `sim-core/examples/all_decks.rs`: JD's decks skip tiers outside their own (t0 is now not exercised there; pool decks at t0
  still never play as "me" in any run).
- Not changed: `practice` crate (`catalog.rs`, `session`, routes: the human picks any tier for a practice game, not a simulation);
  `decks.rs` `TIERS` constant and its test (pool layout).

## Tests
Unit tests: args (`--any-tier`, prefix `--any`), poolmode (`select_tiers`, refusal + override, `tier_range`, the tier table
against `decks.json`, pool decks have none), cardgen (`get_tiers`). `cargo test --release`: all pass; `cargo build --release
--all-targets`: no warnings. `all_decks adaptive 3` (72 games) and `lookahead 1` (24 games): no panic.

## Checks
- `--deck sephfs --pool all --games 48 --ai lookahead --profile loose --brief`: ran t1, t2, t3 only.
- `--deck yshtola --pool t1`: refused; with `--any-tier` it runs. `--deck kefka --pool t1 --trace 0` refused.
- Same seed (7, 48 games, lookahead, loose, brief), old binary (origin/main) vs new, output identical apart from the
  timing line: sauron t3 (35.4%), cloud t1 (4.2%), seph t4 (41.7%).

## Shared files touched
`sim-core/src/export.rs` (`tiers` field), `sim-core/src/testkit.rs` (one line). Docs: `commander-sim/CLAUDE.md` ("Opponent
tiers"), `rust/agents/OVERNIGHT-RULES.md`.
