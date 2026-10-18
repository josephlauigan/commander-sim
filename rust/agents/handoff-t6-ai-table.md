# Handoff t6-ai-table
Status: started. Order: (1) plan slots (item 6) + commit, (2) audit AI, (3) rules 1-5, (4) tests, (5) before/after table, report.
Done: nothing yet.
Building release in background (started)
Step 1 (plan slots): files impls/t6_plan_*.rs + impls/t6_plans.rs (dispatch, is_t6, contract) written and wired:
prio -> ai/decks.rs deck_prio_f; wish -> ai/plans.rs wish_raw; keep -> engine/turn.rs mulligan ok; hold_up -> brain main_options stop_u;
go_now -> combos.rs combo_options. Needs compile check (a first release build is running), then commit.
