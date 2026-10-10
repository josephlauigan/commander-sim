# Rules for agents working on the Rust port

You are one of several agents porting a Python Magic: The Gathering Commander simulator (`commander_sim/`) to Rust
(`rust/`). The user has decided to move fully to Rust and remove Python later. Read this file, then your task brief
(`rust/agents/<task>.md`), then your handoff file if it exists.

## Working
- **Your worktree is your whole world.** `cd` into it for every command. First run `git merge --no-edit rust-port`
  to pick up the latest merged work (resolve conflicts keeping both sides' intent).
- **Python is the frozen reference**: read it, never edit it. Port faithfully, bugs included (so Rust can be
  validated against Python), except bug fixes your brief lists, which you make in Rust only with a test and a
  comment saying what Python did. Other Python bugs: port as is and list them in your report.
- Never let an unsigned counter go below zero (release builds have overflow-checks: it would panic).
- **Memory cap on everything** (cargo build/test, example binaries, difftest.py, Python scripts):
  `systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0 <command>`. Exit 137 = the cap was hit: a bug
  (something growing without bound), never a flake. An uncapped runaway once took down the user's editor.
- Toolchain: `export PATH=$HOME/.cargo/bin:$PATH`; build from `rust/`. In zsh a command stored in a variable isn't
  word-split: write commands out.
- Conventions: `rust/PORTING_CARDS.md`; the pattern `rust/crates/sim-core/src/impls/t1.rs`; how the engine calls
  card code: `hooks.rs`, `engine/hooks.rs`, `cardcode.rs` (placeholders `PORT(M5)` / `PORT(phase 6)`);
  registration order `impls/mod.rs` (Python's import order); shared machinery `impls/common.rs` (reuse, don't copy).
- Edit only the files your brief gives you. Shared files (cardcode.rs, state.rs, hooks.rs, engine/*, ai/*): smallest
  additive change, listed in your report. Format only your files: `rustfmt --edition 2024 <files>`, never
  `cargo fmt` on the workspace.
- Checks before you finish: `cargo test --release` passes; `cargo build --release --all-targets` has no warnings;
  `./target/release/examples/all_decks adaptive 3` and `all_decks lookahead 1` finish without a panic; and the
  difftest command in your brief (0 AI and 0 build differences, none of the card-code differences from your cards
  alone).

## Keeping context small (important)
- **Commit often** (standalone `git add -A rust/ && git commit -m "..." -m "Co-Authored-By: Claude Sonnet 5.5
  <noreply@anthropic.com>"`), at least after each finished piece. Don't push.
- Keep **`rust/agents/handoff-<task>.md`** in your worktree up to date: what's done, what's next (in order), and
  gotchas you found. Commit it with your work.
- Don't read huge files whole: grep, then read the parts you need. Keep command output short (`| tail`, `grep`).
- **When your context gets long** (roughly 60+ tool calls, or you notice you're re-reading things), stop at a clean
  point: everything committed, handoff file updated, then end with a report whose first line is `HANDOFF` (a fresh
  agent will continue from your handoff file).
- **Reports are short**: at most 15 lines (commit hash, done / not done, test and difftest results, anything I must
  act on). Put the full report (cards ported, engine changes, Python bugs found, merge notes) in
  `rust/agents/report-<task>.md` in your worktree and commit it.
