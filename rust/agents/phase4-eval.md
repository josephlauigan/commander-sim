# Brief: Phase 4, PR 4: a learned evaluation

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/phase4-eval` on branch
`phase4-eval`, stacked on `phase4-values` (Phase 4 PR 3, #173, not merged yet). Run every command from there. Read
`CLAUDE.md` first: memory cap, no Python, loose profile, opponent tiers. You are the only agent: `-j 22` for cargo,
`--jobs 22` for sims, one sim at a time, in the background; check rc=0 on every run you cite.

## The goal (JD's plan, AI point 3)

"Record positions from millions of sim games together with who went on to win. Fit the evaluation to predict the
winner from features the code already computes (life, board value, cards, mana, combo progress, threat), and later
from richer ones. Start with a linear or small boosted-tree model trained and run in Rust. Refit after each big
change. This turns the evaluation's weights from guesses into measurements, and fixes problems like the look-ahead
holding mana for nothing."

Read first: `ai/search.rs` (`evaluate`, `strength`, `win_score`, the output terms), `ai/evdiag.rs` (the existing
evaluation diagnostic), `audit/ai-foundations/` and `audit/ai-eval*` if present, `audit/phase4/values-design.md`
(PR 3's derived values and `threat`, usable as features), and `audit/phase4/pair.sh`.

## Scope

- In: the value `evaluate` returns for a non-terminal position, for **generic-policy decks** (decider-gated as
  `values::on` is: `values::generic_deck`). Win/loss scores, `WIN_FLOOR`, `win_score` stay.
- Out: JD's decks and Tier 6 keep today's evaluation; they move in Phase 4's last step. No search changes (ISMCTS is
  PR 5), no style values, no recalibration.

## Work, staged

1. **Design** (`audit/phase4/eval-design.md`): the feature list (from `strength`'s terms per seat, relative to the
   strongest opponents, plus PR 3's worth / threat sums, combo distance from the plan/catalogue, turn, seat order,
   tier), the sampling (which decisions, how many positions per game, so positions in a game aren't over-weighted),
   the label (did the evaluated seat win; draws), the model (start with regularised logistic regression; a small
   boosted-tree model only if it clearly beats it on held-out log loss), and how the model's win probability maps
   onto `evaluate`'s scale (below `WIN_FLOOR`, monotone). Commit before code.
2. **Recording:** a recorder behind an env switch (`AI_EVAL_RECORD=<path>`) that changes nothing about play (byte-identical
   output with it on vs off, 300 games, t1 and t5). Compact binary or CSV, written per thread; budget the disk: at
   most ~20 GB total, in the scratchpad above, and delete raw data you no longer need before you finish. Record
   tiers t1-t5 (generic decks' seats), with seeds outside 500000-501999 (keep those for the paired tests).
3. **Training in Rust** (an example or a `cardgen`/`commander-sim` subcommand, not Python): fit on most games, report
   held-out log loss, Brier and AUC against today's `evaluate` mapped through a fitted logistic (so the baseline is
   fair). Split by game, not by position. The fitted weights become a data file (e.g. `data/eval/model.toml`, with
   the commit, data hash, sample size and held-out scores in it), embedded and loader-validated.
4. **Stage A:** the learned evaluation beside the old, switch `AI_EVAL` (`0` old, `1` learned for generic decks, a
   key list for those decks); unset = old, byte-identical to the base on t1, t2, t4, t5 (300 games).
5. **Stage B, paired tests:** as PR 3 (the shares in a tier sum to a constant): each tier t1-t5, the two fixed halves
   from `audit/phase4/values-handoff.md`, `AI_EVAL=0` vs learned on that half only; 2000 games per arm, seeds
   500000-501999, the look-ahead AI (it is the one that reads `evaluate`). Level = CI includes 0, or better. Also
   report the speed (games per second) of both arms: a slowdown over 10% needs a reason.
6. **Stage C:** where level or better on a tier, make it the default. Where worse, keep the old, write why and what
   would close it. Check the "holding mana for nothing" case named in the plan with the diagnostic: does it change?
7. Commit after each step; keep `audit/phase4/eval-handoff.md` current.

## Checks

- `cargo test --release --no-fail-fast -j 22` passes; `cargo build --release --all-targets -j 22` 0 warnings;
  `cardgen validate` passes; `all_decks` adaptive 3 / lookahead 1 rc=0.
- Unit tests: the model loads and refuses bad files; the mapping is monotone and stays below `WIN_FLOOR`; a few
  hand-built positions (far ahead, far behind, even) order as expected.
- With no env set, the head matches the chosen arm byte for byte on each tier (300 games).
- JD's decks seated only in their own tiers; don't seat `omnath` or `godo` in new tests.

## Rules

- No deck list changes; no recalibration; style values stay. Format only new files (`rustfmt --edition 2024 <file>`),
  lines up to 120. New crates only if small and well-known; say which and why.
- Commit often, no Co-Authored-By line, never push. Scratch files in
  `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase4-eval/`.
- Keep the handoff current so another agent can continue if you run out of time.
- Report in 15 lines or fewer, then the full report as text in your reply (the harness refuses report files from
  subagents): the features, the data (games, positions, disk), held-out scores old vs learned, each tier's two pairs
  (both arms, delta, CI), speed, what became default, what was kept and why.
