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
- [x] Stage A (committed): `engine/repl.rs` (event kinds, `NAMED` + `TAGGED`, `CardDef::repl`, the counters step with a
      `Path` per old site, `Mods`, tokens, damage, life gain, graveyard, draw, trigger, enter readers); the old helpers
      (`cardcode::scales/vorinclex/put_plus/add_counters`, `values::melira/no_damage`, `solemnity`, `dsl::token_mult/
      counter_mult/lifegain_mod`, `zones::hushed`, `cardcode::damage_prevented`, `partials::tajic_protects`, Spelunking,
      the Doubling Season / Manufactor / Lab Man name checks, `life::gain`) read through it. `tests/repl.rs` (9).
      Checks: 0 warnings; 3649 passed / 0 failed / 3 ignored (222 binaries); practice 104; cardgen clean; clone_cost
      2.93 / 3.01; repl_cost +1/+1 350 / 344, -1/-1 112 / 115, poison 112 / 115, tokens 180-185 / 235-237, damage 1.2,
      life gain 358-360 / 373-381 (same as base, within noise).
- [x] Stage B (committed): every site calls the step for its event kind (`repl::put_plus/add_counters/counters/
      give_poison/poison_ok/minus_ok/enter_counters/tokens/token_mult/no_damage/source_prevented/dovin/tajic_prevents/
      no_graveyard/hushed/trigger_copies/enters_tapped/labman/solemnity`); the old helpers are deleted
      (`cardcode::scales/vorinclex/put_plus/add_counters/damage_prevented/dovin_blocked/tajic_protects`,
      `values::melira/no_damage`, `t45_heliod_zur::solemnity`, `dsl::token_mult/counter_mult/lifegain_mod`,
      `zones::hushed`, `hooks::enters_tapped`, `partials::tajic_protects`). Tests moved to the new paths.
      Checks: 0 warnings; 3649 passed / 0 failed / 3 ignored; practice 104; cardgen clean; clone_cost 2.86 / 2.90;
      repl_cost +1/+1 344 / 337-344, -1/-1 112-114 / 115, poison 112 / 115, tokens 182-190 / 229-236, damage 1.4,
      life gain 353-359 / 375-378 (same as base).
      Left for Stage C (each changes results): `zones::amass` and `sweepers` read Mauhúr with `values::has` (abilities
      on); the direct `.plus +=` writes in `t12_isshin`, `common` (enter copy), `legends`.
- [x] Stage C (committed): one counters path (`repl::put_counters`, `Path::ALL`) for every placement by an effect;
      Doubling Season by its compiled doubler, by the permanent's controller (name entry gone); 616.1 order by the
      affected controller (`repl::ordered`; a person is asked, kind `replacement_order`); abilities lost stop every
      entry (`src_on`) and the compiled counter / token doublers; infect and Phyrexian Unlife poison through the
      step; Solemnity on kept counters and players; direct writes (Breena, Hydaelyn, Spark Double) through the step;
      Tajic respects unpreventable; one pass per counters ask (`gather`, `CardDef::counters_mult`). Tests: 6 more in
      tests/repl.rs (15). Checks: 0 warnings; 3655 passed / 0 failed / 3 ignored (222 binaries); practice 104;
      cardgen clean (0 doubles, paths_known empty, no data change); clone_cost 2.89-3.03; repl_cost (step) +1/+1
      74-107 / 76-81, -1/-1 47-74 / 68-70, poison 46-56 / 51-52, tokens 173-181 / 225-233, life gain 354-367 /
      366-377: faster than base or the same. Decks touched: `repl-design.md`, last section.

## Next (follow-ups, not in this PR)
See "What doesn't fit (yet)" in repl-design.md: a damage event (amounts and Gisela's order), destroy replacements,
Erebos / Iroas into the table, two Manufactors, Mimic's Army counter.

## Notes
- Scratch: `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase3-repl/`
  (`checks.sh <tag>`: build, export/validate/audit, tests, practice, clone_cost, repl_cost; `<tag>.out`, logs
  `<tag>-*.log`).
