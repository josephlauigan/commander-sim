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

## Untimely Malfunction
Needs: the removal-response hook as for Redirect Lightning; and a way for the AI to choose the "can't block" mode on its own turn (the cast decision is driven by the `rem`/`tgt=a` tags, so it is only cast at an artifact).
Where it would hook: `ai::protect_response`, `engine/stack.rs` `ai_respond`; the attack planning in `ai/` for the can't-block mode.
Card side done: `malfunction_resolve` (artifact destroyed, else two best blockers get `cant_block` until end of turn via `partials::eot_kw`), `redirect_removal` (redirect mode, {1}{R}).
Test to add once the engine has it: a lethal attack stopped only by two blockers is cast through them and wins.

## Pyrokinesis
Needs: a per-card alternative cost for a hand spell (also asked for by `engine-needs-counters.md`: Daze, Snapback, Misdirection ...): cast it for "exile a red card from your hand" instead of {4}{R}{R}, free on the opponent's turn or in response.
Where it would hook: `engine/cast.rs` where `cost_of` / `pay` run for a hand spell, and the AI's instant-speed removal in `ai/`.
Card side done: `pay_pyro_pitch` / `red_pitch` (exile the least useful red card), `pyro_split` and `pyro_resolve` (divided damage). The card carries `rem=dmg4 tgt=c` so the AI casts it as 4-damage removal for its mana cost.
Test to add once the engine has it: with a red card in hand and no mana, Pyrokinesis kills two 2/2s and the red card is exiled.

## Scout's Warning
Needs: "play this creature card as though it had flash" as a per-player turn grant: the cast-timing check for creature spells reads `t6_redirects::scout_flash_active` and calls `scout_flash_used` after the creature is cast.
Where it would hook: `engine/cast.rs` `castable` (the timing test) and the AI's end-of-turn / in-response creature casting in `ai/`.
Card side done: the draw and the turn flag (`scout_resolve`, `scout_flash_active`, `scout_flash_used`).
Test to add once the engine has it: after the Warning, a creature card in hand is cast in the opponent's end step and the grant is spent.
