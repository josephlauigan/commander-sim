# Engine needs: batch legends2

Card side for all of these is in `crates/sim-core/src/impls/t6_legends2.rs` (tests in `tests/cards_t6_legends2.rs`).

## Magda, Brazen Outlaw
Needs: a general "permanent becomes tapped" event (any tap: a tap cost, a "tap target" effect), not only the attack declaration and mana abilities. `impls/t0_captain.rs::became_tapped` is already called at the attack declaration, mana abilities and the ability language's tap cost; a hook slot fired from the same places would serve Magda.
Where it would hook: `engine/mana.rs` (Source::Perm), `engine/combat.rs` ~1068, `dsl/mod.rs` ~1719 (the three `became_tapped` call sites).
Card side done: Treasure on a Dwarf tapping for mana (`mana_tapped` slot) and attacking (`attack` slot); the +1/+0 and the five-Treasure search.
Test to add once the engine has it: a Dwarf tapped by an opposing "tap target creature" effect still gives Magda's controller a Treasure.
