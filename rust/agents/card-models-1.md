# Brief: card models, part 1: Partial and Unmodeled cards in JD's decks, then the pool's most-played Partials

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/card-models-1` on branch
`card-models-1`, from origin/main. Run every command from there. Read `CLAUDE.md` first: memory cap, no Python, loose
profile, opponent tiers, "Check every swap". **Another agent runs sims in parallel** (Phase 4 PR 5b, worktree
`phase4-search-b`): use `--jobs 12` for sims and `-j 12` for cargo, one sim of yours at a time, in the background;
check rc=0 on every run you cite.

## The goal (JD, 2026-10-30)

By the end of Phase 4, every Partial and Unmodeled card in JD's decks is modelled, and the pool's Partial cards that
come up often in games are too. Approximate cards stay unless they skew a result. This PR does as much of that as fits,
JD's decks first.

## Work

1. **Build and list:** `cargo build --release -j 12`, then `cardgen audit --mine` and `cardgen audit` (the pool). Write
   the list in `audit/card-models/handoff.md`: each Partial or Unmodeled card, its deck(s), what is missing, and the
   Oracle text from `data/scryfall_cache.json`. At the time of the brief, JD's decks had about 15:
   - karn: Treasure Vault (the {X}{X}, {T}, sacrifice ability; the ability language has no {X} activation cost);
   - thanos: Orim's Chant (kicker), Redirect Lightning, Tinder Wall, Gemstone Caverns (the opening-hand clause),
     Shifting Woodland (delirium);
   - urdragon: Dragonlord Ojutai (body only), Nicol Bolas, the Ravager (the flip), Peter Parker's Camera;
   - yshtola: Venat, Heart of Hydaelyn // Hydaelyn;
   - cloud: Buster Sword;
   - seph: Glimpse the Unthinkable (Unmodeled).
2. **Model JD's cards** through the ability language first (data, the compiled ability text), extending the language
   when a general feature is missing (an {X} activation cost, a kicker on a spell, an opening-hand effect, delirium):
   a general feature helps every card that needs it. Rust card code only for real one-offs. Each card's behaviour
   must match its Oracle text; write down any approximation left and mark the status honestly (Full only when
   nothing is left out).
3. **Unit tests** for each modelled card (`tests/cards_<deck>.rs` or the existing card-test file), checking the Oracle
   behaviour, including the AI using it where that matters (e.g. Treasure Vault's X chosen sensibly).
4. **The pool's Partials:** rank the pool's Partial and Unmodeled cards by how often they are drawn in games (a
   300-game run per tier t1-t6 with the per-card drawn counts the reports print, or a `cardgen audit` frequency if
   there is one). Model them in that order as time allows, the same way. Leave Approximate cards alone.
5. **Check the effect** of JD's cards: for each JD deck with a changed card, run that deck on its own tiers (loose,
   2000 games, seeds 500000-501999) before and after, and update `audit/<deck>/FINDINGS.md` (model status, and the
   number with why it moved). For the pool cards, one 2000-game run per touched tier before and after is enough.
6. Run `cardgen export` and `cardgen validate` after data changes. Regenerate generated files with the tools, never by
   hand.

## Checks

- `cargo test --release --no-fail-fast -j 12` passes; `cargo build --release --all-targets -j 12` 0 warnings;
  `cardgen validate` passes; `cardgen audit --mine` shows the new statuses.
- JD's decks seated only in their own tiers; don't seat `omnath` or `godo` in new tests.

## Rules

- Stay out of the AI files Phase 4 is changing: `ai/search.rs`, `ai/ismcts.rs`, `ai/learned.rs`, `ai/values.rs`,
  `ai/brain.rs`, `data/eval/`, `audit/phase4/`, `data/values_overrides.toml`. If a card needs an AI hook there, write
  it down in the handoff and skip it.
- No deck list changes; no recalibration.
- Format only new files (`rustfmt --edition 2024 <file>`), lines up to 120. Commit often (one card or one language
  feature a commit), no Co-Authored-By line, never push. Scratch files in
  `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/card-models-1/`.
- Keep `audit/card-models/handoff.md` current, so another agent can continue.
- Report in 15 lines or fewer, then the full report as text in your reply (the harness refuses report and FINDINGS
  files from subagents: give the FINDINGS text in the reply if a write is refused): each card's old and new status and
  what was built, the language features added, the before/after numbers, and what is left.
