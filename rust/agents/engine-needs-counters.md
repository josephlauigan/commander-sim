# Engine needs: batch counters

Card side for all of these is in `crates/sim-core/src/impls/t6_counters.rs` (tests in `tests/cards_t6_counters.rs`).

## Daze
Needs: alternative costs for counterspells: `pick_counter` / `pay_counter` / `free_counter` ask `t6_counters::alt_cost_ok` / `pay_alt_cost` (return an Island, sacrifice a blue creature, exile 1-2 blue cards, X-pitch) instead of only the Force-of-Will special cases. One general hook serves Daze, Flare of Denial, Disrupting Shoal, Misdirection, Snapback, Commandeer, Mindbreak Trap.
Where it would hook: engine/stack.rs `pick_counter`, `free_counter`, `pay_counter` (and ai/brain.rs line ~94 and ~191 which call `free_counter`).
Card side done: `alt_cost_ok`, `pay_alt_cost` (Island returned to hand through `turn::remove_land`); the soft counter {1} already works through the `soft` tag.
Test to add once the engine has it: a seat with Daze, an Island and no other mana counters an opposing spell and ends with the Island in hand.

## Flare of Denial
Needs: the same alternative-cost hook as Daze.
Where it would hook: engine/stack.rs `pick_counter` / `pay_counter`.
Card side done: `flare_fodder`, `pay_alt_cost` (sacrifices the least valuable nontoken blue creature with `zones::die(.., "sac")`).
Test to add once the engine has it: with a nontoken blue creature and no mana, Flare counters a spell and the creature is sacrificed.

## Disrupting Shoal
Needs: a counter kind "only if the spell's mana value is X": `counter_ok` (a new `ctr` value such as `mvx`), `counter_cost` giving {X}{U}{U} for X = the spell's mana value, and the alternative cost above. Until then the card has no `ctr` tag (the old `ctr=any` would counter anything for {U}{U}).
Where it would hook: engine/stack.rs `counter_ok`, `counter_cost`, `resolve_counter`.
Card side done: `shoal_pitch` (blue card with that mana value), `shoal_counters`, `pay_alt_cost`.
Test to add once the engine has it: a Shoal with a blue 2-drop in hand counters a 2-mana-value spell for free and does nothing against a 3-mana-value spell.

## Snapback
Needs: the alternative-cost hook for a non-counter instant (exile a blue card), in the code that pays for and casts a removal spell from hand.
Where it would hook: engine/cast.rs where `cost_of` / `pay` run for a hand spell (a per-card alternative cost slot).
Card side done: `pay_alt_cost` (exile the least useful blue card); the bounce itself is the existing `rem=bounce` DSL spell.
Test to add once the engine has it: Snapback with no mana and a blue card in hand bounces an opposing creature and exiles the blue card.

## Mindbreak Trap
Needs: the free mode (a per-card alternative cost, see Daze) and a response that targets any number of spells (all opposing spells on the stack).
Where it would hook: engine/stack.rs `ai_respond` / `cast_counter_spell` (resolve through `t6_counters::mindbreak_exile` instead of `resolve_counter`).
Card side done: `mindbreak_free`, `mindbreak_exile` (marks every opposing spell countered and exiled); hard-cast it is the existing `ctr=any ctrexile` counter.
Test to add once the engine has it: after an opponent casts three spells, Trap is cast for free and all three are exiled.

## Misdirection
Needs: the AI answering a spell with a card that is not tagged `ctr` (a response that changes the stack item), plus the alternative cost.
Where it would hook: engine/stack.rs `ai_respond` (next to the counterspell branch), and a new stack-response slot on `CardImpl`.
Card side done: `misdirect` (single target to the caster's best permanent / the caster; spells with no or two targets are left alone), `pay_alt_cost`.
Test to add once the engine has it: a removal spell aimed at our creature is redirected at the caster's creature.

## Commandeer
Needs: a non-counter response (as Misdirection), the alternative cost (two blue cards), and a spell resolving under its stack item's controller: `cast_card` resolves with the original caster `p` and its own ctx snapshot, so the controller change on the item is not seen.
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
