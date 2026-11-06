# Report: t5-hold (measure Tier 5 counter-mana holding; no behaviour change)

## Built
- `sim-core/src/holdstats.rs`: trace-only `hd|...` counters in each AI seat's `stats`. At the end of the second main phase the turn
  is put in a group: a card held by `brain::hold_value` (counter / protection / instant removal / Ad Nauseam / other), or `idle`
  (nothing held, 2+ mana unused, a castable nonland card that is not a held kind). The group is followed to the holder's next turn
  start and ends as: used (a same-kind card hit something), cast with no hit, mana spent elsewhere, or nothing spent. Hits are
  classed combo piece or anything cast mid combo attempt / commander / creature / other permanent / removal spell / other spell /
  ability. Also kept: turns, "2+ unused and any castable card" (the C3 measure), the same split into held vs not held, unused
  mana, hold value, mana spent and other instants cast in the window. Each is kept for own turns 5-10 and for all own turns.
- Hooks: `engine/turn.rs` (snapshot after `mana_held`; outcome at turn start), `engine/cast.rs::on_cast`,
  `engine/mana.rs::spend_unit`, `t6stats::counter_hit`, `engine/removal.rs` (`apply_removal_rest`, the "protects" branch).
  `Player::hw` holds the snapshot; `Settings::hold_trace` (default on) exists so the identity test can switch it off.
  Skipped for people's seats and look-ahead copies. No RNG, no log lines, no decision reads a counter.
- `sim-cli/src/holdreport.rs`, printed by `--realism` after the flow tables: pooled table and per-deck, for turns 5-10 and all turns.
- `tests/t5_hold.rs`: three seeded Tier 5 pods (two adaptive, one look-ahead) play identically (winner, rounds, full log, every
  non-`hd|` counter) with the counters off; and a scripted held Counterspell that counters Sheoldred is a `used` hold with hit
  class `creature`, and a turn with nothing cast is `unused`.
- `rust/agents/t5-hold-queue.sh`: build, then `--realism --pool t5` and `--pool t3`, 2000 games, seed 900000, adaptive, to
  `audit/realism/t5-hold-2026-11-06/{t5,t3}-adaptive-2000.txt`, then `ALL DONE`. Not run by me.

## Checks run
`cargo build --release --all-targets`: no warnings. Passing: t5_hold, c3_casual_speed, ai, ai_flow, ai_eval, cards_yshtola,
cards_thanos, cards_sauron, cards_sauron_speed, cards_sauron_sweepers, cards_sauron_threats, cards_sheoldred, cards_sheoldred_sac,
and sim-cli's holdreport unit test. No sims were run, so there are no measured numbers yet.

## Limits (see NEEDS JD in the reply)
- Counter target: the topmost uncountered stack item of the countered player (not a stored target id).
- Removal and protection hits are classed by the permanent, found at removal time; a removal spell that is countered or fizzles
  is "cast, no hit". Protection saved by a sacrifice-style response (no mana) still counts as a hit, so protection "used" may be
  slightly high.
- Mana "spent elsewhere" counts any mana spent by the holder in the window (abilities included), not only instants.
- `--ai adaptive` (the queue) does not use `search::hold_bonus` / `hold_credit`; Tier 5 pool seats under the look-ahead hold with
  those extra terms, so a look-ahead run could differ from the adaptive one.

## Proposed hold rules, by outcome (not implemented)
All change `ai/brain.rs`; the held-mana value `v` is made in `hold_of` (line ~241) and applied by `reserve_penalty` (~258, called at
~1192 in `card_utility`); the look-ahead adds it in `search::hold_bonus` (~465), `hold_credit` (~660) and the `evdiag` hold (~749).
1. Most holds wasted (used well under half of held turns, mostly "unused"): shrink the outside-deck floor in `hold_of`, i.e. the
   `v = v.max(1.5 + 2.5 * caution * n.min(2))` line for `!is_main(...)` decks, e.g. scale it by the measured use rate, or
   drop it to 0 unless `s.max_threat` is high.
2. Holds used mostly against low-value targets (spells, ability, engine) but seldom against combo or commanders: hold only when an
   opponent is close to winning: gate the floor on `s.combo_near || s.cmdr_combo || s.max_threat >= X`, keeping the
   `3.0 * caution * threat` term.
3. Hold used but only the cheapest answer needed while more mana sits idle (high "unused mana" with a castable card, low
   mana per hold): cap the reserve to the held card's cost: `reserve_penalty` already uses `seq_cost(c, h)`; the idle cases come
   from the main-2 stop in `main_options`, so offer castable cards when the reserve is still payable after them.
4. Most holds used (counters hit combo pieces / commanders): leave the rule; a Tier 5 table of counter decks then really does
   stonewall, and the Tier 3 comparison shows whether it is the same at Tier 3.
5. Mixed by kind (e.g. counters wasted, protection used): split the floor by kind: the `n` count in `hold_of` is of `Tag::Ctr`
   cards only; give removal and protection their own terms.
