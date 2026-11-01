# Brief: plan files good enough to replace the hand AI of yshtola, sheoldred and cloud

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/plan-files-3` on branch
`plan-files-3` (origin/main). Run every command from there. Read `CLAUDE.md`, `rust/agents/RULES.md`,
`audit/phase4/plans-design.md` (the plan schema and how the generic policy reads it) and
`audit/phase4/hand-off-design.md` first: memory cap, no Python.

**A sim agent runs at the same time** (22 threads): use **`-j 6`** for cargo, in the background. **Run no
`commander-sim` and no game-playing example yourself**: cargo tests only. Wait on background work by its completion
notice or an output file, never `until ! pgrep -f ...`.

## Why

In the recalibration's paired tests (`audit/recal/pairs-2026-10-31.md` section 4), these three decks did better with
their hand-written AI than with the generic policy plus their plan file (`data/plans/<deck>.toml`):
cloud +6.6 over t1-t3, yshtola +4.1 on t5, sheoldred +1.9 over t3-t4. The goal is a plan file (and, where needed, a
general schema feature) with which the generic policy plays these decks as well as their hand AI, so the hand AI can
be deleted later. The raw runs, with each arm's game shape (plan timing, how it wins and dies, cards cast, mana and
hand at turns 4 and 8): `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/recal/runs/`
`dk-<deck>-<tier>.txt` (hand AI off) and `ha-<deck>-<tier>.txt` (on). Cloud and yshtola changed lists on 10/31
(#190, #192: Forge Anew, Teferi's Protection; Sphere of Safety): fit the plan files to the lists in `data/decks.json` now.

## Work, for each deck (cloud first: the biggest gap)

1. Read its hand AI (`hand_key(..) == "<deck>"` sites, its `impls/` and `ai/` modules) and list each decision it makes
   differently from the generic policy (priorities, holds, keeps and mulligans, targets, timing, attacks, responses).
   Compare with the two arms' game shape to pick the ones that most plausibly explain the gap.
2. Put each into the plan file with the schema as it is. Where the schema can't say it, add a **general** schema
   feature (one any deck's plan file can use, documented in `plans-design.md`, read by the generic policy, with unit
   tests) instead of deck-specific Rust. Keep each feature small; if one would take more than about a third of the
   session, write down what it would need and move on.
3. Unit tests: with the hand AI off (`force_hand(Some(false))`), the generic policy now makes the hand AI's choice in
   a hand-built position for each decision you encoded.
4. Write `audit/<deck>/plan-file-2026-11-01.md` per deck: the decisions listed, which you encoded and how, which not
   and why. The coordinator will run the hand AI on/off paired test afterwards.

## Checks

- `cargo test --release --no-fail-fast -j 6` passes; `cargo build --release --all-targets -j 6` 0 warnings;
  `cardgen validate` passes; `cardgen export` leaves no diff.
- **With the defaults, the three decks still play their hand AI**, so nothing you add may change any deck's default
  play except through these three plan files (which are read only with their hand AI off) and schema features no
  existing plan file uses. Say how you checked.

## Rules

- Don't change the hand AI itself, deck lists, `ai/search.rs`, `ai/ismcts.rs`, `ai/learned.rs`, `ai/rules.rs`,
  `data/eval/`, `audit/recal/`, other decks' plan files. No sims.
- Format only new files (`rustfmt --edition 2024 <file>`), lines up to 120. Commit per deck, no Co-Authored-By line,
  never push. Scratch in `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/plan-files-3/`.
- Report in 15 lines or fewer, then the full report as text (the harness refuses report files from subagents).
