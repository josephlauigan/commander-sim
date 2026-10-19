# Report t6-cal-rthras (Rograkh/Thrasios) - NOT solved
- Modeling: Oboro line's 6 creatures and Gates requirement are correct; no modeling error found.
- Diagnosis (plan_a fail reasons, rounds >=5, 300 games): Cradle missing ~45% of checks (in library 90% of those), Gates missing ~35% (in library 98%), Oboro missing 12%, finisher/creature count rare. The deck has one Cradle, one Gates and 3 land/creature tutors, so readiness is bounded by draws.
- Changes (tuning): go_now Oboro round>=3 (was 5), Witness round>=4 (was 7) - no effect (14.0%); Crop Rotation/Gamble prio 86 to fetch Gates once Cradle is out - no effect (13.7% at 600).
- Final 2000: Thrasios 13.4% (11.7-15.3), 0.09 combo tries/game; others Dack 34.7, Kinnan 31.2, Kraum 27.8, Silas 22.5, Sisay 20.5.
- Still to try: tutors for Cradle in the wish list (Chord/GSZ can't), mulligan keeping Cradle/tutor hands, lines B-G (Badgermole Cub, Vibrance), a sandbox check that Cradle is not held in hand (49k noCradle-inHand checks vs 1.39M inLib: it is not).
