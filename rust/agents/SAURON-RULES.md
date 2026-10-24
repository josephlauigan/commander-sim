# Rules for the Sauron weakness agents (2026-10-12)

You are one of four agents working on the Sauron deck (`sauron`, Grixis Orc Army) and its Breach package. The
2026-10-12 audit found: no answer to fast combo at Tiers 4-5, one engine (the Army) that starts late, symmetric
wipes that kill its own Army, and an Underworld Breach line that wins when cast but is rarely cast. JD wants tests of
(1) a one-sided board-control package, (2) other packages in the Breach slots and the Breach package moved to Kefka,
(3) a faster deck. You model the cards and AI for those tests; the lead runs the tests. Read this file, then your
brief (`rust/agents/<task>.md`), then your handoff file `rust/agents/handoff-<task>.md` if it exists.

## Hard rules
- **Never run Python.** No `python3`, no old audit scripts, not even a one-liner (a hook refuses it). Use shell
  tools (`grep`, `sed`, `awk`, `jq`), `cardgen` and `commander-sim`, or a Rust test or example.
- **Your worktree is your whole world.** `cd` into it for every command. Its branch was made from `sauron-weakness`;
  don't merge anything else in, and don't push.
- **Memory cap on every build, test and binary:** `systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0
  <command>`. Exit 137 means the cap was hit: that's a bug (something growing without bound), never a flake.
- **Four agents share the machine:** build and test with `-j 6` (`cargo test --release -j 6`), sim runs with
  `--jobs 6`.
- Toolchain: `export PATH=$HOME/.cargo/bin:$PATH`; build from `rust/`. In zsh, write commands out (a command stored in
  a variable isn't word-split).
- Never let an unsigned counter go below zero (release builds have overflow checks).
- **No wait loops** (`until ! pgrep ...; do sleep ...`). Run builds and tests in the foreground with a long timeout.
  Leave nothing running when you finish.
- **Don't change any decklist** (`decklists/`, `data/my_decks.toml`, `cardgen deck`). The lead tests packages with
  `--swap`, which works on cards that are modeled but not in the list.
- **Edit only the files your brief gives you.** Shared files (`cardcode.rs`, `state.rs`, `hooks.rs`, `engine/*`,
  `ai/*`, `dsl/*`, `impls/common.rs`, `impls/mine.rs`): only the smallest additive change, and list each one in your
  report. The other three agents edit the same shared files in parallel. Format only your own files:
  `rustfmt --edition 2024 <files>`, never `cargo fmt` on the workspace.
- **Stay in your lane.** If you need a mechanic another agent owns, model what you can, leave the card Partial, and
  say so in your report.

| Area | Owner |
|---|---|
| Speed / plan metrics in the analysis report, Sauron's plan milestone | sauron-speed |
| One-sided sweepers, "creature deals damage equal to its power to each ..." effects, the AI choosing between wipes for Sauron | sauron-sweepers |
| Amass creatures, Dualcaster Mage + Twinflame, Torment of Hailfire, mana rocks, Sauron's non-wipe cast priorities | sauron-threats |
| The Underworld Breach line (`impls/mine.rs` `breach_line` and friends) for decks other than Sauron, Kefka's AI | kefka-breach |

## How cards are modeled
- Read `documents/archive/PORTING_CARDS.md` (the hook-slot pattern) and follow `rust/crates/sim-core/src/impls/t1.rs` as the
  pattern. Card data: `data/carddb.txt` (hand tags), `data/card_overrides.toml` (overrides and the audit verdict),
  Scryfall text compiled by `cardgen`. Engine entry points: `hooks.rs`, `engine/hooks.rs`, `cardcode.rs`. Deck AI:
  `ai/decks.rs` (`deck_prio`, tutor picks) and `ai/brain.rs`. Sauron's own code is in `impls/mine.rs`.
- **Model each card from its Oracle text.** `jq '.["card name lowercased"]' data/scryfall_cache.json`. If the card
  is missing there, add its `card_overrides.toml` entry and run `rust/target/release/cardgen export` (online) to
  fetch it; commit bc75607 (Latulla's Orders) shows the whole path for one card. Reuse machinery in
  `impls/common.rs` and the engine before writing new machinery: grep first (amass, proliferate, wipes, copy spell,
  token copies, damage to each creature all exist in some form).
- "Fully modeled" means `rust/target/release/cardgen audit --mine` lists the card as **Full** (see how other cards
  are marked in `data/card_overrides.toml`). Mark a card Full only when its whole Oracle text is implemented and
  covered by a test; if a clause truly can't matter, say so in the card's note and in your report.
- After changing card data, run `rust/target/release/cardgen export`.
- Tests go in `rust/crates/sim-core/tests/cards_<your task with _>.rs`, built with `sim_core::testkit` (`table`,
  `hand`, `lands`, `perm`, `token`). One test per card at least, checking the behaviour that matters, plus a test for
  each AI decision you add.

## Checks before you finish
1. `cargo test --release -j 6` passes (all of it, not only your tests).
2. `cargo build --release -j 6 --all-targets` has no warnings.
3. `./target/release/examples/all_decks adaptive 3` and `./target/release/examples/all_decks lookahead 1` finish
   with no panic (under the memory cap).
4. `cardgen audit --mine`: every card your brief lists is Full.
5. The smoke run your brief names finishes.

## Keeping context small
- **Commit often** (`git add -A rust/ data/ && git commit -m "..."`, no Co-Authored-By line), at least after each finished card group. Never push.
- Keep `rust/agents/handoff-<task>.md` up to date (done / next in order / gotchas) and commit it with your work.
- Grep, then read the parts you need; keep command output short (`| tail`, `| grep`).
- **When your context gets long** (about 60 or more tool calls, or you notice you're re-reading things), stop at a
  clean point: everything committed, handoff file updated, then end with a report whose first line is `HANDOFF`.
- **Your final report is at most 15 lines**: line 1 `DONE` or `HANDOFF`, line 2 your model ID, then the commit hash,
  done / not done, test results, the shared-file changes, anything the lead must act on. Put the full report in
  `rust/agents/report-<task>.md` and commit it. Your work will be checked: the diff is read, the tests rerun and the
  claims spot-checked against Oracle text before anything is merged.
