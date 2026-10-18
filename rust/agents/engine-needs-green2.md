# Engine needs: batch green2

## Sowing Mycospawn
Needs: kicker (an additional cost {1}{C} chosen as the spell is cast, with a "was kicked" flag the cast trigger can read); also devoid (colourless card) is not modelled anywhere.
Where it would hook: engine/cast.rs `cast_card` (cost payment) and `cardcode::self_cast` (would need the flag, e.g. on Ctx).
Card side done: the cast trigger fetches a land (even if countered); `t6_green2::mycospawn_kicked` exiles the best opposing land (not yet called).
Test to add once the engine has it: cast kicked with {1}{C} spare, an opposing land is exiled and the unkicked cast leaves it.

## Enduring Vitality
Needs: a permanent that is not a creature although its card is (a type override on a returned/animated-off permanent: "it's an enchantment, it's not a creature"); and the generic "creatures you control have {T}: add any colour" (Tag::Rite) path in `engine/mana.rs` only counts creatures tagged noatk with the commander's identity colours, so Cryptolith Rite-style effects are approximated there.
Where it would hook: `state.rs` `Game::is_creature` (a DataKey/flag that turns the creature type off) and every `d.creature` read of a card; `engine/mana.rs` `mana_units` rite branch.
Card side done: `t6_green2::vitality_mana` (extra_mana hook) gives every untapped, non-sick creature a unit of any-colour mana; `vitality_dies` returns the card to the battlefield the first time it dies as a creature, marked DataKey::Final, and the mana hook ignores the marked permanent as a creature. It still counts as a creature everywhere else (Gaea's Cradle count, attacks, blocks, creature removal, "dies" a second time works).
Test to add once the engine has it: after it returns, `is_creature` is false (not counted by Gaea's Cradle, cannot be attacked/blocked with); creature removal can't target it.

## Shifting Woodland
Needs: "becomes a copy of a card until end of turn" for a land: a land turning into a creature/permanent copy and back at end of turn without entering (no enters-the-battlefield triggers, no new permanent, keeps summoning-sickness history).
Where it would hook: `state.rs` (Land vs Perm are separate structs; a land cannot become a Perm and back), `impls/alela.rs` `set_face` / `mirror_revert` (Mirage Mirror does it for a Perm only).
Card side done: enters tapped unless you control a Forest (data enters_rule), {T}: {G}, `delirium()` (four card types in the graveyard), and the {2}{G}{G} land option: pays, taps the land, makes a temporary token copy of the best creature card in the graveyard (Oboro Breezecaller first), not summoning sick, exiled at the end step.
Test to add once the engine has it: the copy's enters triggers don't fire, the land is the same object and is a land again at end of turn.
