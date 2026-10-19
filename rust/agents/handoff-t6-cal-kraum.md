# Handoff: t6-cal-kraum (branch t6-cal-kraum off t6-cedh)
Task: Kraum/Tymna 34.3% -> ~28-31% (brief rust/agents/t6-calib.md, task kraum).
## Status
- [ ] build release + cardgen export --offline
- [ ] check modeling generosity (Oracle lines, Tainted Pact, free counters, Ad Nauseam)
- [ ] tune impls/t6_plan_kraum.rs
- [ ] 600-game measure, 2000 confirm, t3 lock diff, sim-core tests
- [ ] report rust/agents/report-t6-cal-kraum.md
## Log
(started)
- baseline 600 (seed 500000): conservative 34.8, loose 35.3 (Kraum ~80% seated). Oracle line 79-84% of wins. No modeling error found yet
  (Pact stops at duplicate names; Pact of Negation/FoW/Force of Negation costs modeled; no basics in list).
- exp A: go_now Oracle can_answer threshold stop_chance 0.3 -> 0.12 (needs creature cover more often). Scripts in /tmp/claude-1000/t6cal-kraum/.
