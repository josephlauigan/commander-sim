# Brief: Tier 6 Stage 6 — calibration (one deck or one rule per agent)

Read `rust/agents/RULES.md`, `rust/agents/t6-ai.md` (the Stage 5 brief: plan slots, table rules, report tool) and the
targets `decklists/pool/TARGETS-t6.md`. Stage 6 tunes **plans and thresholds, never decklists**, until Tier 6 matches
the targets on two seeds: seat shares of decisive games 30-38 / 24-31 / 19-26 / 14-21; each deck except Kinnan 18-32%
of its games; best-worst gap among those five 8 points or less; no deck winning more than half its games through a
line its spec calls a backup. Kinnan may stay the strongest (JD).

Starting point (2026-10-18, 2,000 games per profile, seed 500000; `/tmp/claude-1000/-home-joseph-projects-mtg-sims/ca30944f-e06c-4aa9-9c35-c9fae50d3a2b/scratchpad/s6/base-s500000.txt`):
Dack 33-34%, Kinnan 32-34, Kraum/Tymna 28-29, Rograkh/Silas 21-23, Sisay 19-21, Rograkh/Thrasios 13-14 (LOW); seat 1
26-29% and seat 4 22-23% (both out); average game 9.5 rounds; mulligans 41-50% of first sevens.

## Rules

- Fix modeling errors first (a combo that needs more than its real pieces, a step that fails for no game reason, a
  card that doesn't do what its Oracle text says). Then tune the deck's plan slots. Say in the report which each
  change was.
- Tiers 0-5 stay identical: the t3 lock `rust/agents/perf-lock-t3.txt` (diff after
  `grep -vEi "progress|elapsed|minutes|seconds|games/h|worker threads|s total"`), unless you fix a real rules error
  (say so).
- Measure with `--calibrate within --pool t6 --games 600 --jobs N` while working (the deck's report block); confirm the
  final change with `--games 2000` once. Traces: `--calibrate within --pool t6 --trace N`.
- Shared machine: CARGO_BUILD_JOBS and --jobs as your prompt says, every cargo/sim under `nice -n 19 systemd-run --user
  --scope -q -p MemoryMax=14G -p MemorySwapMax=0`. No Python. Temporary files under /tmp/claude-1000/t6cal-<task>/.
- Create `rust/agents/handoff-t6-cal-<task>.md` FIRST, commit it, update it after each step and before any long run.
  Commits with `git add -A rust data audit`, no Co-Authored-By. Timebox ~90 minutes.
- Report in `rust/agents/report-t6-cal-<task>.md`: what was wrong (modeling vs tuning), each change, before/after
  numbers (600 and the final 2000 run), what still differs. Final message 12 lines or fewer.
