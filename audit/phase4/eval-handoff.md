# Handoff: Phase 4 PR 4, a learned evaluation

Brief: `rust/agents/phase4-eval.md`. Design: `audit/phase4/eval-design.md`. Branch `phase4-eval` (on
`phase4-values`). Scratch: `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase4-eval/`
(`sim.sh BIN POOL GAMES OUT AI_EVAL|- [SEED] [K=V..]`: one capped sim, `OUT.rc` has rc, seconds, games/s;
`half.sh OLD NEW "Name;Name"`; `bin/commander-sim-base` = the base, built at 0a5f9007 before any change).

## State

- [x] Design.
- [x] Code: `ai/learned.rs` (features, model, switch `AI_EVAL`, recorder), hook in `search::evaluate` (`raw_term`), `run_rounds`, `Engine::play`; trainer `examples/eval_train.rs`; tests `tests/learned.rs` (need the fitted model).
- [x] Identity (binary `bin/commander-sim-headA`, e40505bf): AI_EVAL unset = base byte for byte on t1, t2, t4, t5 and recorder on = off on t1, t5 (300 games, all rc=0; `stageA-head.sh`, outputs `A/`). Recorder: 2.5 MB per 300 games.
- [x] Recording t1-t5: `record.sh` (5000 games a tier, seeds 1,000,000 + 100,000 per tier; data `rec/data/tN/`, 220 MB; all rc=0; 24,992 games, 830,119 positions).
- [x] Training (`train2.txt`): held-out (4,990 games) log loss / Brier / AUC: baseline 0.4963 / 0.1608 / 0.7168, logistic 0.4781 / 0.1547 / 0.7424, logistic + 600 trees 0.4617 / 0.1492 / 0.7675 (kept). Model committed.
- [x] Stage A: see Identity (and binary B unset = base on t1, `A/B-unset-t1.txt`).
- [ ] Stage B: `chainB.sh` -> `stageB.sh` (binary `bin/commander-sim-B`, outputs `B/tN-{old,A,B}.txt`, resumable).
- [ ] Stage C, hold-mana check, final checks.

## Gotchas

- `cargo build -p sim-cli --example X` builds only the example: build `-p sim-cli` alone for the binary and `cmp` it.
- The trainer: `target/release/examples/eval_train DATA_DIR OUT.toml COMMIT --gbt` (DATA_DIR has `t1`..`t5`). A trial on
  the 600 check games (`rec/checkdata`): raw features lost to the baseline (heavy tails); log-scale features fixed it
  (0.490 vs 0.500).
- `pgrep -f X` inside an `until` loop matches the loop's own command line: wait on output text instead.
