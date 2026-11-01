# Brief: delete the hand-written deck AI of the 11 decks whose generic policy was level or better

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/hand-ai-delete` on branch
`hand-ai-delete` (origin/main: #186 hand AI switch, #191 defaults, #193 Dack, #195 Veyran taken apart). Run every
command from there. Read `CLAUDE.md` and `rust/agents/RULES.md` first: memory cap, no Python. Evidence:
`audit/recal/pairs-2026-10-31.md` section 4 and `audit/recal/dack-check.md`; design: `audit/phase4/hand-off-design.md`.

**A sim agent runs at the same time** (worktree `overnight-stack`, 22 threads): use **`-j 6`** for cargo, in the
background. **Run no `commander-sim` and no game-playing example yourself**: cargo tests only. Wait on background work
by its completion notice or an output file, never `until ! pgrep -f ...`.

## Work

The decks: `veyran`, `sauron`, `galadriel`, `alela`, `kefka`, `karn`, `thrasios`, `brago-azorius-blink-control`,
`aurelia-boros-extra-combats`, `sisay-legends-toolbox`, `dack-fayden-white-stax`. Keep the hand AI of `yshtola`,
`sheoldred`, `cloud` (`ai::HAND_KEEP`) and of every deck outside `HAND_DECKS` untouched.

1. For each of the 11, delete its hand-written decision code (what `hand_ai_key` gates: priorities, holds, keeps,
   mulligans, targeting and timing hooks, deck modules such as `ai/veyran.rs`, `impls/t6_plan_dack.rs`,
   `impls/t6_plan_sisay.rs`, the AI parts of `impls/kefka.rs`, `impls/karn*.rs`, `impls/alela.rs`, ...). **Keep every
   card's rules behaviour** (what a card does when it resolves, its triggers, costs, combo traces such as the Dack
   flip tables in `impls/t6_dack.rs`): only the deciding goes. Where a file mixes the two, split carefully; if a piece
   is both (e.g. a card's own choice made by its controller), keep it and say so.
2. Remove the 11 from `HAND_DECKS`; `HAND_OFF_PLANS` / `plan_unread` and anything else that only existed for the
   switch become what the generic path needs. The `AI_HAND` switch stays for the three kept decks.
3. **Behaviour must not change with the defaults:** after the deletion, every one of the 11 plays exactly as it does
   on origin/main with `AI_HAND` unset (the generic policy and its plan file). Don't change plan files, values,
   card code behaviour or any other deck. The coordinator will check this byte for byte with sims afterwards; list
   anything you think could still move.
4. Tests: the tests that force the hand AI on for these decks (`force_hand(Some(true))`, `AI_HAND=...`) test deleted
   code: delete those tests, or where a test also checks card rules, keep that part with the hand AI off. Say for each
   test file what was deleted and why. Keep the switch tests for the three kept decks.
5. Update `audit/phase4/hand-off-design.md` (a dated "Deleted" section: which decks, which files) and the doc comments.

## Checks

- `cargo test --release --no-fail-fast -j 6` passes; `cargo build --release --all-targets -j 6` 0 warnings (no dead
  code left behind: a `#[allow(dead_code)]` is not a fix); `cardgen export` leaves no diff; `cardgen validate` passes.

## Rules

- Stay out of `ai/search.rs`, `ai/ismcts.rs`, `ai/learned.rs`, `ai/rules.rs`, `data/plans/`, `data/eval/`,
  `audit/recal/`, deck lists. No sims.
- Format only new files (`rustfmt --edition 2024 <file>`), lines up to 120. Commit per deck or group of decks, no
  Co-Authored-By line, never push. Scratch in
  `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/hand-ai-delete/`.
- Report in 15 lines or fewer, then the full report as text (the harness refuses report files from subagents): files
  and lines deleted per deck, what was kept and why, tests deleted per file, check results, what could still move.
