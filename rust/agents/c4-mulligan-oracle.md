# Brief: C4 mulligan oracle (measuring tool only)

JD approved this on 2026-11-07. Branch `c4-mulligan-oracle`, cut from origin/main.

## Goal
A **trace-only** CLI report that grades the AI's keep/mulligan rule against an "oracle": for each sampled opening
hand, play many quick goldfish games from KEEP and from MULLIGAN, and see which one kills faster. It must **not change
any game's behaviour**. Plain sims, swaps and goldfish runs must give exactly the same numbers as before.

## Background (read these)
- `rust/crates/sim-core/src/engine/turn.rs`: `keeps_hand` (~L1253), `bottom_pick`, `mulligan` (~L1285). The
  rule order is: Tier 6 `t6_plans::keep`, then the plan file's rule (`ai::plan_data::keeps_hand`), then 2-5 lands.
  `mull_try` is in `stats` for T6 decks.
- `rust/crates/sim-core/src/impls/t6_keep.rs`: the Tier 6 hand-mana scoring.
- Goldfish mode: `rust/crates/sim-core/src/goldfish.rs` (inert seats) and `rust/crates/sim-cli/src/goldfish.rs` (the
  `--goldfish KEYS` CLI and its kill-turn report). Build the oracle on this machinery. Don't write a new game loop.
- `rust/crates/sim-cli/src/args.rs`: flag parsing.

## What to build
`commander-sim --mulligan-oracle KEYS [--hands N] [--playouts M] [--seed S] [--ai adaptive|lookahead]`
(KEYS work like `--goldfish`: JD's deck keys, Tier 6 pool keys, or `t6`). Defaults: N=1000, M=40, adaptive AI.
1. For each of N seeds, deal the deck's opening 7 exactly as `mulligan` does (try 0), and record the rule's decision
   (`keeps_hand`). Also do this for try 1 (the second 7) when the rule mulligans try 0.
2. Oracle for that hand: M goldfish playouts with the hand **fixed** and the rest of the library **reshuffled** per
   playout (the player doesn't know the library order). KEEP = start from this hand. MULLIGAN = put the hand back and
   follow the normal `mulligan` procedure from the next try (rule-driven keeps, bottoming included). Use the same
   playout seeds for both arms (paired).
3. Score: the kill turn, with no kill by turn 10 counted as 11. Report the mean for each arm. The oracle says "mulligan"
   when MULLIGAN's mean is lower than KEEP's by more than 2 standard errors of the paired difference, "keep" when it is
   higher by that margin, and "unclear" otherwise.
4. Report per deck:
   - the agreement table, rule keep/mull × oracle keep/mull/unclear, for try 0 and try 1;
   - the same table split by land count (0-1, 2, 3, 4, 5+) and by the `t6_keep` effective mana for T6 decks;
   - the 5 worst disagreements of each kind, with the hand's card names, both means and the gap;
   - the run time.
5. The playouts must run in parallel across hands, like the other modes do. Print progress lines.

## Rules
- Follow `rust/agents/RULES.md` where it still applies. Ignore its Python, difftest and `rust-port` parts. **No
  Python at all** (a hook refuses it). Base the branch on origin/main.
- Memory cap on every cargo/test/binary command:
  `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 <cmd>`.
- Smallest additive changes to shared files. If you need a hook into `mulligan`, add a separate function rather than
  changing the existing path.
- Tests: a unit/integration test that (a) the mode runs on a small N and M for one JD deck and one T6 deck, and (b) a
  plain goldfish run gives the same output before and after your change (same seed).
- Before finishing: `cargo test --release` passes, and `cargo build --release --all-targets` has no warnings.
- **Do not run big measurement runs.** Only smoke-run with N≤20 and M≤8. The coordinator runs the real ones.
- Commit (no Co-Authored-By line, don't push). Put the full report in `rust/agents/report-c4-mulligan-oracle.md`. Your
  final reply is at most 15 lines: the commit hash, the exact command line for a real run, a run-time estimate from
  your smoke run, and anything left undone.
