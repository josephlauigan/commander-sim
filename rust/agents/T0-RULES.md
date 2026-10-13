# Rules for the Tier 0 card-modeling agents (2026-10-12)

You are one of five agents writing Rust card code for **Tier 0**, a new opponent tier of five unmodified WotC Commander
precons (`decklists/pool/t0-b2-precons/`), so they play at full strength. Read this file, then
your brief (`rust/agents/<task>.md`), then your handoff file `rust/agents/handoff-<task>.md` if it exists.

These rules replace `rust/agents/RULES.md` (written for the Python port): **Python is gone**.

## Hard rules
- **Never run Python.** No `python3`, no `rust/tools/difftest.py`, no old audit scripts, not even a helper one-liner
  (a hook refuses it). Use shell tools (`grep`, `sed`, `awk`, `jq`), `cardgen` and `commander-sim`, or a Rust test or
  example.
- **Your worktree is your whole world.** `cd` into it for every command. Its branch was made from
  `tier0-precons`; don't merge anything else in, and don't push.
- **Memory cap on every build, test and binary:** `systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0
  <command>`. Exit 137 means the cap was hit: that's a bug (something growing without bound), never a flake.
- **Five agents share the machine:** build and test with `-j 4` (`cargo test --release -j 4`), smoke runs with
  `--jobs 4`.
- Toolchain: `export PATH=$HOME/.cargo/bin:$PATH`; build from `rust/`. In zsh, write commands out (a command stored in
  a variable isn't word-split).
- Never let an unsigned counter go below zero (release builds have overflow checks).
- **No wait loops.** Don't write `until ! pgrep -f "cargo build"; do sleep ...; done` or anything like it. Run the
  build or test in the foreground (with a long timeout). Leave nothing running when you finish.
- **Edit only the files your brief gives you.** Shared files (`cardcode.rs`, `state.rs`, `hooks.rs`, `engine/*`,
  `ai/*`, `dsl/*`, `impls/common.rs`): only the smallest additive change, and list each one in your report. The other
  four agents edit the same shared files in parallel; keep your changes there small so merges stay easy. Don't touch
  `impls/mod.rs` (all five modules are already registered). Format only your own files:
  `rustfmt --edition 2024 <files>`, never `cargo fmt` on the workspace.
- **Stay in your lane.** Each shared mechanic has one owner (table below). If you need one you don't own, don't
  build it: model what you can, leave the card Partial, and say in your report what you were waiting for.

| Mechanic | Owner |
|---|---|
| Cycling and landcycling (the "when you cycle" triggers), the monarch, Clues and investigate, lifegain triggers | t0-giada |
| Myriad, goad, mobilize, fabricate, hideaway, lieutenant, token doubling | t0-zurgo |
| Plot, suspend, storm, cascade, jump-start, copying spells, adapt, Vehicles and crew | t0-stella |
| Sagas and read ahead, constellation, devotion, channel, landfall | t0-anikthea |
| Heroes and Hero cost reductions, Backgrounds (Folk Hero), casting spells as though they had flash, "becomes tapped" triggers | t0-captain |

## How cards are modeled
- Read `rust/PORTING_CARDS.md` (written for the port, but the hook-slot pattern still applies) and follow
  `rust/crates/sim-core/src/impls/t1.rs` as the pattern. Card data: `data/carddb.txt` (hand tags),
  `data/card_overrides.toml` (overrides and the audit verdict), Scryfall text compiled by `cardgen`. Engine entry
  points: `hooks.rs`, `engine/hooks.rs`, `cardcode.rs`. Deck AI dispatch: `ai/decks.rs` (`deck_prio`, tutor picks
  and friends) and `ai/brain.rs` (deck options). These are **outside (pool) decks**: they have no hand-written plan and
  play through the generic outside-deck AI (`ai/pool.rs`): give every card a sensible default use there or in its
  card code (when to cast it, its targets), and check in a trace that the deck actually uses it.
- **Model each card from its Oracle text** (`jq '.["card name lowercased"]' data/scryfall_cache.json`, or `cardgen`
  output). Reuse machinery in `impls/common.rs` and the engine before writing new machinery: grep first (equip,
  hexproof, ward, proliferate, face_down, tempt/ring_bearer, warp, spree and kicker all exist in some form).
- "Fully modeled" means `rust/target/release/cardgen audit --mine` lists the card as **Full**. Find out how the
  audit decides that (`rust/crates/cardgen/`, and how earlier agents marked cards: `git log -p -S'verdict'
  -- data/card_overrides.toml | head`), and mark a card Full only when its whole Oracle text is implemented and covered
  by a test. If some clause truly can't matter in these games, say so in the card's doc comment and in your report;
  don't hide it.
- After changing card data, run `rust/target/release/cardgen export` (it rewrites `data/cards.json` and
  `data/decks.json`).
- Tests go in `rust/crates/sim-core/tests/cards_<your module>.rs`, built with `sim_core::testkit` (`table`, `hand`,
  `lands`, `perm`, `token`). One test per card at least, checking the behaviour that matters.

## Checks before you finish
1. `cargo test --release -j 4` passes (all of it, not only your tests).
2. `cargo build --release -j 4 --all-targets` has no warnings.
3. `./target/release/examples/all_decks adaptive 3` and `./target/release/examples/all_decks lookahead 1` finish
   with no panic (under the memory cap).
4. `cardgen audit --deck <your deck key>`: every card your brief lists is Full.
5. A short smoke run with the deck in the field finishes: `./target/release/commander-sim --deck elrond --pool t0 --profile
   loose --games 48 --ai lookahead --jobs 4 --brief`.

## Keeping context small
- **Commit often** (`git add -A rust/ data/ && git commit -m "..." -m "Co-Authored-By: Claude Sonnet 5.5
  <noreply@anthropic.com>"`), at least after each finished card group. Never push.
- Keep `rust/agents/handoff-<task>.md` up to date (done / next in order / gotchas) and commit it with your work.
- Grep, then read the parts you need; keep command output short (`| tail`, `| grep`).
- **When your context gets long** (about 60 or more tool calls, or you notice you're re-reading things), stop at a
  clean point: everything committed, handoff file updated, then end with a report whose first line is `HANDOFF`.
  A fresh agent continues from your handoff file.
- **Your final report is at most 15 lines**: line 1 `DONE` or `HANDOFF`, line 2 your model ID, then the commit hash,
  done / not done, test results, the shared-file changes, anything the lead must act on. Put the full report (cards
  done, how each was modeled, engine changes, open questions) in `rust/agents/report-<task>.md` and commit it. Your
  work will be checked: the diff is read, the tests rerun and the claims spot-checked against Oracle text before
  anything is merged.
