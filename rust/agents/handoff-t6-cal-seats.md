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
