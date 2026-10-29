# Handoff: AI foundations step 3, Tier 6 card and mana flow (brief `ai-t6-flow.md`)

Branch `ai-foundations`, worktree `.claude/worktrees/ai-foundations`. Report: `audit/ai-foundations/step3-flow.md`;
raw output: `audit/ai-foundations/raw/step3/`.

## State (2026-10-28)

- **Part 1:** the flow counters (stats only, `// --- ai-flow`):
  - `t6stats.rs` flow section and `Player::flow_src`, set in `stack.rs` and `cast.rs::resolve`, counted in
    `zones.rs`, `mana.rs`, `turn.rs`, `cast.rs`, `rules.rs`, `common.rs`, `tutors.rs`, `decks.rs`, `topdeck.rs` and
    `legends.rs`;
  - `sim-cli/src/flow.rs`: the report, printed after `--realism --pool t6` and in each goldfish deck report.

  The hand reference and the gaps are in the report.
- **Part 2:** the card models are right (Oracle checked). The gap is the AI: the look-ahead votes "stop" over
  engines, commanders and mana permanents the heuristic rates 95-100%.
- **Part 3 commits:**
  - eefe4e34 (A): `t6_table::engine_first`: engines and commanders before the look-ahead, Tier 6;
  - 40bc2a8e (B): mana permanents too;
  - then the goldfish record sort fix.

  Fix C (wheels by net cards) changed nothing and was reverted: `raw/step3/C-wheels-reverted/`.
- **Result:** seats and spread unchanged within noise (29.7/26.3/21.9/22.1, spread 11.6); median round 8 → 7. What
  is left isn't flow (see the report).
- **The full set** (t0-t5 realism, all-decks) runs base against final: scratchpad `fullset.sh`, into
  `raw/step3/full-base/` and `full-final/`. "base" is the instrumented binary (step 2 play); "final" is HEAD.

## Next

- Read the full set, fill in the report's "full set" section, and commit with the raw output.
- `cargo test --release --no-fail-fast` passed 3548 with 1 failure, the goldfish record order, since fixed; rerun it
  at the end. `cargo build --release --all-targets` had no warnings.

## Gotchas

- Perl one-liners with `|` delimiters and `\|` in the pattern silently misbehave; sed with `|` delimiters breaks on
  `fl|` keys. Use the Edit tool.
- A new run overwrites NAME.txt: give each run its own name. Count "done" lines in progress.log, don't grep for one.
- One sim or cargo command at a time, `--jobs 16` / `-j 16`; Tier 6 4000 games take about 12-15 min.
- The scratchpad (binaries, run scripts) is lost when the session restarts: rebuild from the commits.
