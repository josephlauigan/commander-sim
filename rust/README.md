# The Rust code

The whole project: the simulator's engine, AI and cards, the three programs, and the browser page for practice mode.
The overview and the common commands are in the top-level `README.md`; how the code fits together is in
`documents/architecture.md` and `documents/practice-mode.md`. The history of the move from Python (which is gone) is in
`documents/rust-rewrite-scope.md`.

```
rust/
  Cargo.toml          the workspace
  crates/sim-core/    engine, rules, AI, cards (a library)
  crates/sim-cli/     commander-sim, the command-line program that runs simulations (see its README.md)
  crates/practice/    commander-practice, practice mode's server and terminal client; the browser page is in
                      crates/practice/static/
  crates/cardgen/     cardgen: builds data/cards.json and data/decks.json, edits decks, audits cards (see its README.md)
```

Card data comes from `cardgen`: `cargo run --release -p cardgen -- export` (from `rust/`) writes `data/cards.json`
and `data/decks.json`, which Rust loads, from `data/carddb.txt`, `data/card_overrides.toml`, the Scryfall cache and the
deck lists in `decklists/`. Re-export after any change to cards or deck lists. `cardgen deck` replaces a deck's list,
`cardgen audit` reports how faithfully cards are modeled; how to add a card or edit a deck is in
`crates/cardgen/README.md`.

Running simulations: `cargo build --release -p sim-cli`, then `rust/target/release/commander-sim`
(`--deck sauron --pool t1 --profile loose --games 960`, `--swap`, `--calibrate`, `--analyze`, `--trace` ...). See
`crates/sim-cli/README.md`.

Toolchain: Rust via rustup (`~/.cargo/bin`). Run the tests with `cargo test --release` from `rust/`.

## Practice mode

```
cd rust && cargo build --release -p practice
./target/release/commander-practice [--port 8765] [--no-browser] [--lan] [--data DIR] [--static DIR]
./target/release/commander-practice --text --deck sauron --tier t4 [--step]      # the terminal client
```

Serves the page in `crates/practice/static/` (default data folder `data/`, saves in `data/saves/`) and opens it in the
browser unless `--no-browser`. `--lan` listens on the local network too for a two-player game and prints the
addresses. `--text` plays in the terminal.

Tests: `cargo test --release -p practice`. The HTTP suite (`tests/http_*.rs`) starts the server in-process with a fake
Scryfall (`PRACTICE_SCRYFALL`) and temp saves/images folders; set `PRACTICE_URL=host:port` to run the same suite
against a server that is already running. Tests that need a long human-seat game are `#[ignore]`d with the reason.
