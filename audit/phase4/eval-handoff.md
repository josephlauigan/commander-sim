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
- [x] Stage B: `chainB.sh` -> `stageB.sh` (binary `bin/commander-sim-B`, acb20670, the fitted model; outputs `B/tN-{old,A,B}.txt`; all 15 arms rc=0).
- [x] Stage C (4747f616): learned is the default for every generic-policy deck. Hold-mana check and final checks below.

## Gotchas

- `cargo build -p sim-cli --example X` builds only the example: build `-p sim-cli` alone for the binary and `cmp` it.
- The trainer: `target/release/examples/eval_train DATA_DIR OUT.toml COMMIT --gbt` (DATA_DIR has `t1`..`t5`). A trial on
  the 600 check games (`rec/checkdata`): raw features lost to the baseline (heavy tails); log-scale features fixed it
  (0.490 vs 0.500).
- `pgrep -f X` inside an `until` loop matches the loop's own command line: wait on output text instead.

## Stage B results (2000 games per arm, seeds 500000-501999, loose, look-ahead; all 15 arms rc=0)

The half's summed share of the decisive games its decks sat in, `AI_EVAL=0` against `AI_EVAL=<half>` (one shared old
arm per tier); Wald 95% CIs, the decks' variances summed. Level = the CI includes 0. Speed: games per second (22 jobs).

| tier | half | old (95% CI) | learned (95% CI) | learned - old (95% CI) | g/s old / learned | verdict |
|---|---|---|---|---|---|---|
| t1 | A (Isshin, Light-Paws, Teysa) | 90.1% [86.3, 93.9] | 90.6% [86.8, 94.4] | +0.5 [-4.8, +5.8] | 3.26 / 3.23 | level |
| t1 | B (Lathril, Tatyova) | 35.0% [32.4, 37.6] | 36.2% [33.5, 38.9] | +1.2 [-2.6, +5.0] | 3.26 / 3.79 | level |
| t2 | A (Brago, Lord Windgrace, Sythis) | 81.0% [77.3, 84.7] | 84.3% [80.6, 88.0] | +3.3 [-1.9, +8.5] | 1.79 / 1.98 | level |
| t2 | B (Kaalia, Meren) | 44.2% [41.3, 47.1] | 43.6% [40.7, 46.5] | -0.6 [-4.7, +3.5] | 1.79 / 1.86 | level |
| t3 | A (Atraxa, Korvold, Tergrid) | 78.3% [74.6, 82.0] | 77.7% [74.1, 81.3] | -0.6 [-5.8, +4.6] | 7.83 / 7.97 | level |
| t3 | B (Aurelia, Marwyn) | 46.7% [43.8, 49.6] | 48.8% [45.8, 51.8] | +2.1 [-2.1, +6.3] | 7.83 / 7.18 | level |
| t4 | A (Chulane, Krenko, Yuriko) | 63.4% [60.0, 66.8] | 65.5% [62.0, 69.0] | +2.1 [-2.8, +7.0] | 4.48 / 4.44 | level |
| t4 | B (Edgar Markov, Prosper) | 62.0% [59.0, 65.0] | 63.8% [60.8, 66.8] | +1.8 [-2.5, +6.1] | 4.48 / 4.14 | level |
| t5 | A (Kinnan, Urza, Zur) | 75.7% [72.1, 79.3] | 79.4% [75.7, 83.1] | +3.7 [-1.5, +8.9] | 6.54 / 6.22 | level |
| t5 | B (Miirym, Yawgmoth) | 49.2% [46.2, 52.2] | 48.4% [45.5, 51.3] | -0.8 [-5.0, +3.4] | 6.54 / 6.33 | level |

Deck level (`pair.sh`): one learned deck's own CI excludes 0, upwards: Zur (t5 A) +3.4 [+0.6, +6.2]; its old-arm
tablemate Miirym fell -3.7 [-6.8, -0.6] in that pairing (the shares sum to a constant). No learned deck's CI is below
0; Kaalia leans down when its tablemates learn (-2.6 [-5.6, +0.4], t2 A, Kaalia on the old arm). Speed: the slowest
learned arm is 8% under its old arm (t3 B 7.18 / 7.83, t4 B 7.6%); the others are within 5% or faster (the learned
evaluation reads per seat about twice what `strength` reads, and 600 trees are 1,800 comparisons: small next to the
playouts).

## Stage C

All five tiers level in both halves: the learned evaluation is the default for every generic-policy deck (4747f616).
`AI_EVAL=0` keeps today's evaluation (a measurement arm); JD's decks and Tier 6 always keep it (they move in Phase 4's
last step).

Holding mana for nothing (`AI_EVAL_DIAG=1`, t3, 1000 games, `hold.awk`; decisions whose heuristic top is an engine,
commander or mana permanent with a hold card, i.e. the stop line gets a hold bonus):

| arm | step | decisions | stop picked | hold bonus | top - stop before the bonus | stop kept the mana up |
|---|---|---|---|---|---|---|
| old | main 1 | 205 | 2.9% | 2.83 | +4.33 | 94.4% |
| learned | main 1 | 206 | 3.4% | 2.99 | +4.76 | 95.9% |
| old | main 2 | 31 | 6.5% | 2.35 | +16.31 | 100% |
| learned | main 2 | 35 | 11.4% | 2.41 | +11.55 | 100% |

The case barely happens on t3 since the ai-eval margin rule (the look-ahead overrules the heuristic only beyond two
standard errors): stop is picked in 3% of main-1 hold decisions either way, and when it is, the mana really stays up.
The learned evaluation doesn't change it measurably (main 2: 2 of 31 against 4 of 35 decisions). The hold bonus keeps
its meaning because the mapping puts the learned value on today's scale (same odds, same number).

## Final checks (head)

- `cargo build --release --all-targets -j 22`: 0 warnings. `cargo test --release --no-fail-fast -j 22`: 3692 passed,
  0 failed. `cardgen validate`: all decks passed. `all_decks adaptive 3` and `lookahead 1`: rc=0 (lookahead 21 games
  in 70.4 s; PR 3: 77.2 s).
- Defaults (no env, 300 games, binary at 4747f616) = the learned arm (binary B, `AI_EVAL=1`) byte for byte on t1, t2,
  t3, t4, t5, all rc=0 (`F/head-tN.txt` vs `F/arm-tN.txt`). Later commits change only comments, tests and docs.
- The raw recordings (220 MB) are deleted; the model file keeps their hash (`5c51874b0ca6d03d`). To refit: rerun
  `record.sh` (same seeds, same binary as the recording: `bin/commander-sim-headA`, AI_EVAL unset there = today's
  evaluation), then `eval_train`.

## Next (not in this PR)

- Refit after each big change (the recording is ~2 h of look-ahead games at 5,000 a tier; `record.sh`). The positions
  were recorded with today's evaluation playing: a refit on games played with the learned evaluation is the next
  step (one round of the fit-play loop).
- Richer features (the plan brief's "later"): per-card output, open mana and held interaction, the stack of threats
  each opponent shows.
- JD's decks and Tier 6 move in Phase 4's last step (they keep today's evaluation; `values::generic_deck` gates both).
