# Meren of Clan Nel Toth — Golgari Recursion

Opponent pool deck — **Tier 2: Mid Bracket 3**. Axis: Grind / recursion value.

- **Commander:** Meren of Clan Nel Toth
- **Color identity:** Black, Green
- **Game Changers (1):** Demonic Tutor
- **Combos:** Yawgmoth, Thran Physician + Mikaeus, the Unhallowed + Blood Artist or Zulaport Cutthroat (three cards and two non-Human creatures: each sacrifice returns by undying and drains the table). No two-card infinite.
- **Lands:** 35 (17 nonbasic + 18 basic)

## Role in the pool

Long games. Punishes decks that trade one-for-one and have no graveyard hate: every removal creature comes back. Bojuka Bog and Deathrite Shaman also interact with Sephiroth's yard.

## At the table

- **Plan:** Develops mana (Birds of Paradise, Llanowar Elves, Elvish Mystic, Elves of Deep Shadow, Sakura-Tribe Elder, Wood Elves, Cultivate) and plays value creatures. Meren returns a creature from the graveyard each end step; sacrifice outlets (Viscera Seer, Carrion Feeder, Ashnod's Altar, Phyrexian Tower, Yawgmoth, Warren Soultrader) feed her experience. Buried Alive, Entomb and Satyr Wayfinder fill the graveyard for Reanimate, Animate Dead, Victimize and Living Death; Twilight Diviner copies whatever comes back.
- **Wins by:** Drains: Gray Merchant of Asphodel, Blood Artist, Zulaport Cutthroat, Syr Konrad and Cauldron of Essence; Massacre Wurm and Grave Pact grind the opponents' boards away; Yawgmoth with Mikaeus and a drain payoff ends it at once.
- **Watch out for:** Trading one-for-one loses to Meren, so exile the graveyard or kill Meren (her tax grows). Ravenous Chupacabra, Shriekmaw, Plaguecrafter and Fleshbag Marauder return again and again. Grave Pact and Dictate of Erebos turn each of its sacrifices into one of yours. Journey to Eternity brings its creature back and becomes Atzal, a land that reanimates.
- **Interaction:** Ravenous Chupacabra, Shriekmaw, Plaguecrafter, Fleshbag Marauder, Accursed Marauder, Massacre Wurm, Haywire Mite, Reclamation Sage, Cankerbloom, Assassin's Trophy, Beast Within, Putrefy, Toxic Deluge, Grave Pact, Dictate of Erebos and Bojuka Bog. No counterspells.

## Strategy

Every creature is a value ETB or a death payoff, every sacrifice is an experience counter, and Meren returns one to the battlefield each end step. Birthing Pod, Eldritch Evolution, Fiend Artisan, Demonic Tutor and Diabolic Intent find the piece; Skullclamp and Grim Haruspex draw off the deaths; Protean Hulk and Sheoldred, Whispering One are the top end. Wight of the Reliquary and Fiend Artisan grow with the graveyard.

## Sim modeling notes

Model these first; if any is Partial or unmodeled, this deck plays below its real strength and its results should be flagged.

Meren's end-step reanimation (to battlefield if MV ≤ experience, else to hand); ETB removal re-use; sac outlets and Grave Pact / Dictate of Erebos edicts; graveyard filling (Buried Alive, Entomb, Satyr Wayfinder) and reanimation; Twilight Diviner's copies of creatures that come back (the game's record of creature cards put into graveyards); Journey to Eternity // Atzal; the Yawgmoth combo. All of the list's cards are modeled (impls/t12_brago_meren.rs for the Tier 2 additions).

## Decklist by type (100)

**Commander (1).** Meren of Clan Nel Toth

**Creatures (37).** Accursed Marauder, Birds of Paradise, Blood Artist, Braids, Arisen Nightmare, Cankerbloom, Carrion Feeder, Deathrite Shaman, Elves of Deep Shadow, Elvish Mystic, Eternal Witness, Fiend Artisan, Fleshbag Marauder, Gray Merchant of Asphodel, Grim Haruspex, Haywire Mite, Llanowar Elves, Massacre Wurm, Mikaeus, the Unhallowed, Plaguecrafter, Protean Hulk, Ravenous Chupacabra, Reclamation Sage, Sakura-Tribe Elder, Satyr Wayfinder, Sheoldred, Whispering One, Shriekmaw, Solemn Simulacrum, Spore Frog, Syr Konrad, the Grim, Twilight Diviner, Umbral Collar Zealot, Viscera Seer, Warren Soultrader, Wight of the Reliquary, Wood Elves, Yawgmoth, Thran Physician, Zulaport Cutthroat

**Enchantments (5).** Animate Dead, Dictate of Erebos, Grave Pact, Journey to Eternity // Atzal, Cave of Eternity, Moldervine Reclamation

**Artifacts (6).** Arcane Signet, Ashnod's Altar, Birthing Pod, Cauldron of Essence, Skullclamp, Sol Ring

**Sorceries (10).** Buried Alive, Culling Ritual, Cultivate, Demonic Tutor, Diabolic Intent, Eldritch Evolution, Living Death, Reanimate, Toxic Deluge, Victimize

**Instants (6).** Assassin's Trophy, Beast Within, Dark Ritual, Entomb, Putrefy, Village Rites

**Lands (35).** Bojuka Bog, Command Tower, Deathcap Glade, Golgari Rot Farm, High Market, Llanowar Wastes, Necroblossom Snarl, Overgrown Tomb, Phyrexian Tower, Tainted Wood, Temple of Malady, Twilight Mire, Underground Mortuary, Undergrowth Stadium, Urborg, Tomb of Yawgmoth, Verdant Catacombs, Woodland Cemetery, 8 Forest, 10 Swamp

## Import list (100)

```
1 Accursed Marauder
1 Animate Dead
1 Arcane Signet
1 Ashnod's Altar
1 Assassin's Trophy
1 Beast Within
1 Birds of Paradise
1 Birthing Pod
1 Blood Artist
1 Bojuka Bog
1 Braids, Arisen Nightmare
1 Buried Alive
1 Cankerbloom
1 Carrion Feeder
1 Cauldron of Essence
1 Command Tower
1 Culling Ritual
1 Cultivate
1 Dark Ritual
1 Deathcap Glade
1 Deathrite Shaman
1 Demonic Tutor
1 Diabolic Intent
1 Dictate of Erebos
1 Eldritch Evolution
1 Elves of Deep Shadow
1 Elvish Mystic
1 Entomb
1 Eternal Witness
1 Fiend Artisan
1 Fleshbag Marauder
1 Golgari Rot Farm
1 Grave Pact
1 Gray Merchant of Asphodel
1 Grim Haruspex
1 Haywire Mite
1 High Market
1 Journey to Eternity // Atzal, Cave of Eternity
1 Living Death
1 Llanowar Elves
1 Llanowar Wastes
1 Massacre Wurm
1 Meren of Clan Nel Toth
1 Mikaeus, the Unhallowed
1 Moldervine Reclamation
1 Necroblossom Snarl
1 Overgrown Tomb
1 Phyrexian Tower
1 Plaguecrafter
1 Protean Hulk
1 Putrefy
1 Ravenous Chupacabra
1 Reanimate
1 Reclamation Sage
1 Sakura-Tribe Elder
1 Satyr Wayfinder
1 Sheoldred, Whispering One
1 Shriekmaw
1 Skullclamp
1 Sol Ring
1 Solemn Simulacrum
1 Spore Frog
1 Syr Konrad, the Grim
1 Tainted Wood
1 Temple of Malady
1 Toxic Deluge
1 Twilight Diviner
1 Twilight Mire
1 Umbral Collar Zealot
1 Underground Mortuary
1 Undergrowth Stadium
1 Urborg, Tomb of Yawgmoth
1 Verdant Catacombs
1 Victimize
1 Village Rites
1 Viscera Seer
1 Warren Soultrader
1 Wight of the Reliquary
1 Wood Elves
1 Woodland Cemetery
1 Yawgmoth, Thran Physician
1 Zulaport Cutthroat
8 Forest
10 Swamp
```
