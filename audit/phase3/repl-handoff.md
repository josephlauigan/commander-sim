# Handoff: Phase 3, PR 3, one replacement step (branch `phase3-repl`)

Brief: `rust/agents/phase3-repl.md`. Design: `audit/phase3/repl-design.md`. Stacked on `phase3-chars`.

**JD: no simulation runs of any kind until Phase 4.** Checks: release test suite, 0 warnings, practice tests,
cardgen export/validate/audit, and the microbenchmarks `examples/clone_cost`, `examples/repl_cost` (not sims).

Base (4d9d0ca4): build 0 warnings; 3640 passed / 0 failed / 3 ignored (221 binaries); practice 104; export +
validate pass, 0 doubles, paths_known empty; clone_cost 2.95 / 3.13 µs; repl_cost (ns per ask, empty / doublers):
+1/+1 328-334 / 342-346, -1/-1 112-115 / 115-116, poison 112-115 / 115-116, tokens 186-196 / 228-245, damage 1.4,
life gain 358-368 / 373-374.

## Status
- [x] Design (`repl-design.md`) and `examples/repl_cost`
- [ ] Stage A: `engine/repl.rs`, `repl::NAMED`, `CardDef::repl`; helpers read through it; tests/repl.rs
- [ ] Stage B: call sites call the step; old helpers gone
- [ ] Stage C: coverage and order fixes, tests, decks touched

## Notes
- Scratch: `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase3-repl/`
  (`checks.sh <tag>`: build, export/validate/audit, tests, practice, clone_cost, repl_cost; `<tag>.out`, logs
  `<tag>-*.log`).
