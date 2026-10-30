# Brief: Phase 3, PR 2: the characteristics pass

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/phase3-chars` on branch
`phase3-chars`, stacked on `phase3-arena` (Phase 3 PR 1, a PR not merged yet; lands are `Perm`s now). Run every
command from there. Read `CLAUDE.md` first: memory cap, no Python. You are the only agent: `-j 22` for cargo.

## JD's rules for Phase 3 (2026-10-29)

- **No simulation runs of any kind** until Phase 4: no commander-sim realism / deck / goldfish / trace runs, no
  byte-identity or timing sims. Checks are the cargo test suite (release, all pass, 0 warnings) and the practice
  crate's tests. Microbenchmarks in `rust/crates/sim-core/examples/` (e.g. `clone_cost`) are fine.
- Phase 3's gate is "the test suite passes". JD recalibrates once, after Phase 4.

## Background

`documents/realism.md`, Phase 3: a characteristics pass, one replacement step, a mana pool with restrictions,
identity for exiled cards, one permanent arena (done in PR 1: `audit/phase3/arena-design.md`).

Today a permanent's characteristics are worked out in many places, each with its own special cases:
- `Game::is_creature` (`state.rs`, ~910 uses): token / creature card / `DataKey::Anim`, minus `NotCreature`;
- `values::epow` (~394 uses) and its toughness twin, plus `values::static_bonus`, `dsl::pt`: base P/T, counters,
  anthems, equipment tags (Sword, Helm), Adeline, Norn, ...;
- `values::has_type` (~153), `values::subtypes`, `values::colors_of`, `galadriel::card_is`: types, with Maskwood
  Nexus / "all types" special-cased;
- `dsl::has_kw` (~89), `cards::CardDef::has_kw`, `eot_kw`: keywords;
- name checks spread through `impls/` for effects that change characteristics (Blood Moon on lands, animation,
  Humility-like effects if any, copy effects via `CopyOf`).

## Goal

One place computes a permanent's characteristics, in the rules' layer order (613: copy, control, text, type,
colour, ability, P/T with its sublayers), and every rule reads them from there. Stages, each committed, compiling,
and passing every test:

1. **Design** (`audit/phase3/chars-design.md`, short): the `Chars` struct (card types, subtypes, supertypes,
   colours, power, toughness, keywords, and whether its abilities are on), the function that builds it
   (`chars(g, m)`), how each existing effect maps onto a layer, and the speed plan: these are hot paths, so decide
   between computing on demand with cheap fast paths and caching per state version; measure with `clone_cost` or
   a small new `examples/chars_cost` microbenchmark (not a sim). List what does NOT fit yet.
2. **Stage A, results identical:** implement `chars` so it reproduces today's answers exactly, and turn
   `is_creature`, `epow`/toughness, `has_type`, `subtypes`, `colors_of`, `has_kw` into thin readers of it.
   No behaviour change; the existing tests are the check, plus new unit tests that compare `chars` with the old
   functions on a set of tricky boards (anthems + counters + equipment, Maskwood, animated lands, a manland, a
   Blood Moon board, a copy).
3. **Stage B, move the special cases in:** each scattered name check that changes characteristics becomes a layer
   entry fed from one table, deleting the special case at its use site. Still results identical, same check.
4. **Stage C, behaviour (tests required):** fix the layer bugs the design found (wrong order, effects that miss a
   source, e.g. an anthem that misses lands that are creatures). Each fix gets a test. List which tiers and JD decks
   (`data/my_decks.toml`) play an affected card, for the recalibration; no sims.

## Checks (every stage)

- `cargo test --release --no-fail-fast -j 22` passes; `cargo build --release --all-targets -j 22` 0 warnings.
- `cardgen export --offline` + `cardgen validate` pass; `cardgen audit --paths` 0 doubles; `data/paths_known.txt`
  stays empty.
- The practice crate builds and its tests pass (JD plays it).
- The microbenchmark before and after: say if the characteristics pass is slower and by how much.

## Rules

- Card data only through `data/card_overrides.toml` and `cardgen export`; no deck list changes.
- Don't seat `omnath` or `godo` in new tests. Format only files you touch (`rustfmt --edition 2024 <file>`, never
  `cargo fmt`), lines up to 120. Mechanical renames with `sed` are fine; check the diff.
- Commit at each stage (and more often), no Co-Authored-By line, never push. Scratch files in
  `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase3-chars/`.
- Keep `audit/phase3/chars-handoff.md` current, so the next agent can continue from it.
- Report in 15 lines or fewer, then the full report as text in your reply (the harness refuses report files from
  subagents): what each stage did, the tests added, the behaviour changes and which tiers / JD decks they touch, the
  microbenchmark numbers, and what doesn't fit yet.
