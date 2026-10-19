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
- exp A result: 35.9 / 34.6 (no effect, reverted to 0.3). exp B: go_now Oracle needs a free counter or creature cover in hand while any opponent has cards + open mana (spec conditions to go)
- exp B result: 33.0 / 35.1 (small). exp C+D: any_open also when stop_chance>=0.05; keep() 3-4 lands: lone engine not a keep
- exp CD result: 35.9 / 34.8 (no effect). exp E: Oracle needs creature_cover whenever any opp has cards+mana (or stop>=0.05); override only near>=8 and round>=7
- exp E result: 35.6 / 33.2 (avg 34.4 vs base 35.05; plan knobs weak). Final = C+D+E kept; running lock then 2000 final
- DONE: final 2000 = 32.9 / 29.5; lock identical; tests pass; report written
