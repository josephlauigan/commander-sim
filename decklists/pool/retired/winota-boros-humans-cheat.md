# Winota — Boros Humans & Hatebears

Opponent pool deck — **Tier 5: High Bracket 4**. Axis: Aggro-cheat with stax.

- **Commander:** Winota, Joiner of Forces
- **Color identity:** White, Red
- **Game Changers (2):** Chrome Mox, Drannith Magistrate
- **Combos:** Rionya, Fire Dancer + Combat Celebrant (a hasty Celebrant copy each combat: infinite combats). Extra combats from Breath of Fury and Combat Celebrant; Windcrag Siege doubles Winota's triggers.
- **Lands:** 32 (16 nonbasic + 16 basic), plus Sundering Eruption and Witch Enchanter as land faces

## Role in the pool

Turn-three board explosion that also locks down commanders, free spells and graveyards (Magus of the Moon hits every multicolor deck you own).

## At the table

- **Plan:** Fast mana (Sol Ring, Chrome Mox, Lotus Petal, Simian Spirit Guide) puts Winota, Joiner of Forces down turn two or three, with cheap non-Humans like Ornithopter, Signal Pest, Gingerbrute, Phoenix Chick and Goblin tokens (Goblin Rabblemaster, Legion Warboss, Loyal Apprentice) to attack. Hatebears slow you down.
- **Wins by:** Winota puts a Human such as Greymond, Avacyn's Stalwart, Lena, Myrel, Drannith Magistrate or Angrath's Marauders onto the battlefield attacking, indestructible, for each non-Human attacker (twice each with Windcrag Siege). Breath of Fury and Combat Celebrant add combats, and Rionya, Fire Dancer + Combat Celebrant is an infinite-combat combo.
- **Watch out for:** A turn-three board explosion. Magus of the Moon turns off your nonbasic lands, Drannith Magistrate stops your commander, Thalia taxes you, Boromir counters free spells, High Noon and Eidolon of Rhetoric allow one spell a turn, Myrel and Grand Abolisher shut you out on its turn, and Soulless Jailer stops reanimation. Kill Winota or the small attacker before combat.
- **Interaction:** Swords to Plowshares, Path to Exile, Boros Charm, Red Elemental Blast, Silence, High Noon (5 damage), Sundering Eruption (a land, and ground creatures can't block). Mother of Runes, Giver of Runes, Selfless Spirit, Boromir, Lena, Flawless Maneuver, Flare of Fortitude and Deflecting Swat protect its board. No counterspells.

## Strategy

Attack with cheap non-Humans (Ornithopter, Signal Pest, Gingerbrute, Phoenix Chick, Ragavan, goblin token makers) and Winota puts a Human from the top six onto the battlefield attacking and indestructible for each one; Windcrag Siege (Mardu) makes each of those triggers happen twice. The Humans are hatebears and payoffs: Drannith Magistrate, both Thalias, Grand Abolisher, Myrel, Magus of the Moon, Greymond (first strike, lifelink, +2/+2 with four Humans), Lena, Angrath's Marauders (double damage), Alexios. Breath of Fury on a small attacker trades it for another combat each time it connects, Combat Celebrant exerts for one, and Rionya copying Combat Celebrant every combat is the infinite-combat finish. Fast mana gets Winota down on turn two or three.

## Sim modeling notes

Model these first; if any is Partial or unmodeled, this deck plays below its real strength and its results should be flagged.

Winota's attack trigger (non-Human attacks → Human from top 6, indestructible this turn) and Windcrag Siege's doubling; extra combats (Breath of Fury, Combat Celebrant, the Rionya combo); stax pieces (taxes, Drannith, Magus of the Moon, High Noon, Myrel, Boromir, Soulless Jailer); protection (Flare of Fortitude, Lena, Boromir, Selfless Spirit). The new list's cards are in impls/t45_winota.rs (Archivist of Oghma and Eidolon of Rhetoric are modelled with other pool decks' cards).

## Decklist by type (100)

**Commander (1).** Winota, Joiner of Forces

