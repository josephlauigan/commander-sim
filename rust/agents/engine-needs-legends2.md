# Engine needs: batch legends2

Card side for all of these is in `crates/sim-core/src/impls/t6_legends2.rs` (tests in `tests/cards_t6_legends2.rs`).

## Magda, Brazen Outlaw
Needs: a general "permanent becomes tapped" event (any tap: a tap cost, a "tap target" effect), not only the attack declaration and mana abilities. `impls/t0_captain.rs::became_tapped` is already called at the attack declaration, mana abilities and the ability language's tap cost; a hook slot fired from the same places would serve Magda.
Where it would hook: `engine/mana.rs` (Source::Perm), `engine/combat.rs` ~1068, `dsl/mod.rs` ~1719 (the three `became_tapped` call sites).
Card side done: Treasure on a Dwarf tapping for mana (`mana_tapped` slot) and attacking (`attack` slot); the +1/+0 and the five-Treasure search.
Test to add once the engine has it: a Dwarf tapped by an opposing "tap target creature" effect still gives Magda's controller a Treasure.

## Teferi, Who Slows the Sunset
Needs: emblem hooks on the turn structure: "untap all permanents you control during each opponent's untap step" and "you draw a card during each opponent's draw step" for a player holding the emblem.
Where it would hook: `engine/turn.rs` where each player's untap step and draw step run (the emblem is the string `"teferi_sunset"` in `Player::emblems`; `cardcode::emblem_draw` is the existing per-draw emblem hook).
Card side done: +1 and -2 are complete; the -7 ability pushes the emblem (`teferi_ult`) but its value is `None`, so the AI never uses it until the effects exist (then give `teferi_ult` a value in `TEFERI`).
Test to add once the engine has it: with the emblem, an opponent's untap step untaps our lands and their draw step draws us a card.
