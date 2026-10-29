# Handoff: ai-rollouts (adaptive rollouts for the margin's hand-backs)

Brief: `rust/agents/ai-rollouts.md`. Report: `audit/ai-foundations/rollouts.md`. Raw: `audit/ai-foundations/raw/rollouts/`
(`run.sh` runs one sim, `progress.log` lists every run with its time, `evf.awk` summarises the EVF lines).

## State

- Part 1 (code + tests): `search::adapt_rollouts`, `pick_with`, `hopeless`, `rollout_vals`; knobs `AI_EVAL_ADAPT=1`,
  `AI_EVAL_ADAPT_CAP` (24), `AI_EVAL_ADAPT_BATCH` (6), `AI_EVAL_ADAPT_STOP` (0.5). EVF lines gained columns 16-18
  (rollouts used, handed back at six, rollouts the second set used). Tests: `tests/ai_rollouts.rs`.
- Part 2 (measurements): in progress, see `progress.log`.
