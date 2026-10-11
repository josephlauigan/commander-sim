# The Rust port

The simulator's engine, AI, cards and practice mode, ported from the Python package `commander_sim`. Scope, phases
and status: `documents/rust-rewrite-scope.md`.

```
rust/
  Cargo.toml          the workspace
  crates/sim-core/    engine, rules, AI, cards (a library)
  crates/sim-cli/     commander-sim, the command-line program that runs simulations (the run harness, no Python)
  crates/sim-py/      the Python module rust_sim, so the Python run harness can play games in Rust
  crates/practice/    practice mode's server, commander-practice (replaces python3 -m commander_sim.play)
  crates/cardgen/     card data and deck lists without Python: builds data/cards.json and data/decks.json, edits
                      decks, audits cards (see its README.md)
  build-py.sh         builds rust_sim and copies it to commander_sim/rust_sim.so
```

Card data comes from `cardgen`: `cargo run --release -p cardgen -- export` (from `rust/`) writes `data/cards.json`
and `data/decks.json`, which Rust loads, from `data/carddb.txt`, `data/card_overrides.toml`, the Scryfall cache and the
deck lists in `decklists/`. Re-export after any change to cards or deck lists. `cardgen deck` replaces a deck's list,
`cardgen audit` reports how faithfully cards are modeled; how to add a card or edit a deck is in
`crates/cardgen/README.md`. (`python3 -m commander_sim.tools.export_cards` is the frozen Python original; it writes
the same files.)

Running simulations: `cargo build --release -p sim-cli`, then `rust/target/release/commander-sim` with the options
`python3 -m commander_sim` took (`--deck sauron --pool t1 --profile loose --games 960`, `--swap`, `--calibrate`,
`--analyze`, `--trace` ...; no `--engine`): the same seats, seeds, statistics and reports. See
`crates/sim-cli/README.md`.

Toolchain: Rust via rustup (`~/.cargo/bin`). Run the Rust tests with `cargo test` from `rust/`.

`python3 rust/tools/difftest.py` (from the repository root) is the differential harness: the same positions in both
engines, the AI's options and scores compared (see `PORT_STATUS.md`).

## Practice mode

```
cd rust && cargo build --release -p practice
./target/release/commander-practice [--port 8765] [--no-browser] [--lan] [--data DIR] [--static DIR]
```

Serves the same page as `python3 -m commander_sim.play` (default data folder `data/`, saves in `data/saves/`, page
files from `rust/crates/practice/static/`) and opens it in the browser unless `--no-browser`. `--lan` listens on the
local network too for a two-player game and prints the addresses. The terminal table (`--text`) and its options are
accepted but not ported yet. The human seat's full game needs the practice-core and practice-cards work; until then a
person is only asked the mulligan.

Tests: `cargo test --release -p practice`. The HTTP suite (`tests/http_*.rs`) starts the Rust server in-process with
a fake Scryfall (`PRACTICE_SCRYFALL`) and temp saves/images folders; set `PRACTICE_URL=host:port` to run the same
suite against another server (for example the Python one). Tests that need the full human seat are `#[ignore]`d with
the reason; run them with `-- --ignored` once practice-core lands.
