# Handoff: Phase 4 PR 4, a learned evaluation

Brief: `rust/agents/phase4-eval.md`. Design: `audit/phase4/eval-design.md`. Branch `phase4-eval` (on
`phase4-values`). Scratch: `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase4-eval/`
(`sim.sh BIN POOL GAMES OUT AI_EVAL|- [SEED] [K=V..]`: one capped sim, `OUT.rc` has rc, seconds, games/s;
`half.sh OLD NEW "Name;Name"`; `bin/commander-sim-base` = the base, built at 0a5f9007 before any change).

## State

- [x] Design.
- [ ] Recorder (`AI_EVAL_RECORD`), identity check t1 + t5.
- [ ] Recording t1-t5.
- [ ] Training (`examples/eval_train.rs`), `data/eval/model.toml`.
- [ ] Stage A (t1, t2, t4, t5, 300 games, AI_EVAL unset = base).
- [ ] Stage B.
- [ ] Stage C, hold-mana check, final checks.

## Gotchas

- `cargo build -p sim-cli --example X` builds only the example: build `-p sim-cli` alone for the binary and `cmp` it.
