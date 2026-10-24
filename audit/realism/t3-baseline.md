# Tier 3 realism baseline, 2026-10-24

`commander-sim --realism --pool t3 --ai adaptive --games 8200` (loose profile, adaptive AI, seeds 500000-508199, the
five Tier 3 pool decks, 4 a pod), against `data/realism/t3.toml`, the first casual targets (Playgroup.gg). This is
the nightly job's Tier 3 run. Raw output: `t3-adaptive-8200.txt`. 7,512 of 8,200 games were decisive (8.4% hit the
round cap).

**Result: 1 of 9 targets pass** (poison). The tier is much too slow, wins almost never by combo, and its two
strongest real commanders are its two weakest decks.

| Statistic | Sim (95% CI) | Target | Band | Result |
| --- | --- | --- | --- | --- |
| Seat 1 share of decisive games | 26.5% (25.5-27.5) | 31.3% | 30.3-32.3 | FAIL |
| Seat 2 share | 26.1% (25.1-27.1) | 27.8% | 26.8-28.8 | FAIL |
| Seat 3 share | 23.7% (22.8-24.7) | 21.8% | 20.8-22.8 | FAIL |
| Seat 4 share | 23.7% (22.8-24.7) | 19.1% | 18.1-20.1 | FAIL |
| Median round of a decisive win | 13 | 8 | 7-9 | FAIL |
| Win round spread (75th - 25th percentile) | 5 rounds | none | | no target |
| Wins by combo or alternate win | 1.9% (1.7-2.3) | 20.0% | 19-21 | FAIL |
| Wins by damage | 85.6% (84.8-86.4) | 62.2% | 61.2-63.2 | FAIL |
| Wins by commander damage | 12.4% (11.7-13.2) | 17.4% | 16.4-18.4 | FAIL |
| Wins by poison | 0.0% (0.0-0.1) | 0.4% | 0-1.4 | PASS |
| Deck win spread (best - worst) | 29.9 pts | none | | no target |
| Games over 15 rounds | 33.0% (32.0-34.1) | none | | no target |

Each deck's share of the decisive games it sat in: Aurelia 41.6%, Atraxa 34.6%, Tergrid 23.0%, Korvold 13.9%,
Marwyn 11.7%. On Playgroup the order is close to the reverse: Marwyn wins 37.8% of its games, Korvold 34.8%, Aurelia
32.4%, Tergrid 26.2%.

## Reading it

- **Speed.** The median win is round 13 against about 8, and a third of games run past round 15. As at Tier 6, a slow
  table flattens the seat edge: seat 1 wins 26.5% against 31.3%.
- **Combo wins** are 1.9% against 20%. On Playgroup, combo is 18.5% of Korvold's wins and 12.5% of Marwyn's (Aurelia
  3.9%, Tergrid 8.0%), and those two are the weakest decks here. Their engines are the first place to look,
  before any knob: a deck whose win line the sim doesn't play can't be fitted into shape.
- **Atraxa** is superfriends in the pool; most Atraxa decks logged on Playgroup are infect (45% of its wins are
  poison), so it is left out of the targets (`data/realism/t3.toml`). JD (2026-10-24) allowed pool decks to be
  updated to reflect real data, so the pool's Atraxa could become the deck players actually log, and rejoin the
  targets.
- Adaptive AI only. The look-ahead is about 60 times slower; Tier 3 has no nightly look-ahead run.
