# Report: Phase 3, PR 1: one permanent arena (branch `phase3-arena`)

Brief: `rust/agents/phase3-arena.md`. Design: `audit/phase3/arena-design.md`. Handoff: `audit/phase3/arena-handoff.md`.
Commits: 6fecef22 (Stage A), cf07bfe1 (Stage B), b60fe55c (Stage C).

## No sims (JD's choice)
No simulation runs at all until Phase 4: no byte-identity runs and no timing sims. **Stages A and B were checked as
"no behaviour change" by the test suite only** (3616 passed at the base and after each stage), not by byte-identical
runs. Iteration order: `player.lands` and `player.perms` are filled and emptied exactly where they were; ids still
increase, so comparisons between ids give the same answers; no map keyed by id is iterated in hash order; four loops
walk the whole arena, and none of them can match a land.

## Stage A (6fecef22): lands are `Perm`s
`Land` and `Game::lands` are deleted; `add_land` makes a `Perm` (tapped, not sick, on the battlefield). `LandId` was
an alias of `PermId` during the move. `Perm::card()` returns the card of a permanent that must have one.

## Stage B (cf07bfe1): call sites
Mechanical rename (`g.land(` → `g.perm(`, `land_mut` → `perm_mut`, `LandId` → `PermId`), diff checked. The alias
and `Game::land` / `land_mut` are gone. `Source::Land`, `Val::Land` keep their names.

## Stage C (b60fe55c): the ability language reads lands
- `dsl::land_sources` lists a player's lands with a compiled ability (not a plain Mountain under Blood Moon); they
  come after `player.perms` in `dsl::fire`, `statics`, `pt` and `ability_options`, so nothing that ran before
  changes order.
- Temples and gain lands: the compiled enter trigger (through `zones::lands_entered` → `dsl::fire`);
  `land_etb_fx` is deleted from the engine, `cards.json` and cardgen. The four surveil lands now surveil.
- Eight lands' activated abilities are offered: Castle Ardenvale, High Market, Maelstrom of the Spirit Dragon,
  Skemfar Elderhall, Hanweir Battlements, Room of Refuge, Sinister Hideout, Urza's Factory. Seraph Sanctuary's
  Angel trigger runs.
- Fetches keep `turn::crack_fetch` (their compiled fetch is not offered); channel lands channel from the hand
  through card code; lands with card-code `land_options` are not offered their compiled abilities too; Reliquary
  Tower keeps `nomax`.
- Treasure Vault: `dsl = []`, because its compiled ability had lost its {X}{X} cost and would have made Treasures
  for free (the AI never used it before either; it is in JD's karn).
- 44 land `paths_ok` notes rewritten to say what runs now.
- Tests, `tests/arena_lands.rs`: a temple scries once; the surveil lands surveil; Seraph Sanctuary; a fetch cracks
  once and its compiled fetch is not offered; Boseiju channels from the hand only; Castle Ardenvale makes a 1/1
  paid by Plains; Skemfar Elderhall; Mutavault animates and connects for 2.

## Checks
- `cargo test --release --no-fail-fast`: 3624 passed, 0 failed, 3 ignored (rerun by the coordinator). Build: 0
  warnings. Practice: 104/104.
- `cardgen export --offline` + `validate` pass; `data/paths_known.txt` empty; `audit --paths` 0 doubles.
- Copying a game (`examples/clone_cost`): 2.91 µs against 2.69-2.83 at the base (about 5% slower: a land is now a
  full `Perm`). The only speed figure (no timing sims).

## Decks that play a land whose behaviour changed (for the recalibration)
"offered" = activated ability now used; "surveil"; "seraph"; "etb_stack" = the same scry or life, now through the
compiled trigger on the stack (it can be countered, and doubled by Yarok-like effects).
- JD's decks: alela [t1] surveil (Hidden Grotto); galadriel [t1, t2] etb_stack (Temple of Mystery, Temple of
  Plenty); karn [t4, t5] offered (Urza's Factory).
- t0: anikthea (etb_stack); giada (seraph; etb_stack); zurgo (offered: Castle Ardenvale; etb_stack).
- t1: isshin (etb_stack); lathril (offered: Skemfar Elderhall; etb_stack); tatyova (etb_stack); teysa (offered:
  High Market; etb_stack).
- t2: brago (etb_stack); lord-windgrace (surveil); meren (offered: High Market; surveil; etb_stack); sythis
  (etb_stack).
- t3: aurelia (etb_stack). t5: miirym (offered: Maelstrom of the Spirit Dragon). t6: rograkh-silas (surveil).

## Follow-ups
- Practice mode doesn't offer a person their lands' compiled abilities yet.
- Manlands still animate as a stand-in token; the land itself could now become the creature.
- Compiled effects still choose targets among nonland permanents only.
- `Player::lands` and `Player::perms` remain two lists.
