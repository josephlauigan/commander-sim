# Handoff: t6-ai kraum-tymna-blue-breach-midrange
Branch t6-ai-kraum. Plan file impls/t6_plan_kraum.rs, tests tests/ai_t6_kraum.rs.
## Steps
1. [now] build + "before" run (300 games, jobs 3) -> /tmp/claude-1000/t6ai-kraum/before.txt
2. write keep/prio/wish/hold_up/go_now; tests; after run; t3 lock; full tests; report
## Notes
base binary saved; before-run started -> /tmp/claude-1000/t6ai-kraum/before.txt; plan file drafted (uncommitted)
tests written; waiting for before-run (~10 min) before building the plan
before-run done; building plan + tests
plan v1 + 9 tests pass, committed; after-run -> /tmp/claude-1000/t6ai-kraum/after1.txt
v1 after: 22.6/20.0% (before 29.7/32.8), mull 36% (was 48). v2: looser keep (counter-only 3-4 land keeps), stop_chance .3; rerun after2.txt
