# Handoff t6-ai-keeps
Task: `keeps` in rust/agents/t6-ai.md (keep fns in impls/t6_plan_*.rs, Mox Diamond discard, tests/ai_t6_keeps.rs).
## Status
- [x] measuring tool: tests/ai_t6_keeps.rs (`rates`, `first_hand_by_lands`), run with --nocapture
- [x] shared helper impls/t6_keep.rs (mana by turn 2 from the hand; need_one_lander by mulligan try; settle) used by the keep fns of dack, kinnan, kraum, silas, thrasios, sisay; engine/turn.rs sets stat mull_try (Tier 6 only)
- [ ] BASELINE sim (git stash -u of the above, build, sim to /tmp/claude-1000/t6ai-keeps/base.txt) then pop + AFTER sim -> after.txt
- [ ] Mox Diamond discard choice (engine/zones.rs mox_discard_key, Tier 6 colour-aware)
- [ ] t3 lock, full tests, no warnings (dead consts FAST/SLOW/ACCEL/mana_maker now unused: remove)
- [ ] report rust/agents/report-t6-ai-keeps.md
Gotchas: Godo keep untouched. Pre-change keep rates (1500 hands): mulligan 29-39%, triple 2.5-6.9%, 1-land keeps 9-35%.
