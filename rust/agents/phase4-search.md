# Brief: Phase 4, PR 5: tree search (ISMCTS) where it matters

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/phase4-search` on branch
`phase4-search`, stacked on `phase4-eval` (Phase 4 PR 4, #174, not merged yet). Run every command from there. Read
`CLAUDE.md` first: memory cap, no Python, loose profile, opponent tiers. You are the only agent: `-j 22` for cargo,
`--jobs 22` for sims, one sim at a time, in the background; check rc=0 on every run you cite.

## The goal (JD's plan, AI point 4)

"Information-set Monte Carlo tree search (ISMCTS) over the same determinized copies, with the learned evaluation at
the leaves, a horizon of one to two turns, and a time budget per decision instead of fixed counts. Spend it on
high-stakes choices (combo turns, wipes, counter wars) and play obvious ones from the policy. Add the decisions still
heuristic today: blocks, protection, wipe responses." And from the plan's statistics section: "A look-ahead game makes
about 125 searched decisions and 270,000 playout steps. Searching only high-stakes decisions and stopping rollouts
early when a candidate is clearly ahead should cut the per-game cost by a large factor; measure it before and after."

Read first: `ai/search.rs` (today's look-ahead: determinized copies, playouts, `evaluate`, the hold bonus and margin
rules), `ai/learned.rs` and `audit/phase4/eval-design.md` (PR 4), `audit/phase4/values-handoff.md` (the half-tier
paired method and the halves), `audit/ai-foundations/`, `audit/ai-eval*` and `audit/t6/speed-2026-10-19/` if present.

## Scope

- In: the look-ahead for **generic-policy decks** (decider-gated, `values::generic_deck`).
- Out: JD's decks and Tier 6 keep today's search; they move in Phase 4's last step. No style values, no
  recalibration, no evaluation refit (a later PR).
- This may land as two PRs: (a) ISMCTS + high-stakes gating + time budget + early stopping; (b) blocks, protection
  and wipe responses as searched decisions. Do (a) fully first; start (b) only if (a) is finished and checked, on a
  new commit series in the same branch, and say clearly where you stopped.

## Work, staged

1. **Design** (`audit/phase4/search-design.md`): how ISMCTS samples determinizations from what the decider knows
   (hidden hands and libraries), the tree (decider's own choices; opponents' replies from their policy or a shallow
   node), selection (UCT or similar, its constant), the horizon (one to two turns), the leaf value (the learned
   evaluation, `win_score` at terminal), the budget (a per-decision limit in playout steps, not wall-clock, so runs
   stay reproducible with seeds), early stopping (a candidate clearly ahead), and the high-stakes test (which
   decisions get search; the rest play from the policy). First measure today's cost: searched decisions and playout
   steps per game, per tier. Commit before code.
2. **Stage A:** behind `AI_SEARCH` (`0` today's, `1` ISMCTS for generic decks, a key list for those decks; unset =
   today's): byte-identical to the base on t1, t2, t4, t5 (300 games). Determinism: two runs with the same seed and
   `--jobs` must match.
3. **Stage B, paired tests:** each tier t1-t5, both halves, `AI_SEARCH=0` vs ISMCTS on that half; 2000 games per arm,
   seeds 500000-501999, the look-ahead AI. Report games per second and playout steps per game for both arms. The aim
   is level or better at a lower cost; if it is level only at a higher cost, say how much.
4. **Stage C:** default where level or better; else keep today's and write why.
5. Commit after each step; keep `audit/phase4/search-handoff.md` current.

## Checks

- `cargo test --release --no-fail-fast -j 22` passes; `cargo build --release --all-targets -j 22` 0 warnings;
  `cardgen validate` passes; `all_decks` adaptive 3 / lookahead 1 rc=0.
- Unit tests: the tree finds a forced win in one on a hand-built position that the policy misses; the budget is
  respected; early stopping fires on a dominant candidate; same seed, same choice.
- With no env set, the head matches the chosen arm byte for byte on each tier (300 games).
- JD's decks seated only in their own tiers; don't seat `omnath` or `godo` in new tests.

## Rules

- No deck list changes; no recalibration; style values stay. Format only new files (`rustfmt --edition 2024 <file>`),
  lines up to 120. No new crates unless small and well-known; say which and why.
- Gotcha from PR 3: naming `--example`/`--test` targets in a cargo build skips the main binary; `cmp` the binary after
  each rebuild. Gotcha from PR 4: `pgrep -f` inside an `until` loop matches the loop itself.
- Commit often, no Co-Authored-By line, never push. Scratch files in
  `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase4-search/`.
- Keep the handoff current so another agent can continue if you run out of time.
- Report in 15 lines or fewer, then the full report as text in your reply (the harness refuses report files from
  subagents): today's cost, the design choices, each tier's two pairs (both arms, delta, CI, games/s, steps/game),
  what became default, what was kept and why, and whether part (b) was started.
