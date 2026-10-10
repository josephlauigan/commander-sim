# The Rust port

The simulator's engine, AI, cards and practice mode, ported from the Python package `commander_sim`. Scope, phases
and status: `documents/rust-rewrite-scope.md`.

```
rust/
  Cargo.toml          the workspace
  crates/sim-core/    engine, rules, AI, cards (a library)
  crates/sim-py/      the Python module rust_sim, so the Python run harness can play games in Rust
  crates/practice/    practice mode's server (later; it will replace python3 -m commander_sim.play)
  build-py.sh         builds rust_sim and copies it to commander_sim/rust_sim.so
```

Card data comes from Python: `python3 -m commander_sim.tools.export_cards` writes `data/cards.json` and
`data/decks.json`, which Rust loads. Re-export after any change to cards or deck lists.

Toolchain: Rust via rustup (`~/.cargo/bin`). Run the Rust tests with `cargo test` from `rust/`.

`python3 rust/tools/difftest.py` (from the repository root) is the differential harness: the same positions in both
engines, the AI's options and scores compared (see `PORT_STATUS.md`).
