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
  - `impls/combos.rs`: read-only accessors (`plan_lines`, `group_held`, `group_out`, `plan_mana_targets`);
  - `ai/brain.rs`: `main` (before `must_option`) and `end_of_turn_window`.
- Tests: `rust/crates/sim-core/tests/ai_planner.rs` (8), `sim-cli` knobs test `the_planner_knob_is_read_applied_and_written`.

## Next

- (filled in as the work goes)

## Gotchas

- Most shared files are not rustfmt-clean at HEAD: format only `planner.rs` and `tests/ai_planner.rs`.
- Perl one-liners with `|` delimiters and `\|` in the pattern silently match the empty string: use the Edit tool or
  a perl script with `\Q...\E`.
- One sim or cargo command at a time, `--jobs 16` / `-j 16`.
