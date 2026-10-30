# Card paths (cardgen audit --paths)

Every path a card's behaviour reaches the sim by, and the cards where two paths run the same thing. A report only; see `rust/crates/cardgen/src/paths.rs` for the classes and groups.

## Summary

| | all cards | cards the decks play |
|---|---|---|
| cards with a path | 2749 | 1906 |
| with compiled abilities | 893 | 543 |
| with executing tags | 366 | 267 |
| with code rule slots | 1680 | 1233 |
| with code AI slots only | 180 | 137 |
| double: hard | 0 | 0 |
| double: shadowed | 25 | 20 |
| double: competing | 21 | 14 |
| double: split | 161 | 122 |
| double: split (paths_ok) | 29 | 20 |
| rule code: dsl-covers | 20 | 14 |
| rule code: dsl-partial | 186 | 138 |
| rule code: dsl-none | 1474 | 1081 |

## Hard, shadowed and competing doubles

| Card | Class | Why | DSL | Executing tags | Rule slots | AI slots | Rule code |
|---|---|---|---|---|---|---|---|
| Damn | shadowed | spell + resolve (the compiled spell never runs) | spell | - | resolve | hand_options | dsl-partial |
| Drown in Ichor | shadowed | spell + resolve (the compiled spell never runs) | spell | - | resolve | prio | dsl-partial |
| Eldrazi Confluence | shadowed | spell + resolve (the compiled spell never runs) | spell | - | resolve | - | dsl-partial |
| Elemental Eruption | shadowed | spell + resolve (the compiled spell never runs) | spell | - | resolve self_cast | prio | dsl-partial |
| Entish Restoration | shadowed | spell + resolve (the compiled spell never runs) | spell | - | resolve | - | dsl-partial |
| Flare of Malice | shadowed | spell + resolve (the compiled spell never runs) | spell | - | resolve | hand_options prio | dsl-partial |
| Galadhrim Ambush | shadowed | spell + resolve (the compiled spell never runs) | spell | - | resolve | prio | dsl-partial |
| Get Lost | shadowed | spell + resolve (the compiled spell never runs) | spell | - | resolve | - | dsl-partial |
| Heroic Return | shadowed | spell + resolve (the compiled spell never runs) | spell | - | resolve self_cost | - | dsl-partial |
| Ignite the Future | shadowed | spell + resolve (the compiled spell never runs) | spell | - | resolve self_cast | - | dsl-partial |
| Infectious Bite | shadowed | spell + resolve (the compiled spell never runs) | spell | - | resolve | prio | dsl-partial |
| Lock and Load | shadowed | spell + resolve (the compiled spell never runs) | spell | - | resolve | hand_options prio | dsl-partial |
| Malakir Rebirth // Malakir Mire | shadowed | spell + resolve (the compiled spell never runs) | spell | - | resolve land_cols | hand_options prio | dsl-partial |
| Mnemonic Betrayal | shadowed | spell + resolve (the compiled spell never runs) | spell | - | resolve | - | dsl-partial |
| Phyresis Outbreak | shadowed | spell + resolve (the compiled spell never runs) | spell | - | resolve | - | dsl-partial |
| Pox | shadowed | spell + resolve (the compiled spell never runs) | spell | - | resolve | prio | dsl-partial |
| Pyretic Charge | shadowed | spell + resolve (the compiled spell never runs) | spell | - | resolve | hand_options prio | dsl-partial |
| Return Upon the Tide | shadowed | spell + resolve (the compiled spell never runs) | spell | - | resolve | - | dsl-partial |
| Soul Partition | shadowed | spell + resolve (the compiled spell never runs) | spell | - | resolve | - | dsl-partial |
| Time Wipe | shadowed | spell + resolve (the compiled spell never runs) | spell | - | resolve | - | dsl-partial |
| Desperate Ritual | shadowed | spell + resolve (the compiled spell never runs); (no deck plays it) | spell | - | resolve | prio | dsl-partial |
| Emerald Charm | shadowed | spell + resolve (the compiled spell never runs); (no deck plays it) | spell | - | resolve | - | dsl-partial |
| Kozilek's Command | shadowed | spell + resolve (the compiled spell never runs); (no deck plays it) | spell | - | resolve | - | dsl-partial |
| Sundering Eruption // Volcanic Fissure | shadowed | spell + resolve (the compiled spell never runs); (no deck plays it) | spell | - | resolve land_cols | hand_options prio | dsl-partial |
| Untimely Malfunction | shadowed | spell + resolve (the compiled spell never runs); (no deck plays it) | spell | - | resolve | - | dsl-partial |
| Buried Ruin | competing | activated + land_options | activated | - | land_etb | land_options | dsl-partial |
| Castle Embereth | competing | activated + land_options | activated | - | land_etb | land_options | dsl-partial |
| Demon of Fate's Design | competing | activated + options | activated | - | - | options | - |
| Dragon's Hoard | competing | activated + options | activated | rock(mana) | etb | options | dsl-partial |
| Ghost Quarter | competing | activated + land_options | activated | - | land_etb | land_options | dsl-partial |
| Gingerbrute | competing | activated + options | activated | - | - | options | - |
| Karn's Bastion | competing | activated + land_options | activated | - | land_etb | land_options | dsl-partial |
| Mount Doom | competing | activated + land_options | activated | - | on_tap_land dyn_mana_land land_etb | land_options | dsl-partial |
| Slayers' Stronghold | competing | activated + land_options | activated | - | land_etb | land_options | dsl-partial |
| Staff of Compleation | competing | activated + options | activated | rock(mana) | - | options | - |
| Staff of Domination | competing | activated + options | activated | - | - | options | - |
| Sunhome, Fortress of the Legion | competing | activated + land_options | activated | - | land_etb | land_options | dsl-partial |
| Vault of the Archangel | competing | activated + land_options | activated | - | land_etb | land_options | dsl-partial |
| Waterlogged Grove | competing | activated + land_options | activated | - | on_tap_land land_etb | land_options | dsl-partial |
| Boiling Rock Prison | competing | activated + land_options; (no deck plays it) | activated | - | land_etb | land_options | dsl-partial |
| Castle Locthwain | competing | activated + land_options; (no deck plays it) | activated | - | land_etb | land_options | dsl-partial |
| Horizon Canopy | competing | activated + land_options; (no deck plays it) | activated | - | on_tap_land land_etb | land_options | dsl-partial |
| Silent Clearing | competing | activated + land_options; (no deck plays it) | activated | - | on_tap_land land_etb | land_options | dsl-partial |
| Sunbaked Canyon | competing | activated + land_options; (no deck plays it) | activated | - | on_tap_land land_etb | land_options | dsl-partial |
| Tectonic Edge | competing | activated + land_options; (no deck plays it) | activated | - | land_etb | land_options | dsl-partial |
| Weathered Wayfarer | competing | activated + options; (no deck plays it) | activated | - | - | options | - |

