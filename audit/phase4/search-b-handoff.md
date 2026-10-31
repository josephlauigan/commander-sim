# Handoff: Phase 4 PR 5b, blocks, protection and wipe responses as searched decisions

Brief: `rust/agents/phase4-search-b.md`. Design: `audit/phase4/search-design.md`, "Part (b)". Branch `phase4-search-b`
(on `phase4-search`, PR 5). Scratch: `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase4-search-b/`
(`sim.sh BIN POOL GAMES OUT AI_SEARCH_B|- [SEED] [K=V..]`, 12 jobs, unsets every `AI_*` switch first; `half.sh`,
`pair.sh` as PR 5's; `bin/commander-sim-base` = PR 5's head (76160538, built from this branch before any change);
`bin/commander-sim-m` = the survey counters only; `bin/commander-sim-b1` = the searches behind `AI_SEARCH_B`).

## State

- [ ] Survey (`survey.sh`, `M/`): base vs m, 300 games t1-t5.
- [ ] Design (Part (b) in the design doc).
- [ ] Code: `ismcts.rs` part (b) (switch `AI_SEARCH_B`, `choose_blocks` / `choose_protect` / `choose_wipe`,
  `block_plans`, survey `note*`), `search::playout_with` / `play_on_resume`, splits `combat::resolve_blocks`,
  `removal::apply_removal_rest`, `removal::wipe_victim` / `wipe_finish` / `WIPE_CTX`, `pool::protect_cands` etc.
  Tests `tests/ismcts_b.rs` (7).
- [ ] The lean: Meren diagnostic (`meren.sh`, `X/meren-*.txt`), ablations on t2 half B.
- [ ] Stage A (`stageA.sh`, `A/`).
- [ ] Stage B, Stage C, final checks.

## Gotchas

- `cargo build -p sim-cli --example X` / `--test X` builds only that target: build `-p sim-cli` alone and `cmp`.
- `pgrep -f` inside an `until` loop matches the loop itself: wait on output text.
- `testkit::token(g, p, 0)` makes no token: make a 1/1 and set `pow`.
