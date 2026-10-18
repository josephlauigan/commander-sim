# Engine needs: batch redirects

Card side for all of these is in `crates/sim-core/src/impls/t6_redirects.rs` (tests in `tests/cards_t6_redirects.rs`).

## Redirect Lightning
Needs: the removal-response hook (`ai::protect_response`, `ai/mod.rs` ~304 and `ai/decks.rs` ~1232) calling `t6_redirects::redirect_removal` for any deck holding the card (today only Veyran/Kefka call Deflecting Swat by name); and a stack-response slot that changes the target of a spell or ability on the stack (reuse `t6_counters::misdirect` for a spell aimed at us). Abilities on the stack have no re-targeting yet.
Where it would hook: `ai::protect_response` and `engine/stack.rs` `ai_respond`.
Card side done: `redirect_removal` (pays {R}+{2} or {R}+5 life, aims at the caster's best permanent, applies the removal there), `redirect_lightning_life` (which additional cost the AI pays).
Test to add once the engine has it: an opponent's Swords to Plowshares on our key creature, with the card in hand, exiles the caster's own creature instead.

## Ricochet Trap
Needs: the same removal-response hook as Redirect Lightning. The alternative cost ({R} after an opponent's blue spell) is already worked out in the card code (`blue_spell_this_turn`), so no separate alternative-cost hook is needed.
Where it would hook: `ai::protect_response` and `engine/stack.rs` `ai_respond`.
Card side done: `redirect_removal` (the Trap branch), `blue_spell_this_turn`.
Test to add once the engine has it: after an opponent casts a blue spell, removal aimed at our creature is re-aimed at the caster's creature for {R}.
