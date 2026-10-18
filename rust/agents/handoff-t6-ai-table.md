# Handoff t6-ai-table
Status: started. Order: (1) plan slots (item 6) + commit, (2) audit AI, (3) rules 1-5, (4) tests, (5) before/after table, report.
Done: nothing yet.
Building release in background (started)
Step 1 (plan slots): files impls/t6_plan_*.rs + impls/t6_plans.rs (dispatch, is_t6, contract) written and wired:
prio -> ai/decks.rs deck_prio_f; wish -> ai/plans.rs wish_raw; keep -> engine/turn.rs mulligan ok; hold_up -> brain main_options stop_u;
go_now -> combos.rs combo_options. Needs compile check (a first release build is running), then commit.
Step 1 committed (plan slots). Baseline t6 300 games running -> scratchpad base-t6.txt. Next: ai/t6_table.rs (archenemy, counters, hold-up, go, protection).
Step 2 (rules): ai/t6_table.rs written + wired (stack.rs ai_respond verdict, brain.rs Situation/reserve/removal/attack, combos.rs go_ok + attempt wrapper w/ protect + t6_going, life.rs Angel's Grace, pool.rs Ad Nauseam imp 8). Builds. Baseline (300/profile): see report. Next: run new t6 calib, tests (tests/ai_t6_table.rs), lock check t3.
Gaps so far: Silence/Chant only cast right before going (not upkeep / in response).
Step 3: tests/ai_t6_table.rs (12 tests pass) committed. First after-run (new1-t6.txt) running; it is ~2x slower than baseline (archenemy scoring: consider caching).
Next: results table, perf cache if needed, t3 lock diff, full cargo test, all-targets warnings, report rust/agents/report-t6-ai-table.md.
Baseline (300/profile cons|loose): Dack 27.6|37.4, Kinnan 26.4|23.9, Kraum 32.3|33.3, Rograkh(Silas?) 29.1|27.0, Rograkh 11.8|9.5, Sisay 23.7|19.7; rounds 9.4|9.3, timeouts 2.0|2.7.
Experiments running (exp.sh): T6_NOGO/NOCTR/NOARCH env toggles temporary in code (REMOVE before commit); game length rose 9.4->11 rounds with rules
Removed env toggles; go gate relaxed (0.6 base, 0.75 at r>=4, always from r6) because go gate caused the length rise (NOGO 9.0 rounds); cached lines_of for speed. Running new2-t6.txt
