# Brief: Tier 6 Stage 6 — calibration (one deck or one rule per agent)

Read `rust/agents/RULES.md`, `rust/agents/t6-ai.md` (the Stage 5 brief: plan slots, table rules, report tool) and the
targets `decklists/pool/TARGETS-t6.md`. Stage 6 tunes **plans and thresholds, never decklists**, until Tier 6 matches
the targets on two seeds: seat shares of decisive games 30-38 / 24-31 / 19-26 / 14-21; each deck except Kinnan 18-32%
of its games; best-worst gap among those five 8 points or less; no deck winning more than half its games through a
line its spec calls a backup. Kinnan is held to the same band (JD withdrew the exception after #100).

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

## Task `dack` — Dack Fayden is too strong (2026-10-18 night)

New baseline after main's #100 (2,000 games per profile, seed 500000; scratchpad s6/base2-s500000.txt): Dack 34.8 /
36.3% (target 18-32), 84% of its wins by "Dack Fayden: the first flip wins the turn (35 piles)", median win round 7,
stopped only 9-11% of its tries. The 35-pile shortcut (impls/t6_dack.rs, `record_first_flip`, `pile_wins`) was built
from a hand trace that found ~68% same-turn wins UNOPPOSED (audit/t6/dack-fayden-white-stax/RULES-SHEET.md,
flips-*.md). Check, in this order:

1. Does the shortcut give the table every answer window a real game would (the Dack cast itself, Dack's trigger,
   each creature that flips in, the Preston/Felidar/Solar loop steps), and is the 68% applied only when nobody
   answers? If the shortcut wins in one step, opponents get one window where a real game gives several: fix that.
2. Is the pile table right for our list (pieces that are removed, exiled or stolen before the flip)?
3. Do opponents see Dack as the archenemy (piece_gain/line_progress: Dack's line is Dack alone, castable from the
   command zone) and hold answers for it?
4. Only then tune Dack's plan (impls/t6_plan_dack.rs).

Start from a new branch t6-cal-dack off t6-cedh. Machine: CARGO_BUILD_JOBS=8, sims --jobs 8, all in the background.
Timebox ~2 hours.

## Task `rthras3` — Rograkh/Thrasios: find the pieces sooner (2026-10-18 night)

After rthras and rthras2 (reports `report-t6-cal-rthras.md`, `report-t6-cal-rthras2.md`; lines A-D modeled, tutors
fixed, the plan's on_bf now sees lands) the deck is still ~14% (target 18-32), median win round ~12, 0.13 combo tries
per game: it assembles 3-4 specific cards too late. Try, measuring each at 600 games (seed 500000) and keeping only
what helps:

1. Keep rule: favour opening hands with Gaea's Cradle, Talon Gates or a tutor that finds them (Crop Rotation,
   Expedition Map, Sowing Mycospawn, Gamble, Vibrance), per the spec's mulligan section.
2. Thrasios's activation ({4}: scry 1, reveal the top card, a land onto the battlefield, else draw it): use spare
   mana at the end of each opponent's turn (and with leftover mana on your own turn) to dig, from the first turn
   Thrasios is out.
3. The combo risk check: with 0.13 tries per game, check `go_ok`/`go_now` aren't refusing lines that are ready.
4. Any other draw/tutor the spec names that the AI casts too late (Rhystic Study, Mystic Remora, Sylvan Library).

Branch t6-cal-rthras (merge t6-cedh first). Machine: CARGO_BUILD_JOBS=8, sims --jobs 8, all in the background.
Timebox ~90 minutes; confirm the kept changes once at 2000 games.

## Task `kraum` — Kraum/Tymna is a little too strong (2026-10-18 night)

After the Dack calibration (2,000 games, seed 500000), Kraum/Tymna is 34.3% (target 18-32); before it, 31.8 / 33.2%.
82% of its wins are Thassa's Oracle + Demonic Consultation / Tainted Pact, median round 9. Real data (TARGETS-t6.md)
puts it near the top of the five established decks, so bring it to about 28-31%, not lower. Check first that nothing
is modeled too generously (e.g. Oracle lines going off through counters the table holds, Tainted Pact with the
library, free counters it pitches with no blue card, Ad Nauseam life), then tune its plan
(impls/t6_plan_kraum.rs: go_now, hold_up, keep). Branch t6-cal-kraum off t6-cedh. Machine: CARGO_BUILD_JOBS=8,
sims --jobs 8, all in the background. Timebox ~90 minutes; confirm once at 2000.

## State after dack / rthras3 / kraum (2026-10-19 01:20; 2,000 games, seeds 500000 and 900000, scratchpad s6/cal3-s*.txt)

Kraum/Tymna 29-32, Dack 27-30, Kinnan 27-28, Sisay 22-27, Rograkh/Silas 22-26, Rograkh/Thrasios 15-17 (LOW). Seat 1
26-28% (target 30-38), seat 4 22% (target 14-21). Average game ~10.2 rounds, timeouts 2-3%.

## Task `rthras4` — keep Thrasios alive

rthras3 found the deck's pieces arrive (Mycospawn fetches Cradle by round 4) but its commanders are removed ~240 times
per 600 games and each recast costs {2} more and a turn. 1. Check the removal it eats is chosen fairly (the table's
removal targets: is Rograkh/Thrasios over-targeted vs the archenemy rules? Dack's Fiend Hunter / All-Fates Stalker
loops). 2. Plan: don't recast Thrasios/Rograkh into open removal (opponents' open mana + cards), hold protection
(Veil of Summer, Snap/Chain to save) for the commanders, prefer to rebuild the line when the table is tapped.
Branch t6-cal-rthras (merge t6-cedh first). 8 build jobs / --jobs 8, background. Timebox 90 minutes; confirm at 2000.

## Task `seats` — why is going first worth so little?

Real cEDH: seat 1 wins 34% of decisive games, seat 4 17%. Ours: 26-28 / 22. Find why, with numbers: win round by seat,
mulligan rate by seat, how often seat 1's line is stopped vs others', whether turn 1-3 plays (fast mana, the seat that
acts first) give seat 1 the tempo they should, whether the archenemy/answer rules dogpile the first player too hard
(the leader eats every answer), and whether the opening (draw on turn 1? mulligans? London mulligan free 7 in
multiplayer) follows the rules. Fix modeling errors; tuning of the shared table rules (ai/t6_table.rs) is allowed,
gated to Tier 6. Branch t6-cal-seats off t6-cedh. 8 build jobs / --jobs 8, background. Timebox 90 minutes.
