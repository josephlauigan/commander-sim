# Report t6-cal-rthras4 (Rograkh/Thrasios, keep the commanders alive) - improved, 17.5 / 17.1% (target 18-32), just short
Branch t6-cal-rthras (t6-cedh merged first). Runs: /tmp/claude-1000/t6cal-rthras4/ (base600, RT4_*-600, final2000, traces tr1-60).

## 1. Is the deck over-targeted? No (checked, no change)
60 traced within-tier games (43 with the deck): 49 removals on its permanents, vs Sisay 51 and Kinnan 46 (each seated about as often). In the
600-game reports "removal resolved ... hit" per seated game: Kinnan 0.74, Thrasios 0.65, Sisay 0.6. Thrasios (engine) is the usual target (bounce 13
of 33: Sisay's Otawara 5), Rograkh much less. Nothing in the table rules favours it: no table change made (ai/t6_table.rs untouched).
Real finding: the two commanders were cast 3.6 times per game (156 casts in 43 games), Rograkh again at tax 6-8 in rounds 12-14.

## 2. Changes (plan tuning only, impls/t6_plan_rograkh_thrasios.rs `prio`; no modeling error found)
- A: Rograkh with tax >= 4 (third cast from the command zone) -> prio 40 instead of 97 ("must"): the search decides, it stops paying 4+ for a 0/1.
- B: Thrasios with tax >= 2 (already removed once) while an opponent has mana open and >= 40% chance of an instant answer in hand
  (`removal_open`) -> prio 60 instead of 93: the search may wait. Hard hold (prio 0) was worse: 15.2 / 14.2, dropped.
- Not done: protection for the commanders (Veil of Summer / Snap / Chain in response) needs a response hook, no plan slot for it.

## Numbers (seed 500000, conservative / loose)
| | 600 | 
|---|---|
| baseline | 16.1 / 15.7 |
| A | 18.3 / 15.7 |
| B (prio 60) | 17.6 / 15.9 |
| B hard hold | 15.2 / 14.2 (dropped) |
Final A+B at 2000: Thrasios 17.5% (15.6-19.6) / 17.1% (15.1-19.2), avg 17.3 (rthras3 final2000: 15.9 / 13.3, avg 14.6). Wins by line (profile 1): A 141, damage 60,
B 18, C 8; 0.16 combo tries per game, 21% stopped; median win round 12. Average game 10.2-10.3 rounds, timeouts 2.2-2.5%.

## Still differs
Below 18 by under a point; median win round 12 and 0.15 tries per game remain the gap (pieces arrive late, Oboro/Gates missing). Gains are within one CI
of baseline each, the 2000-game pair is the evidence. Next: response-time protection for the commanders, Oboro tutors held for instant speed.

## Checks
t3 lock diff: identical. Tests passed: ai_t6_rograkh_thrasios, ai_t6_keeps, ai_t6_pieces, engine_t6_combo2, engine_t6_combo3.
