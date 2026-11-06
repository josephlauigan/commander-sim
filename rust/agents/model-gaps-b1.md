# Task: practice mode: probe the unprobed cards and fix what they find

Worktree: `/home/joseph/projects/commander-sim/.claude/worktrees/model-gaps-b1` (branch model-gaps-b1, from model-gaps).
Task name: `model-gaps-b1`. Practice mode only: **AI paths must not change** (no sims follow this task).

## Background
`rust/crates/sim-core/tests/practice_trigger_probe.rs` drives each owned card's triggers and activated abilities with the
person at P0 and flags any choice the engine makes for the person. `UNPROBED` (line ~782) lists 23 entries the probe's
table can't reach, each with why. `rust/crates/sim-core/tests/practice_choice_probe.rs` lists Urza's Cave (karn, yshtola,
sauron) as a gap left to the ability probe (`practice_ability_probe.rs`). Read `rust/agents/report-trigger-fixes.md`,
`rust/agents/report-walker-fixes.md` and `rust/agents/report-opponent-fixes.md` first (the fix patterns).

Note: Mishra's Factory / Foundry, Steel Hellkite, Tezzeret's emblem and Liquimetal Torque got new abilities in the model-gaps
A1 work (`rust/agents/report-model-gaps-a1.md`); check the person can use each new ability (with their own choices) too.

## Goal
1. Give the probe the preconditions it lacks so each `UNPROBED` card is actually exercised: an attacking/blocking creature
   (Angelic Page, Ballista Squad), an attacking Assembly-Worker / a second Worker (Mishra's lands), an Equipment or Vehicle
   card in the graveyard (Cid), a legendary creature (Plaza of Heroes), a red / black creature (Lawbringer, Lightbringer), a
   Rebel in the graveyard (Ramosian Revivalist), Walking Ballista entering with counters (X > 0), an ability to copy (Peter
   Parker's Camera), a void-counter exiled card (Dauthi Voidwalker), Maelstrom of the Spirit Dragon and Loran of the Third
   Path (find out why they never reach the stack), Urza's Cave's activation, and as many of the trigger entries (Karmic
   Justice, Jocasta, Rings of Brighthearth, Urza's Saga's chapters, The Ozolith, Vision, Tymna) as a bounded setup allows.
   Prefer a small per-card setup hook in the probe over rewriting the probe.
2. For each card now probed: fix every real gap (the person is asked what the Oracle text lets them choose), following the
   existing patterns (`human/gaps_a.rs`, `human/gaps_c.rs`, `human/walkers.rs`, `human/cards.rs`). Take the card off
   `UNPROBED` (the test fails on stale entries). Genuine false alarms go to `FALSE_ALARMS` with the reason.
3. A card you can't reasonably probe stays in `UNPROBED` with an updated, exact reason.

Out of scope: the AI-side rules mismatches in `report-opponent-fixes.md` (Council's Judgment's AI votes, Liliana of the Veil's
piles, Chain of Vapor's copy chain): they change the AI and need sims; the coordinator raises them separately. Don't touch
them, but if you find more AI-side mismatches, list them in your report.

## Tests
`practice_trigger_probe`, `practice_ability_probe`, `practice_choice_probe` pass; a new `tests/practice_model_gaps.rs` with a
test per fixed card (the person is asked, and their answer is what happens). Also run the existing test files for code you
touched (`practice_*`, `play_*` covering those cards, `model_gaps_karn`, `model_gaps_others`).

## Done when
- Every `UNPROBED` entry is probed and fixed, a false alarm, or still unprobed with an exact reason; the tests above pass;
  `cargo build --release --all-targets` has no warnings in files you touched.
- `rust/agents/report-model-gaps-b1.md`: per card: probed or not, what the person now chooses, files, tests; any AI-side
  mismatch found.

## Ground rules (these override the older rust/agents/RULES.md, which is from the Python-port era)
- Work only in your worktree (path above); `cd` into it for every command. Don't merge other branches. Don't push.
  Toolchain: `export PATH=$HOME/.cargo/bin:$PATH`; build from `rust/`.
- **Never run Python** (a hook refuses it). Use jq, shell tools, or Rust.
- **Memory cap on every cargo command**: `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 cargo ...`
  (run from `rust/`). The shell is zsh: write the prefix out in full, don't store it in a variable. Exit 137 = cap hit = a bug.
- **Don't run the full suite** (`cargo test --release` with no filter) and don't run sims: the coordinator does that. Build and
  run only your own test file and the existing test files you touched or that test the code you changed
  (`cargo test --release -q -p sim-core --test <name>`). Run test commands in the foreground with a long timeout; don't start
  background jobs and don't leave anything running when you finish.
- Practice mode: the person's seat is `crate::human::is_human(g, p)`; questions go through `human::choices::{choose,
  choose_one, pick_cards, pick_any, yes_no, search, ...}`. A new ability a person can use needs a person path too (look at
  how `human/` handles similar abilities); AI and person code stay separate (`if is_human`).
- Never let an unsigned counter go below zero (release builds have overflow checks).
- `rustfmt --edition 2024 <your new files>` only; never `cargo fmt` on the workspace (the repo isn't fmt-clean).
- Commit often (`git add -A rust/ data/ && git commit -q -m "..."`); **no Co-Authored-By line**. Keep
  `rust/agents/handoff-model-gaps-b1.md` current (done, next, gotchas). If your context gets long (~60+ tool calls), stop at a
  clean point with everything committed and end your reply with a first line `HANDOFF`.
- Finish with: everything committed; full details in the report file (committed); a reply of at most 15 lines (commit hash,
  done / not done per card, test results, anything the coordinator must act on). Then stop.
