# Report t6-ai-table (Stage 5, task `table`)

## What was built (all gated by `impls/t6_plans::is_t6`, the one helper: the 6 pool keys + `godo`)
- **Plan slots** (item 6): `impls/t6_plans.rs` (contract at the top, the only `match` on the key, `T6_KEYS`, `is_t6`) and
  `impls/t6_plan_{dack,kinnan,kraum,rograkh_silas,rograkh_thrasios,sisay,godo}.rs` with `prio`, `wish`, `keep`,
  `hold_up`, `go_now`, all `None` (no opinion). Wired: `prio` in `ai/decks.rs deck_prio_f`; `wish` in `ai/plans.rs
  wish_raw`; `keep` in `engine/turn.rs mulligan`; `hold_up` added to the "stop (hold mana)" utility in `brain::main_options`;
  `go_now` in `combos::combo_options` (`Some(false)` blocks; `Some(true)` forces past the shared check).
- **`ai/t6_table.rs`** (new): 
  1. archenemy: `archenemy_score` 0-10 per opponent (best line's pieces on the battlefield, a `ready` line, open mana with
     cards, tutor / Ad Nauseam odds), `archenemy`, `threat_bonus`. Feeds `Situation` (threat, leader, `combo_near` at 5+),
     attack choice, and instant-removal choice (with a close opponent, removal targets only its pieces; with none, instant
     removal is held for targets worth 4.5+).
  2. answers spent on the win: `counter_verdict` in `stack::ai_respond` replaces the look-ahead/roll for Tier 6 players.
     A step of a line being gone for (`Game.t6_going`, set around `combos::attempt`) and Ad Nauseam (importance 8) are always
     countered when any counter, free ones included, can be cast; the archenemy's spells get a lower bar; a bystander's
     spells are passed while an archenemy is close; with no archenemy a lone counter is held (threshold +1). Every opponent
     is asked in turn order for each window (it already was; the first opponent declining no longer ends it).
  3. hold up vs develop: with an opponent close (`combo_near`) a Tier 6 player holding a mana-cost counter/protection
     takes -8 on any non-ramp spell that would tap it below that mana (`reserve_penalty`).
  4. go when it can't be stopped: `go_ok` = `go_risk` (free counters + paid counters from open blue + instant removal, from
     deck composition and hand size, reduced by own Silence/Chant/free counters) against a bar that falls with the round (0.6,
     0.75 from round 4, always from round 6; 0.75-0.85 when another player is close). The plan slot `go_now` goes first.
  5. Silence / Orim's Chant: `protect_before_go` (from a new `combos::attempt` wrapper) casts Silence (2+ opponents could
     answer) or Chant (one) before the line starts if the line's mana is still left. Angel's Grace: `grace_save` in
     `check_state` (a Tier 6 player about to lose, Grace in hand, {W} open: paid and resolved at once, split second, life to 1);
     `check_state` now honours `t6_stax::grace_active` ("can't lose this turn" - was a model gap, now real). Ad Nauseam is a
     win spell for counters (`pool::spell_importance` = 8 for Tier 6 casters).
- Shared-file edits (additive, all Tier-6-gated): `state.rs` (+`t6_going` field), `engine/stack.rs`, `engine/life.rs`,
  `engine/turn.rs`, `ai/{brain,decks,plans,pool,mod}.rs`, `impls/combos.rs` (`line_progress`, `lines_of` cached per deck,
  `mana_need`, `attempt` wrapper), `impls/mod.rs`.

## Tests
`tests/ai_t6_table.rs` (12): gate, Force of Will counters Oracle + Consultation (12 seeds, three opponents), nobody to answer
-> wins, win step always countered / first spell held, Ad Nauseam importance, archenemy found, counters to the archenemy not
a bystander, Swan Song player keeps mana up (Kinnan + Basalt Monolith on the board), go waits when the table can answer,
Silence cast before the line, Angel's Grace saves / Tier 0-5 does not. Full `cargo test --release` passes, `cargo build
--release --all-targets` has no warnings. **t3 lock identical** (`perf-lock-t3.txt`).

## Tier 6 before / after (`--calibrate within --pool t6 --games 300`, conservative | loose, 300 games per profile; rows in the report's order, the two "Rograkh" rows are as printed, names are the `report` task's)
| Deck | win % before | win % after |
| --- | --- | --- |
| Dack Fayden | 27.6 / 37.4 | 29.6 / 26.6 |
| Kinnan | 26.4 / 23.9 | 27.9 / 32.5 |
| Kraum | 32.3 / 33.3 | 30.3 / 33.3 |
| Rograkh (row 1) | 29.1 / 27.0 | 20.9 / 16.3 |
| Rograkh (row 2) | 11.8 / 9.5 | 10.0 / 10.9 |
| Sisay | 23.7 / 19.7 | 32.3 / 31.3 |
| avg game rounds | 9.4 / 9.3 | 9.8 / 9.9 |
| timeouts | 2.0% / 2.7% | 3.3% / 1.7% |
Runs: 215 s before, 240 s after. Noise is about +-6 points per cell; Stage 6 tunes numbers.

## What I found / what is left
- First version (go bar 0.35) lengthened games to 11 rounds and 4.7% timeouts: the go gate was the cause (switching it off gave
  9.0 rounds; counters and archenemy scoring alone ~11.0 / 11.2); relaxed to the numbers above. Archenemy scoring was a ~1.5x
  slowdown until the per-deck line list was cached.
- Gaps: Silence / Chant are cast right before going (main phase), not in the upkeep or in response to an opponent's win
  attempt; opponents can counter the Silence itself (the AI does, via the normal roll). No Angel's Grace for Ad Nauseam's
  life payment before the cast (Grace only saves from a lost state). Free counter cards are recognised by tag (Free/Pact/
  Misstep/Fon/Fierce with commander out, Swat with commander out), deck-composition odds stand in for the hand (no hidden-hand
  peeking except the archenemy `ready` line check). The `hold_up` slot is added to the stop utility only (`reserve_penalty`
  is archenemy-driven, not slot-driven). `Rograkh` row 1 dropped ~8-10 points; a plan for it should look at why its
  line is being answered more (counters now go to its steps).
