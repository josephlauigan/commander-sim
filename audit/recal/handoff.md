# Handoff: recalibration stage 1, paired tests of the Phase 4 switches

Brief: `rust/agents/recal-1-pairs.md`. Branch `overnight-stack`. Results: `audit/recal/pairs-2026-10-31.md`.
Scratch: `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/recal/`:

- `bin/commander-sim-stack` (this branch, `bin/stack.rev`), `bin/commander-sim-main` (origin/main from a clean
  `git archive` in `main-src/`, `bin/main.rev`; run from `main-src/` so it reads main's data).
- `lib.sh` (sourced): `arm NAME BIN ROOT "K=V ..."|- ARGS...` = one capped sim at `--jobs 22` into `runs/NAME.txt`
  (`.rc` rc/secs/games-per-s, `.err`, `.cpu`); unsets every `AI_*` first; skips an arm already done with rc=0, so a
  queue script reruns to resume. Halves `A_tN` / `B_tN` (values-handoff).
- Reading: `half.sh OLD NEW "Name;Name"` (a half's summed share), `pair.sh OLD NEW` (each deck), `dp.sh OLD1 NEW1
  [OLD2 NEW2 ..]` (a `--deck` run's win rate per tier and the average).
- Queues: `q1.sh` (identity), then `q2.sh` ... one per step, each logging to `qN.log`, ending "qN all done".

## State

- [x] 1. Identity (300 games t1-t5): origin/main vs stack `AI_HAND=1 AI_RULES=0`: t1, t2, t4, t5 byte for byte; t3 moved by card models 2 commit 13e01e84 (Atraxa: Dreamtide Whale, Norn's Decree), bisected with `pre-src` (a9766cf8) and `mid-src` (a9766cf8 + bad6c75a), both = main. Results file section 1.
- [x] 2. Stage B, t2 lean, t1 lean (results section 2): all level.
- [x] 3. `AI_RULES`: all level (results section 3).
- [~] 4. `AI_HAND`: q4 running (log q4.log).
- [ ] 5. Results table.

# Stage 2: evaluation refit, style fit, pool check, JD's decks remeasured

Brief: `rust/agents/recal-2.md`. Results: `audit/recal/stage2-2026-11-01.md`. Base = `overnight-stack` at 37fd6b90
(origin/main + #191 + #193), defaults, `--jobs 22`, loose. Scratch:
`/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/recal2/`:

- `bin/commander-sim-old` (37fd6b90, the current model), `bin/eval_train`; later `bin/commander-sim-new` (the same
  source with the refitted `data/eval/model.toml`, nothing else).
- `lib.sh` (sourced): `arm NAME BIN "K=V ..."|- ARGS...` = one capped sim at `--jobs 22` into `runs/NAME.txt` (`.rc`,
  `.err`, `.cpu`), run from the worktree, every `AI_*` unset first, skipped when already done with rc=0 (queues rerun
  to resume). `half.sh`, `pair.sh`, `dp.sh`, `names.sh` as in stage 1. `loss.awk`: a `--realism` scorecard's pass
  count and the `--fit` loss.
- `score/`: a scratch crate (path dependency on sim-core, not in the repo): `score/target/release/evalscore DATA
  MODEL.toml...` scores models on the held-out (seed % 5 == 0) recorded positions, weighted as `eval_train` does.
- Queues: `q1.sh` (recording), ... each logging to `qN.log`, ending "qN all done".

## Stage 2 state

- [x] 1. Evaluation refit (16:05-20:00, 3h55): refitted model NOT adopted (coordinator: no measurable gain, fails ismcts_b protection_is_held_for_a_low_value_target); committed 9f0b47f1 and reverted; the file is kept in `recal2/model-new.toml` (old: `model-old.toml`). Binary for steps 2+: `bin/commander-sim-final` = rebuilt from the reverted tree (current model).
- [x] 2. Style fit t0-t6 (7h): held t1 t2 t3 t4 t6 (adaptive files), t5 (look-ahead refit file); t0 built-in (neither held). Results section 2.
- [~] 3. Pool check: `q5.sh` (chained by chain45.sh after q4): `cal-base` = `--calibrate all --games 1500 --seed 500000` (defaults), then `cal-knobs-tN` = `--calibrate all --pool tN --knobs <file>` per held tier. Started 01:56 MST 11/01.
- [ ] 4. JD's decks remeasured: ON HOLD (coordinator, 10/31 18:30): hand back after step 3; the coordinator merges origin/main (urdragon #194, kefka, sauron, thrasios #195 list changes) first and sends step 4 with a rebuilt binary. Don't merge main.
