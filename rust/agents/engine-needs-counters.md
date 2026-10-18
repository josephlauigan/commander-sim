# Engine needs: batch counters

Card side for all of these is in `crates/sim-core/src/impls/t6_counters.rs` (tests in `tests/cards_t6_counters.rs`).

## Misdirection
Needs: the alternative cost is DONE (t6-e-altcost: `engine/altcost.rs` slot, registered). Still needs: the AI answering a spell with a card that is not tagged `ctr` (a response that changes the stack item).
Where it would hook: engine/stack.rs `ai_respond` (next to the counterspell branch), and a new stack-response slot on `CardImpl`.
Card side done: `misdirect` (single target to the caster's best permanent / the caster; spells with no or two targets are left alone), `pay_alt_cost`.
Test to add once the engine has it: a removal spell aimed at our creature is redirected at the caster's creature.

## Commandeer
Needs: the alternative cost is DONE (t6-e-altcost). Still needs: a non-counter response (as Misdirection), and a spell resolving under its stack item's controller: `cast_card` resolves with the original caster `p` and its own ctx snapshot, so the controller change on the item is not seen.
Where it would hook: engine/cast.rs `cast_card` after `stack_window_keep` (resolve with `item.controller`, `item.ctx`), and `ai_respond`.
Card side done: `commandeer` (controller, owner for the card's destination, new targets).
Test to add once the engine has it: we commandeer an opposing artifact spell and the artifact enters under our control.

## Sudden Substitution
Needs: split second (no responses while it is on the stack: `take_priority` / `ai_respond` must skip players while a split-second item is on the stack), a non-counter response, and a spell resolving under its stack item's controller (see Commandeer).
Where it would hook: engine/stack.rs `take_priority`, `ai_respond`; engine/cast.rs `cast_card`.
Card side done: `substitute` (exchange control of the creature with `marchesa::steal`, controller swap on the stack item, new targets).
Test to add once the engine has it: nobody can respond to it, and the exchanged spell resolves for the creature's old controller.

## Bilbo's Gambit
Needs: a gift choice on cast (a Treasure for an opponent) and "players can't cast spells this turn" (read `t6_counters::lock_flag` on the caster in `castable`/`cast_card`, any seat).
Where it would hook: engine/cast.rs `castable`; the AI deciding to promise the gift when the lock wins the game.
Card side done: `bilbo_bounce` (spell back to its owner's hand, gift Treasure, lock flag); without the gift the card is the existing `ctr=any remand` counter (cast.rs returns the card to hand).
Test to add once the engine has it: with the gift promised, no player can cast a spell for the rest of the turn.

## Abeyance
Needs: a per-player "can't cast instants or sorceries / can't activate non-mana abilities this turn" check: `castable`, ability activation and the AI's option lists read `t6_counters::abeyance_flag` (also what Silence, Orim's Chant and Mandate of Peace style cards need, so a general player-level cast lock flag would serve all of them).
Where it would hook: engine/cast.rs `castable`; the ability activation path.
Card side done: draws a card and sets the flag on the opponent with the most cards in hand.
Test to add once the engine has it: the flagged opponent's instant is refused this turn and allowed next turn.

## Tibalt's Trickery
Needs: a per-card side-effect slot when a counterspell counters (`counter_side_effects` only knows tags and a few names).
Where it would hook: engine/stack.rs `counter_side_effects` (call `t6_counters::tibalt_after(g, q, p, spell)` when the counterspell is Tibalt's Trickery, with q the caster and p the countered spell's controller).
Card side done: `tibalt_after` (mill 1-3 at random, exile to a nonland card with a different name, cast it free unless it is a counterspell, the rest on the bottom in random order).
Test to add once the engine has it: countering a spell with it gives the countered player a free spell and a shorter library.
