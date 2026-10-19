# Report t6-cal-rthras2 (Rograkh/Thrasios, lines C/D/G and tutors) - NOT solved, ~14%
Branch t6-cal-rthras (t6-cedh merged first). Runs: /tmp/claude-1000/t6cal-rthras2/ (r600.txt, r2000.txt).

## Modeling errors found (fixed)
- t6_plan_rograkh_thrasios.rs `on_bf` only looked at permanents; lands are not permanents here, so Cradle/Gates on the
  battlefield were invisible. Previous agent's "Crop Rotation/Gamble fetch Gates" change was dead code (explains no effect).
- Tutor picks: `put_creature` (Chord/GSZ) took the first wished card in library order, not the best-ranked (T6 only now;
  lower tiers unchanged, t3 lock safe). Sowing Mycospawn never fetched Gates and only Cradle with 2+ creatures: T6 now
  uses the wish list. Checked as correct already: Expedition Map, Crop Rotation, Gamble (wish list), Vibrance land mode
  (Cradle, then Gates), Spellseeker (instant/sorcery MV<=2 incl. Snap, Finale), Nature's Rhythm/Chord (MV filter, wish).

## New lines (t6_combos_t6.rs, Oracle-counted; sim has no earthbend so the creature-lands are not played)
- C: Cub + Mockingbird/Phantasmal Image + Oboro + Cradle. Clone copies the earthbent land (MV 0) as a plain land; loop
  Cradle N + copy {U} - Oboro 2 - recast {U}: net N-2 (Mockingbird) / N-3 (Image); N counts Cub, land, Oboro: ready at N>=3/4,
  needs a {U} land. Clever Impersonator (no lands) and Flesh Duplicate ({U}{U}) can't.
- D: Cub + clone of Cub + Witness + Snap, no Cradle: 2 creature-lands x (1+2 Cub G) = 6 in, 5 out, Snap untaps both: +1.
- G (Nantuko + Vibrance): NOT unbounded: needs {R}{R} spent per damage, loop makes one renewable {R} (Gates filter); loop
  costs 10 vs Cradle N+1. Skipped.
- Tuning: clones held (prio 0), wish list has Cub/Mockingbird/Image; go_now covers C/D.

## Numbers (seed 500000, both profiles)
- Baseline 2000: 13.7 (earlier agent 13.4). 600: 14.5 / 14.5. Final 2000: 15.4 and 13.3 (avg 14.4, +-1.8).
- Wins by line (2000, profile 1): A 111, damage 59, B 17, C 9, D 3; tries/game 0.13 (A 134, B 29, C 10, D 5).
- C/D add ~12 wins/2000 per profile at most; within noise. Tests (combo2/3, green1/2, ai_t6_rograkh_thrasios, pieces) pass.

## Still open
Readiness is bound by assembling 3-4 specific cards by round ~12 (win round avg 11.8); damage wins are 30%. Candidates: mulligan
toward Cradle/tutor hands, Thrasios dig timing, a keep rule for mana creatures. Deck stays LOW (target 18+).
