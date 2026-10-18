# Engine needs: batch `blue`

Card side for all of these is in `crates/sim-core/src/impls/t6_blue.rs` (tests in `tests/cards_t6_blue.rs`).

## Borne Upon a Wind
Needs: the player-level "you may cast spells this turn as though they had flash" grant (same need as Emergence Zone in `engine-needs-lands.md`): the cast-timing check and the AI's end-of-turn / in-response casting read the one-turn flag.
Where it would hook: impls/t0_captain.rs `as_flash` (read by `ai/brain.rs` ~line 1264): also return true when `t6_blue::flash_turn_grant(g, p)` (the flag `t6_lands::FLASH_FLAG`, shared with Emergence Zone); `engine/cast.rs` `castable` for the timing test.
Card side done: draws a card and sets the caster's turn flag (`t6_blue::flash_turn_grant`).
Test to add once the engine has it: after Borne Upon a Wind on the opponent's turn, a sorcery in hand is cast in their end step.
