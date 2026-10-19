# Handoff: t6-ai-dack (deck:dack-fayden-white-stax)
Branch t6-ai-dack. Files: impls/t6_plan_dack.rs, tests/ai_t6_dack.rs.
## Status
- [x] baseline done (agents/dack-before.txt; 2 games panic at seed 500059 subtract overflow, pre-existing)
- [ ] plan: keep, prio, wish, hold_up, go_now
- [ ] tests
- [ ] after run, t3 lock, full tests
- [ ] report
## Notes
- 1st draft of t6_plan_dack.rs written (keep/prio/wish/hold_up/go_now), NOT yet built. Baseline running into agents/dack-before.txt (ends with "done").
- Next: build (after baseline ends, binary is in use), write tests/ai_t6_dack.rs, run after, t3 lock.
- tests file written (7 tests); next build+run tests, then after-run
- tests pass; running after-run -> agents/dack-after1.txt
- keep loosened; rebuilding; after-run2 -> agents/dack-after2.txt. trace helper test removed
- keep v3 (66% keep offline); running agents/dack-after3.txt
- after3 good (28.1%, mull 27.6%); now t3 lock + full tests + report
