# The Rust code

The whole project is this workspace: the engine, the AI, the cards, the three programs, and practice mode's page.

```
rust/
  Cargo.toml            the workspace; Cargo.lock pins every dependency
  rust-toolchain.toml   the pinned compiler (rustup installs it on the first cargo command)
  rustfmt.toml          max width 120
  crates/sim-core/      the library: game state, rules (engine/), AI (ai/), the ability language (dsl/), card code
                        (impls/), a person's seat (human/); tests in tests/, examples in examples/
  crates/sim-cli/       commander-sim: runs simulations, swap tests, analysis, calibration and the realism scorecard
                        (realism.rs) with the AI knobs file (knobs.rs); every option is in src/args.rs
  crates/practice/      commander-practice: practice mode's server, session, Hint and review, the terminal client,
                        and the browser page in static/
  crates/cardgen/       cardgen: builds data/cards.json and data/decks.json, edits decks, validates the pool, audits
                        cards (see its README.md)
  agents/               briefs, handoffs and reports for agent tasks (RULES.md first); finished ones in archive/
```

Build and test from `rust/`, under the memory cap in [`../CLAUDE.md`](../CLAUDE.md):

```
cargo build --release                 # rust/target/release/commander-sim, commander-practice, cardgen
cargo build --release -p sim-cli      # one program
cargo test --release                  # every test (about 3,000; about 10 minutes)
cargo test --release -p practice      # one crate
```

`make setup`, `make build` and `make test` from the repository root do the same. Format only the files you change
(`rustfmt --edition 2024 <file>`), never the whole workspace.

Where to read next:

- [`../README.md`](../README.md): what the project is, and the commands.
- [`../documents/how-it-works.md`](../documents/how-it-works.md): the engine, the card model, the AI, the run harness.
- [`../documents/realism.md`](../documents/realism.md): the realism scorecard and the plan it drives.
- [`../documents/cards.md`](../documents/cards.md): the card pipeline and model statuses.
- [`../documents/testing-decks.md`](../documents/testing-decks.md): how a deck change is tested.
- [`../documents/practice-mode.md`](../documents/practice-mode.md): practice mode, how it is built and tested.

The practice HTTP tests (`crates/practice/tests/http_*.rs`) start the server in-process with a fake Scryfall
(`PRACTICE_SCRYFALL`); set `PRACTICE_URL=host:port` to run the same suite against a server that is already running.
