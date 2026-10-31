# Brief: card models, part 3: the pool's remaining Partial and Unmodeled cards

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/card-models-3` on branch
`card-models-3`, stacked on `card-models-2` (card models part 2, PR to come). Run every command from there. Read
`CLAUDE.md` and `rust/agents/RULES.md` first: memory cap, no Python. Read `audit/card-models/handoff.md` (parts 1 and
2: the ranking and what each card needs) and keep it current as part 3.

**A sim agent runs at the same time** (worktree `overnight-stack`, 22 threads): use **`-j 4`** for cargo, in the
background. **Run no `commander-sim` and no game-playing example yourself**: cargo tests only. Wait on background work
by its completion notice or an output file, never `until ! pgrep -f ...`.

## Work

1. The pool's remaining Partial and Unmodeled cards (36 + 1 after part 2), in the handoff's rank order: first the
   multi-deck ones (Hidden Strings: cipher; Invasion of Ikoria: battles; Magda, Brazen Outlaw: a general "becomes
   tapped" event; Astral Cornucopia: an X cast for an artifact and mana per counter), then the 30 single-deck ones.
   Each through a general engine or ability-language feature where one fits; Rust card code only for real one-offs;
   behaviour matching the Oracle text in `data/scryfall_cache.json`. Leave Approximate cards alone. If a feature
   would take more than about a third of the session, write down what's built and what's left and move on.
2. Part 2 left Redirect Lightning and Peter Parker's Camera Approximate because hand-written abilities call
   `stack::ability_window` (no target on the stack). If time allows after step 1, move the hand-written abilities that
   have a single target to `ability_window_ex` and re-mark the two cards.
3. Unit tests for each card; honest statuses; in the handoff, the decks and tiers that play each changed card.
4. `cardgen export` and `cardgen validate` after data changes; regenerate generated files with the tools.

## Checks

- `cargo test --release --no-fail-fast -j 4` passes; `cargo build --release --all-targets -j 4` 0 warnings;
  `cardgen validate` passes; `cardgen audit` shows the new statuses.
- Don't seat `omnath` or `godo` (the pool deck) in new tests.

## Rules

- Stay out of the AI policy files: `ai/search.rs`, `ai/ismcts.rs`, `ai/learned.rs`, `ai/values.rs`, `ai/brain.rs`,
  `ai/rules.rs`, `ai/decks.rs`, `ai/mod.rs`, `data/plans/`, `data/eval/`, `audit/phase4/`, `audit/recal/`.
- No deck list changes; no recalibration.
- Format only new files (`rustfmt --edition 2024 <file>`), lines up to 120. Commit often, no Co-Authored-By line, never
  push. Scratch in `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/card-models-3/`.
- Report in 15 lines or fewer, then the full report as text (the harness refuses report files from subagents): each
  card's old and new status and what was built, the features added, what moved (decks and tiers), what is left.
