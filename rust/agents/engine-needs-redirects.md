# Engine needs: batch redirects

Card side for all of these is in `crates/sim-core/src/impls/t6_redirects.rs` (tests in `tests/cards_t6_redirects.rs`).

## Redirect Lightning
Needs: the removal-response hook (`ai::protect_response`, `ai/mod.rs` ~304 and `ai/decks.rs` ~1232) calling `t6_redirects::redirect_removal` for any deck holding the card (today only Veyran/Kefka call Deflecting Swat by name); and a stack-response slot that changes the target of a spell or ability on the stack (reuse `t6_counters::misdirect` for a spell aimed at us). Abilities on the stack have no re-targeting yet.
Where it would hook: `ai::protect_response` and `engine/stack.rs` `ai_respond`.
Card side done: `redirect_removal` (pays {R}+{2} or {R}+5 life, aims at the caster's best permanent, applies the removal there), `redirect_lightning_life` (which additional cost the AI pays).
Test to add once the engine has it: an opponent's Swords to Plowshares on our key creature, with the card in hand, exiles the caster's own creature instead.
