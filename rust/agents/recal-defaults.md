# Brief: set the Phase 4 switch defaults from the stage 1 paired tests

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/recal-defaults` on
branch `recal-defaults` (origin/main + #185 rule checks + #186 hand AI switch). Run every command from there. Read
`CLAUDE.md` and `rust/agents/RULES.md` first: memory cap, no Python. The evidence is
`audit/recal/pairs-2026-10-31.md` (sections 2-5).

**Another agent may run sims at the same time:** use **`-j 6`** for cargo, in the background. **Run no
`commander-sim` and no game-playing example yourself**: cargo tests only. Wait on background work by its completion
notice or an output file, never `until ! pgrep -f ...`.

## Work

Change only what an unset variable means; every explicit value keeps working as now (the paired tests and later
stages pass explicit values).

1. `AI_SEARCH_B` (`ai/ismcts.rs` `parse_mode_b`): unset or empty = `All` (was Off). `0` stays Off, `1` All, a key
   list those keys.
2. `AI_SEARCH_OPP` (`ai/ismcts.rs`): unset = on (opponent nodes chosen against the decider). `0` = off, `1` = on.
3. `AI_RULES` (`ai/rules.rs` `parse_mode`): unset or empty = `ALL` (was 0). `0` stays none.
4. `AI_HAND` (`ai/mod.rs` `hand_mode`): unset or empty = the hand AI on for exactly these four keys: `yshtola`,
   `sheoldred`, `cloud`, `dack-fayden-white-stax` (a const list, e.g. `HAND_KEEP`, with a comment pointing at the
   results file; Dack is held pending a check of its -13.9). `0` = off for every deck, `1` = all, a key list = those.
   Check `values::generic_deck` and anything else keyed on the hand AI still follows `hand_ai_key`.
5. Doc comments, `--help` text, `audit/phase4/*-design.md` default lines and any README or doc listing these
   variables: say the new defaults and where the evidence is.
6. Tests: tests that relied on the old defaults must pin what they test (`force`, `force_b`, `force_hand`, or the
   test's own setup), not be deleted or loosened. Add tests that the unset value parses to the new default for each
   of the four, and that `0` turns each off.

## Checks

- `cargo test --release --no-fail-fast -j 6` passes; `cargo build --release --all-targets -j 6` 0 warnings.
- `cardgen export` leaves no diff.
- Don't seat `omnath` or `godo` in new tests.

## Rules

- Stay out of card code, `data/plans/`, `data/eval/`, deck lists. No sims; no recalibration.
- Format only new files (`rustfmt --edition 2024 <file>`), lines up to 120. Commit often, no Co-Authored-By line,
  never push. Scratch in `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/recal-defaults/`.
- Report in 15 lines or fewer, then the full report as text (the harness refuses report files from subagents): each
  default changed, each test changed and why, the check results.
