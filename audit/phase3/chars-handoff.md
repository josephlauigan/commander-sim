# Handoff: Phase 3, PR 2, the characteristics pass (branch `phase3-chars`)

Brief: `rust/agents/phase3-chars.md`. Design: `audit/phase3/chars-design.md`. Stacked on `phase3-arena`.

**JD: no simulation runs of any kind until Phase 4.** Checks: release test suite, 0 warnings, practice tests,
cardgen export/validate/audit, and the microbenchmark `examples/chars_cost` (not a sim).

Base (515d67a9): build 0 warnings; 3624 passed / 0 failed / 3 ignored (220 binaries); practice 104; export +
validate pass, 0 doubles, paths_known empty; clone_cost 2.95 / 3.04 µs; chars_cost (ns per permanent):
is_creature 1.5-1.6, power 271-345, toughness 231-236, has_type 22, subtypes 15, colors_of 1.3-1.4, has_kw 407-417,
all 923-935.

## Status
- [x] Design (`chars-design.md`, committed 9ce2e44a)
- [x] Stage A (committed): `engine/chars.rs` (moved bodies of is_creature, epow/etgh, static_bonus, dsl::pt, has_kw, subtypes,
      has_type, ptypes, maskwood, colors_of); the old functions are thin wrappers; `tests/chars.rs` (8). Checks: 0 warnings;
      3632 passed / 0 failed / 3 ignored (221 binaries); practice 104; export/validate/audit clean; clone_cost 2.93 / 2.97;
      chars_cost is_creature 1.5, power 281-342, toughness 233, has_type 22, subtypes 15, colors_of 1.4, has_kw
      415-428, all 941-945 ns (same as base, within noise); a whole `chars()` 7.98-8.10 µs.
- [ ] Stage B: `chars::NAMED` table (Maskwood, Yavimaya, Urborg, Colossus Hammer, Wrecking Ball Arm / Hulkbuster,
      Commander's Insignia, Steelshaper), name checks deleted at their sites
- [ ] Stage C: layer fixes (neutered sources' statics; Blood Moon vs land types and Urborg; Adeline vs base P/T)

## Notes
- Scratch: `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase3-chars/`
  (`checks.sh <tag>`: build, export/validate/audit, tests, practice, clone_cost; logs `<tag>-*.log`).
- `dsl::statics` and `dsl::noncreature_ok` are `pub(crate)` now (chars reads them).
