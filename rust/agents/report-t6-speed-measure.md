# Report t6-speed measure (2026-10-19)

Run: `--calibrate within --pool t6 --games 1000 --seed 500000` (both profiles; win rates identical to the run before the
stats were added, so the measurement changes no game). Raw output: `audit/t6/speed-2026-10-19/cal1000.txt`.
Code: `t6stats.rs` (`sp|ready`, `sp|reach`, `sp|go`, `spx|<option>` per game), `impls/combos.rs` (`t6_measure`,
`one_tutor_away`, called from `combo_options` for Tier 6 players at each main-phase decision), `ai/mod.rs` (`explain`
passes the chosen option to `speed_decided`), `t6report.rs` (`render_speed`, one test). Stats only: no decision, no RNG.

## What the numbers mean
- **ready**: first round (g.round) some modeled line (`impls/combos.rs` COMBOS: the deck's lines plus lines a held piece
  gates) had every piece in hand/play and the mana (`ready` + `affordable`) at a main-phase decision. Instant-window
  readiness is not measured (a line only ready at instant speed counts when the deck reaches a main phase with it).
  A `go` also marks ready (some stepped lines are only seen ready inside the go).
- **reachable**: exactly one piece group missing, a piece of it in the library, a tutor-tagged card (Tag Tut/Xtutor) in hand,
  and mana for tutor + that piece + the line. Any tutor counts as able to find any piece (the spec tutor tables are not
  machine-readable), so this is an upper bound.
- Medians are over games that reached the event (share of games in brackets). Win = median round of real wins.

## Table (conservative profile; loose in brackets), medians
| Deck | ready | reachable | go | win | ready before go | Verdict |
| --- | --- | --- | --- | --- | --- | --- |
| Dack | 7 (20% of games) [7, 24%] | 7 [7] | 7 [7] | 7.5 [8] | goes the same decision it is ready (0 of 136 waited) | draw-limited: a line is ready in only ~20% of games |
| Kinnan | 7 (25%) [7, 28%] | 6 (52%) [6] | 8 (22%) [8] | 9 [8] | 20 of 164 ready games went later; 20 never | mostly draw-limited, a little AI-limited |
| Kraum/Tymna | 7 (51%) [7] | 7 (60%) [7] | 9 (30%) [9] | 10 [9] | went later in 120 of 331 ready games (median wait 1, mean 1.7); never in 141 | AI-limited (and ready itself is late) |
| Rograkh/Silas | 6 (40%) [7] | 6 (48%) [6] | 7 (36%) [8] | 7 [8] | later in 84 of 264 (mean 0.7); never in 27 | mostly draw-limited, small AI delay |
| Rograkh/Thrasios | 10 (16%) [10] | 10 [10] | 10 (15%) [10] | 12 [11] | later in 26 of 114 (mean 0.35) | draw-limited: ready in 13-16% of games, round 10 |
| Sisay | 8 (14%) [8] | 8 [8] | 8 (14%) [8] | 12 [12] | later in 4 of 94 | draw-limited for lines; she wins by damage at round 12 in the other games |

What each deck did instead while a line was ready and it had not gone (most common): Kraum/Tymna "stop (hold mana)" 656
and "stop" 235 decisions, casting Tymna 139 and Kraum 77 (conservative; so it develops or passes with mana up instead of
Oracle: the table gate `go_ok`/hold-up logic); Rograkh/Silas passes (150 hold, 98 stop) and casts Silas Renn 61 or
Rograkh 24 before going; Kinnan passes (56 stop, 38 hold) or picks another of its lines (Thrasios line 77, Hullbreaker
37, Mirror 30); Dack and Sisay and Rograkh/Thrasios: almost nothing (the go comes at once).

## Findings
1. **Most of the delay is draw/assembly, not hesitation.** Even the fast decks get a line ready at round 6-7 (turn 6-7
   of their own), in only 14-50% of games, and "one tutor away" is no earlier (6-7). Tutors in hand don't speed it:
   reachable median equals ready median for Dack, Kraum, Silas. Real cEDH has the line ready on turn 3-4.
2. **Only Kraum/Tymna (and a little Silas, Kinnan) is AI-limited**: ready in half of Kraum/Tymna games, gone for in 30%;
   in 141 of 331 ready games it never went (it passes with mana up, "stop (hold mana)", or develops). Waiting adds a
   median 1 round (mean 1.7) when it does go. Fixing hold-up/go_ok for Kraum would give about 1-2 rounds on its
   wins and could turn some of the never-went games into wins.
3. **Dack, Sisay, Rograkh/Thrasios**: the modeled lines are ready in 14-24% of games and the deck goes at once. Slow
   wins (Sisay 12, Rograkh/Thrasios 11-12) come from the other 80%+ of games, where no modeled line ever becomes
   ready and they win by damage. The lever is getting lines ready (mulligans/keeps, fast mana, Sisay's fetching), or modeling
   cheaper lines, not the go decision.
4. Median ready round 6-7 with 4 players means the lines are not assembled before the table has had 6-7 turns;
   faster assembly needs earlier mana (T4 mana, cheaper pieces) and keeps that favor fast mana: look at mana at
   turn 3-4 and mulligan rates next (seat block of the same report).

## Checks
- t3 lock: identical after `grep -vEi "progress|elapsed|minutes|seconds|games/h|worker threads|s total"` (see end of file).
- Tests: `cargo test --release -p sim-cli t6report` (5 pass, 1 new), `-p sim-core --test t6_stats`.
- t3 lock: LOCK IDENTICAL (checked after the last code change). t6_stats 10 pass, sim-cli t6report 5 pass.
