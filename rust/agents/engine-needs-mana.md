# Engine needs: batch `mana`

## Simian Spirit Guide
Needs: mana from a card in hand that is exiled as the cost, for any card, not by name: `partials::hand_mana` returns a unit only for "Elvish Spirit Guide" ({G}); a data-driven list (card name -> colour: Simian Spirit Guide {R}) or a `hand_mana` slot on `CardImpl` would cover both. `special_unit_paid` (Source::HandCard) already exiles the card.
Where it would hook: `rust/crates/sim-core/src/impls/partials.rs` `hand_mana` (called by `engine/mana.rs` `adjust_mana` through `cardcode::hand_mana`).
Card side done: nothing needed: the 2/2 body is modeled from its tags; the exile-for-{R} ability is the engine part.
Test to add once the engine has it: a hand with Simian Spirit Guide and no red source can pay {R} (`can_pay(.., 0, "R")`), paying exiles it.

## Apple of Eden, Isu Relic
Needs: delayed "at the beginning of the next end step" effects that outlive their source permanent, and a way to play cards owned by another player from exile (lands included, with "its owner draws" on each play).
Where it would hook: a `Game`-level delayed-trigger list fired next to `Event::EndStep` in `engine/turn.rs` (line ~679); `yshtola::take_card` / `Player::stolen` for held cards; a land played from the held cards would need `play_land_card` to put it under the thief's control but return it to its owner.
Card side done: sacrifice, 4 life, take every nonland card from the opponent with the most, the AI casts what it can immediately with any-type mana, the rest return to the owner's hand, the owner draws once per spell played.
Test to add once the engine has it: the cards stay held until the end step (an opponent can't use them meanwhile), unplayed lands return, and a land played from the held cards draws a card for its owner.
