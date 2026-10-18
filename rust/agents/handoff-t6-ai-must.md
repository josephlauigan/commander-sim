# Handoff t6-ai-must
Design: `t6_table::must_option` (ai/t6_table.rs) called in `brain::main` before search for T6 players: cast with plan prio>=90, card with piece_gain>=1.0, Sisay fetch ("Sisay: " ability). Skipped when hold_up>0.
Status: code written (uncommitted at this point). Next: build baseline binary (stash) -> /tmp/claude-1000/t6ai-must/base, baseline sims, build new, compare, tests, t3 lock, report.
Status: must_option + brain wiring + tests/ai_t6_must.rs done, built. Baseline sims running (/tmp/claude-1000/t6ai-must/base-*.txt). Next: new-binary sims, t3 lock, full tests, report.
Baseline done; new sims running to /tmp/claude-1000/t6ai-must/new-*.txt
Sims done: new vs base similar; investigating Sisay late (median 12) via trace
Added cheap-rock must (turns<=4). Rebuilding, sims to /tmp/claude-1000/t6ai-must/new2-*.txt
Sims done (new2). Running t3 lock then full tests, then report.
