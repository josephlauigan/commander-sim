# Rules for the overnight card-modeling agents (2026-10-11)

You are one of three agents writing Rust card code so two new decks, **Kefka** (`kefka`) and **Zada** (`zada`), are
fully modeled and can be measured against the other decks. Read this file, then your brief
(`rust/agents/<task>.md`), then your handoff file `rust/agents/handoff-<task>.md` if it exists.

These rules replace `rust/agents/RULES.md`, which was written for the Python port: **Python is gone**.

## Hard rules
- **Never run Python.** No `python3`, no `rust/tools/difftest.py`, no old audit scripts, not even a helper one-liner.
  Use shell tools (`grep`, `sed`, `awk`, `jq`), `cardgen` and `commander-sim`, or a Rust test or example.
- **Your worktree is your whole world.** `cd` into it for every command. Its branch was made from
  `overnight-models`; don't merge anything else in, and don't push.
- **Memory cap on every build, test and binary:** `systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0
  <command>`. Exit 137 means the cap was hit: that's a bug (something growing without bound), never a flake.
- Toolchain: `export PATH=$HOME/.cargo/bin:$PATH`; build from `rust/`. In zsh, write commands out (a command stored in
  a variable isn't word-split).
- Never let an unsigned counter go below zero (release builds have overflow checks).
- **Edit only the files your brief gives you.** Shared files (`cardcode.rs`, `state.rs`, `hooks.rs`, `engine/*`,
  `ai/*`, `impls/mod.rs`, `impls/common.rs`): only the smallest additive change, and list each one in your report.
  Other agents are editing the same shared files in parallel; keep your changes there small so merges stay easy.
  Format only your own files: `rustfmt --edition 2024 <files>`, never `cargo fmt` on the workspace.

## How cards are modeled
- Read `rust/PORTING_CARDS.md` (written for the port, but the hook-slot pattern still applies) and follow
  `rust/crates/sim-core/src/impls/t1.rs` as the pattern. Card data: `data/carddb.txt` (hand tags),
  `data/card_overrides.toml` (overrides and the audit verdict), Scryfall text compiled by `cardgen`. Engine entry
  points: `hooks.rs`, `engine/hooks.rs`, `cardcode.rs`. Deck AI dispatch: `ai/decks.rs` (`deck_prio`, tutor picks
  and friends) and `ai/brain.rs` (deck options); a deck's own module (for example `impls/yshtola.rs`, `impls/jodah.rs`)
  holds its plan.
- **Model each card from its Oracle text** (`jq '.["card name lowercased"]' data/scryfall_cache.json`, or
  `cardgen` output). Reuse machinery in `impls/common.rs` and the engine before writing new machinery.
- "Fully modeled" means `rust/target/release/cardgen audit --mine` lists the card as **Full**. Find out how the
  audit decides that (`rust/crates/cardgen/`), and mark a card Full only when its whole Oracle text is implemented
  and covered by a test. If some clause truly can't matter in these games, say so in the card's doc comment and in
  your report; don't hide it.
- After changing card data, run `rust/target/release/cardgen export` (it rewrites `data/cards.json` and
  `data/decks.json`).
- Tests go in `rust/crates/sim-core/tests/cards_<task>.rs`, built with `sim_core::testkit` (`table`, `hand`, `lands`,
  `perm`, `token`). One test per card at least, checking the behaviour that matters.

## Checks before you finish
1. `cargo test --release` passes (all of it, not only your tests).
2. `cargo build --release --all-targets` has no warnings.
3. `./target/release/examples/all_decks adaptive 3` and `./target/release/examples/all_decks lookahead 1` finish
   with no panic (under the memory cap).
4. `cardgen audit --mine`: every card your brief lists is Full.
5. A short smoke run of your deck finishes: `./target/release/commander-sim --deck <kefka|zada> --pool t3 --profile
   loose --games 48 --ai lookahead --jobs 8 --brief`.

## Keeping context small
- **Commit often** (`git add -A rust/ data/ && git commit -m "..." -m "Co-Authored-By: Claude Sonnet 5.5
  <noreply@anthropic.com>"`), at least after each finished card group. Never push.
- Keep `rust/agents/handoff-<task>.md` up to date (done / next in order / gotchas) and commit it with your work.
- Grep, then read the parts you need; keep command output short (`| tail`, `| grep`).
- **When your context gets long** (about 60 or more tool calls, or you notice you're re-reading things), stop at a
  clean point: everything committed, handoff file updated, then end with a report whose first line is `HANDOFF`.
  A fresh agent continues from your handoff file.
- **Your final report is at most 15 lines**: the commit hash, done / not done, test results, the shared-file changes,
  anything the lead must act on. Put the full report (cards done, how each was modeled, engine changes, open
  questions) in `rust/agents/report-<task>.md` and commit it. Your work will be checked: the diff is read, the tests
  rerun and the claims spot-checked before anything is merged.
