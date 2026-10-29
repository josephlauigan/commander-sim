# Handoff: ai-rollouts (adaptive rollouts for the margin's hand-backs)

Brief: `rust/agents/ai-rollouts.md`. Report: `audit/ai-foundations/rollouts.md`. Raw: `audit/ai-foundations/raw/rollouts/`
(`run.sh` runs one sim, `phase1.sh` / `phase2.sh` the sets, `progress.log` lists every run with its time; `evf.awk`,
`gf.awk`, `flow.awk`, `cmp.sh` summarise).

## State: done

- Part 1 (bedc035d): `search::adapt_rollouts`, `pick_with`, `hopeless`, `rollout_vals`; knobs `AI_EVAL_ADAPT`,
  `AI_EVAL_ADAPT_CAP` (24), `AI_EVAL_ADAPT_BATCH` (6), `AI_EVAL_ADAPT_STOP` (0.5). EVF columns 16-18. Tests:
  `tests/ai_rollouts.rs`.
- Part 2 (df1bc649): on by default (cap 24, batch 6, stop 0.5), chosen over cap 30 and stop 1.0 at Tier 6 / Tier 3.
  `search_hold.rs`'s Ad Nauseam test pinned to six rollouts (decision for JD).
- Part 3: the full set (goldfish, t0-t6 at 4,000, all-decks at 1,000) and the report.
- `AI_EVAL_ADAPT=0` reproduces a08af77a's games byte for byte (checked at Tier 6, 1,000 games).

## Open (for JD, see the report's last section)

Keep it on; a sequential boundary for the repeated margin tests; the Ad Nauseam hold; runs not made (Tier 5 timing,
thrasios t5 and alela t1 at 4,000 games, Tymna traces). Tiers are not recalibrated (that comes at the very end).
