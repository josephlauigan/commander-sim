# Handoff t6-ai-kinnan
Worktree .claude/worktrees/t6-ai-kinnan, branch t6-ai-kinnan. Plan file impls/t6_plan_kinnan.rs, tests tests/ai_t6_kinnan.rs.
## Steps
1. [x] build + "before" run (--calibrate within --pool t6 --games 300 --jobs 3) -> /tmp scratchpad before.txt
2. [ ] keep (mulligan) + prio + wish + go_now + hold_up
3. [ ] tests, after run, t3 lock, full tests, report
## Notes
- plan file written (keep/prio/wish/hold_up/go_now), needs build; before binary at /tmp/claude-1000/-home-joseph-projects-mtg-sims/ca30944f-e06c-4aa9-9c35-c9fae50d3a2b/scratchpad/cs-before (built from stub); before run -> /tmp/claude-1000/-home-joseph-projects-mtg-sims/ca30944f-e06c-4aa9-9c35-c9fae50d3a2b/scratchpad/before.txt (running)
- tests written (tests/ai_t6_kinnan.rs); building+testing
- tests pass (7). next: release build, after run -> scratchpad after.txt, compare
- before run done (before.txt; 2 preexisting 'subtract with overflow' panics seed 500059, stub binary). after run -> after.txt
- keep loosened (mull 43%->~35% in unit sample; deck has 24 lands so ~35% floor). rerun after -> after2.txt
- go_now: dropped Some(true) (win rate HIGH 37-41%); outlet wait added; rerun -> after3.txt
- after3 done (41.3/35.2 HIGH, mulls 32.7%). next: t3 lock, full tests, report
