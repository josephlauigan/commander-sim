# Report t6-cal-rthras3 (Rograkh/Thrasios, finding the pieces sooner) - NOT solved, ~14.6%
Branch t6-cal-rthras (t6-cedh merged first). Runs: /tmp/claude-1000/t6cal-rthras3/ (base600, E1/E2/E4-600, final2000).

## Measured at 600 games, seed 500000 (conservative / loose)
| change | result | verdict |
|---|---|---|
| baseline | 14.5 / 14.5 | - |
| E2: Thrasios digs with spare {4} on its own main 2 (utility -2 -> 2, no counter reserve) | 13.3 / 13.5 | dropped (tuning) |
| E1: keep rule: 2-5 lands + Cradle/Gates/finder (Crop Rotation, Gamble, Map, Mycospawn, Vibrance, Chord, Spellseeker) always kept | 15.7 / 14.7, mulligans 41% -> 38% | KEPT (tuning; spec's mulligan section) |
| E4: Rhystic Study / Mystic Remora / Sylvan Library prio 90 | 14.2 / 12.5 | dropped (tuning) |
- go_ok/go_now (read, no run): rthras go_now returns Some(true) for the Oboro lines from round 3 (or with cover) and Witness lines from round 4, so go_ok is not what refuses ready lines. Lines C/D contain "Oboro"/"Eternal Witness" so are covered. No modeling error found.
- Thrasios's end-of-turn dig already fires (EOT utility 3 -> 99%); the own-turn dig is rejected by the search ("stop (hold mana)"), and forcing it did not help.

## Final 2000 with E1 (seed 500000): Thrasios 15.9% (14.0-17.9) / 13.3% (11.6-15.2), avg 14.6 (before: 15.4 / 13.3, avg 14.4). Within noise.
Wins by line (profile 1): A 115, damage 60, B 17, C 10, D 3; 0.14 combo tries per game, 22% stopped.

## What the traces show (why it stays low)
Not draw speed alone: in traces the deck finds Cradle (Mycospawn fetched it round 4) but Thrasios and the commanders are removed
or exiled again and again (removal hits on this deck ~240 per 600 games, 2nd-3rd most at the table) and each recast costs {2} more plus
the turn; Oboro is the missing piece in several games. Opponents' counters also hit it often. Next ideas: protect Thrasios (hold a
free counter / Veil for it), do not recast Thrasios into open removal when the engine is not needed, hold Oboro tutors for instant speed.

## Checks
t3 lock diff: identical. Tests (ai_t6_rograkh_thrasios, ai_t6_keeps, engine_t6_combo2/3, ai_t6_pieces): see handoff/final message.
Code: t6_plan_rograkh_thrasios.rs only (FINDERS, keep()).
