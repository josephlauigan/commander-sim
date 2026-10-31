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
