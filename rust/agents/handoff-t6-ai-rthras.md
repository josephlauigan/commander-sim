# Handoff t6-ai-rthras (deck:rograkh-thrasios-creature-combo)
Done: read spec/brief; traced 12 games (tr*.txt in scratchpad). Findings: combos need Cradle bf + Thrasios bf + 6 creatures
(+Oboro in hand, Gates in reach); generic AI casts Oboro as a plain creature, tutors take Sol Ring/Mana Vault, Rograkh
cast turn 2, Thrasios often removed, 36% mulligans. Pact of Negation can be cast with <5 usable U lands and loses at upkeep.
Step 2 (now): plan file written (prio hold Oboro/Witness/Snap/Finale/GSZ, commanders high, wish list, keep, go_now). Next: build,
test, measure 300 games `--calibrate within --pool t6`, tests file ai_t6_rograkh_thrasios.rs, t3 lock, report.
Step 3: keep loosened (mulligans were 49%), tests file written+passing. Next: measure m2 (300 games), diagnose slow assembly via traces, Pact guard, t3 lock, full tests, report.
Step 4: Gamble/Crop prio; Pact upkeep guard (t6-gated, engine/stack.rs pact_affordable: 4 of ~21 traced Rthras games were lost to a missed Pact upkeep). Next: measure m3, t3 lock, full tests, report.
