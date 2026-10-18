# Tier 6 engine pass: regression check of Tiers 0-5 (2026-10-17/18)

Within-tier calibration (`commander-sim --calibrate within --pool tN --games 1500 --seed 500000`, the "loose"
profile; two result blocks per tier), on the engine before Tier 6 (`a6bc50b`, worktree t6-baseline) and on t6-cedh
after the engine pass (`a207e07`). Same seeds, same pool lists. Raw outputs are in this folder. 95% intervals are about
±2.5 points per deck, so a shift counts only if it is larger than that and both blocks agree.

| Tier | Result | Notes |
| --- | --- | --- |
| 0 | no change | every deck within 1.7 points |
| 1 | **Isshin up about 6 points** (24.5 -> 30.8, block 2: 25.8 -> 29.7) | the only shift beyond noise. Isshin plays Karlach, Fury of Avernus and Legion Warboss with its attack-trigger doubling; the combat changes (uncapped extra combats, the each-combat event) are the likely cause. Not yet confirmed from game logs. Tatyova and Lathril down 1-2 (noise) |
| 2 | no change | blocks disagree in direction |
| 3 | no change | Aurelia +2.4 / 0.0, Atraxa -2.7 / 0.0 |
| 4 | no change | Krenko +2.6 / +0.8 |
| 5 | small, within noise | Zur -2.1 / -2.8, Yawgmoth +2.3 / +1.4 (consistent direction, at the edge of noise) |

## Speed

Runs with the machine to itself: t3 395 s -> 537 s (+36%), t4 339 s -> 458 s (+35%), t5 176 s -> 244 s (+39%).
**The engine pass made games about 35-40% slower.** Earlier tiers overlapped another session's sims or a pause and
are not comparable. Candidates: the legend-rule and toughness checks at every `check_state`, the hand scan for Spirit
Guides in every mana computation, the cast-lock / cast-counter checks on every castability test. To profile and fix.
