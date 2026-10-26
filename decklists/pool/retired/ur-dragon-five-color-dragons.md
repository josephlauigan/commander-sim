# The Ur-Dragon — Five-Color Dragons

Opponent pool deck — **Tier 5: High Bracket 4**. Axis: Dragon tribal ramp and combat.

- **Commander:** The Ur-Dragon
- **Color identity:** White, Blue, Black, Red, Green
- **Game Changers (7):** Cyclonic Rift, Demonic Tutor, Rhystic Study, Smothering Tithe, Teferi's Protection, Vampiric Tutor, Worldly Tutor
- **Combos:** none of two or three cards: the deck wins in combat. Call the Spirit Dragons is an alternate win (at your upkeep, +1/+1 counters on five Dragons of different colours).
- **Lands:** 35 (28 nonbasic + 7 basic)

## Role in the pool

The pool's big-flier deck: five-colour ramp into Dragons that cost less or nothing, card draw from attacking, and an alternate win. It replaced Winota at Tier 5, and it tests whether a deck can race or answer a board of 6/6 fliers from turn four or five.

## At the table

- **Plan:** Ramp (Birds of Paradise, Bloom Tender, Sol Ring, Arcane Signet, Chromatic Lantern, Fellwar Stone, Mox Jasper, Farseek, Nature's Lore, Three Visits, Cultivate, Kodama's Reach, Smothering Tithe) and cost cutters: The Ur-Dragon's eminence works from the command zone, with Dragonlord's Servant, Dragonspeaker Shaman, Sarkhan, Soul Aflame, Urza's Incubator, Herald's Horn and Morophon (which takes off all the coloured mana). Dracogenesis makes every Dragon free.
- **Wins by:** Flying damage. The Ur-Dragon draws a card per attacking Dragon and puts a permanent from hand onto the battlefield; Hellkite Courser brings it in early with haste. The Ancient Dragons roll a d20 on combat damage (Treasures, Faerie Dragons, cards, or creatures from every graveyard), Miirym and Lathliss double the Dragons that enter, Utvara Hellkite makes 6/6s, Atarka gives double strike, Terror of the Peaks, Scourge of Valkas and Dragon Tempest burn as Dragons enter, and Twinflame Tyrant doubles it all. Call the Spirit Dragons wins outright at upkeep with five Dragons of different colours.
- **Watch out for:** Tiamat (five Dragons from the library), Ureni (a Dragon from the top eight on entering and attacking), Rivaz recasting a Dragon from the graveyard each turn. Call the Spirit Dragons makes the Dragons indestructible, Rith gives them ward {2}, and Roaming Throne doubles their triggers. Crux of Fate kills only your non-Dragon creatures.
- **Interaction:** Swords to Plowshares, Path to Exile, Cyclonic Rift, Crux of Fate, Mana Drain, Heroic Intervention, Teferi's Protection, Lightning Greaves; Dragonlord Dromoka stops spells on its turn.

## Strategy

Ramp for two or three turns, then cast Dragons at a discount: The Ur-Dragon's {1} off from the command zone stacks with the Shamans, Sarkhan, Urza's Incubator, Herald's Horn and Morophon, and Dracogenesis removes the cost altogether. Every attack with Dragons draws cards through The Ur-Dragon and puts another permanent in for free, so the board grows faster than opponents can answer one threat at a time. Wipes are the weak point (Teferi's Protection, Heroic Intervention and Call the Spirit Dragons' indestructibility cover them), and Tiamat, Rivaz and Haven of the Spirit Dragon refill afterwards.

## Sim modeling notes

Model these first; if any is Partial or unmodeled, this deck plays below its real strength and its results should be flagged.

