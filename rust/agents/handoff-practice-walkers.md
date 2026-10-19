# Handoff: practice-walkers
- [x] baseline sims: /tmp/pw-before-{sheoldred,cloud,kefka}.txt (binary built from unchanged source)
- [x] step 1 code: Game.human_walker_atk (state.rs), human_attack asks player-or-walker (human/combat.rs),
      engine/combat.rs to_walker uses it for a person (+ no player damage if walker gone)
- [ ] test tests/play_walker_attacks.rs written, being run
- [ ] step 2 several blockers: engine damage step is single blocker (blocker_of = first); AI never multi-blocks;
      per brief: do NOT rewrite combat, say so in report. Step 3 likewise skipped.
- [ ] after sims (/tmp/pw-after-*), diff, workspace test, no-warnings build, report-practice-walkers.md, final commit