**Creatures (47).** Alexios, Deimos of Kosmos, Alseid of Life's Bounty, Angrath's Marauders, Archivist of Oghma, Archon of Emeria, Aven Mindcensor, Blade Historian, Boromir, Warden of the Tower, Cathar Commando, Combat Celebrant, Drannith Magistrate, Eidolon of Rhetoric, Esper Sentinel, Ethersworn Canonist, Gingerbrute, Giver of Runes, Goblin Rabblemaster, Goldnight Commander, Grand Abolisher, Greymond, Avacyn's Stalwart, Hope of Ghirapur, Imperial Recruiter, Legion Warboss, Lena, Selfless Champion, Loyal Apprentice, Magus of the Moon, Mother of Runes, Myrel, Shield of Argive, Ornithopter, Ornithopter of Paradise, Phoenix Chick, Phyrexian Revoker, Phyrexian Walker, Professional Face-Breaker, Ragavan, Nimble Pilferer, Rionya, Fire Dancer, Rograkh, Son of Rohgahh, Selfless Spirit, Signal Pest, Simian Spirit Guide, Skrelv, Defector Mite, Soulless Jailer, Spirit of the Labyrinth, Thalia, Guardian of Thraben, Thalia, Heretic Cathar, Voice of Victory, Witch Enchanter // Witch-Blessed Meadow

**Enchantments (4).** Breath of Fury, Deafening Silence, High Noon, Windcrag Siege

**Artifacts (6).** Arcane Signet, Boros Signet, Chrome Mox, Lotus Petal, Sol Ring, Talisman of Conviction

**Sorceries (2).** Shatterskull Smashing, Sundering Eruption // Volcanic Fissure

**Instants (8).** Boros Charm, Deflecting Swat, Flare of Fortitude, Flawless Maneuver, Path to Exile, Red Elemental Blast, Silence, Swords to Plowshares

**Lands (32).** Arid Mesa, Battlefield Forge, Blinkmoth Nexus, Cavern of Souls, Clifftop Retreat, Command Tower, Eiganjo, Seat of the Empire, Exotic Orchard, Furycalm Snarl, Mishra's Factory, Needleverge Pathway // Pillarverge Pathway, Rugged Prairie, Sacred Foundry, Sokenzan, Crucible of Defiance, Spectator Seating, Sunbaked Canyon, 8 Plains, 8 Mountain

## Import list (100)

```
1 Alexios, Deimos of Kosmos
1 Alseid of Life's Bounty
1 Angrath's Marauders
1 Arcane Signet
1 Archivist of Oghma
1 Archon of Emeria
1 Arid Mesa
1 Aven Mindcensor
1 Battlefield Forge
1 Blade Historian
1 Blinkmoth Nexus
1 Boromir, Warden of the Tower
1 Boros Charm
1 Boros Signet
1 Breath of Fury
1 Cathar Commando
1 Cavern of Souls
1 Chrome Mox
1 Clifftop Retreat
1 Combat Celebrant
1 Command Tower
1 Deafening Silence
1 Deflecting Swat
1 Drannith Magistrate
1 Eidolon of Rhetoric
1 Eiganjo, Seat of the Empire
1 Esper Sentinel
1 Ethersworn Canonist
1 Exotic Orchard
1 Flare of Fortitude
1 Flawless Maneuver
1 Furycalm Snarl
1 Gingerbrute
1 Giver of Runes
1 Goblin Rabblemaster
1 Goldnight Commander
1 Grand Abolisher
1 Greymond, Avacyn's Stalwart
1 High Noon
1 Hope of Ghirapur
1 Imperial Recruiter
1 Legion Warboss
1 Lena, Selfless Champion
1 Lotus Petal
1 Loyal Apprentice
1 Magus of the Moon
1 Mishra's Factory
1 Mother of Runes
1 Myrel, Shield of Argive
1 Needleverge Pathway // Pillarverge Pathway
1 Ornithopter
1 Ornithopter of Paradise
1 Path to Exile
1 Phoenix Chick
1 Phyrexian Revoker
1 Phyrexian Walker
1 Professional Face-Breaker
1 Ragavan, Nimble Pilferer
1 Red Elemental Blast
1 Rionya, Fire Dancer
1 Rograkh, Son of Rohgahh
1 Rugged Prairie
1 Sacred Foundry
1 Selfless Spirit
1 Shatterskull Smashing
1 Signal Pest
1 Silence
1 Simian Spirit Guide
1 Skrelv, Defector Mite
1 Sokenzan, Crucible of Defiance
1 Sol Ring
1 Soulless Jailer
1 Spectator Seating
1 Spirit of the Labyrinth
1 Sunbaked Canyon
1 Sundering Eruption // Volcanic Fissure
1 Swords to Plowshares
1 Talisman of Conviction
1 Thalia, Guardian of Thraben
1 Thalia, Heretic Cathar
1 Voice of Victory
1 Windcrag Siege
1 Winota, Joiner of Forces
1 Witch Enchanter // Witch-Blessed Meadow
8 Plains
8 Mountain
```
