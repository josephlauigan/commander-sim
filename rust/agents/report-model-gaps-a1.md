# Report model-gaps-a1 (the karn deck's non-Full cards)

Branch model-gaps-a1. `cardgen audit --mine`: karn now has 1 card below Full (Mystic Forge, Approximate); the other 12 are Full.
Tests: `rust/crates/sim-core/tests/model_gaps_karn.rs` (19 tests, all pass) plus the existing files that cover the touched code
(cards_karn*, cards_t5, cards_t6_godo, cards_alela, cards_tymna, cards_sauron, cards_thanos, cards_titania, cards_legends,
play_karn_*, play_cloud_karn_gaps, play_abilities/combat/altcost/hand/cards/marchesa, practice_* probes, ai, ai_table_fixes,
ai_refused_options, altcost, rules, repl, generic_paths, ...): all pass. `cargo build --release --all-targets`: no warnings.
Not run: the full suite and any sim (the coordinator does both).

## Per card (before -> after)
1. **Tezzeret, Cruel Captain** Partial -> Full. -7 emblem: `Player::emblems` key `tezzeret_cc` (`t6_godo::TEZ_EMBLEM`); at the
   beginning of combat on its owner's turn (engine/combat.rs, beside Liliana's emblem) `t6_godo::tez_emblem` puts three +1/+1
   counters on an artifact (AI: an artifact creature first, else the best non-rock; a person is asked), and a noncreature
   artifact becomes a 0/0 Robot artifact creature for good (`DataKey::Anim`, subtype robot). Test: -7 option, Mind Stone ->
   3/3 Robot, second trigger -> 6/6, and combat() fires it. practice_trigger_probe: new FALSE_ALARMS entry (the -7 only gives the
   emblem; its target is asked at combat).
2. **Blightsteel Colossus** Approximate -> Full. The -1/-1 counters for infect damage to a creature were already in
   engine/combat.rs (the note was stale; test added). New `zones::put_in_gy` (shuffle into the library instead) used by mill,
   discard_cards, discard_index and the two countered-spell sites (stack.rs, cast.rs); `to_zone_card` already did the battlefield.
3. **Cityscape Leveler** Approximate -> Full. `zones::to_zone_card` sends any permanent with the `DataKey::Unearth` mark to exile
   (dies, bounce, anything). That also applies to the other unearth cards (Archfiend of Sorrows, Alela's), which is the rule.
4. **Darksteel Citadel** Approximate -> Full. Both `destroy_land` copies (impls/fixes.rs, impls/lands.rs) ask
   `values::indestructible`.
5. **Liberator, Urza's Battlethopter** Approximate -> Full. Rules were right (flash, mana-spent counter). AI: at the end of the
   turn before its own, `wipe_options(eot)` and the instant-removal loop also take a colourless/artifact sorcery when Liberator
   is out (`liberator_flash`): All Is Dust. (Permanent spells were already cast in that window.)
6. **Liquimetal Torque** Approximate -> Full. AI use (karn_engines::torque_options): before combat, Karn, Legacy Reforged ready
   to attack and a nonartifact permanent of ours with mana value >= Karn's size + 2 (Ugin, the Spirit Dragon; Karn Liberated)
   -> make it an artifact (Karn grows by the difference, `open_attack` required). Otherwise it taps for {C}. `karn::max_mv` now
   reads `t5::is_artifact`, so the Torque mark counts for Karn's size (it did not before). Other uses (Construct, upkeep mana,
   Darksteel Forge protection) were checked and never beat tapping for {C}.
7. **Mirrorworks** Approximate -> Full. New `land_play` hook `mirror_land`: an artifact land played as the land drop (Darksteel
   Citadel, Treasure Vault) is copied for {2}: a token land of the same card in `Player::lands` (landfall fires). `mirror_pay`
   is the shared ask/pay. `lands::sac_land` no longer puts a token land's card in the graveyard. Caveat: other code paths that
   remove a land and push its card to a graveyard (mass sacrifice in rules.rs, Kefka) do not know about token lands, so a
   copied Citadel sacrificed that way leaves a duplicate card in the graveyard (rare).
8-9. **Mishra's Factory / Foundry** Approximate -> Full. A person already had the pumps (human/karn.rs). AI: new
   `karn::combat_pumps`, called from `combat::resolve_blocks` after blockers: an untapped Factory (land, or the animated token
   that did not attack) taps to give a worker in a fight +1/+1, a Foundry (land or animated, untapped) pays {1} for +2/+2 on an
   attacking worker; only when the pump saves the worker or lets it kill its opponent (Foundry: gain >= 2). Honours
   Pithing Needle / locks.
10. **Mystic Forge** Approximate -> Approximate (note rewritten). Removal and wipes on top now go through `removal_options` /
   `wipe_options` and their executors (`brain::spell_sources`, `source_zone`; `forge_top_card` is pub), Tokx X spells through
   `forge_cast`. Still missing: a counterspell on top is never cast (pick_counter / cast_counter_spell / the response gate count
   the hand only; no colourless counterspell is in the karn list or the pool except Not of This World, a protector).
11. **Pithing Needle** Approximate -> Full. Permanents' abilities already checked `ability_locked` (abilities_of). Added: a person's
   land abilities (`human::use_land`), and graveyard / hand ability options (unearth, cycling ... `human/alt.rs`, label based).
   The AI's Leveler unearth checks `needle_stops`. Mana abilities are exempt as in the Oracle text (tapping goes through the
   mana code, which never asks the lock).
12. **Steel Hellkite** Approximate -> Full. Person: already had the pump. AI: `karn_cards::hellkite_pump_lethal` (after blockers):
   an unblocked attacking Hellkite is pumped with {2} per +1/+0 when that makes the attack lethal on the defender.
13. **Ugin, the Ineffable** Approximate -> Full. The gap: the exiled card was handed back only when the Spirit *died* or when Ugin
   *left* (early). Now the Spirit holds the card (`DataKey::Held` on the token) and `karn_cards::ugin_token_left`, called from
   `zones::leave`, returns it when that token leaves in any way and never early. The old dies/leaves hooks are gone.

## Shared files changed
engine/zones.rs (put_in_gy, mill, discard_cards, discard_index, to_zone_card unearth, leave: Ugin token hook), engine/stack.rs
(countered spell -> put_in_gy), engine/cast.rs (countered spell -> put_in_gy), engine/combat.rs (calls to combat_pumps,
hellkite_pump_lethal, tez_emblem), ai/brain.rs (spell_sources/source_zone in removal_options, cast_removal, wipe_options,
cast_wipe; Liberator end-of-turn sorceries). Not touched: state.rs, cardcode.rs, hooks.rs. Others: human/human.rs, human/alt.rs,
impls/fixes.rs, impls/lands.rs, impls/karn.rs, impls/karn_cards.rs, impls/karn_engines.rs, impls/t6_godo.rs, data/card_overrides.toml,
data/cards.json (re-exported), tests/model_gaps_karn.rs, tests/practice_trigger_probe.rs.

## For the coordinator (sims before/after)
AI behaviour that changes: Factory/Foundry combat pumps; Hellkite lethal pump; Torque for Karn; Liberator + All Is Dust at end of
turn; Forge casts All Is Dust (and removal) from the top; Mirrorworks pays {2} for a Citadel / Treasure Vault copy on the land
drop (a tempo cost on that turn; an extra mana land); Blightsteel milled/discarded/countered now returns to the library;
Tezzeret may -7 (the AI values it 9 whenever it has an artifact; at 7+ loyalty it will take it over -3).
