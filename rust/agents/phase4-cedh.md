# Brief: Phase 4, PR 2: plan files for Yuriko and the cEDH decks

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/phase4-cedh` on branch
`phase4-cedh`, stacked on `phase4-plans` (Phase 4 PR 1; it and the Phase 3 PRs are not merged yet). Run every command
from there. Read `CLAUDE.md` first: memory cap, no Python, loose profile, opponent tiers. You are the only agent:
`-j 22` for cargo, `--jobs 22` for sims, one sim at a time, in the background; check rc=0 on every run you cite.

## Background

Read first: `rust/agents/phase4-plans.md` (the PR 1 brief), `audit/phase4/plans-design.md` (the plan-file schema,
the derived tutor priority, the `AI_PLAN_DATA` switch: `=0` hand code, `=1` plans, or a comma-separated list of deck
keys), `audit/phase4/plans-handoff.md` (paired numbers so far and what the schema can't express yet), and
`audit/phase4/pair.sh` (the CI computation). Phase 4's gate: each deck level with its hand code in a paired test.
JD recalibrates every tier once, after Phase 4: don't retune or refit styles.

PR 1 moved chulane, yawgmoth, kaalia and light-paws to plan files (hand code deleted). Yuriko's plan lost
(-2.8, CI -5.5..-0.1): its derived wish list is the cause. PR 1 listed what the schema can't express:
- a group that counts only when on the battlefield (for Yuriko, Scroll Rack or Top in hand is not a way to put a
  card back);
- engines ranked ahead of a line's pieces (Yuriko's hand list fetches Rhystic Study first);
- a card's value depending on the rest of the hand (Brainstorm is 45 without a big card to put back);
- "N cards of a group" (Yawgmoth's loop: two undying creatures or Mikaeus);
- the cEDH slots: a keep rule that counts fast mana, hold-up utility, per-combo go-now gates;
- colour fixing in mulligans and keeping mana up for holds (in the schema, not read yet).

## Work

1. **Schema extensions** for the gaps above, each with a unit test, each loader-validated as in PR 1.
2. **Yuriko:** with the extensions, rewrite its plan, re-run its paired test (t4, 2000 games per arm, same seeds,
   `AI_PLAN_DATA=0` vs `=yuriko-dimir-ninjas`). Level or better: delete its hand code. Else keep it and say why.
3. **Yawgmoth:** with "N of a group", re-run its pair; report whether the point estimate (-1.9 in PR 1) closes.
4. **The cEDH decks, one at a time** (`impls/t6_plan_*.rs` + `t6_plans.rs`: scion, dack, kinnan, kraum,
   rograkh_silas, rograkh_thrasios, sisay; skip godo, which isn't seated): plan file, paired test on t6 (2000 games
   per arm, same seeds), delete the hand plan code where level or better, keep it where worse with the reason.
   Start with scion (only a `keep` rule). Note: `scion` is JD's deck; its pair is against its own tier t6 only.
5. Commit after each deck; update `audit/phase4/plans-handoff.md`'s deck table each time.

## Checks

- `cargo test --release --no-fail-fast -j 22` passes; `cargo build --release --all-targets -j 22` 0 warnings;
  practice tests pass; `cardgen validate` passes; the plan-file test passes.
- With no env set, the head binary must match each deck's chosen arm (the plan for decks moved, hand for decks kept)
  byte for byte on that deck's tier (300 games), as PR 1 checked.

## Rules

- No deck list changes; no recalibration; style values stay as they are.
- Don't seat `omnath` or `godo` in new tests. Format only new files (`rustfmt --edition 2024 <file>`), lines up to
  120.
- Commit often, no Co-Authored-By line, never push. Scratch files in
  `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase4-cedh/`.
- Keep the handoff current so another agent can continue if you run out of time.
- Report in 15 lines or fewer, then the full report as text in your reply (the harness refuses report files from
  subagents): the schema extensions, each deck's paired result (both arms, delta, CI), what was deleted, what still
  can't be expressed.
