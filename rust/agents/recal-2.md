# Brief: recalibration, stage 2: evaluation refit, style fit, pool check, JD's decks remeasured

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/overnight-stack` on
branch `overnight-stack`: origin/main (Phase 4, card models 2 and 3, the 10/31 deck swaps for cloud, yshtola and
ghave) + #191 (switch defaults: `AI_SEARCH_B`, `AI_SEARCH_OPP` and `AI_RULES` on) + #193 (Dack flip fix; hand AI on
only for yshtola, sheoldred and cloud). Run every command from there. Read `CLAUDE.md`, `rust/agents/RULES.md`, `documents/realism.md`,
`audit/phase4/eval-handoff.md` and `audit/recal/pairs-2026-10-31.md` first: memory cap, no Python, loose profile,
opponent tiers.

**You are the only sim agent:** `--jobs 22`, one sim at a time, each under
`systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0`, in the background; check rc=0 on every run you
cite. Wait on a task's completion notice or an output file, never `until ! pgrep -f ...`; one queue script per batch,
written so a rerun resumes (skip arms already done with rc=0); don't leave wait loops behind. Every arm of one
comparison uses the same binary and `--jobs`. Base = this branch's defaults (no env).

## Work, in this order (commit results and the handoff after each step)

1. **Evaluation refit.** Record t1-t5 as `eval-handoff.md` says (5000 games a tier, seeds 1,000,000 + 100,000 per
   tier, recorder on) with this branch's binary and defaults; train with `examples/eval_train DATA OUT.toml COMMIT
   --gbt`; report held-out log loss / Brier / AUC against the current `data/eval/model.toml` scored on the same new
   held-out games. The model is embedded at build time: build a second binary with the new model and run the
   half-tier pairs (method in `audit/phase4/values-handoff.md`; 2000 games, seeds 500000-501999, t1-t5, both halves)
   old model vs new. Commit the new model only if the held-out metrics improve and every half is level or better;
   else keep the old one and say why. Delete the raw recordings after training (keep their hash in the model file).
2. **Style fit per tier, t0-t6** (JD, 2026-11-01: the fast AI, then a check). With the model from step 1:
   `--fit --pool tN --ai adaptive --games 2000 --fit-gens 15` (shared scope, every target), output under
   `audit/realism/fit/` with today's date. Then the check for each tier: the look-ahead AI (the default), 2000 games
   on fresh seeds 900000-901999, `--realism --pool tN` with the built-in knobs against `--knobs <fitted file>`;
   report each scorecard (pass count and the loss as `--fit` computes it) both ways. A tier's fitted knobs "hold"
   when the look-ahead loss is lower with them. For a tier where they don't hold: refit with the look-ahead AI,
   `--games 500 --fit-gens 8`, and check that the same way. Don't change the built-in values; list each tier's
   recommended knobs file (or "built-in").
3. **Pool check:** `--calibrate all` with this branch's defaults and, for each tier whose knobs held, its knobs file
   (as the CLI allows; say how it was run). Report every flagged deck.
4. **JD's decks remeasured:** every deck in `data/decks.json` `mine`, on each of its own tiers only, 2000 games,
   seeds 500000-501999, defaults (and the tier's knobs file if it held). Report each deck's win rate per tier and its
   average over its tiers, next to the last numbers in its `audit/<deck>/FINDINGS.md` (say which build those were; cloud, yshtola and ghave changed lists on 10/31, so their old numbers are
   from older lists).
   Add one dated line to each deck's FINDINGS.md with the new number and the build.

## Rules

- No deck list changes; no swaps; don't change switch defaults or built-in knob values (the coordinator decides
  from your tables); no Rust changes except the refitted model file (and a bug fix only if a run crashes; say so).
- Results in `audit/recal/stage2-2026-11-01.md`, handoff `audit/recal/handoff.md` (add a stage 2 part, keep it current
  so another agent can continue), commit often, no Co-Authored-By line, never push. Scratch in
  `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/recal2/`.
- Post short progress in the handoff after each step, with the time each step took.
- Report in 15 lines or fewer, then the full report as text (the harness refuses report files from subagents): each
  step's numbers and the recommendations (model kept or not, knobs per tier, flagged pool decks, JD decks' new
  numbers).
