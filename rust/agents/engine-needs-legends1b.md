# Engine needs: batch legends1b

Card side in `crates/sim-core/src/impls/t6_legends1b.rs` (tests `tests/cards_t6_legends1b.rs`).

## Shang-Chi, Master of Kung Fu (mana)
Needs: restricted mana ("two mana of any one colour, spend only to activate abilities of creature sources"): mana in the pool tagged with what it may pay for.
Where it would hook: the mana pool in `engine/mana.rs` (`pay` and the pool type).
Card side done: Shang-Chi taps for one {C} through its `dork` tag (an unrestricted approximation).
Test to add once the engine has it: Shang-Chi's mana cannot cast a spell but pays Ioreth's or Marvin's costs.

## Marvin, Murderous Mimic
Needs: copying every activated ability of the controller's other creatures onto a permanent (including mana abilities, so a tapped Marvin can tap for what a Selvala makes), e.g. a way to run another card's `options` hook with Marvin as `src`.
Where it would hook: `ai/` option collection (calls each permanent's `options` slot) and `engine/mana.rs` for mana abilities.
Card side done: Ioreth's two untap abilities (shared `untap_options`).
Test to add once the engine has it: Marvin repeats a Selvala/Candelabra ability.