## Declared splits (paths_ok)

| Card | Class | Why | DSL | Executing tags | Rule slots | AI slots | Rule code |
|---|---|---|---|---|---|---|---|
| Allosaurus Shepherd | split (paths_ok) | uncounterable: tag unc + uncounterable; paths_ok: the unc tag is "this spell can't be countered" (on the stack); the card code's uncounterable slot is "green spells you control can't be countered" (on the battlefield) | - | unc(cast) | uncounterable | options | dsl-none |
| Borne Upon a Wind | split (paths_ok) | spell + resolve (the compiled spell never runs); paths_ok: the hand-written resolve does the card; the compiled spell, the draw only (no flash for the turn), never resolves and stays as the AI's value of the card (dsl::card_value) until the AI phase | spell | - | resolve | - | dsl-covers |
| Calamity's Wake | split (paths_ok) | spell + resolve (the compiled spell never runs); paths_ok: the hand-written resolve does the card; the compiled spell, the graveyard exile only (no noncreature-spell lock, not exiled itself), never resolves and stays as the AI's value of the card (dsl::card_value) until the AI phase | spell | - | resolve | - | dsl-covers |
| Contaminant Grafter | split (paths_ok) | toxic: static:toxic + combat_damage; paths_ok: toxic 1 is compiled (tymna_cards::toxic_hit); the card code's combat_damage slot is the proliferate trigger, its end_step the corrupted draw | static:toxic | - | end_step combat_damage | - | dsl-partial |
| Demonic Counsel | split (paths_ok) | spell + resolve (the compiled spell never runs); paths_ok: the hand-written resolve does the card; the compiled spell, a search for any card (no Demon search, no delirium rule), never resolves and stays as the AI's value of the card (dsl::card_value) until the AI phase | spell | - | resolve | - | dsl-covers |
| Erode | split (paths_ok) | spell + resolve (the compiled spell never runs); paths_ok: the hand-written resolve does the card; the compiled spell, the destroy only (no basic land for its controller), never resolves and stays as the AI's value of the card (dsl::card_value) until the AI phase | spell | - | resolve | - | dsl-covers |
| Finale of Revelation | split (paths_ok) | spell + resolve (the compiled spell never runs); paths_ok: the hand-written resolve does the card; the compiled spell, draw X only (no X of 10 or more, not exiled), never resolves and stays as the AI's value of the card (dsl::card_value) until the AI phase | spell | - | resolve | hand_options prio | dsl-covers |
| Hexing Squelcher | split (paths_ok) | uncounterable: uncounterable + uncounterable; paths_ok: the compiled uncounterable (the unc tag) is "this spell can't be countered" (on the stack); the card code's uncounterable slot is "spells you control can't be countered" (on the battlefield) | uncounterable | unc(cast) | uncounterable | - | dsl-partial |
| Kaya's Wrath | split (paths_ok) | spell + resolve (the compiled spell never runs); paths_ok: the hand-written resolve does the card; the compiled spell, the wipe only (no life gain), never resolves and stays as the AI's value of the card (dsl::card_value) until the AI phase | spell | - | resolve | - | dsl-covers |
| Liliana's Triumph | split (paths_ok) | spell + resolve (the compiled spell never runs); paths_ok: the hand-written resolve does the card; the compiled spell, the sacrifice only (no discard with a Liliana), never resolves and stays as the AI's value of the card (dsl::card_value) until the AI phase | spell | - | resolve | - | dsl-covers |
| Natural Order | split (paths_ok) | spell + resolve (the compiled spell never runs); paths_ok: the hand-written resolve does the card; the compiled spell, any creature card (not green; the sacrifice is a note), never resolves and stays as the AI's value of the card (dsl::card_value) until the AI phase | spell | - | resolve | prio | dsl-covers |
| Open the Armory | split (paths_ok) | spell + resolve (the compiled spell never runs); paths_ok: the hand-written resolve does the card; the compiled spell, a search for any card (not an Aura or Equipment), never resolves and stays as the AI's value of the card (dsl::card_value) until the AI phase | spell | - | resolve | - | dsl-covers |
| Rite of Flame | split (paths_ok) | spell + resolve (the compiled spell never runs); paths_ok: the hand-written resolve does the card; the compiled spell, {R}{R} only (no extra {R} per Rite of Flame in graveyards), never resolves and stays as the AI's value of the card (dsl::card_value) until the AI phase | spell | - | resolve | prio | dsl-covers |
| Scout's Warning | split (paths_ok) | spell + resolve (the compiled spell never runs); paths_ok: the hand-written resolve does the card; the compiled spell, the draw only (no flash for the next creature), never resolves and stays as the AI's value of the card (dsl::card_value) until the AI phase | spell | - | resolve | - | dsl-covers |
| Starfield Mystic | split (paths_ok) | static:cost + code dies; paths_ok: the enchantment discount is compiled; the +1/+1 counter when an enchantment of yours goes to the graveyard is card code (impls/partials.rs) | static:cost | - | dies | - | dsl-partial |
| Static Net | split (paths_ok) | trigger: triggered:etb + etb; paths_ok: two enter triggers: the compiled one exiles the opponent's permanent, the card code (impls/common.rs) gains 2 life and makes the tapped Powerstone | triggered:etb | - | etb | - | dsl-covers |
| Tataru Taru | split (paths_ok) | enter: tag draw + etb; paths_ok: the enter trigger has two parts: you draw (the draw tag), the opponent may draw (the card code's etb, t6_legends2.rs) | - | draw(enter) | etb draw | - | dsl-none |
| Timetwister | split (paths_ok) | spell + resolve (the compiled spell never runs); paths_ok: the hand-written resolve does the card; the compiled spell, you draw seven only (no shuffle, the others draw nothing), never resolves and stays as the AI's value of the card (dsl::card_value) until the AI phase | spell | - | resolve | - | dsl-covers |
| Twilight Diviner | split (paths_ok) | trigger: triggered:etb + etb; paths_ok: two enter triggers: the compiled one is its own (surveil 2), the card code's etb (t12_brago_meren.rs) is the token copy when other creatures enter from a graveyard | triggered:etb | - | etb | - | dsl-partial |
| Vona's Hunger | split (paths_ok) | spell + resolve (the compiled spell never runs); paths_ok: the hand-written resolve does the card; the compiled spell, one sacrifice only (no ascend), never resolves and stays as the AI's value of the card (dsl::card_value) until the AI phase | spell | - | resolve | - | dsl-covers |
| Daxos, Blessed by the Sun | split (paths_ok) | trigger: triggered:etb + etb; paths_ok: the compiled triggers gain 1 life when another creature of yours enters or dies; the card code's etb only switches on its devotion toughness (self_pt); (no deck plays it) | triggered:dies-other triggered:etb | - | etb self_pt | - | dsl-partial |
| Elemental Mascot | split (paths_ok) | trigger: triggered:cast + cast; paths_ok: opus: the compiled cast trigger gives +1/+0, the card code (tymna_cards.rs) exiles the top card when five or more mana was spent; (no deck plays it) | triggered:cast | - | cast | - | dsl-covers |
| Heartless Act | split (paths_ok) | spell + resolve (the compiled spell never runs); paths_ok: the hand-written resolve does the card; the compiled spell, destroy any creature (no counters condition, no second mode), never resolves and stays as the AI's value of the card (dsl::card_value) until the AI phase; (no deck plays it) | spell | - | resolve | - | dsl-covers |
| Mardu Charm | split (paths_ok) | spell + resolve (the compiled spell never runs); paths_ok: the hand-written resolve does the card; the compiled spell, two modes, its first strike given to all your creatures (no discard mode), never resolves and stays as the AI's value of the card (dsl::card_value) until the AI phase; (no deck plays it) | spell | - | resolve | hand_options | dsl-covers |
| Molten Duplication | split (paths_ok) | spell + resolve (the compiled spell never runs); paths_ok: the hand-written resolve does the card; the compiled spell, haste for your creatures only (no token copy), never resolves and stays as the AI's value of the card (dsl::card_value) until the AI phase; (no deck plays it) | spell | - | resolve | prio | dsl-covers |
| Rapid Hybridization | split (paths_ok) | spell + resolve (the compiled spell never runs); paths_ok: the hand-written resolve does the card; the compiled spell, the destroy only (no Frog Lizard), never resolves and stays as the AI's value of the card (dsl::card_value) until the AI phase; (no deck plays it) | spell | - | resolve | - | dsl-covers |
| Ribskiff | split (paths_ok) | static:toxic + code etb crew; paths_ok: toxic 2 is compiled (tymna_cards::toxic_hit); the enter draw and crew are card code (impls/sheoldred.rs); (no deck plays it) | static:toxic | - | etb crew | - | dsl-partial |
| Sunfall | split (paths_ok) | spell + resolve (the compiled spell never runs); paths_ok: the hand-written resolve does the card; the compiled spell, the exile only (no Incubator), never resolves and stays as the AI's value of the card (dsl::card_value) until the AI phase; (no deck plays it) | spell | - | resolve | - | dsl-covers |
| Venomous Brutalizer | split (paths_ok) | static:toxic + code etb; paths_ok: toxic 3 is compiled (tymna_cards::toxic_hit); the enter trigger (pay {1}{G}: proliferate) is card code (impls/green_cards.rs); (no deck plays it) | static:toxic | - | etb | - | dsl-partial |

## Undeclared splits (counted as doubles until an override declares them)

| Card | Class | Why | DSL | Executing tags | Rule slots | AI slots | Rule code |
|---|---|---|---|---|---|---|---|
| Alela, Artful Provocateur | split | static:anthem + code cast | static:anthem | - | cast | - | dsl-partial |
| Alesha, Who Laughs at Fate | split | triggered:attack + code end_step | triggered:attack | - | end_step | - | dsl-partial |
| Andúril, Narsil Reforged | split | static:equip_cost + code attack | static:equip_cost | - | attack | prio | dsl-partial |
| Angel of Invention | split | static:anthem + code etb | static:anthem | - | etb | - | dsl-partial |
| Arid Mesa | split | activated + code land_etb | activated | - | land_etb | - | dsl-partial |
| Arrest | split | tag rem + code aura | - | rem(enter-static) | aura | - | dsl-none |
| Batterskull | split | static:equip_bonus static:equip_cost static:equip_keyword + code etb | static:equip_bonus static:equip_cost static:equip_keyword | - | etb | options | dsl-partial |
| Battle Angels of Tyr | split | tag treas + code combat_damage | - | treas(enter) | combat_damage | - | dsl-none |
| Blade of the Bloodchief | split | static:equip_cost + code dies | static:equip_cost | - | dies | - | dsl-partial |
| Blighted Woodland | split | activated + code land_etb | activated | - | land_etb | - | dsl-partial |
| Bloodstained Mire | split | activated + code land_etb | activated | - | land_etb | - | dsl-partial |
| Bonny Pall, Clearcutter | split | triggered:attack + code etb | triggered:attack | - | etb | - | dsl-partial |
| Boseiju, Who Endures | split | activated + code land_etb | activated | - | land_etb | hand_options | dsl-partial |
| Bound in Silence | split | tag rem + code aura | - | rem(enter-static) | aura | - | dsl-none |
| Canopy Tactician | split | static:anthem + code dyn_mana_perm | static:anthem | dork(mana) | dyn_mana_perm | - | dsl-partial |
| Captain Lannery Storm | split | triggered:attack + code sacrifice | triggered:attack | - | sacrifice | - | dsl-partial |
| Castle Ardenvale | split | activated + code land_etb | activated | - | land_etb | - | dsl-partial |
| Champions of the Perfect | split | triggered:cast + code leaves | triggered:cast | - | leaves | - | dsl-partial |
| Chatterstorm | split | spell + code self_cast | spell | - | self_cast | - | dsl-partial |
| Commercial District | split | triggered:etb + code land_etb | triggered:etb | - | land_etb | - | dsl-partial |
| Contagion Engine | split | activated + code etb | activated | - | etb | - | dsl-partial |
| Cryogen Relic | split | tag draw + code leaves | - | draw(enter) | leaves | options | dsl-none |
| Daze | split | spell + code alt_cost | spell | - | alt_cost | - | dsl-partial |
| Docent of Perfection // Final Iteration | split | triggered:cast + code etb grant_kw sba | triggered:cast | - | etb grant_kw sba | - | dsl-partial |
| Doubling Season | split | replacement:counters replacement:tokens + code etb | replacement:counters replacement:tokens | - | etb | pval | dsl-partial |
| Dour Port-Mage | split | activated + code leaves | activated | - | leaves | - | dsl-partial |
| Dragonlord Dromoka | split | tag unc + code can_cast | - | unc(cast) | can_cast | - | dsl-none |
| Dreadhorde Invasion | split | triggered:upkeep + code attack | triggered:upkeep | - | attack | - | dsl-partial |
| Elenda, the Dusk Rose | split | triggered:dies-other + code self_dies | triggered:dies-other | - | self_dies | - | dsl-partial |
| Elvish Archdruid | split | static:anthem + code etb dyn_mana_perm | static:anthem | dork(mana) | etb dyn_mana_perm | - | dsl-partial |
| Elvish Champion | split | static:anthem + code grant_kw | static:anthem | - | grant_kw | - | dsl-partial |
| Emeritus of Ideation // Ancestral Recall | split | tag prepare + code attack | - | prepare(enter-static) | attack | options | dsl-none |
| Emiel the Blessed | split | activated + code etb | activated | - | etb | - | dsl-partial |
| Encrust | split | tag rem + code aura | - | rem(enter-static) | aura | - | dsl-none |
| Ezuri, Stalker of Spheres | split | triggered:proliferate + code etb | triggered:proliferate | - | etb | - | dsl-partial |
| Firebender Ascension | split | tag tok + code attack | - | tok(enter) | attack | - | dsl-none |
| Flare of Denial | split | spell + code alt_cost | spell | - | alt_cost | - | dsl-partial |
| Flooded Strand | split | activated + code land_etb | activated | - | land_etb | - | dsl-partial |
| Goblin Chieftain | split | static:anthem + code grant_kw | static:anthem | - | grant_kw | - | dsl-partial |
| Goblin King | split | static:anthem + code grant_kw | static:anthem | - | grant_kw | - | dsl-partial |
| Grinding Station | split | activated + code etb | activated | - | etb | - | dsl-partial |
| Hangarback Walker | split | activated + code self_dies self_cost | activated | - | self_dies self_cost | prio | dsl-partial |
| Heliod, God of the Sun | split | activated static:keyword + code etb grant_kw sba | activated static:keyword | - | etb grant_kw sba | - | dsl-partial |
| Helm of Kaldra | split | static:equip_cost static:equip_keyword + code self_regen | static:equip_cost static:equip_keyword | - | self_regen | options | dsl-partial |
| Helm of the Host | split | static:equip_cost + code combat_start | static:equip_cost | - | combat_start | options pval | dsl-partial |
| Hero's Blade | split | static:equip_bonus static:equip_cost + code etb | static:equip_bonus static:equip_cost | - | etb | - | dsl-partial |
| Hidden Grotto | split | triggered:etb + code land_etb | triggered:etb | - | land_etb | - | dsl-partial |
| High Market | split | activated + code land_etb | activated | - | land_etb | - | dsl-partial |
| Hullbreaker Horror | split | tag unc + code cast | - | unc(cast) | cast | pval | dsl-none |
| Icetill Explorer | split | triggered:landfall + code extra_lands lands_from_gy | triggered:landfall | - | extra_lands lands_from_gy | - | dsl-partial |
| Inferno Titan | split | activated + code etb attack | activated | - | etb attack | - | dsl-partial |
| Iroas, God of Victory | split | static:keyword + code etb grant_kw sba | static:keyword | - | etb grant_kw sba | - | dsl-partial |
| Jungle Hollow | split | triggered:etb + code land_etb | triggered:etb | - | land_etb | - | dsl-partial |
| Karn, Scion of Urza | split | tag tok + code walker | - | tok(enter) | walker | options | dsl-none |
| Knight of the Ebon Legion | split | activated + code end_step | activated | - | end_step | - | dsl-partial |
| Koma, Cosmos Serpent | split | triggered:upkeep uncounterable tag unc + code each_combat self_regen | triggered:upkeep uncounterable | unc(cast) | each_combat self_regen | - | dsl-partial |
| Krosan Verge | split | activated + code land_etb | activated | - | land_etb | - | dsl-partial |
| Leaf-Crowned Visionary | split | static:anthem + code cast | static:anthem | - | cast | - | dsl-partial |
| Luminous Bonds | split | tag rem + code aura | - | rem(enter-static) | aura | - | dsl-none |
| Lumra, Bellow of the Woods | split | static:self_scaling + code etb | static:self_scaling | - | etb | - | dsl-partial |
| Maelstrom of the Spirit Dragon | split | activated + code land_etb | activated | - | land_etb | - | dsl-partial |
| Marsh Flats | split | activated + code land_etb | activated | - | land_etb | - | dsl-partial |
| Mirari's Wake | split | static:anthem + code land_mana | static:anthem | - | land_mana | - | dsl-partial |
| Misty Rainforest | split | activated + code land_etb | activated | - | land_etb | - | dsl-partial |
| Mossborn Hydra | split | triggered:landfall + code as_enters | triggered:landfall | - | as_enters | - | dsl-partial |
| Myr Battlesphere | split | triggered:etb + code attack | triggered:etb | - | attack | - | dsl-partial |
| Myriad Landscape | split | activated + code land_etb | activated | - | land_etb | - | dsl-partial |
| Neriv, Crackling Vanguard | split | triggered:etb + code attack | triggered:etb | - | attack | - | dsl-partial |
| Nyxborn Behemoth | split | activated + code self_cost | activated | - | self_cost | - | dsl-partial |
| Otawara, Soaring City | split | activated + code land_etb | activated | - | land_etb | hand_options | dsl-partial |
| Paladin Class | split | static:anthem + code attack cost | static:anthem | - | attack cost | options | dsl-partial |
| Pawn of Ulamog | split | tag tok + code dies | - | tok(enter) | dies | - | dsl-none |
| Polluted Delta | split | activated + code land_etb | activated | - | land_etb | - | dsl-partial |
| Prismatic Vista | split | activated + code land_etb | activated | - | land_etb | - | dsl-partial |
| Prison Sentence | split | tag rem + code aura | - | rem(enter-static) | aura | - | dsl-none |
| Radiant Fountain | split | triggered:etb + code land_etb | triggered:etb | - | land_etb | - | dsl-partial |
| Rain of Riches | split | triggered:etb + code cast | triggered:etb | - | cast | - | dsl-partial |
| Reliquary Tower | split | static:no_max_hand + code land_etb | static:no_max_hand | - | land_etb | - | dsl-partial |
| Rhys the Exiled | split | triggered:attack + code self_regen | triggered:attack | - | self_regen | - | dsl-partial |
| Sakura-Tribe Elder | split | tag lr + code blocks | - | lr(perm-resolve) | blocks | options | dsl-none |
| Sandwurm Convergence | split | triggered:end_step + code attack_cap | triggered:end_step | - | attack_cap | - | dsl-partial |
| Sangromancer | split | triggered:dies-other + code discard | triggered:dies-other | - | discard | - | dsl-partial |
| Scalding Tarn | split | activated + code land_etb | activated | - | land_etb | - | dsl-partial |
| Scoured Barrens | split | triggered:etb + code land_etb | triggered:etb | - | land_etb | - | dsl-partial |
| Seraph Sanctuary | split | triggered:etb + code land_etb | triggered:etb | - | land_etb | - | dsl-partial |
| Shield of Kaldra | split | static:equip_cost static:equip_keyword + code self_regen | static:equip_cost static:equip_keyword | - | self_regen | - | dsl-partial |
| Sifter of Skulls | split | tag tok + code dies | - | tok(enter) | dies | - | dsl-none |
| Skemfar Elderhall | split | activated + code land_etb | activated | - | land_etb | - | dsl-partial |
| Skyclave Apparition | split | tag rem + code leaves | - | rem(enter) | leaves | - | dsl-none |
| Snapback | split | spell + code alt_cost | spell | - | alt_cost | - | dsl-partial |
| Soul of Windgrace | split | activated + code etb attack | activated | - | etb attack | - | dsl-partial |
| Sowing Mycospawn | split | tag lr + code self_cast | - | lr(perm-resolve) | self_cast | - | dsl-none |
| Springheart Nantuko | split | tag tok + code landfall | - | tok(enter) | landfall | - | dsl-none |
| Sting, the Glinting Dagger | split | static:equip_bonus static:equip_cost static:equip_keyword + code each_combat | static:equip_bonus static:equip_cost static:equip_keyword | - | each_combat | - | dsl-partial |
| Sunscour | split | spell + code alt_cost | spell | - | alt_cost | - | dsl-partial |
| Sword of Kaldra | split | static:equip_bonus static:equip_cost + code self_regen | static:equip_bonus static:equip_cost | - | self_regen | - | dsl-partial |
| Tainted Observer | split | static:toxic + code etb | static:toxic | - | etb | - | dsl-partial |
| Takenuma, Abandoned Mire | split | activated + code land_etb | activated | - | land_etb | - | dsl-partial |
| Temple of Enlightenment | split | triggered:etb + code land_etb | triggered:etb | - | land_etb | - | dsl-partial |
| Temple of Malady | split | triggered:etb + code land_etb | triggered:etb | - | land_etb | - | dsl-partial |
| Temple of Mystery | split | triggered:etb + code land_etb | triggered:etb | - | land_etb | - | dsl-partial |
| Temple of Plenty | split | triggered:etb + code land_etb | triggered:etb | - | land_etb | - | dsl-partial |
| Temple of Silence | split | triggered:etb + code land_etb | triggered:etb | - | land_etb | - | dsl-partial |
| Temple of Triumph | split | triggered:etb + code land_etb | triggered:etb | - | land_etb | - | dsl-partial |
| The Meathook Massacre | split | triggered:dies-other + code etb | triggered:dies-other | - | etb | prio | dsl-partial |
| The Raven Man | split | activated + code end_step | activated | - | end_step | - | dsl-partial |
| Thornbite Staff | split | static:equip_cost + code etb | static:equip_cost | - | etb | pval | dsl-partial |
| Thornwood Falls | split | triggered:etb + code land_etb | triggered:etb | - | land_etb | - | dsl-partial |
| Thought Monitor | split | triggered:etb + code self_cost | triggered:etb | - | self_cost | - | dsl-partial |
| Thoughtcast | split | spell + code self_cost | spell | - | self_cost | - | dsl-partial |
| Thran Power Suit | split | static:equip_cost + code etb | static:equip_cost | - | etb | - | dsl-partial |
| Treasure Vault | split | activated + code land_etb | activated | - | land_etb | - | dsl-partial |
| Undercity Sewers | split | triggered:etb + code land_etb | triggered:etb | - | land_etb | - | dsl-partial |
| Underground Mortuary | split | triggered:etb + code land_etb | triggered:etb | - | land_etb | - | dsl-partial |
| Urza's Blueprints | split | activated + code upkeep | activated | - | upkeep | - | dsl-partial |
| Urza's Factory | split | activated + code land_etb | activated | - | land_etb | - | dsl-partial |
| Verdant Catacombs | split | activated + code land_etb | activated | - | land_etb | - | dsl-partial |
| Vexing Bauble | split | activated + code cast_counter | activated | - | cast_counter | - | dsl-partial |
| Vito, Thorn of the Dusk Rose | split | activated + code gain_life | activated | - | gain_life | - | dsl-partial |
| Weaver of Harmony | split | static:anthem + code cast | static:anthem | - | cast | - | dsl-partial |
| Windswept Heath | split | activated + code land_etb | activated | - | land_etb | - | dsl-partial |
| Wooded Foothills | split | activated + code land_etb | activated | - | land_etb | - | dsl-partial |
| Andúril, Flame of the West | split | static:equip_bonus static:equip_cost + code attack; (no deck plays it) | static:equip_bonus static:equip_cost | - | attack | - | dsl-partial |
| Archfiend of Sorrows | split | triggered:etb + code end_step; (no deck plays it) | triggered:etb | - | end_step | gy_options | dsl-partial |
| Beastmaster Ascension | split | static:bma_anthem + code attack; (no deck plays it) | static:bma_anthem | - | attack | pval | dsl-partial |
| Boromir, Warden of the Tower | split | activated + code cast_counter; (no deck plays it) | activated | - | cast_counter | - | dsl-partial |
| Boughside Wanderers | split | triggered:landfall + code etb; (no deck plays it) | triggered:landfall | - | etb | - | dsl-partial |
| Call the Spirit Dragons | split | static:keyword + code upkeep; (no deck plays it) | static:keyword | - | upkeep | - | dsl-partial |
| Cleric Class | split | static:lifegain_plus + code gain_life; (no deck plays it) | static:lifegain_plus | - | gain_life | options | dsl-partial |
| Coerced to Kill | split | tag rem + code aura; (no deck plays it) | - | rem(enter-static) | aura | - | dsl-none |
| Conformer Shuriken | split | static:equip_cost + code attack; (no deck plays it) | static:equip_cost | - | attack | - | dsl-partial |
| Dragoon's Lance | split | static:equip_bonus static:equip_cost + code etb grant_kw; (no deck plays it) | static:equip_bonus static:equip_cost | - | etb grant_kw | - | dsl-partial |
| Embercleave | split | static:equip_bonus static:equip_cost static:equip_keyword + code etb hand_attack; (no deck plays it) | static:equip_bonus static:equip_cost static:equip_keyword | - | etb hand_attack | prio | dsl-partial |
| Explorer's Scope | split | static:equip_cost + code attack; (no deck plays it) | static:equip_cost | - | attack | - | dsl-partial |
| Ghalta the Unstoppable | split | static:keyword + code self_cost; (no deck plays it) | static:keyword | - | self_cost | - | dsl-partial |
| Hammer of Nazahn | split | static:equip_bonus static:equip_cost static:equip_keyword + code etb; (no deck plays it) | static:equip_bonus static:equip_cost static:equip_keyword | - | etb | - | dsl-partial |
| Hanweir Battlements | split | activated + code land_etb; (no deck plays it) | activated | - | land_etb | - | dsl-partial |
| Hero's Heirloom | split | static:equip_bonus static:equip_cost + code grant_kw; (no deck plays it) | static:equip_bonus static:equip_cost | - | grant_kw | - | dsl-partial |
| Kasmina's Transmutation | split | tag rem + code aura; (no deck plays it) | - | rem(enter-static) | aura | - | dsl-none |
| Legion's Landing // Adanto, the First Fort | split | tag tok + code attack; (no deck plays it) | - | tok(enter) | attack | - | dsl-none |
| Maelstrom Wanderer | split | static:keyword + code self_cast; (no deck plays it) | static:keyword | - | self_cast | - | dsl-partial |
| Magitek Scythe | split | static:equip_bonus static:equip_cost + code etb; (no deck plays it) | static:equip_bonus static:equip_cost | - | etb | prio | dsl-partial |
| Mirri, Weatherlight Duelist | split | static:self_keyword + code attack_cap blocks; (no deck plays it) | static:self_keyword | - | attack_cap blocks | - | dsl-partial |
| Mithril Coat | split | static:equip_cost static:equip_keyword static:self_keyword + code etb; (no deck plays it) | static:equip_cost static:equip_keyword static:self_keyword | - | etb | - | dsl-partial |
| Moseo, Vein's New Dean | split | tag tok + code end_step; (no deck plays it) | - | tok(enter) | end_step | - | dsl-none |
| Myrel, Shield of Argive | split | triggered:attack + code can_cast; (no deck plays it) | triggered:attack | - | can_cast | - | dsl-partial |
| Orysa, Tide Choreographer | split | triggered:etb + code self_cost; (no deck plays it) | triggered:etb | - | self_cost | - | dsl-partial |
| Postmortem Professor | split | static:cant_block + code attack; (no deck plays it) | static:cant_block | - | attack | gy_options | dsl-partial |
| Requiem Angel | split | tag tok + code dies; (no deck plays it) | - | tok(enter) | dies | - | dsl-none |
| Room of Refuge | split | activated + code land_etb; (no deck plays it) | activated | - | land_etb | - | dsl-partial |
| Samurai's Katana | split | static:equip_bonus static:equip_cost static:equip_keyword + code etb; (no deck plays it) | static:equip_bonus static:equip_cost static:equip_keyword | - | etb | - | dsl-partial |
| Sinister Hideout | split | activated + code land_etb; (no deck plays it) | activated | - | land_etb | - | dsl-partial |
| Smaug the Magnificent | split | triggered:upkeep + code attack; (no deck plays it) | triggered:upkeep | - | attack | - | dsl-partial |
| Squire's Lightblade | split | static:equip_bonus static:equip_cost + code etb; (no deck plays it) | static:equip_bonus static:equip_cost | - | etb | prio | dsl-partial |
| Starbreach Whale | split | triggered:etb + code end_step; (no deck plays it) | triggered:etb | - | end_step | hand_options | dsl-partial |
| Temple of Deceit | split | triggered:etb + code land_etb; (no deck plays it) | triggered:etb | - | land_etb | - | dsl-partial |
| Temple of Malice | split | triggered:etb + code land_etb; (no deck plays it) | triggered:etb | - | land_etb | - | dsl-partial |
| Textbook Tabulator | split | triggered:etb + code cast; (no deck plays it) | triggered:etb | - | cast | - | dsl-partial |
| Voice of the Blessed | split | triggered:gain_life + code grant_kw; (no deck plays it) | triggered:gain_life | - | grant_kw | - | dsl-partial |
| White Mage's Staff | split | static:equip_bonus static:equip_cost triggered:attack + code etb; (no deck plays it) | static:equip_bonus static:equip_cost triggered:attack | - | etb | - | dsl-partial |
| Zhalfirin Void | split | triggered:etb + code land_etb; (no deck plays it) | triggered:etb | - | land_etb | - | dsl-partial |

## dsl-covers: card code the compiled abilities may replace (verify by hand)

- Borne Upon a Wind: code resolve; compiled spell
- Calamity's Wake: code resolve; compiled spell
- Demonic Counsel: code resolve; compiled spell
- Elemental Mascot: code cast; compiled triggered:cast (no deck plays it)
- Erode: code resolve; compiled spell
- Finale of Revelation: code resolve; compiled spell
- Heartless Act: code resolve; compiled spell (no deck plays it)
- Kaya's Wrath: code resolve; compiled spell
- Liliana's Triumph: code resolve; compiled spell
- Mardu Charm: code resolve; compiled spell (no deck plays it)
- Molten Duplication: code resolve; compiled spell (no deck plays it)
- Natural Order: code resolve; compiled spell
- Open the Armory: code resolve; compiled spell
- Rapid Hybridization: code resolve; compiled spell (no deck plays it)
- Rite of Flame: code resolve; compiled spell
- Scout's Warning: code resolve; compiled spell
- Static Net: code etb; compiled triggered:etb
- Sunfall: code resolve; compiled spell (no deck plays it)
- Timetwister: code resolve; compiled spell
- Vona's Hunger: code resolve; compiled spell

## Path 5: card names as string literals in engine/, ai/ and cardcode.rs (inventory)

447 literals naming 279 distinct cards (test modules left out)

| File | Literals | Distinct cards |
|---|---|---|
| ai/decks.rs | 72 | 47 |
| ai/plans.rs | 63 | 48 |
| ai/seph.rs | 48 | 34 |
| engine/turn.rs | 41 | 31 |
| ai/pool.rs | 37 | 29 |
| ai/veyran.rs | 37 | 31 |
| engine/stack.rs | 22 | 18 |
| engine/zones.rs | 20 | 18 |
| engine/mana.rs | 19 | 18 |
| ai/mod.rs | 17 | 17 |
| ai/evdiag.rs | 13 | 13 |
| ai/search.rs | 12 | 12 |
| engine/combat.rs | 11 | 11 |
| cardcode.rs | 9 | 9 |
| engine/cast.rs | 6 | 6 |
| ai/t6_table.rs | 5 | 4 |
| engine/removal.rs | 5 | 5 |
| ai/planner.rs | 3 | 3 |
| engine/values.rs | 3 | 3 |
| ai/brain.rs | 2 | 2 |
| engine/tutors.rs | 2 | 1 |
