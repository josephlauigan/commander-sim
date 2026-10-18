# Engine needs: batch `mana`

## Simian Spirit Guide
Needs: mana from a card in hand that is exiled as the cost, for any card, not by name: `partials::hand_mana` returns a unit only for "Elvish Spirit Guide" ({G}); a data-driven list (card name -> colour: Simian Spirit Guide {R}) or a `hand_mana` slot on `CardImpl` would cover both. `special_unit_paid` (Source::HandCard) already exiles the card.
Where it would hook: `rust/crates/sim-core/src/impls/partials.rs` `hand_mana` (called by `engine/mana.rs` `adjust_mana` through `cardcode::hand_mana`).
Card side done: nothing needed: the 2/2 body is modeled from its tags; the exile-for-{R} ability is the engine part.
Test to add once the engine has it: a hand with Simian Spirit Guide and no red source can pay {R} (`can_pay(.., 0, "R")`), paying exiles it.
