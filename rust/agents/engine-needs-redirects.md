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

## Get Lost
Needs: (1) noncreature artifact tokens with an activated ability (Map: {1},{T}, sacrifice: target creature you control explores, sorcery speed; likewise Treasure/Clue/Food-style tokens made by a spell for an opponent); (2) a `tgt` code for "creature, enchantment, or planeswalker" in `legal_targets` so the AI can aim removal at the latter two.
Where it would hook: `engine/zones.rs` `make_tokens` (tokens are always creatures today), `engine/removal.rs` `legal_targets` (new `tgt` such as "cep").
Card side done: `get_lost_resolve` (destroy the chosen creature, else the best opposing enchantment or planeswalker).
Test to add once the engine has it: Get Lost on an opposing creature gives its controller two Map tokens.

## Soul Partition
Needs: "may play from exile" as a lasting permission (a per-card marker on the exiled card, read by `castable` / the cast options for hand-like zones) with a cost tax for an opponent of the caster ({2} more).
Where it would hook: `engine/cast.rs` `castable` and `cost_of`, plus the AI's cast options in `ai/brain.rs`.
Card side done: `soul_partition_resolve` (exile, then the card returns to its owner's hand as a stand-in; tokens vanish).
Test to add once the engine has it: after Soul Partition on an opposing creature, the creature stays in exile, can be cast by its owner for {2} more, and cannot be cast for less.

## Calamity's Wake
Needs: "players can't cast noncreature spells this turn" in the cast check (the same lock Silence / Bilbo's Gambit / Abeyance need: a per-turn "can't cast" flag read by `castable` for every player).
Where it would hook: `engine/cast.rs` `castable` (and the counter / response casting in `engine/stack.rs`).
Card side done: graveyards and the Wake exiled, `noncreature_lock` (true for the rest of the turn).
Test to add once the engine has it: after the Wake, an opponent holding Lightning Bolt cannot cast it that turn but can cast a creature.

## Emerald Charm
Needs: removing a keyword from a creature until end of turn (the `eot_kw` list only adds keywords); a `tgt` code for "enchantment" alone so the AI aims the destroy mode (today it uses `tgt=nl`).
Where it would hook: `dsl::has_kw` / `Perm::fly` readers in `engine/combat.rs` (a "loses flying" eot list), `engine/removal.rs` `legal_targets`.
Card side done: `emerald_charm_resolve` (destroy a non-Aura enchantment, else untap a land or the best tapped permanent).
Test to add once the engine has it: a 2/2 flyer loses flying and is blocked by a ground creature that turn.

## Sunscour
Needs: the per-card alternative cost for a hand spell (same hook as Pyrokinesis and `engine-needs-counters.md`): cast it for "exile two white cards from your hand" instead of {5}{W}{W}.
Where it would hook: `engine/cast.rs` where `cost_of` / `pay` run for a hand spell; the wipe decision in `ai/` (`wipe_options`).
Card side done: `pay_sunscour_pitch` / `white_pitch` (exile the two least useful white cards, refuse with fewer than two). The wipe is the existing `wipe=destroy` tag.
Test to add once the engine has it: with two white cards in hand and no mana, the AI casts Sunscour on a losing board and both cards are exiled.