The Ur-Dragon's eminence and attack trigger, the cost cutters (Dragonlord's Servant, Dragonspeaker Shaman, Sarkhan, Urza's Incubator, Morophon, Dracogenesis), the Ancient Dragons' d20 triggers, Miirym and Lathliss, Hellkite Courser, Tiamat, Ureni, Twinflame Tyrant, Call the Spirit Dragons' win, Rivaz's graveyard casts and Dragon-only mana. All of these are modeled (impls/t45_urdragon.rs); The Ur-Dragon, Klauth, Rith, Rivaz, Roaming Throne, Sarkhan and Scion of the Ur-Dragon are Approximate (data/audit.toml says what is left out: Sarkhan's and Scion's copies, Rith's excess-damage test).

## Decklist by type (100)

**Commander (1).** The Ur-Dragon

**Creatures (31).** Ancient Brass Dragon, Ancient Copper Dragon, Ancient Gold Dragon, Ancient Silver Dragon, Atarka, World Render, Balefire Dragon, Birds of Paradise, Bloom Tender, Dragonlord Dromoka, Dragonlord's Servant, Dragonspeaker Shaman, Goldspan Dragon, Hellkite Courser, Klauth, Unrivaled Ancient, Lathliss, Dragon Queen, Miirym, Sentinel Wyrm, Morophon, the Boundless, Old Gnawbone, Rith, Liberated Primeval, Rivaz of the Claw, Roaming Throne, Sarkhan, Soul Aflame, Scion of Draco, Scion of the Ur-Dragon, Scourge of Valkas, Terror of the Peaks, Tiamat, Twinflame Tyrant, Two-Headed Hellkite, Ureni of the Unwritten, Utvara Hellkite

**Enchantments (8).** Call the Spirit Dragons, Dracogenesis, Dragon Tempest, Garruk's Uprising, Rhystic Study, Rhythm of the Wild, Smothering Tithe, Temur Ascendancy

**Artifacts (9).** Arcane Signet, Chromatic Lantern, Dragon's Hoard, Fellwar Stone, Herald's Horn, Lightning Greaves, Mox Jasper, Sol Ring, Urza's Incubator

**Sorceries (7).** Crux of Fate, Cultivate, Demonic Tutor, Farseek, Kodama's Reach, Nature's Lore, Three Visits

**Instants (8).** Cyclonic Rift, Heroic Intervention, Mana Drain, Path to Exile, Swords to Plowshares, Teferi's Protection, Vampiric Tutor, Worldly Tutor

**Lands (35).** Arid Mesa, Blood Crypt, Bloodstained Mire, Breeding Pool, Cavern of Souls, Command Tower, Exotic Orchard, Flooded Strand, Godless Shrine, Hallowed Fountain, Haven of the Spirit Dragon, Ketria Triome, Maelstrom of the Spirit Dragon, Marsh Flats, Misty Rainforest, Overgrown Tomb, Path of Ancestry, Polluted Delta, Reflecting Pool, Sacred Foundry, Scalding Tarn, Steam Vents, Stomping Ground, Temple Garden, Verdant Catacombs, Watery Grave, Windswept Heath, Wooded Foothills, 1 Plains, 2 Mountain, 2 Forest, 1 Island, 1 Swamp

**Planeswalkers (1).** Sarkhan Unbroken

## Import list (100)

```
1 Ancient Brass Dragon
1 Ancient Copper Dragon
1 Ancient Gold Dragon
1 Ancient Silver Dragon
1 Arcane Signet
1 Arid Mesa
1 Atarka, World Render
1 Balefire Dragon
1 Birds of Paradise
1 Blood Crypt
1 Bloodstained Mire
1 Bloom Tender
1 Breeding Pool
1 Call the Spirit Dragons
1 Cavern of Souls
1 Chromatic Lantern
1 Command Tower
1 Crux of Fate
1 Cultivate
1 Cyclonic Rift
1 Demonic Tutor
1 Dracogenesis
1 Dragon Tempest
1 Dragon's Hoard
1 Dragonlord Dromoka
1 Dragonlord's Servant
1 Dragonspeaker Shaman
1 Exotic Orchard
1 Farseek
1 Fellwar Stone
1 Flooded Strand
1 Garruk's Uprising
1 Godless Shrine
1 Goldspan Dragon
1 Hallowed Fountain
1 Haven of the Spirit Dragon
1 Hellkite Courser
1 Herald's Horn
1 Heroic Intervention
1 Ketria Triome
1 Klauth, Unrivaled Ancient
1 Kodama's Reach
1 Lathliss, Dragon Queen
1 Lightning Greaves
1 Maelstrom of the Spirit Dragon
1 Mana Drain
1 Marsh Flats
1 Miirym, Sentinel Wyrm
1 Misty Rainforest
1 Morophon, the Boundless
1 Mox Jasper
1 Nature's Lore
1 Old Gnawbone
1 Overgrown Tomb
1 Path of Ancestry
1 Path to Exile
1 Polluted Delta
1 Reflecting Pool
1 Rhystic Study
1 Rhythm of the Wild
1 Rith, Liberated Primeval
1 Rivaz of the Claw
1 Roaming Throne
1 Sacred Foundry
1 Sarkhan Unbroken
1 Sarkhan, Soul Aflame
1 Scalding Tarn
1 Scion of Draco
1 Scion of the Ur-Dragon
1 Scourge of Valkas
1 Smothering Tithe
1 Sol Ring
1 Steam Vents
1 Stomping Ground
1 Swords to Plowshares
1 Teferi's Protection
1 Temple Garden
1 Temur Ascendancy
1 Terror of the Peaks
1 The Ur-Dragon
1 Three Visits
1 Tiamat
1 Twinflame Tyrant
1 Two-Headed Hellkite
1 Ureni of the Unwritten
1 Urza's Incubator
1 Utvara Hellkite
1 Vampiric Tutor
1 Verdant Catacombs
1 Watery Grave
1 Windswept Heath
1 Wooded Foothills
1 Worldly Tutor
1 Plains
2 Mountain
2 Forest
1 Island
1 Swamp
```
