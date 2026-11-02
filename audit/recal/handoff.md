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
- [x] 3. Pool check (02:37 crash; fix 3b63138f; reruns 02:44-09:52 MST): results section 3, outputs `audit/recal/stage2-calibrate-base.txt` and `stage2-calibrate-knobs/`.
- [x] 4. JD's decks remeasured (done 11/01, see "Stage 2 DONE" below; was: ON HOLD (coordinator, 10/31): handed back after step 3; the coordinator merges origin/main (urdragon #194, kefka, sauron, thrasios #195) and sends step 4 with a rebuilt binary. Knobs files to use per tier (if held): results section 2, "Recommended knobs per tier". Galadriel has no `audit/galadriel/FINDINGS.md` yet (decks.json tiers t1, t2; CLAUDE.md says t1).)

## Stage 2, steps 4-5 (coordinator, 11/01: on 7221712e = main #194 #195 + generic-paths)

- Binary `recal2/bin/commander-sim-s4` (7221712e). `q7.sh`: every JD deck on its own tiers, 2000 games, seeds
  500000-501999, defaults (`d-KEY-tN`); then the fitted knobs on t3 / t5 / t6 only (`k-KEY-tN`: t3-shared,
  t5-shared-lookahead, t6-shared); then step 5, `AI_HAND=0` for yshtola t5, sheoldred t3-t4, cloud t1-t3 (`h0-KEY-tN`;
  base = the `d-` run). Read with `dp.sh`. Thrasios t5 seed 500193 panics on main too (zones.rs ~1917): count it as the
  run reports, don't fix. Galadriel t1-t2: create its FINDINGS.md.
- PAUSED (coordinator, 11/01 13:10, JD's lists all about to change): q7 stopped after the arm running finished; no sim or wait loop left. Done (rc=0, pre-change lists, results section 4 "partial"): d-alela-t1, d-cloud-t1/t2/t3, d-galadriel-t1/t2, d-ghave-t3/t4, d-karn-t4/t5, d-kefka-t3, d-sauron-t3/t4 (13 of 22 default arms). Not run: d- for sheoldred t3/t4, thanos t6, thrasios t5, urdragon t5, veyran t1/t2/t3, yshtola t5; all 11 k- arms; all 6 h0- arms. To resume on the new lists: move every `runs/d-*` aside (they are stale; `q7.sh` skips arms already done with rc=0), rebuild `bin/commander-sim-s4` from the new head, rerun `q7.sh` (log q7.log). No FINDINGS.md updated yet; galadriel's still to create.
- RESUMED (coordinator, 11/01): head efe45425 (lists final). Stale arms moved to `runs/stale-7221712e/`; `bin/commander-sim-s4` rebuilt from efe45425; q7 rerun in full (log q7.log, the old log q7-7221712e.log).
- **Stage 2 DONE (11/01 20:17 MST q7 finished; written up the same evening).** q7 on `bin/commander-sim-s4` rebuilt from efe45425:
  39 runs, all rc=0 (22 defaults, 11 knobs t3/t5/t6, 6 `AI_HAND=0`), 16:13-20:17. Results: `stage2-2026-11-01.md` section 4
  (4a defaults per tier + average vs stage 1, 4b knobs, 4c step 5, 4d last FINDINGS numbers, veyran footnote only).
  Step 5: yshtola t5, sheoldred t3/t4, cloud t1-t3 are all level (hand AI on minus generic: +0.5, -0.8 avg, +1.0 avg).
  One dated line added to each `audit/<deck>/FINDINGS.md` (build efe45425); `audit/galadriel/FINDINGS.md` created.
  Veyran (retired, #201) skipped. Oddities for the coordinator: overflow panics in t5 only (1-2 games a run, different
  seeds, skipped by the run); thanos t6 29.6% (27.6-31.6) equals the pre-fix FINDINGS number digit for digit, stage 1
  had 25.6%; sheoldred t3 with the knobs +2.8 [+0.5, +5.1] is the one knob arm outside the noise. Scratch helper:
  `recal2/s4tab.sh` (defaults table). No sim is running.
