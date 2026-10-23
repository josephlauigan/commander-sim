# Handoff: realism scorecard (plan Phase 0 done, Phase 1 next)

Branch `claude/realism-phase0` (not merged, no PR yet). Plan document:
https://claude.ai/artifact/Fa1usjF6hHZiSUBXwLzGNJ ("Commander Sim: Improvement and Refactoring Plan"; sections
"Ground truth and validation", "Real game data online", "Logging JD's pod games", "Roadmap").

## Done (Phase 0, ground truth)

- `data/realism/<tier>.toml`: one table per scorecard statistic (value, lo, hi, n, source, date, optional note).
  t6 is filled; t0-t5 have only `tier = "tN"` plus comments naming their future sources.
- `commander-sim --realism --pool tN`: plays the tier's own decks (the `--calibrate within` games, shared through
  `Ctx::tier_games` in `poolmode.rs`), prints sim value, 95% CI, target, band, PASS/FAIL. Code:
  `rust/crates/sim-cli/src/realism.rs`; tests there and in `rust/crates/sim-cli/tests/realism.rs`.
- `data/realism/pod_games.toml`: template for logging JD's pod games (format in its header).
- Baseline: `audit/realism/t6-baseline.md` (2,200 look-ahead games): 3 of 10 pass. Seat 1 27.5% vs 32.5, seat 4
  22.7% vs 17.9, median win round 9 vs 7, combo/alt wins 76% vs 47.8, commander damage 0.6% vs 11.6, deck spread
  16.3 vs 0-8 (Kraum/Tymna 32.8%, Rograkh/Thrasios 16.5%).

## JD's decisions (2026-10-23)

1. Seat targets are EDHTop16's monthly-report shares (Apr-Sep 2026): 32.5 / 26.9 / 22.7 / 17.9.
2. Every share target is held to its real value +/- 1 point (already in t6.toml). A run that checks +/- 1 needs about
   8,200 Tier 6 games (look-ahead: about 5 hours on 4 cores).
3. Win types stay scored, although their source is casual Playgroup logs.
4. Fitting may overrule any hand-set value or threshold (including the loose profile's).
5. Each deck may get its own fitted settings (to close the deck spread).

## Next (Phase 1: fit the AI's knobs to the scorecard)

1. Find the free parameters: per-deck `temp`, `aggression`, `caution`, counter thresholds, removal eagerness,
   evaluation weights (sim-core `ai/`, `settings.rs`). Expose them as data a fit can set, without changing behaviour
   at their current values (a paired run must show 0 difference).
2. Add a fit command (`commander-sim --fit --pool t6`): a derivative-free search (CMA-ES or similar, in Rust) on the
   adaptive AI, same seeds for every candidate, loss = distance to each target weighted by confidence.
3. Confirm the winner with the look-ahead AI at about 8,200 games; write `audit/realism/t6-fit-<date>.md`.

## Gotchas

- Cloud containers here have no systemd: `systemd-run` fails. The cap was applied with a cgroup v1 memory group
  (12 GB, swap 0), giving the same exit 137 on a breach. On JD's machine use the CLAUDE.md command.
- Data sources and terms: Playgroup's `api/public/v1/commanders.json` (CC BY 4.0, credit it), EDHTop16's
  `/reports/monthly_seat_winrates` (public page; its GraphQL API takes persisted queries only), TopDeck.gg's API
  (free key, visible credit required). Details and citations are in the plan document.
