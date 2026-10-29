# Handoff: AI foundations step 2, the line planner (brief `ai-planner.md`)

Branch `ai-foundations`, worktree `.claude/worktrees/ai-foundations`. Report: `audit/ai-foundations/step2-planner.md`;
raw output: `audit/ai-foundations/raw/step2/`.

## State

- `rust/crates/sim-core/src/ai/planner.rs` (new): the search, the tutor model, the next-turn view, the decision.
- Shared files, each change marked `// --- ai-planner`:
  - `state.rs`: `Player::plan_tutor` (the forced tutor target), `Game::plan_for` (a planner copy: only its player
    takes priority);
  - `engine/stack.rs` (`take_priority`): the `plan_for` gate;
  - `ai/decks.rs` (`tutor_pick`) and `ai/mod.rs` (`dsl_search_pick`): the forced target, when legal;
  - `settings.rs`: `Settings::planner` (env `PLANNER=0` turns it off); `sim-cli/src/knobs.rs`: `[ai] planner`;
  - `impls/combos.rs`: read-only accessors (`plan_lines`, `group_held`, `group_out`, `plan_mana_targets`,
    `audit_loop_pips`);
  - `ai/brain.rs`: `main` (before `must_option`) and `end_of_turn_window`, with the `plan|...` counters;
  - `engine/turn.rs`: `play_land` (the planner's land drop), `crack_fetch` (the forced fetch), `true_fetch` pub;
  - `gf_audit.rs` + `sim-cli/src/gf_audit.rs`: the legal-tutor best case; a tutored top card is not "seen";
  - `sim-cli/src/goldfish.rs`: `plan|...` counters in the goldfish report.
- Tests: `rust/crates/sim-core/tests/ai_planner.rs` (9), and the `sim-cli` knobs test
  `the_planner_knob_is_read_applied_and_written`.

## Done (2026-10-27)

- Commits: 542b0902 (planner + integration), 4dcfcdb5 (legal best case), ddf05cd5 (plan slot obeyed: the Dack
  41% bug), 1d41a79f (counters, colour land drop), ab58377d (best-case double count), then the report.
- Measured (raw/step2/): goldfish off/on, fresh seeds, Tier 6 table off/on, scion vs t6 off/on, the gate check
  (identical), 30 traces, 6 gap traces. The planner gains about 1.5 points of kills by turn 3 for scion; the
  remaining gap to the (now legal) best case is colours and draws.

## Next

- Step 3, the Tier 6 recalibration (the planner barely moved the table).
- Possible follow-ups: a colour-aware best case; the planner reading `hold_up`; X tutors as planner plays.

## Gotchas

- Most shared files are not rustfmt-clean at HEAD: format only `planner.rs` and `tests/ai_planner.rs`.
- Perl one-liners with `|` delimiters and `\|` in the pattern silently match the empty string: use the Edit tool or
  a perl script with `\Q...\E`.
- One sim or cargo command at a time, `--jobs 16` / `-j 16`.
- `progress.log` in raw/step2 has several runs' lines: an `until grep "done X"` loop matches an old line. Count
  occurrences, or grep after the newest marker.
- `PLANNER_PROGRESS=1` turns on the (slower) progress fallback; `PLANNER=0` turns the planner off.
