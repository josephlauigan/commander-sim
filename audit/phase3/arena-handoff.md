# Handoff: Phase 3, PR 1, one permanent arena (branch `phase3-arena`)

Brief: `rust/agents/phase3-arena.md`. Design: `audit/phase3/arena-design.md`.

**JD (2026-10-29, via the coordinator): no simulation runs at all until Phase 4**, including the brief's
200-game byte-identity runs, goldfish compares and the t6 timing. Stages A and B are checked by the test suite
(release, all pass, 0 warnings) and the practice crate's tests only, with extra care that `player.lands` /
`player.perms` keep their order. `examples/clone_cost.rs` (not a sim) gives the copy cost.

Base: 515c00a4 (phase2-competing): build 0 warnings; 3616 tests passed, 0 failed, 3 ignored (219 binaries);
clone_cost 2.83 / 2.69 µs per copy (60 permanents, 32 lands).

## Status
- [x] Design (`arena-design.md`)
- [x] Stage A (committed): lands are `Perm`s in `Game::perms`; `type LandId = PermId` and a `LandId(n)` fn during the move;
      `Land` and `Game::lands` gone; `g.land(l)` / `land_mut` return the `Perm`; `.cd` on a land is `.card()`.
      Checks: build 0 warnings; 3616 passed / 0 failed / 3 ignored (same as base); practice 104 passed;
      clone_cost 3.05 / 2.99 µs (base 2.83 / 2.69: about 10% more per copy, a `Perm` is bigger than a `Land`).
- [x] Stage B (committed): call sites onto the `Perm` API (`g.land(` -> `g.perm(`, `land_mut` -> `perm_mut`,
      `LandId` -> `PermId`); the alias, the `LandId(n)` fn and `Game::land` / `land_mut` are gone. Checks: build 0
      warnings; 3616 passed / 0 failed / 3 ignored; practice 104; `cardgen export --offline` (data unchanged) and
      `validate` pass; clone_cost 3.07 / 2.94 µs.
- [ ] Stage C: the ability language reads lands; special paths deleted where compiled abilities cover them; tests.

## Notes for whoever continues
- zsh doesn't word-split `$files`: pipe file lists through `xargs`.
- Scratch: `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase3-arena/`
  (`checks.sh <tag>` runs build + tests + practice tests + clone_cost; logs `<tag>-build.log`, `<tag>-test.log`).
