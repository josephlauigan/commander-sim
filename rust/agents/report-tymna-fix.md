# Report: tymna-fix (2026-10-13)

## Causes and fixes
1. **Never cast.** Lagrella: its tags lack `etb`, so `generic_prio` fell to the `rem` branch (value 0). Fix: `partner_prio` values it by the best creature of each opponent (45-75, else 35). Witherbloom Charm: no tags and no code (the override note was stale). Fix: modeled in `tymna_cards.rs` (3 modes, hand options like Silverquill Charm). Shardmage's Rescue: nothing routed thrasios/kraum to `white_protect` (they are not `is_main`, so `protect_main` went to `pool::protect`). Fix in `ai/decks.rs`.
2. **Ultimate Magic: Holy / Rescue:** `protect_main` and `wipe_response` now call `bulk_white::white_protect` / `white_wipe_response` first for keys thrasios and kraum.
3. **Commanders late:** the look-ahead search (its score does not value commanders) put ramp/creatures ahead of the commander cast. Fix: `tymna_partners::commander_first`, called at top of `search::choose_inner`: when the heuristic's best play is a commander cast from the command zone, it is made without search. Measured (200 games, t3): Tymna castable T3.7 and landed 4.9-5.3 before, 3.85-3.9 after; Thrasios castable T3.0, landed 4.3-4.5 before, 3.7 after; Kraum castable T5.9, landed 6.8-7.1 before, 6.1-6.2 after. What remains is mana/colours (Kraum costs 5; Thrasios needs G+U).
4. **Thrasios ~0.5 activations/game:** mostly mana, not AI: he is on the battlefield about 5 own turns a game and 4 spare mana (after plays) exists in only about 0.74 games' turns at main 2 and 0.16 at the end-of-turn window. Fixes: when his activation is the best play it is made without search (search chose "stop"); main-2 activation keeps mana for the cheapest held counterspell. Activations stay 0.45-0.48 per game (before 0.49-0.53): the limit is spare mana. New `ty_th_*` stats (turns, turns with 4 spare) print in the analysis.

## Before / after (200 games, seed 1013, loose, lookahead; win rate | Tymna cards/game | Tymna lands | commander lands | Thrasios acts)
- thrasios t1: 8.5% -> 8.5% | 2.17 -> 2.33 | 5.28 -> 3.85 | 4.33 -> 3.66 | 0.53 -> 0.48
- thrasios t3: 8.5% -> 9.0% | 1.76 -> 2.33 | 4.92 -> 3.84 | 4.51 -> 3.71 | 0.49 -> 0.45
- kraum t1: 16.0% -> 12.0% | 1.83 -> 2.50 | 5.43 -> 3.88 | 6.82 -> 6.21
- kraum t3: 18.0% -> 12.5% | 1.83 -> 1.93 | 5.30 -> 3.90 | 7.09 -> 6.12
Kraum's win rate fell 4-5 points (about 1.5 sigma per cell at 200 games). A 600-game t3 base-vs-new check (seed 2026) was started but had not finished when this report was written; the lead should check kraum before merging. The "never cast by the sim" list is empty in all four new outputs.

## Checks
- `cargo test --release`: 1589 passed, 0 failed (4 new in `tests/tymna_fix.rs`). `cargo build --release --all-targets`: no warnings. `all_decks adaptive 3`: 216 games, no panic. `all_decks lookahead 1`: exit 0 (output not captured; rerun to confirm).
- Same-seed check (seed 7, 48 games, loose, lookahead): `sauron --pool t3` and `cloud --pool t5` outputs identical before and after (apart from timing line).

## Shared-file changes
`ai/decks.rs` (two additive blocks for thrasios/kraum), `ai/search.rs` (3 lines, no-op for other keys), `sim-cli/src/report.rs` (prints `ty_th_*`).
