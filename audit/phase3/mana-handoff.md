# Handoff: Phase 3, PR 4, a mana pool with restrictions (branch `phase3-mana`)

Brief: `rust/agents/phase3-mana-exile.md`. Design: `audit/phase3/mana-design.md`. Stacked on `phase3-repl`.

**JD: no simulation runs of any kind until Phase 4.** Checks: release test suite, 0 warnings, practice tests,
cardgen export/validate/audit, and the microbenchmarks `examples/clone_cost`, `examples/repl_cost`, `examples/mana_cost`
(not sims).

Base (5e8fb584): build 0 warnings; 3655 passed / 0 failed / 3 ignored (222 binaries); practice 104; export +
validate pass, 0 doubles, paths_known empty; clone_cost 3.03 / 3.03 µs; mana_cost (ns, two runs): empty units 150-152, can_pay {3}{G} 355-402 /
{6}{U}{R} 503-505, pay 633-639; float units 134-135, can_pay 625-636 / 762-777, pay 919-928.

## Status
- [x] Design (`mana-design.md`) and `examples/mana_cost`
- [x] Stage A: `engine/pool.rs` (`Pool`, `Mana`, `Key { restr, kind, keep }`); `Player::floating` is `Player::mana`;
      every add / read goes through the pool; `Source::Float*` are `Source::Pool(key)`; restricted pool mana is offered
      only where `mana::restr_ok`. Checks: 0 warnings; 3655 passed / 0 failed / 3 ignored; practice 104; cardgen clean;
      clone_cost 2.95-3.04; mana_cost the same as base (empty units 145-147, pay 615-624; float pay 937-957).
- [x] Stage B: the source restrictions are `Restr` values (table in the design), one test (`mana::restr_ok(g, p, r)`):
      `CardDef::mana_restr` (`mana::restr_of`, at load) for Workshop, the Powerstones, Giada, Hydro-Channeler, Rivaz;
      the colour code of Plaza (two copies), Unclaimed Territory / Cavern (two), Secluded Courtyard, Haven, Voldaren
      Estate, Avengers Tower, Delighted Halfling asks `restr_ok`. Deleted: `art_mana_ok`, `dragon_only_mana`, the name
      checks in `mana_units`. Giada and Avengers Tower now count changelings, Territory / Cavern Maskwood Nexus (one
      predicate per restriction; no deck plays both, so results are identical). Checks: 0 warnings; 3655 passed;
      practice 104; cardgen clean; clone_cost 2.93-2.94; mana_cost (ns) empty units 160-162, can_pay 359-399 /
      509-514, pay 620-628; float units 135-136, can_pay 618-620 / 748-760, pay 962-998.
- [ ] Stage C: emptying (500.4), leftovers keep their restriction, Photon's white, Cavern / Halfling riders,
      Maelstrom; tests

## Notes
- Scratch: `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase3-mana/`
  (`checks.sh <tag>`: build, export/validate/audit, tests, practice, clone_cost, repl_cost, mana_cost; `<tag>.out`,
  logs `<tag>-*.log`). `basewt/`: a detached worktree at the base for the base `mana_cost` (its own target dir;
  remove it with `git worktree remove` when done).
