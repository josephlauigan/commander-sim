# Knobs as data: the zero-change check and a one-at-a-time screen, 2026-10-24

`--knobs FILE` (`rust/crates/sim-cli/src/knobs.rs`) overrides the AI's hand-set values. All runs here:
`commander-sim --realism --pool t6 --ai adaptive --games 2000` (loose, seeds 500000-501999).

**Zero change.** `t6-identity.toml` restates every built-in value (the six Tier 6 decks' pool default style 1.0 /
0.6 / 0.5, cthresh_default 6, instant_extra 0, temp_scale 1.0, the look-ahead defaults). Its output
(`t6-identity-2000.txt`) is byte-identical to the run without a knobs file (`t6-none-2000.txt`). `t6-aggr09.toml`
(aggression 0.9 for every deck) differs, so the knobs reach the AI.

**Screen** (`screen-*.toml`, each knob alone at both ends, for every Tier 6 deck; `screen-all` sets several at once):

| Setting | Seat 1-4 (%) | Median win round | Combo or alt (%) |
| --- | --- | --- | --- |
| built in | 27.2 / 23.8 / 25.8 / 23.2 | 10 | 68.7 |
| aggression 0 | 26.9 / 24.6 / 25.3 / 23.1 | 10 | 70.9 |
| aggression 1 | 26.1 / 25.1 / 26.1 / 22.8 | 10 | 67.6 |
| caution 0 | 26.8 / 25.8 / 25.3 / 22.1 | 9 | 68.9 |
| caution 1 | 27.0 / 23.9 / 26.0 / 23.0 | 10 | 70.0 |
| temp 0.3 | 26.3 / 25.1 / 25.2 / 23.3 | 10 | 68.3 |
| temp 2.0 | 28.0 / 25.3 / 24.6 / 22.1 | 10 | 66.4 |
| cthresh_default 3 | 25.3 / 25.1 / 25.5 / 24.1 | 9 | 70.9 |
| cthresh_default 10 | 26.0 / 25.6 / 24.7 / 23.7 | 10 | 66.5 |
| instant_extra 4 | 27.1 / 23.6 / 25.4 / 24.0 | 10 | 69.7 |
| temp 0.3, aggression 1, caution 0, cthresh 10, instant_extra 4 | 26.9 / 24.7 / 24.0 / 24.5 | 9-10 | 67.0 |
| target | 32.5 / 26.9 / 22.7 / 17.9 (each +/- 1) | 7 (6-8) | 47.8 |

At 2,000 games a seat share is known to about +/- 2 points, so no setting moves the seat edge, and none moves the
median win round off 9-10. Fitting these knobs cannot reach the Phase 1 gate (Tier 6 seat edge inside its band). It
agrees with `audit/t6/speed-2026-10-19/REPORT.md`: every deck goes the round its line is ready, and lines are ready
late (median round 7-10, in 10-49% of games), so speed is set by the deck lists and their plans, not by these knobs.
