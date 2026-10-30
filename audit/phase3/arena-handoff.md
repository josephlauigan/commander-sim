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
- [x] Stage C (committed): the ability language reads lands (`dsl::land_sources`, after `player.perms`, in `fire`,
      `statics`, `pt`, `ability_options`); `land_etb_fx` deleted (engine, `cards.json`, cardgen); land paths_ok notes
      rewritten (44 + Reliquary Tower); Treasure Vault `dsl = []`. Table of what changed: `arena-design.md`.
      Tests: `sim-core/tests/arena_lands.rs` (8 new), `paths_splits.rs` (offered test rewritten). Checks: build 0
      warnings; 3624 passed / 0 failed / 3 ignored (220 binaries); practice 104; export + validate pass, audit
      --paths 0 doubles (208 declared splits); clone_cost 2.91 / 2.91 µs.

## Decks that play a land whose behaviour changed (for the recalibration after Phase 4; no sims run)
Kinds: offered = activated ability now used; surveil = now surveils; seraph = Angel trigger now runs; etb_stack = the
same scry / life, now through the compiled trigger on the stack (Stifle-able, doubled by Yarok-like effects).
- JD's decks: alela [t1] (surveil: Hidden Grotto); galadriel [t1,t2] (etb_stack: Temple of Mystery, Temple of
  Plenty); karn [t4,t5] (offered: Urza's Factory).
- Pool, by tier: t0 anikthea (etb_stack x3), giada (seraph: Seraph Sanctuary; etb_stack: Radiant Fountain), zurgo
  (offered: Castle Ardenvale; etb_stack x2); t1 isshin (etb_stack), lathril (offered: Skemfar Elderhall; etb_stack
  x2), tatyova (etb_stack x2), teysa (offered: High Market; etb_stack x2); t2 brago (etb_stack), lord-windgrace
  (surveil: Commercial District, Underground Mortuary), meren (offered: High Market; surveil: Underground Mortuary;
  etb_stack), sythis (etb_stack); t3 aurelia (etb_stack); t5 miirym (offered: Maelstrom of the Spirit Dragon);
  t6 rograkh-silas (surveil: Undercity Sewers).
- Tiers touched: t0, t1, t2, t3, t5, t6 (t4 only through JD's karn). Treasure Vault (karn) is unchanged (never ran).

## Follow-ups (not in this PR)
- Practice mode: a person's lands still use practice's own land abilities (`human/karn.rs`, `seam`), not the compiled
  ones (`dsl::human_abilities` reads a permanent the person picks; lands aren't offered there yet).
- Manlands still animate as a stand-in token `Perm`; with one arena the land itself could become the creature.
- Targets: compiled effects still choose among `player.perms` only (lands as targets go through card code).
- `Player::lands` / `Player::perms` stay two lists; one battlefield list is possible now but would reorder loops.

## Notes for whoever continues
- zsh doesn't word-split `$files`: pipe file lists through `xargs`.
- Scratch: `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase3-arena/`
  (`checks.sh <tag>` runs build + tests + practice tests + clone_cost; logs `<tag>-build.log`, `<tag>-test.log`).
