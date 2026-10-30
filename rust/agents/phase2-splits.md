# Brief: Phase 2, PR 5b: judge the 161 undeclared splits

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/phase2-splits` on
branch `phase2-splits`, based on `phase2-coverage` (PR #163). Run every command from there. Read `CLAUDE.md` first:
memory cap, no Python. Another agent works in parallel on `phase2-competing` and runs the sims: **you run no sims**,
and use `-j 8` for cargo.

## Background

PR #161 removed every hard double (method and report: `rust/agents/phase2-doubles.md`,
`rust/agents/report-phase2-doubles.md`; read both, especially its "False positives (`paths_ok`)" section). PR #163
made `cardgen export` refuse any double not declared with `paths_ok`, and froze the ones left in
`data/paths_known.txt`, which only shrinks. JD chose (2026-10-29) to clear the list before Phase 3. Your part is its
**161 `split` lines**: cards where the compiled abilities and the card code both reach the sim, for what
`audit --paths` thinks are different parts of the card.

## Work

For each of the 161 cards: read the Oracle text (`data/scryfall_cache.json`), the compiled abilities
(`data/cards.json`) and the card code (`cardgen audit --paths`, grep `impls/`). Put it in one group:

1. **A real split**: the two paths do different parts of the card, and nothing runs twice. Add
   `paths_ok = "<what each path does>"` to its entry in `data/card_overrides.toml` (one short line, as in PR #161:
   e.g. "compiled: the cost reduction; card code: the +1/+1 counter"), and delete its line from
   `data/paths_known.txt`.
2. **Really a double** (some part runs twice, or two paths compete for one ability): **don't fix it.** Leave its
   line, and list it in your report with the evidence (what runs twice, and when). Fixing changes behaviour, which
   needs measured runs: that is a follow-up PR.
3. **Unsure**: leave its line and list it with the question.

A `paths_ok` note is a claim that nothing runs twice, so check it: for a card whose two paths act on the same event
(both on enter, both on attack ...), read both bodies, and where it isn't obvious, add a test that the effect happens
once. Don't add `paths_ok` to make the list shrink.

## Checks

- `cardgen export` (no change besides the overrides' `paths_ok`, which doesn't change behaviour; confirm
  `data/cards.json` doesn't change except for that field, if it's exported at all) and `cardgen validate` pass.
- `cargo test --release --no-fail-fast -j 8` passes (the known-list test fails on stale lines, so every card you
  declare must leave the list); `cargo build --release --all-targets -j 8` 0 warnings.
- No sim-core code change. If you find you need one, stop and list it instead.
- Update `documents/realism.md`'s Phase 2 paragraph with the new known-list counts.

## Rules

- Card data only through `data/card_overrides.toml` and `cardgen export`; no deck list changes.
- Commit often (e.g. every 30 cards), no Co-Authored-By line, never push. Scratch files in
  `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase2-splits/`.
- Keep `audit/phase2/splits-handoff.md`. Report in 15 lines or fewer, then the full report as text in your reply
  (the harness refuses report files from subagents): counts per group; the real doubles and the unsure cards in a
  table with their evidence; any test you added.
