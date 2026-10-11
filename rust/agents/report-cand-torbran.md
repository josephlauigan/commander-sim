# Report: cand-torbran

All 7 cards are Full in `cardgen audit --mine` (torbran: 65 unique cards, 0 not Full). Tests: tests/cards_torbran.rs (13).

## How each card is modeled
- Torbran, Thane of Red Fell: engine replacement effect. `engine::life::torbran_plus(g, p)` = 2 per Torbran. Applied in `lose_life`
  (damage to an opponent, before Fiery Emancipation's tripling), `apply_removal` (damage to an opponent's creature) and combat
  (blocked-creature kill check). Deck is mono-red so every source is red; damage to you / your permanents is unchanged.
- Impractical Joke: `Game::no_prevent` (new field) + `Game::unpreventable()`; checked in prevents_damage, no_damage, tajic/dovin/spider wrappers
  and the fog check. Not covered: protection from a colour in combat. Card is rem=dmg3 tgt=cp with a resolve hook.
- Borderland Marauder (attack hook, +2/+0), Goblin Medics (attack hook = becomes tapped; nothing else taps it), Karplusan Wolverine
  (blocks hook), Godo's Irregulars (blocks hook, pays {R} per activation until the blocker dies), Rift Bolt (suspend via
  `torbran_options` + shared `jodah::suspend_upkeep`; the compiled damage spell resolves it).
## AI plan
Targeting counts +2 (`legal_targets`, `bolt_something`, dsl any_target, Crackle, brain face option); `ping_floor` makes pings go to the face
unless a value-5 creature can be killed; Torbran prio 92, Helm/Plate 84 once he is out; `torbran_options` adds Rift Bolt suspend and
"burn at a nearly dead face".
## Shared-file changes (all small)
state.rs (no_prevent + unpreventable), engine/life.rs (torbran_plus, ping_floor, prevents_damage), engine/removal.rs, engine/cast.rs,
engine/combat.rs, engine/values.rs (no_damage), cardcode.rs (3 wrappers), dsl/mod.rs, ai/brain.rs (face option, special_options arm).
data/card_overrides.toml (7 entries), data/cards.json (export).
## Checks
cargo test --release all pass; build --all-targets no warnings; all_decks adaptive 3 and lookahead 1 clean; torbran smoke run (48 games) clean.
