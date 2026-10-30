# Brief: Phase 3, PR 1: one permanent arena (lands are permanents)

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/phase3-arena` on branch
`phase3-arena`, based on `phase2-competing` (Phase 2 PR 5a, not merged yet; it will be merged into main before
this PR). Run every command from there. Read `CLAUDE.md` first: memory cap, no Python. You are the only agent
running: `-j 22` for cargo, `--jobs 22` for the short checks below.

## Background

`documents/realism.md`, Phase 3 ("Engine structures"): a characteristics pass, one replacement step, a mana pool
with restrictions, identity for exiled cards, **one permanent arena**. JD chose the arena first (2026-10-29) and
changed Phase 3's gate to "the test suite passes": **no measurement sims** (no before/after realism or deck
campaigns). JD recalibrates once after Phase 4.

Today a land is a `Land` in `Game::lands` (`LandId`, `cd`, `owner`, `tapped`, `data`, `on_bf`) and the player keeps
`lands: Vec<LandId>` apart from `perms: Vec<PermId>`; everything else on the battlefield is a `Perm` in
`Game::perms`. About 1,277 call sites in 198 files touch lands. Consequences found in Phase 2 (PR #164,
`rust/agents/report-phase2-splits.md`): the ability language never runs on lands (`dsl::fire`, `dsl::statics`,
`dsl::ability_options` read only `player.perms`), so the lands' compiled abilities (Boseiju, Castle Ardenvale,
Treasure Vault ...) are never offered, and land effects live in special paths (`land_etb_fx`, `turn::crack_fetch`,
the `f` / `nomax` tags, `t6_lands.rs`, card-code `land_options`). Animated lands (manlands, Nissa) and "lands are
creatures" effects are also special cases; find them.

## Goal

One arena: every permanent on the battlefield, land or not, is a `Perm` with one id type. Do it in stages, each one
committed, compiling, and with the whole test suite passing:

1. **Design first** (in `audit/phase3/arena-design.md`, short): how `Perm` holds a land (types from the card, a
   tapped flag it already has, `PermData`), what replaces `LandId`, how `player.lands` / `player.perms` relate
   (keep two lists as views at first, then decide), how iteration order is kept so results stay the same, and how
   the lands' special paths map onto it. List the risks (iteration order, `on_bf` semantics, mana code, tokens that
   are lands, Dryad Arbor / animated lands, `enter` paths, the AI's land valuation).
2. **Stage A, pure plumbing, results identical:** lands become `Perm`s in the one arena; `LandId` goes (or becomes
   an alias during the move); `player.lands` holds the land `PermId`s and `player.perms` the nonland ones, exactly as
   now. No behaviour change.
3. **Stage B, call sites:** move the land call sites onto the shared `Perm` API (`g.land(x)` → `g.perm(x)` ...),
   delete the `Land` type and `Game::lands`. Still no behaviour change.
4. **Stage C, behaviour (tests required):** the ability language reads lands too (one battlefield list, or the
   DSL iterating both), so lands' compiled abilities, statics and triggers run; delete land special paths the
   compiled abilities now cover (one path per effect, as in Phase 2; `cardgen audit --paths` and the export guard
   must stay clean, and `paths_ok` notes that said "lands' compiled abilities never run" must be revisited); fix the
   four surveil lands. Each changed land behaviour gets a test (e.g. a temple scries once; Boseiju's channel is
   offered and works; a gain land gains once; a fetch cracks once; a manland animates and attacks).

## Checks for "no behaviour change" (Stages A and B)

These are correctness checks, not measurements. Build the base binary first (from `phase2-competing`'s commit
515c00a4, e.g. via `git archive` into your scratch dir). After each of Stages A and B, run
`--realism --pool t3 --games 200 --jobs 22 --quiet` and `--realism --pool t6 --games 200 --jobs 22 --quiet` with the
base binary and with yours: the output must be **byte-identical**. If it isn't, find out why (a `--calibrate within
--pool tN --trace N` replay of the first differing seed on both binaries, diffed) and fix it before moving on. Also
compare `--goldfish` for two of JD's decks at 200 games. Report the timing of the t6 run on both binaries (Phase 3
aims to be no slower; say if it is).

## Checks (every stage)

- `cargo test --release --no-fail-fast -j 22` passes; `cargo build --release --all-targets -j 22` 0 warnings.
- `cardgen export --offline` + `cardgen validate` pass, and the export guard (`data/paths_known.txt` is empty) holds.
- `practice` (the play-it-yourself crate) still builds and its tests pass: JD plays it.
- Stage C: list which tiers and JD decks (`data/my_decks.toml`) play a land whose behaviour changed, for the
  recalibration. No sims for it.

## Rules

- Card data only through `data/card_overrides.toml` and `cardgen export`; no deck list changes.
- Don't seat `omnath` or `godo` in new tests. Format only files you touch (`rustfmt --edition 2024 <file>`, never
  `cargo fmt`), lines up to 120. A mechanical rename across many files is fine with `sed`, but check the diff.
- Commit at each stage (and more often), no Co-Authored-By line, never push. Scratch files in
  `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase3-arena/`.
- Keep `audit/phase3/arena-handoff.md` current: if you run out of time, the next agent continues from it.
- Report in 15 lines or fewer, then the full report as text in your reply (the harness refuses report files from
  subagents): what each stage did, the byte-identity results and timings, the tests added, the land behaviour
  changes, and which tiers / JD decks they touch.
