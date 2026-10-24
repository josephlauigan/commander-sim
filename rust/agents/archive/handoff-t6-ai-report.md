# Handoff t6-ai report
Status: started. Read brief. Next: explore calibration code, deck name helper, stats struct.

## Done so far (uncommitted until noted)
- Names: cardgen decks::display_name -> decks.json names "Rograkh/Silas" etc (sed on data/decks.json too); sim-cli play.rs name_of uses decks.json name for pool decks.
- sim-core/src/t6stats.rs (new): counters as player stats with `|` keys; Player.t6 flag.
## Next
- hook the counters: resolve_counter, removal, on_cast (stax), combos.rs (go/win/stop), mulligan (mull_n)
- sim-cli: t6 report section after balance table (poolmode.rs run_calibration), seat shares, tests
- lock t3 diff, full tests, report

## Update (step 2)
- Hooks wired (resolve_counter, removal/wipe, on_cast stax, combos go/win/stop, mull_n); sim-cli t6report.rs written + hooked in poolmode run_calibration (t6 only; other tiers print nothing new, for the lock).
- tests: sim-core/tests/t6_stats.rs, sim-cli t6report unit tests.
- Running: agents/lock.sh (t3 lock; temp files in /tmp/claude-1000/t6ai-report/). NOTE /tmp/claude-1000 is shared with other agents: use my subdir only.
- Next: build (after lock finishes), run tests t6_stats + sim-cli, run t6 report at 300 games, commit, baseline table, full tests, report.
- Step 3 done: t6 report prints OK (first run saved /tmp/claude-1000/t6ai-report/t6-run1.txt); t6_stats tests + sim-cli tests pass; lock identical (before sim-cli report hook).
- ALL DONE; report written, final commit made.
