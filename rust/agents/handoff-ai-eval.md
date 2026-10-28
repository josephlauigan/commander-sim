# Handoff: fix the look-ahead's evaluation at the cause (brief `ai-eval.md`)

Branch `ai-foundations`, worktree `.claude/worktrees/ai-foundations`. Report: `audit/ai-foundations/eval-fix.md`; raw
output: `audit/ai-foundations/raw/eval/`.

## State (2026-10-28)

- **Part 1 (done):** env `AI_FLOW_MUST=0` turns step 3's A and B off (`Settings::flow_must`). Env `AI_EVAL_DIAG=1`
  writes one `EVD` line per option of each main-phase look-ahead decision whose heuristic top is an engine, an
  engine commander or a mana permanent (`ai/evdiag.rs`; stats only, games byte-identical with it on). Raw lines:
  `raw/eval/part1/evd-t6.tsv.gz`, `evd-t3.tsv.gz`; tables: `summary.txt` (awk scripts beside it); traces in
  `traces/` (5 games, the `[eval ...]` lines).
- **Part 2 (code in, measuring):** `search::output_value` (the board valued for what it will produce) and the kept-hold
  rule (`search::hold_credit`, `kept_up`). Knobs: env `AI_EVAL_OUTPUT`, `AI_EVAL_HOLD`.
- **Part 3:** not started.

## Gotchas

- Perl with `|` delimiters mangles patterns containing `|` (closures): use `#` delimiters or the Edit tool.
- `until ! pgrep -f '<cmd>'` matches its own command line and never ends: wait on the job's output file instead.
- `--trace N` for a pool game needs `--calibrate within --pool tN --trace N` (no --deck).
- Step 3's binaries (base, A, B, final) are in the scratchpad `bin/`; the scratchpad is lost on a session restart.
