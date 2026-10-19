# Report t6-cal-seats: why is going first worth so little?

Branch t6-cal-seats (off t6-cedh). Code change: stats only (sim-cli/src/t6report.rs: a per-seat block in the Tier 6 report plus a test).
No decision, RNG or rule changed; sim-core is identical to t6-cedh. Tiers 0-5 untouched.

## Answer
The sim has no seat bug. Seat 1's edge is small because our games are decided late, not by tempo. 1200 games, seed 500000,
conservative profile, `--calibrate within --pool t6` (the 600-game two-profile run agrees: 26.2/24.3/28.1/21.5 and 24.6/26.1/26.1/23.1):

| seat | share | win round avg/median | mull. hands | combo tries/game | stopped | removal+counters taken/game | died | first out | wins by rd 4 / 6 / 8 | T4 mana | commander out |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | 26.4% | 9.9 / 9 | 43.1% | 0.26 | 28% | 1.29 + 0.93 | 73% | 56% | 15 / 63 / 123 of 309 | 4.6 | turn 3.2 |
| 2 | 25.7% | 10.2 / 10 | 42.2% | 0.26 | 36% | 1.17 + 0.95 | 73% | 54% | 11 / 64 / 118 of 300 | 4.5 | 3.4 |
| 3 | 25.7% | 10.1 / 9 | 43.2% | 0.24 | 32% | 1.06 + 0.92 | 74% | 55% | 11 / 65 / 122 of 301 | 4.4 | 3.2 |
| 4 | 22.2% | 9.7 / 9 | 44.2% | 0.23 | 30% | 0.97 + 0.86 | 77% | 57% | 11 / 55 / 107 of 259 | 4.5 | 3.1 |

1. Wins come late in every seat: only 4% of all wins are by round 4, 9% by round 5, 21% by round 6; the median win round is 9-10 and
   a seat's win round does not depend on where it sits (9.7-10.2). Real cEDH seat 1 (34%) lives on the fast wins; here there are
   almost none, so being a quarter-round ahead is washed out over a 10-round game. Even the wins by round 6 are split 25/26/27/22%.
2. Own-turn tempo is identical by construction: mulligans (42-44%), mana on turn 4 (4.4-4.6), hand on turn 4 (5.0-5.2) and the turn the commander
   resolves (3.1-3.4) do not differ by seat, because they are counted per own turn. Seat 1's edge is only that its turn N comes before the
   others' turn N (seat 4 meets about three more opposing turns at the same own turn); that is a quarter-round of a 10-round game.
3. Dogpile on the first player exists but is mild and does not explain it: seat 1 takes 33% more removal than seat 4 (1.29 vs 0.97 per game)
   and 8% more counters, but dies no more often (73% vs 77%) and is first out no more often (56% vs 57%). It takes more because it
   deploys first and is the only target in the early rounds.
4. Rules check (all correct, no change): the starting player draws on turn 1 (multiplayer rule); the first mulligan is free and later
   London mulligans bottom 1 and 2 (engine/turn.rs `mulligan`); seats are shuffled independent of deck (pyrand::draw_seats); nothing in the
   engine special-cases the first seat. The mulligan rate (43%) is high but equal in all seats.

## Experiments (1200 games, seed 500000, conservative; switches were temporary and are reverted)
| change (Tier 6 only) | seat 1 / 2 / 3 / 4 |
|---|---|
| baseline | 26.4 / 25.7 / 25.7 / 22.2 |
| no x1.25 "leader" bonus on removal | 26.1 / 25.8 / 25.5 / 22.6 (seat 1 still takes 1.33 removal) |
| the leader is the archenemy only (board value ignored in the table's threat reading) | 26.5 / 26.0 / 23.9 / 23.6 |
| both of the above | 25.8 / 24.5 / 25.8 / 23.9 |
| go_ok threshold +0.2 (decks go for a combo sooner) | 27.2 / 26.5 / 25.7 / 20.6 (game still 10.2 rounds) |

Standard error per seat is about 1.3 points, so none of these moves seat 1 out of noise; the table rules are not what flattens the edge, and no
shared-table change was kept.

## What would move it
The pod needs more games that end in rounds 3-6 (today 4% by round 4; combo tries 0.25 per game, average game 10.3 rounds). That is per-deck
speed (go_now, fast mana, keeps) and belongs to the deck calibrations, not to the table: once decks win by round 5-6 in 25-30% of games, the
quarter-round that seat 1 gains starts to count. Re-read the new seat block after those: `seat N: ... wins by round <=4/5/6/8`.

## Checks
t3 lock diff: see the end of this file. Tests: `cargo test --release -p sim-cli t6report`.

t3 lock: diff after the brief grep is empty (identical). Tests: sim-cli t6report 4 passed (new: the_seat_block_counts_wins_rounds_and_removal_taken_per_seat); release build clean.
