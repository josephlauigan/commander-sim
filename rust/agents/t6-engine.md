# Brief: Tier 6 Stage 3 engine pass — implement ONE engine feature

Read `rust/agents/RULES.md` (memory cap, no Python, commits, short reports; ignore the Python-port parts and the
`git merge rust-port` step) and `rust/agents/t6-engine-pass.md` (the plan; your feature is one row of it). Your
feature, its worktree and branch are in the prompt that started you. Work only in your worktree.

## What you do

1. Read every `rust/agents/engine-needs-*.md` entry for the cards your feature lists (grep the card names). Each entry
   says what the card code already does, which function in `impls/t6_*.rs` is waiting to be called (e.g.
   `t6_counters::pay_alt_cost`), where the engine would hook, and the test to add.
2. Implement the feature in the engine ONCE, as a general mechanism (not card-name special cases where a hook, tag or
   slot can do it). Smallest change that does it right; follow the surrounding code's style and comments.
3. Wire the waiting card code to it. Remove any stand-in the card agents left for your feature (their notes say
   which, e.g. a `can_cast` lock standing in for counter-on-cast, or the clones' 1/1 stand-in stats).
4. For each card that now works completely per its Oracle text, set `status = "Full"` (note updated) in
   `data/card_overrides.toml`; still-missing parts stay Partial with the note updated. Re-export with YOUR worktree's
   `rust/target/release/cardgen export --offline` (never the main checkout's binary; never `audit --md`).
5. Tests: the tests the engine-needs entries ask for, plus one engine-level test of the mechanism itself.

## Rules

- **Other decks must not change behaviour** unless your feature fixes a real rules error that affects them (e.g. a
  state-based action). If it does, say so in the report: which decks and how. Keep every existing test passing; if one
  fails, decide whether the test encoded the old wrong rule (fix the test, explain in the report) or your change is
  wrong (fix the change).
- **Only your feature.** If you find another missing engine feature, add it to `rust/agents/engine-needs-found.md`
  (card, need, where) instead of building it.
- **Never run a sim.** Tests only.
- Shared machine: `CARGO_BUILD_JOBS=3 nice -n 19 systemd-run --user --scope -q -p MemoryMax=14G -p MemorySwapMax=0
  cargo ...` from `rust/`. While working, run only the test files you touch
  (`cargo test --release -p sim-core --test <file>`); the full `cargo test --release` once at the end, and
  `cargo build --release --all-targets` with no warnings.

## Progress (a lead watches it)

- Create `rust/agents/handoff-t6-e-<feature>.md` FIRST; update it after each step (done / next / gotchas), and BEFORE
  any long build. A lead checks on an agent silent for 12 minutes and restarts one silent for ~20.
- Commit after each working piece (engine change + its test; then each card wired), standalone commits, no
  Co-Authored-By.
- Timebox: if one sub-part resists for ~15 minutes, commit what works, write the gap in the report, move on.

## Done

Full tests pass, no warnings. Report in `rust/agents/report-t6-e-<feature>.md`: the mechanism (files, functions), the
cards flipped to Full and those still Partial (why), any behaviour change for other decks, and new needs found.
Final message at most 12 lines.
