# Brief: fix the look-ahead's evaluation at the cause (value what permanents will produce)

Work in the worktree `.claude/worktrees/ai-foundations` (branch `ai-foundations`, at e55b0af6 or later). It is your
whole world. Read `CLAUDE.md` first. Don't use `rust/agents/RULES.md`.

## Why

Step 3 (`audit/ai-foundations/step3-flow.md`) found that Tier 6 decks rarely cast their draw engines and engine
commanders: Tymna was out by turn 4 in 38% of games, and 25-65% of seen Rhystic Studies were still uncast at turn 4.
The look-ahead chose "stop" over the heuristic's 95-100% choice. Step 3 then added two workarounds (commits eefe4e34
and 40bc2a8e, "A" and "B") that cast engines, commanders and mana permanents *before* the look-ahead, for Tier 6
only.

JD (2026-10-28): those are band-aids. Fix the look-ahead itself, for every deck, then remove A and B.

The likely causes come from `architecture-review.md` and haven't been proven:
1. `search::strength` scores a card in hand at 0.9, while a permanent's future output is close to 0. Most engines
   and mana rocks have no `pval`, so casting one trades 0.9 for about 0.
2. The horizon ends at the deck's next turn, so the compounding turns after it count for nothing.
3. "Stop (hold mana)" gets the heuristic's hold value plus `search::hold_bonus`, and casting the engine taps that
   mana.

## Part 1: prove the cause (stats only, no play change)

Add a settings knob that turns A and B off (for example env `AI_FLOW_MUST=0`), so the raw look-ahead can be measured.
With A and B off:
- Record every main-phase look-ahead decision where the heuristic's top option was a card engine, an engine
  commander, or a mana permanent, and the look-ahead picked something else.
- For each of these, record per option:
  - the rollout average, split into `strength`'s terms for the decider (life, board `pval`, power, hand, lands,
    treasures, combo) and the opponents' term;
  - the hold and ramp bonuses;
  - the share of playouts won or lost.

Run it on Tier 6 (`--pool t6`) and on one lower tier (`--pool t3`), 1,000 games each, and trace 5 decisions by hand.
Report which terms decide these choices, and by how much. If the cause is not 1-3, say what it is.

## Part 2: fix the evaluation at the cause

Make `strength` value the board for what it will produce, in the same unit as a card in hand, for every deck:
- a draw engine at its expected extra cards;
- a mana permanent at its expected mana, converted to card-equivalents through one stated rate;
- an engine commander likewise.

Value each one over a fixed window of turns past the playout (for example the next 2 own turns, discounted). Derive
the per-card output from what the game state already says, such as tags, `rock=` and the draw-trigger hooks, or from a
small table built from the step 3 flow measurements (Rhystic Study about 3 cards per turn cycle, Remora about 2.7,
Necropotence about 4, a rock its mana). Don't add per-card bonuses inside the decision code. If the hold bonus is
part of the cause, fix how it's computed rather than switching it off.

One stated model, small and explainable. Document the unit and the rate in `search.rs`.

## Part 3: remove the workarounds and measure

1. Remove A and B (eefe4e34, 40bc2a8e). Then check whether the older Tier 6 must-do mana rules (`t6_table::must_option`'s
   rock and dork rules, `chrome_mox_now`, and `mana_first`) are now redundant: measure with each one off and remove it
   if nothing is lost. Say what was removed.
2. Compare three states:
   - "base": step 3's state before A and B, which is A and B off with the old evaluation;
   - "A+B": the current state;
   - "eval": the fixed evaluation, with A and B removed.

   Measure each with:
   - `--goldfish t6,scion --games 2000 --seed 500000`, including the step 3 flow and engine funnel;
   - `--realism --pool tN --games 4000` for t0-t6;
   - `--all-decks --pool all --games 1000`.
3. The target: "eval" is at least as good as "A+B" on Tier 6 flow (Tymna and Rhystic cast by turn 4) and on the
   Tier 6 scorecard. It must also be explainable on Tiers 0-5 and on JD's decks. Those tiers are calibrated against
   the old evaluation, so expect movement there: report it, explain the large moves with traces, and don't retune.
   Flag every realism row that flips PASS/FAIL or moves by more than 1.5 points, and every JD deck that moves outside
   its confidence interval.

Write it all up in `audit/ai-foundations/eval-fix.md`, with the raw output in `audit/ai-foundations/raw/eval/`.

## Rules

- **Commands.** Run every build, test and sim from `rust/` with `run_in_background`, under
  `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 ...`. Use a detached
  `systemd-run --user --unit=eval-<name> ...` for anything over about 10 minutes, and poll its output files. Never use
  one blocking command longer than 10 minutes.
- **Threads.** Use `--jobs 16` and `-j 16`, and run one sim or cargo command at a time.
- **No Python, ever**, in any position in a command. Not even `python3 --version`, and not after `cd` or `&&`.
- **Shared files.** Mark each change `// --- ai-eval`. Comments explain why, briefly, in the surrounding style.
- **Formatting.** Format only the files you write (`rustfmt --edition 2024 <file>`), never `cargo fmt`. Lines go up
  to 120 characters.
- **Commits.** Commit at each part. No Co-Authored-By line. Never push. Keep `rust/agents/handoff-ai-eval.md` up to
  date and commit it with your work.
- **Tests.** Add a test that the look-ahead casts Rhystic Study and Sol Ring on an empty-ish board with no counter to
  hold, with A and B removed. Keep `search_hold` and the step 1 and step 2 tests passing, or say exactly which old
  assertion was wrong and why.
- **Before you finish:** `cargo test --release --no-fail-fast` passes, `cargo build --release --all-targets` has no
  warnings, and nothing of yours is still running (`pgrep -af 'commander-sim|cargo'`).
- **Decisions.** JD may be away. Make the conservative call, record it with the alternative in a "Decisions" list, and
  keep going.
- **Report** in your final reply, in 15 lines or fewer: the proven cause with its numbers, the model, what was removed,
  the three-way comparison, and anything JD must decide.
