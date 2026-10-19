# Isshin T1 regression: RESOLVED - it is a BUG (Karlach double-implemented), not correct rules

## Verdict
The +6 points (24.5 -> 30.8) come from an unbounded extra-combat loop on Karlach, Fury of Avernus, which the old
`ncomb < 4` cap used to hide. Legion Warboss and the new `each_combat` event are not involved (Warboss is in the deck
list but no trace shows duplicate tokens; only Sting uses `each_combat`).

## Oracle text (Scryfall, one POST /cards/collection)
- Karlach, Fury of Avernus: "Whenever you attack, if it's the first combat phase of the turn, untap all attacking
  creatures. They gain first strike until end of turn. After this phase, there is an additional combat phase."
- Isshin: "If a creature attacking causes a triggered ability of a permanent you control to trigger, that ability
  triggers an additional time." So Karlach + Isshin = at most 3 combats a turn (1 + 2), because later combats are not
  the first.

## Cause (code and data)
`data/card_overrides.toml` ["Karlach, Fury of Avernus"] (line ~2518) sets tags and a hand-coded hook
(`rust/crates/sim-core/src/impls/t1.rs` `fn karlach`, guarded by `combat_no == 1`) but, unlike e.g. Kappa Cannoneer,
has NO `dsl = []`. So the auto-compiled DSL ability from the oracle text is still live in `data/cards.json`:
`triggered, event "attack", source "you_any", effects: grant first strike to all your creatures + {"do":"extra_combat"}`.
It has no "first combat phase" condition, so every attack, in every combat, adds one extra combat
(`dsl/mod.rs:757` `extra_combats += 1`), and Isshin doubles it (+2 per combat). The chain only stops at
MAX_COMBATS=200. Baseline stopped it at 4 combats (`while ncomb < 4`).
Aurelia and Scourge of the Throne are fine (their DSL is null; hand-coded hooks have once-per-turn guards).

## Evidence (traces: `--deck seph --pool t1 --any-tier --profile conservative --trace N`, Isshin is seated in the
## draw; --deck cannot be a pool deck, so Seph was used as the observer seat. Seeds 500000+N, N=0..59, both binaries)
- Isshin attack lines per turn, max (>4): OLD: only game 21 (6). NEW: game 12 R8 = 200, game 15 R8 = 67,
  game 13 R5 = 39, game 11 R6 = 28, game 13 R6 = 23, game 3 R5 = 22, game 21 R6 = 20.
- NEW game 12, R8: Isshin casts Karlach, then "combat cap of 200 phases reached" (log line 49678), attacking creature
  counts 20, 35, 54, 77, 104, 135, 170, 209, 252 (Hero of Bladehold / Anim Pakal tokens doubled every combat).
  Isshin wins on turn 9.
- NEW game 13 R5: Isshin casts Karlach, then ~39 consecutive "Isshin attacks Sephiroth with 1 creature(s)" lines in
  one turn; Isshin wins on R7. OLD game 13: Sephiroth wins (combo). NEW game 15: Isshin wins, OLD: Light-Paws.
- Same-seed games where the winner flips to Isshin on NEW: 13, 15 (and Isshin turned a win into a loss on none in
  this sample; other flips are Seph-seat butterfly effects).

## Fix (not applied; no code edits were allowed)
Add `dsl = []` to the ["Karlach, Fury of Avernus"] block in `data/card_overrides.toml` (then regenerate
`data/cards.json`), so only the guarded hand-coded `karlach` runs. Then re-run `--calibrate within --pool t1`:
Isshin should return to about 24-26%. Also worth a sweep for other cards with both a hand-coded hook and a leftover
compiled DSL ability that adds `extra_combat` (only Karlach has a DSL with `extra_combat` in cards.json).
Traces: scratchpad t6-baseline-N.txt / t6-cedh-N.txt (session scratchpad).
