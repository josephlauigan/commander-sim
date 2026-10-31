# Brief: Phase 4, PR 5b: blocks, protection and wipe responses as searched decisions

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/phase4-search-b` on
branch `phase4-search-b`, stacked on `phase4-search` (Phase 4 PR 5, #179: ISMCTS for main-phase, attack and counter
decisions, not merged yet). Run every command from there. Read `CLAUDE.md` first: memory cap, no Python, loose
profile, opponent tiers. **Another agent runs sims in parallel** (card modelling, worktree `card-models-1`): use
`--jobs 12` for sims and `-j 12` for cargo, one sim of yours at a time, in the background; check rc=0 on every run you
cite. Within any byte-for-byte or paired comparison, every arm uses the same `--jobs`.

## The goal (JD's plan, AI point 4, second half)

"Add the decisions still heuristic today: blocks, protection, wipe responses." PR 5 built the ISMCTS
(`ai/ismcts.rs`, design `audit/phase4/search-design.md`, handoff `audit/phase4/search-handoff.md`, report
`rust/agents/report-phase4-search.md`). Read those first, then `ai/search.rs`, `ai/learned.rs`, and
`audit/phase4/values-handoff.md` (the half-tier paired method and the halves).

JD (2026-10-30) kept ISMCTS as the default although its Stage B pairs leaned down (7 of 10, mean -0.85 points, not
significant; Meren -3.7 [-6.3, -1.1]). So this PR has a second job: **find and fix the lean** if it can.

## Scope

- In: generic-policy decks only (decider-gated, `values::generic_deck`), as in PR 5.
- Out: JD's decks and Tier 6 keep today's code. No style values, no recalibration, no evaluation refit, no deck list
  changes.

## Work, staged

1. **Survey and design** (append a "Part (b)" section to `audit/phase4/search-design.md`, commit before code):
   - Where each of the three decisions is made today and how (blocks: the declare-blockers code; protection: the
     responses to removal aimed at the decider's permanents, e.g. the removal-response hook and protection spells;
     wipe responses: what a deck does when a wipe is cast or about to resolve). Measure how often each comes up per
     game, per tier (300 games), using PR 5's cost counter (`state::SearchCost`; add kinds).
   - Which of them gain from search: the candidates (block assignments, or respond / don't with each protection
     option), the stakes test that sends a decision to search (lethal or near-lethal combat, a key permanent
     targeted, a wipe that swings the board), and the budget (attacks and counters were capped at 6 playouts a
     candidate in PR 5 for cost).
2. **The lean, ablations first** (on one or two halves that leaned down, e.g. t2 half B with Meren and t1 half B):
   PR 5's next ideas were slower widening (`AI_SEARCH_WIDEN`) and opponent nodes at their first reply. Also look at
   Meren's games: which decisions the ISMCTS changes for it and whether they are worse (a `--trace` of a few games
   where the arms differ). Keep a change only if it helps on the ablation halves; report every ablation.
3. **Stage A:** the new searched decisions behind a switch (e.g. `AI_SEARCH_B`, unset = PR 5's behaviour):
   byte-identical to PR 5's head on t1, t2, t4, t5 (300 games). Determinism: two runs with the same seed match.
4. **Stage B, paired tests:** each tier t1-t5, both halves, PR 5's head vs this PR on that half; 2000 games an arm,
   seeds 500000-501999, look-ahead AI. Report games per second, CPU seconds and playout steps per game for both arms.
   The aim is better, or level at no more than about 10% more cost.
5. **Stage C:** default where level or better; else keep PR 5's and write why.
6. Commit after each step; keep `audit/phase4/search-b-handoff.md` current.

## Checks

- `cargo test --release --no-fail-fast -j 12` passes; `cargo build --release --all-targets -j 12` 0 warnings;
  `cardgen validate` passes; `all_decks` adaptive 3 / lookahead 1 rc=0.
- Unit tests on hand-built positions: a block the search finds that the heuristic misses (e.g. a chump that prevents
  lethal, or a double block that kills an attacker without losing the game); protection used on the right permanent
  (the one whose loss swings the game); holding a protection spell when the removal is aimed at a low-value target;
  a wipe response that saves the key permanent. Same seed, same choice; the budget is respected.
- With no env set, the head matches the chosen arm byte for byte on each tier (300 games).
- JD's decks seated only in their own tiers; don't seat `omnath` or `godo` in new tests.

## Rules

- Format only new files (`rustfmt --edition 2024 <file>`), lines up to 120. No new crates.
- Gotchas: naming `--example`/`--test` targets in a cargo build skips the main binary, so `cmp` the binary after each
  rebuild; `pgrep -f` inside an `until` loop matches the loop itself.
- Commit often, no Co-Authored-By line, never push. Scratch files in
  `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase4-search-b/`.
- PR 5's scratch (its binaries and arms) is in `.../scratchpad/phase4-search/`: reuse its arms only when the binary
  and env are identical.
- Report in 15 lines or fewer, then the full report as text in your reply (the harness refuses report files from
  subagents): the survey numbers, the design, every ablation, each tier's two pairs (both arms, delta, CI, games/s,
  CPU, steps/game), what became default and why.
