# Report: t6-cal-kraum (Kraum/Tymna too strong), branch t6-cal-kraum

## Modeling check (nothing found)
- Oracle line (combos.rs `oracle`, t6_combos_extra.rs ORACLE_STEPS): three answer windows (Oracle spell, exile spell,
  Oracle trigger); Tainted Pact stops at the first duplicate name (no basics in the list, so it exiles everything);
  Consultation / Pact cost and colours paid by the normal cast path. Correct to the Oracle text.
- Table counters: Force of Will needs another blue card + 1 life, Force of Negation only on others' turns, Pact of
  Negation needs 5 lands per owed pact and two blue lands, Mental Misstep 2 life, Fierce Guardianship needs the commander
  out (engine/stack.rs `pick_counter`/`pay_counter`). No free counter without its cost.
- Ad Nauseam: stops once a line is ready, hand cap, life floor; its line needs the mana left. Fine.
- Not modeled (left as is): an opponent's response forcing a Kraum / Esper Sentinel empty-library draw during the
  Oracle trigger. The plan already refuses to go in that case, so it has no effect on the numbers.
So the high rate is tuning, not a modeling error: ~80% of wins are the Oracle line at median round 9-10, taken when the
table has no cards or mana for it.

## Changes (impls/t6_plan_kraum.rs, tuning only)
1. go_now (Oracle): while any opponent holds cards and has mana up (or a >=5% chance of a free counter), go only with
   creature cover (Force of Will with 2 blue cards, Pact of Negation, Silence, Orim's Chant), per the spec's "conditions
   to go"; the "table is about to win" override is now archenemy >= 8 and round >= 7 (was 6 / round 5).
2. keep (3-4 lands): a lone engine, or lands plus one free counter, is no keep (spec: "a lone engine is not a keep").
Tried and dropped (no effect): stop_chance threshold 0.3 -> 0.12 (35.9 / 34.6).

## Numbers (Kraum win %, conservative / loose, seed 500000)
| run | conservative | loose |
| baseline 600 | 34.8 | 35.3 |
| A threshold 0.12 (600) | 35.9 | 34.6 |
| B free counter needed (600) | 33.0 | 35.1 |
| C+D (600) | 35.9 | 34.8 |
| E = final code (600) | 35.6 | 33.2 |
| before this task, 2000 (brief) | 31.8 / 34.3 after Dack calibration |
| final 2000 | 32.9 | 29.5 (average 31.2) |
Final 2000 others (cons / loose): Dack 25.1 / 29.9, Kinnan 27.2 / 28.1, Silas 22.5 / 24.3, Thrasios 17.1 / 16.4
(LOW, other agent), Sisay 25.5 / 21.9. Seats unchanged (seat 1 26.8 / 26.2, seat 4 22.0 / 22.7: OUT, not this task).
The 600-game runs are within noise of each other (+-4.5); the plan knobs move Kraum only about 1-2 points, the 2000
run lands at ~31 (33 / 29.5, the two profiles differ by 3.4 points).

## Checks
- t3 lock (300 games, seed 500000, filtered diff vs perf-lock-t3.txt): IDENTICAL.
- tests: `cargo test --release -p sim-core --test ai_t6_keeps --test engine_t6_combat` pass (7 + 4).

## Still differs
Kraum conservative profile stays at the top of the band (32.9). Further lowering needs table interaction against the
Oracle line (opponents hold few creature counters; Consultation/Pact is hit by Swan Song, Misstep, Fierce Guardianship),
not Kraum's own plan.
