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

## Task `rthras2` — Rograkh/Thrasios: its other lines and its tutors (2026-10-18 night)

`rust/agents/report-t6-cal-rthras.md` (branch t6-cal-rthras) found no modeling error in line A, and that readiness is
draw-bound: Gaea's Cradle was missing in ~45% of checks (in the library ~90% of those), Talon Gates in ~35%. Two
tuning changes (earlier go_now, Crop Rotation/Gamble for Gates) did nothing measurable; the deck stays ~13%.

The spec (audit/t6/rograkh-thrasios-creature-combo/SPEC.md, "Win lines" and the tutor table) lists lines the sim
doesn't model and tutors it may not use:

1. Model the missing lines as combos (impls/t6_combos_t6.rs style, QUIET, per-step answer windows), Oracle-checked:
   C. Badgermole Cub + Mockingbird (or Clever Impersonator / Phantasmal Image / Flesh Duplicate) with Oboro + Cradle;
   D. Badgermole Cub x2 (Cub + a clone) + two earthbent lands + Eternal Witness + Snap (line B without Cradle);
   G. Springheart Nantuko bestowed on Eternal Witness + Vibrance damage loop with the Oboro/Gates loop.
   Count the mana exactly from the cards' Oracle text; if a line isn't unbounded with our list, say so and skip it.
2. Check every tutor in the spec's table actually fetches the piece the line is missing: Expedition Map, Sowing
   Mycospawn, Vibrance's land mode, Crop Rotation, Gamble for Cradle/Gates; Spellseeker for Snap or Finale (MV 2 in
   the library); Nature's Rhythm, Chord, Finale (X=2/3), Invasion of Ikoria, Formidable Speaker for Oboro/Witness;
   Green Sun's Zenith for Witness. Fix the ones that don't.
3. Measure (600 then 2000 games, seed 500000): win %, wins by line, combo tries per game.

Start from branch t6-cal-rthras (merge t6-cedh first). Machine: CARGO_BUILD_JOBS=8, sims --jobs 8, all in the
background. Timebox ~2 hours.
