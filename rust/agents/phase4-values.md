# Brief: Phase 4, PR 3: values from abilities

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/phase4-values` on branch
`phase4-values`, stacked on `phase4-cedh` (Phase 4 PR 2; it, PR 1 and the Phase 3 PRs are not merged yet). Run every
command from there. Read `CLAUDE.md` first: memory cap, no Python, loose profile, opponent tiers. You are the only
agent: `-j 22` for cargo, `--jobs 22` for sims, one sim at a time, in the background; check rc=0 on every run you cite.

## The goal (JD's plan, AI point 2)

"A card's worth is computed from what it does on the board (mana, cards, removal, damage, board presence) at the
current state, plus its role in a win line. That replaces `bomb=` tags and the 0-90 priority tables, and it gives a
new card a sensible value the day it is added." Strategy tags (`bomb`, role hints) become derived values computed from
abilities, with a short override table where the derivation is wrong.

Read first: `audit/phase4/plans-design.md` and `audit/phase4/plans-handoff.md` (PRs 1-2: plan files, the
`AI_PLAN_DATA` switch, the paired method), `audit/phase4/pair.sh` (CI), `ai/pool.rs` (`generic_prio` and the tables
it falls back to), `ai/plans.rs`, and the compiled ability language (`engine/dsl*`, `cardcode.rs`, `combo_imp`).
Find every place a `bomb` tag or a fixed priority table feeds a decision, and list them in the design.

## Scope

- In: the generic policy's card values (cast order, tutor and wish fallback, keep/mulligan value, discard and
  sacrifice choice, threat reading of an opponent's card where it uses `bomb`), for decks without their own hand code.
- Out: JD's decks' own priority functions (`ai/decks.rs`, `seph`, `veyran`, `impls/<jd deck>.rs`): they stay and are
  replaced last, after Phase 4's other points. Plan-file `[[prio]]` rules and `key_cards` still win over the derived
  value. The learned evaluation (point 3) and search (point 4) are later PRs: no evaluation-weight changes here.

## Work, staged

1. **Design** (`audit/phase4/values-design.md`): the value formula from abilities (mana produced, cards drawn, removal
   reach, damage, board presence, engine/repeatable effects, cost and colour fit), how a win-line role adds to it
   (combo pieces from the catalogue / Spellbook, plan lines), how the derived `bomb` is read, and the override table
   format (data, e.g. `data/values_overrides.toml`, each row with a reason). Commit before code.
2. **Stage A, results identical:** compute the derived value beside the old one, switch `AI_VALUES` (`0` old, `1`
   derived, unset = old). With it unset, output is byte-identical to the base on t1, t2, t4, t5 (300 games). Add a
   diagnostic (Rust test or example, not Python) that prints the old and derived values side by side for every card
   in the pool, and the rank correlation per tier; write the worst disagreements into the design with the cause.
3. **Fix the derivation** where it is wrong for a general reason; use the override table only for real one-offs.
4. **Stage B, paired tests:** in a tier the decks' shares sum to a constant, so flipping every seat measures nothing.
   Split each tier's generic-policy decks into two fixed halves (A, B; listed in the handoff). Per tier t1-t5, two
   pairs: `AI_VALUES=0` vs derived values on half A only (A's summed share), then the same for half B; 2000 games per
   arm, seeds 500000-501999. Add a per-deck switch if `AI_VALUES` needs one (as `AI_PLAN_DATA=<keys>` does). Level means
   the CI includes 0 or the derived arm is better.
5. **Stage C:** where level or better on a tier, make the derived values the default and delete the replaced tables
   and `bomb=` reads for that path. Where worse, keep the old path, write why, and what closes the gap.
6. Commit after each step; keep `audit/phase4/values-handoff.md` current (tier table, both arms, delta, CI).

## Checks

- `cargo test --release --no-fail-fast -j 22` passes; `cargo build --release --all-targets -j 22` 0 warnings;
  `cardgen validate` passes; `all_decks` adaptive 3 / lookahead 1 rc=0.
- Unit tests for the formula on named cards (a rock, a draw engine, a wipe, a bomb, a combo piece) checked against
  their Oracle text.
- With no env set, the head matches the chosen arm byte for byte on each tier (300 games).
- JD's decks are seated only in their own tiers; don't seat `omnath` or `godo` in new tests.

## Rules

- No deck list changes, no recalibration, style values stay. Format only new files (`rustfmt --edition 2024 <file>`),
  lines up to 120.
- Commit often, no Co-Authored-By line, never push. Scratch files in
  `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase4-values/`.
- Keep the handoff current so another agent can continue if you run out of time.
- Report in 15 lines or fewer, then the full report as text in your reply (the harness refuses report files from
  subagents): what feeds on the old values, the formula, the correlation, each tier's two pairs (both arms, delta,
  CI), what was deleted, what was kept and why.
