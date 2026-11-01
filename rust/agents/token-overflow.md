# Brief: a token loop that overflows the "tokens created this turn" count (Thrasios t5, seed 500193)

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/token-overflow` on branch
`token-overflow` (origin/main). Run every command from there. Read `CLAUDE.md` and `rust/agents/RULES.md` first:
memory cap, no Python in any form (the hook misses commands that start with `cd ... &&`: don't run Python that way
either).

**A sim agent runs at the same time** (22 threads): use **`-j 6`** for cargo, in the background. **Run no
`commander-sim` binary yourself**; reproduce through a cargo test or a test-only example run under the memory cap,
one game at a time. Wait on background work by its completion notice or an output file, never `until ! pgrep -f ...`.

## The bug

`commander-sim --deck thrasios --pool t5 --games 300 --seed 500000` on origin/main reports one game panicking:
`seed 500193  panic: attempt to add with overflow (at crates/sim-core/src/engine/zones.rs:1917)`, the `u32` add in
`note_tokens` (`tok_made`): one turn created over 4 billion tokens, so some loop of token creation has no end.

## Work

1. Reproduce that game (the seed's table and seating as the `--deck` run builds them) in a test, find the loop: which
   cards, which trigger or replacement keeps creating tokens, and why it doesn't stop.
2. Fix the cause by the rules (Oracle text in `data/scryfall_cache.json`): a real infinite loop must be handled as
   the engine handles other unbounded loops (look for the existing loop caps or the "draw by an unbreakable loop"
   rule, 104.4b / 732), a mis-modeled card fixed to its Oracle text. Don't just saturate the counter; you may also
   make `note_tokens` saturate as a safety net, in addition.
3. A unit test that fails before and passes after.

## Checks

- `cargo test --release --no-fail-fast -j 6` passes; `cargo build --release --all-targets -j 6` 0 warnings;
  `cardgen export` leaves no diff.

## Rules

- No deck list changes; stay out of `ai/search.rs`, `ai/ismcts.rs`, `ai/learned.rs`, `data/plans/`, `data/eval/`,
  `audit/recal/`. Format only new files, lines up to 120. Commit, no Co-Authored-By line, never push. Scratch in
  `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/token-overflow/`.
- Report in 15 lines or fewer, then the full report as text: the loop, the fix, the test, which decks and tiers can
  reach it.
