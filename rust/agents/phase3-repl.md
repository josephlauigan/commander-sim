# Brief: Phase 3, PR 3: one replacement step

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/phase3-repl` on branch
`phase3-repl`, stacked on `phase3-chars` (Phase 3 PR 2, which is stacked on `phase3-arena`, PR #166; neither is
merged yet). Run every command from there. Read `CLAUDE.md` first: memory cap, no Python. You are the only agent:
`-j 22` for cargo.

## JD's rules for Phase 3 (2026-10-29)

- **No simulation runs of any kind** until Phase 4: no commander-sim realism / deck / goldfish / trace runs. Checks
  are the cargo test suite (release, all pass, 0 warnings) and the practice crate's tests. Microbenchmarks in
  `rust/crates/sim-core/examples/` (`clone_cost`, `chars_cost`) are fine.
- Phase 3's gate is "the test suite passes". JD recalibrates once, after Phase 4.

## Background

`documents/realism.md`, Phase 3. Done: one permanent arena (PR 1, `audit/phase3/arena-design.md`) and the
characteristics pass (PR 2, `audit/phase3/chars-design.md`, `engine/chars.rs`, its `NAMED` table). Read both designs
and follow the same staging.

Replacement and prevention effects (rule 614-616) are scattered as name checks and helper calls at each use site,
about 130 sites across `engine/values.rs`, `engine/zones.rs`, `cardcode.rs`, `hooks.rs` and many `impls/` files:
`cardcode::vorinclex` (counter doubling / halving), `values::melira` (no poison / -1/-1 counters), `cardcode::scales`
(Hardened Scales), Doubling Season (token and counter doubling; its double loyalty was a Phase 2 bug,
`rust/agents/report-phase2-competing.md`), Solemnity, Anafenza / Rest in Peace / Leyline of the Void style "exile
instead", Panharmonicon / Yarok trigger doubling, damage prevention (`values::no_damage`, Iroas), lifegain and
life-loss replacements, "enters tapped" / "enters with counters", draw replacements, and others. Find them all.

## Goal

One replacement step: each kind of event that can be replaced (a permanent entering, counters placed, tokens made,
damage dealt, life gained or lost, a card going to a graveyard, a card drawn, a trigger) goes through one function
that collects the applicable replacement effects from one table and applies them in the right order (the affected
object's controller chooses the order among them, rule 616.1; self-replacement first, 614.15). Stages, each
committed, compiling and passing every test:

1. **Design** (`audit/phase3/repl-design.md`, short): the event kinds, the effect table (fed like `chars::NAMED`,
   looked up once at load, so there is no name compare in the hot path), the ordering rule and how the AI or the
   person (practice mode) picks the order when it matters, how it reaches the existing call sites, the speed plan, and
   every current site mapped to its event kind. List what doesn't fit.
2. **Stage A, results identical:** the step and the table exist, and the existing helpers (`vorinclex`, `melira`,
   `scales`, Doubling Season's checks, Solemnity, `no_damage` ...) become entries read through it, applied in the same
   order they are today. No behaviour change; the existing tests plus new tests comparing old and new on tricky
   boards (Doubling Season + Hardened Scales + Vorinclex, Melira with infect and wither, Rest in Peace and a dying
   creature with a dies trigger, Solemnity with an undying creature, damage prevention).
3. **Stage B, call sites:** each scattered use site calls the step for its event kind and the old helper calls go.
   Still results identical, same checks.
4. **Stage C, behaviour (tests required):** fix the ordering and coverage bugs the design found (an effect that
   misses an event path, a wrong order, the controller's choice of order). Each fix gets a test. List which tiers and
   JD decks (`data/my_decks.toml`) play an affected card, for the recalibration; no sims.

## Checks (every stage)

- `cargo test --release --no-fail-fast -j 22` passes; `cargo build --release --all-targets -j 22` 0 warnings.
- `cardgen export --offline` + `cardgen validate` pass; `cardgen audit --paths` 0 doubles; `data/paths_known.txt`
  stays empty.
- The practice crate builds and its tests pass (JD plays it).
- `clone_cost` and a small replacement microbenchmark before and after: say if it is slower and by how much.

## Rules

- Card data only through `data/card_overrides.toml` and `cardgen export`; no deck list changes.
- Don't seat `omnath` or `godo` in new tests. Format only files you touch (`rustfmt --edition 2024 <file>`, never
  `cargo fmt`), lines up to 120. Mechanical renames with `sed` are fine; check the diff.
- Commit at each stage (and more often), no Co-Authored-By line, never push. Scratch files in
  `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase3-repl/`.
- Keep `audit/phase3/repl-handoff.md` current, so the next agent can continue from it.
- Report in 15 lines or fewer, then the full report as text in your reply (the harness refuses report files from
  subagents): what each stage did, the tests added, the behaviour changes and which tiers / JD decks they touch, the
  microbenchmark numbers, and what doesn't fit yet.
