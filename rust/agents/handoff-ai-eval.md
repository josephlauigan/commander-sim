# Handoff: fix the look-ahead's evaluation at the cause (brief `ai-eval.md`)

Branch `ai-foundations`, worktree `.claude/worktrees/ai-foundations`. Report: `audit/ai-foundations/eval-fix.md`; raw
output: `audit/ai-foundations/raw/eval/`.

## State (2026-10-28)

- **Part 1 (874148a3):** env `AI_EVAL_DIAG=1` writes one `EVD` line per option of each main-phase look-ahead
  decision whose heuristic top is an engine, an engine commander or a mana permanent (`ai/evdiag.rs`; stats only,
  games byte-identical with it on), and one `EVN` line (the paired top-minus-stop noise and the pick on a second set
  of seeds; its playouts spend no game budget). Raw lines in `raw/eval/part1/`, `part2/` (`*.tsv.gz`), tables in each
  `summary.txt` (the awk scripts beside them).
- **Part 2 (f46e6595):** in `ai/search.rs`, on by default:
  - `output_value`: the board valued for what it will produce (env `AI_EVAL_OUTPUT=0` turns it off);
  - `hold_credit` / `kept_up`: the hold bonus to the playouts that kept the mana up (`AI_EVAL_HOLD=0`);
  - `margin_holds`: depart from the heuristic's top play only beyond two standard errors (`AI_EVAL_MARGIN=0`).
- **Part 3 (e408b31b):** A and B removed (and `AI_FLOW_MUST`), and the cheap-rock, Chrome Mox and mana-first must
  rules (measured off: `raw/eval/part3-must/`).
- **Margin z = 2 (cba5580b)** and **deterministic playouts (51afa9f2)**, on by default; the short horizon
  (`AI_EVAL_SHORT=1`) stays an off switch (it moved Tier 3 twice as far). Evidence: `raw/eval/part3-z/`,
  `part4-noise/` (`EVF` lines: env `AI_EVAL_FALLBACK=1`).
- **Full set:** detached unit `eval-full3` (scratchpad `full.sh`) into `raw/eval/full-eval/`, `full-AB/`, `full-base/`.
  "base" and "A+B" reuse step 3's t0-t6 realism and goldfish (`raw/step3/full-base/`, `base/`, `B/`); A+B all-decks
  is run fresh, and base differs from A+B only on Scion's row (`--deck scion --pool t6` on both binaries).

## Next

- Done: the full set and the attribution (`raw/eval/attrib/`) are in the report. Open points are under "Decisions for
  JD" in `eval-fix.md` (keep the margin for every tier or Tier 6 only; rollouts for handed-back decisions; traces of
  the moved decks; the pre-existing all-decks overflow panics).

## Gotchas

- Perl with `|` delimiters mangles patterns containing `|` (closures): use `#` delimiters or the Edit tool.
- `until ! pgrep -f '<cmd>'` matches its own command line and never ends: wait on the job's output file instead.
- `--trace N` for a pool game needs `--calibrate within --pool tN --trace N` (no --deck).
- Step 3's binaries (base, A, B, final) and this task's (`cs-eval`) are in the scratchpad `bin/`; the scratchpad is
  lost on a session restart: rebuild from the commits.
