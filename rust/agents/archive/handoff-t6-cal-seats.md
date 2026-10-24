# Handoff t6-cal-seats (branch t6-cal-seats)
Task `seats` in t6-calib.md: why is going first worth so little (seat 1 26-28% vs real 34%, seat 4 22 vs 17).
## Plan
1. Build release, cardgen export --offline.
2. Add per-seat stats (win round, mulligans, stopped tries, removal received) in t6stats.rs / t6report.rs (stats only).
3. Run baseline --calibrate within --pool t6 --games 600 --jobs 8, read numbers.
4. Check opening rules (draw T1, mulligan), table rules (ai/t6_table.rs) for dogpile on leader.
5. Fix, measure, 2000 games, t3 lock diff, tests, report-t6-cal-seats.md.
## Status
- started
- 10/19: per-seat block added to t6report.rs (uncommitted->commit next). Baseline 600 (seed 500000, /tmp/claude-1000/t6cal-seats/base600.txt): seat1 26.2/24.6, seat4 21.5/23.1. Per seat: win round, tries/game, mulligans identical across seats; seat 1 takes MORE removal (1.28 vs 0.99/game). Games last 10 rounds, so tempo washes out. Next: trace rounds 1-4 for fast-mana/tempo, check mulligan rate (43%).
- 1200-game baseline (conservative, s500000): seat 26.4/25.7/25.7/22.2. Per seat identical: win round ~10, tries .23-.26/game, T4 mana 4.4-4.6, commander out turn 3.1-3.4, mulligans 42-44%. Differences: removal taken 1.29 (s1) vs 0.97 (s4); stopped 28-36%. Wins by round 6 only ~21% in every seat.
- Rules check: T1 draw in multiplayer (right), free first mulligan then London bottoms (right). No seat-specific code.
- EXPERIMENTS (env T6_SEAT_EXP bit 1 = no x1.25 leader bonus on T6 removal, bit 2 = T6 leader threat ignores board; code in ai/t6_table.rs `exp`, brain.rs) -> /tmp/claude-1000/t6cal-seats/exp{0,1,2,3}.txt. REMOVE the exp switches or turn the winner into a Tier-6-gated rule before final.
- DONE 10/19: no behaviour change; report-t6-cal-seats.md written, t3 lock identical, t6report tests pass.
