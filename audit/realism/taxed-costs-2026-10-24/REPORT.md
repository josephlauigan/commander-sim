# Taxed costs in the AI's checks (branch `taxed-costs`, 2026-10-24)

## The bug

Normal casts pay `cost_of`, which includes taxes (Thorn of Amethyst, Sphere of Resistance, Defense Grid, Damping
Sphere). Several AI paths checked the printed cost instead:

- combo affordability (`combos::affordable` / `mana_need`);
- the combo runner's casts, which also *paid* the printed cost, so they underpaid under a tax;
- removal (`rem_cost`), kicker, held answers, burn to the face, end-of-turn and flash casts;
- the `pool.rs` plan casts and the Tier 6 protection check.

A line such as Thassa's Oracle + Demonic Consultation could start on mana for the printed {U}{U} + {B} and then stop
at the taxed second piece.

## The fix

- `engine/mana.rs` `seq_cost(g, p, cards)` gives the cost of casting cards one after another. It is each card's
  `cost_of` plus {1} per Damping Sphere for each spell of the line cast before it.
- Combo checks (`combos::mana_need`, the Tier 6 line helpers in `t6_combos_t6.rs`) use `seq_cost`.
- The other paths use `cost_of`.
- Tests:
  - `engine_t6_combo3.rs`: under Damping Sphere or Thorn of Amethyst, the Oracle line waits on three lands and wins
    on four. Both tests failed before the fix.
  - `cards_t6_stax.rs`: a unit test of `seq_cost`.
- Full suite: 3006 passed, 0 failed, 3 ignored.

## Before and after (Tier 6, adaptive AI, loose, same seeds)

Scorecard, 8200 games (`base-8200.txt`, `fix-8200.txt`). Every change is inside the 95% CI:

| statistic | before | after |
| --- | --- | --- |
| median win round | 9.0 | 9.0 |
| combo / alt wins | 69.2% | 69.2% |
| damage wins | 29.9% | 30.0% |
| deck spread (pts) | 19.4 | 18.8 |
| Dack / Kinnan / Kraum-Tymna / Rograkh-Silas / Rograkh-Thrasios / Sisay | 35.6 / 22.4 / 29.7 / 22.7 / 16.3 / 23.5 | 35.3 / 22.1 / 29.4 / 23.0 / 16.6 / 23.7 |

The direct counts moved. From the `--calibrate within` report, 4000 games (`base-cal.txt`, `fix-cal.txt`):

| count | before | after |
| --- | --- | --- |
| Oracle line stopped "by an unnamed answer" (a cost, a ward or a lock), Kraum/Tymna | 60 | 33 |
| the same, Rograkh/Silas | 132 | 104 |
| combos stopped, Kraum/Tymna | 364 of 945 (39%) | 327 of 913 (36%) |
| combos stopped, Rograkh/Silas | 513 of 1057 (49%) | 496 of 1055 (47%) |

Over 150 traced games, refused plays ("couldn't play") went from 2057 to 1778. Half-cast stops ("fizzles" or "stops
at step") went from 63 to 55.

## What it means

The fix is correct and removes a real class of wasted turns, but taxes were too rare in these games to explain Tier 6
being slow.

Still open:

- The remaining refused plays are mostly zero-cost or cheap cards: Lotus Petal 119, Expedition Map 109, Culling the
  Weak 103, Mana Vault 79, Mox Diamond 75, Chrome Mox 44. Their cause isn't cost. The AI lists something that
  `act::perform` then rejects, probably a lock or a castability rule its options don't check.
- The biggest "unnamed answer" stops are Ad Nauseam (Kraum/Tymna 137) and Sisay's Derevi + Emiel line (96). They
  belong to steps 3 and 4.
- Godo's `t6_combos_godo::affordable` sums `cost_of` per part without Damping Sphere's growing tax. That's minor:
  Godo is not in the current Tier 6 pool.
