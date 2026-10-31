# Brief: card models, part 2: JD's last three Partials, then the pool's Partial and Unmodeled cards

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/card-models-2` on branch
`card-models-2`, from origin/main. Run every command from there. Read `CLAUDE.md` and `rust/agents/RULES.md` first:
memory cap, no Python, "Check every swap". Part 1's handoff is `audit/card-models/handoff.md` (its ranking of the
pool's cards, and what each JD Partial needs): read it first, and keep it current as part 2.

**Two other agents work in parallel** (worktrees `phase4-rules` and `phase4-hand-off`, AI files): use `-j 8` for
cargo, in the background.

## No sims

**Run no `commander-sim` of any kind** and no game-playing example: checks are cargo tests only (JD, 2026-10-31; sims
come later in the night, after all code work). Wait on background work by its completion notice or an output file,
never with `until ! pgrep -f ...`, and stop any wait loop whose job is cancelled.

## Work

1. **JD's last three Partials**, each through a general engine or ability-language feature (Rust card code only for
   real one-offs), behaviour matching the Oracle text in `data/scryfall_cache.json`:
   - **Redirect Lightning** (thanos): "Change the target of target spell or ability with a single target" — a way to
     change the target of a spell or ability on the stack; the AI uses it on removal or a counter aimed at its key
     permanent or spell.
   - **Shifting Woodland** (thanos): "becomes a copy of target permanent card in your graveyard until end of turn"
     (delirium) — a real copy effect on the land itself that ends at end of turn (no enter triggers).
   - **Peter Parker's Camera** (urdragon): "Copy target activated or triggered ability you control. You may choose new
     targets for the copy" — abilities as copyable objects; the AI copies its best ability (e.g. a tutor or a combo
     trigger).
   If one needs more than about a third of the session, write down what's built and what's left and move on.
2. **The pool's Partial and Unmodeled cards**, in part 1's rank order (most decks first), the same way. Mostly Tier 6
   (dack, rograkh-thrasios, sisay, kinnan-bonder, atraxa-infect, kraum-tymna...). Leave Approximate cards alone.
3. **Unit tests** for each card (Oracle behaviour, and the AI using it where that matters). Mark each status honestly
   (Full only when nothing is left out).
4. In the handoff, list for each changed card the JD decks and pool tiers that play it (for the recalibration).
5. Run `cardgen export` and `cardgen validate` after data changes; regenerate generated files with the tools, never by
   hand.

## Checks

- `cargo test --release --no-fail-fast -j 8` passes; `cargo build --release --all-targets -j 8` 0 warnings;
  `cardgen validate` passes; `cardgen audit` and `--mine` show the new statuses.
- Don't seat `omnath` or `godo` (the pool deck) in new tests.

## Rules

- Stay out of the AI files other agents are changing: `ai/search.rs`, `ai/ismcts.rs`, `ai/learned.rs`,
  `ai/values.rs`, `ai/brain.rs`, `ai/rules.rs`, `ai/decks.rs`, `data/eval/`, `audit/phase4/`. If a card needs an AI
  hook there, write it down and skip it.
- No deck list changes; no recalibration.
- Format only new files (`rustfmt --edition 2024 <file>`), lines up to 120. Commit often (one card or one feature a
  commit), no Co-Authored-By line, never push. Scratch files in
  `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/card-models-2/`.
- Report in 15 lines or fewer, then the full report as text in your reply (the harness refuses report files from
  subagents): each card's old and new status and what was built, the features added, what moved (decks and tiers), and
  what is left.
