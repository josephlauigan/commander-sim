# Engine needs: batch `blue`

Card side for all of these is in `crates/sim-core/src/impls/t6_blue.rs` (tests in `tests/cards_t6_blue.rs`).

## Borne Upon a Wind
Needs: the player-level "you may cast spells this turn as though they had flash" grant (same need as Emergence Zone in `engine-needs-lands.md`): the cast-timing check and the AI's end-of-turn / in-response casting read the one-turn flag.
Where it would hook: impls/t0_captain.rs `as_flash` (read by `ai/brain.rs` ~line 1264): also return true when `t6_blue::flash_turn_grant(g, p)` (the flag `t6_lands::FLASH_FLAG`, shared with Emergence Zone); `engine/cast.rs` `castable` for the timing test.
Card side done: draws a card and sets the caster's turn flag (`t6_blue::flash_turn_grant`).
Test to add once the engine has it: after Borne Upon a Wind on the opponent's turn, a sorcery in hand is cast in their end step.

## Mnemonic Betrayal
Needs: a delayed "at the beginning of the next end step" trigger that is not tied to a permanent (any player's end step), here "return the cards still exiled to their owners' graveyards"; and `impulse` clean-up to send a taken card back to its owner (`Player::stolen`) instead of into the caster's exile.
Where it would hook: `engine/turn.rs` end step where `impulse` is cleared (about line 707): for a card in `stolen`, push it to the owner's graveyard and drop it from `stolen`/`agent_ids`; the clean-up also only runs at the caster's own end step.
Card side done: exiles every opponent's graveyard (lands stay in exile), holds the nonland cards in the caster's hand with `yshtola::take_card` (castable with mana of any type, goes to the owner's graveyard when cast) and lists them in `impulse`; Betrayal itself is exiled.
Test to add once the engine has it: an uncast card from an opponent's graveyard is back in that graveyard after the end step, and nothing stays in the caster's exile.

## Rain of Filth
Needs: a mana source "sacrifice a land: add {B}" granted to every land of a player for a turn (a new `Source` variant next to `Scion`/`Treasure`, or a general "land gains an ability until end of turn" layer).
Where it would hook: `engine/mana.rs` (`Source`, `total_mana` / the unit list, `pay` which taps or sacrifices the unit's source), reading `t6_blue::rain_of_filth_active(g, p)`.
Card side done: resolving sets the caster's turn flag (`rain_of_filth_active`); with no usable mana source the AI has no reason to cast it (no tags, no priority).
Test to add once the engine has it: with the flag, a player with three lands and a {B}{B}{B} spell sacrifices all three lands to cast it, from tapped or untapped lands; next turn the lands only tap.
