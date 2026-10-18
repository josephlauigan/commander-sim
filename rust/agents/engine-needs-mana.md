# Engine needs: batch `mana`

## Apple of Eden, Isu Relic
Needs: delayed "at the beginning of the next end step" effects that outlive their source permanent, and a way to play cards owned by another player from exile (lands included, with "its owner draws" on each play).
Where it would hook: a `Game`-level delayed-trigger list fired next to `Event::EndStep` in `engine/turn.rs` (line ~679); `yshtola::take_card` / `Player::stolen` for held cards; a land played from the held cards would need `play_land_card` to put it under the thief's control but return it to its owner.
Card side done: sacrifice, 4 life, take every nonland card from the opponent with the most, the AI casts what it can immediately with any-type mana, the rest return to the owner's hand, the owner draws once per spell played.
Test to add once the engine has it: the cards stay held until the end step (an opponent can't use them meanwhile), unplayed lands return, and a land played from the held cards draws a card for its owner.
