# Handoff: Phase 4 PR 5b, blocks, protection and wipe responses as searched decisions

Brief: `rust/agents/phase4-search-b.md`. Design: `audit/phase4/search-design.md`, "Part (b)". Branch `phase4-search-b`
(on `phase4-search`, PR 5). Scratch: `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase4-search-b/`
(`sim.sh BIN POOL GAMES OUT AI_SEARCH_B|- [SEED] [K=V..]`, 12 jobs, unsets every `AI_*` switch first; `half.sh`,
`pair.sh` as PR 5's; `bin/commander-sim-base` = PR 5's head (76160538, built from this branch before any change);
`bin/commander-sim-m` = the survey counters only; `bin/commander-sim-b1` = the searches behind `AI_SEARCH_B`).

## State

- [x] Survey (`survey.sh`, `M/`): m = base byte for byte on t1-t5 (300 games, all rc=0); numbers in the design doc.
- [x] Design (2f0d8ba4).
- [x] Code (a0a547ff): `ismcts.rs` part (b) (switch `AI_SEARCH_B`, `choose_blocks` / `choose_protect` / `choose_wipe`,
  `block_plans`, survey `note*`), `search::playout_with` / `play_on_resume`, splits `combat::resolve_blocks`,
  `removal::apply_removal_rest`, `removal::wipe_victim` / `wipe_finish` / `WIPE_CTX`, `pool::protect_cands` etc.
  Tests `tests/ismcts_b.rs` (7).
- [ ] The lean: Meren diagnostic (`meren.sh`, `X/meren-*.txt`), ablations on t2 half B.
- [x] Stage A (`stageA.sh`, `A/`, binary b1 = a0a547ff): `AI_SEARCH_B` unset = base byte for byte on t1, t2, t4, t5 (300
  games, rc=0); `AI_SEARCH_B=1` twice on t1 = the same stdout and stderr (play changes against base). Full table with
  every seat on part (b), t1: 180,885 playout steps a game vs 174,366 (+3.7%); blocks searched 2.1 a game (13.2 left
  to the heuristic), protection 0.1, wipes 0.1.
- PR 5's arms (binary D) can't be reused: its worktree is gone and the data changed since (the origin/main merge),
  so D run here fails on a moved decklist. Every reference arm is rerun with b1.
- Jobs: 12 until 2026-10-30 ~22:40, then 8 (coordinator: three agents share the machine; `sim.sh` defaults to 8, cargo
  `-j 8`). Done at 12: survey `M/`, Stage A `A/`, `X/t2-old`, `X/t2-B-ref`. The ablation arms after them run at 8, and
  `abl3.sh` reruns t2-old and t2-B-ref at 8 (`X/*-j8.txt`), so every ablation pair is at one `--jobs` (and checks
  that the output doesn't depend on it). Every Stage B arm runs at 8.
- PR 5 (#179) is merged into main: this branch targets main. Not merged into this branch (main's later data would
  change the base the arms are paired against).
- [ ] Stage B, Stage C, final checks.

## Gotchas

- `cargo build -p sim-cli --example X` / `--test X` builds only that target: build `-p sim-cli` alone and `cmp`.
- `pgrep -f` inside an `until` loop matches the loop itself: wait on output text.
- `testkit::token(g, p, 0)` makes no token: make a 1/1 and set `pow`.
