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
