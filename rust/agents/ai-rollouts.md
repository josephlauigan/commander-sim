# Brief: extra rollouts for the close calls the margin hands back

Work in the worktree `.claude/worktrees/ai-foundations` (branch `ai-foundations`, at a08af77a or later). It is your
whole world. Read `CLAUDE.md` first. Don't use `rust/agents/RULES.md`.

## Why

The eval fix (`audit/ai-foundations/eval-fix.md`) put a margin into the look-ahead: `search::pick_of` and
`margin_holds` depart from the heuristic's top play only when the look-ahead's best beats it by more than z = 2 standard
errors (`Search::margin`). With 6 rollouts the margin hands about half of all main-phase decisions back to the
heuristic (Tier 6 50.6%, Tier 3 42.1%, from `raw/eval/part4-noise/`). The attribution (`raw/eval/attrib/`) shows that
these hand-backs cause nearly all of the Tier 0-5 moves. The remaining noise is hidden information: each playout
re-deals the opponents' hands.

JD (2026-10-29): this is the last AI change before the single recalibration of every tier, which comes at the very
end. **Don't recalibrate or retune anything.**

## What to build

Adaptive rollouts, for every deck. When the margin would hand a main-phase decision back (the look-ahead's best is
not the heuristic's top and `margin_holds` fails), run more rollouts on the same options, in batches, and test again:
- new seeds for each batch, the same seeds across options (common random numbers, as now);
- stop when the margin holds, or when it clearly can't (for example the mean difference is below 0 or under a small
  fraction of its standard error), or at a cap on total rollouts;
- the extra rollouts count toward `search_work` like the first six.

Pick the batch size, the cap and the early stop on the evidence. Start from batches of 6 and a cap of 24 or 30. Add
settings knobs (env, like `AI_EVAL_DET`) for the on/off switch and the cap, so A/B runs are easy. Keep the
diagnostics (`AI_EVAL_FALLBACK`, `EVF` lines) working, and add the number of rollouts each decision used.

## Measure (one sim at a time, `--jobs 22` on every run, under the memory cap)

Against the current eval (a08af77a) and for each setting you try:
- the hand-back rate, the bare pick's flip rate on a second set of seeds, and the rollouts used per decision, split by
  kind of play (`evdiag::kind`), on Tier 6 and Tier 3 at 1,000 games;
- the time per 1,000 games, Tier 6 and Tier 3;
- Tier 6 flow and scorecard (`--realism --pool t6 --games 4000`), for the record, not as a target;
- the goldfish (`--goldfish t6,scion --games 2000 --seed 500000`).

Keep a setting if it cuts the hand-back rate clearly and the time per game stays at or under 2 times the current
eval's. If no setting is worth it, say so; the switch then stays off by default. Then, for the setting you keep, run
realism t0-t5 at 4,000 games and all-decks at 1,000 games, and report how far each tier and each of JD's decks moved
from the current eval (`raw/eval/full-eval/`). Report the moves; don't fix them.

## Rules

- Tests for the new rule in `rust/crates/sim-core/tests/`. `cargo test --release` passes, and
  `cargo build --release --all-targets` has no warnings.
- One commit per part, with no Co-Authored-By trailer. Don't push.
- No Python. Sims and tests run in the background under `systemd-run --user --scope -q -p MemoryMax=24G
  -p MemorySwapMax=0`. Check with `pgrep` that nothing of yours is running when you finish, wait loops included.
- Don't add runs outside this brief. If you think one is needed, list it for JD instead.
- Write `audit/ai-foundations/rollouts.md` (what you built, each measurement, decisions you made and their
  alternatives, decisions for JD). Raw output goes to `audit/ai-foundations/raw/rollouts/`. Keep a handoff at
  `rust/agents/handoff-ai-rollouts.md`.
- Report in 15 lines or fewer.
