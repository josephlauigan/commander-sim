# Teysa Karlov — Orzhov Aristocrats

Opponent pool deck — **Tier 1: High Bracket 2 / Low Bracket 3**. Axis: Aristocrats / incremental drain.

- **Commander:** Teysa Karlov
- **Color identity:** White, Black
- **Game Changers (0):** none
- **Combos:** Pitiless Plunderer + Reassembling Skeleton + Ashnod's Altar or Phyrexian Altar: unbounded deaths (the Treasure pays for the Skeleton's return) → Blood Artist / Zulaport Cutthroat / Cruel Celebrant / Bastion of Remembrance / Syr Konrad drain the table. Three pieces and a payoff; no two-card infinite.
- **Lands:** 35 (14 nonbasic + 21 basic)

## Role in the pool

Damage that bypasses blockers and punishes chump-blocking. Grave Pact effects also test how many creatures your decks commit.

## At the table

- **Plan:** Makes cheap bodies that leave more behind: Doomed Traveler, Hunted Witness, Reassembling Skeleton, Pawn of Ulamog, Infestation Sage, Ophiomancer and the afterlife creatures (Ministrant of Obligation, Orzhov Enforcer, Indebted Spirit). Viscera Seer, Carrion Feeder, Priest of Forgotten Gods, Yahenni and the altars sacrifice them; Skullclamp, Midnight Reaper and Grim Haruspex draw. Teysa Karlov and Drivnod, Carnage Dominus make each death trigger again.
- **Wins by:** Drains the table: Blood Artist, Zulaport Cutthroat, Cruel Celebrant, Bastion of Remembrance, Syr Konrad, Vindictive Vampire, Elas il-Kor and The Meathook Massacre, with Grave Pact, Dictate of Erebos and Butcher of Malakir forcing sacrifices. Luminous Broodmoth, Elenda and Liliana, Dreadhorde General keep the value coming; Pitiless Plunderer with Reassembling Skeleton and an altar is an unbounded loop.
- **Watch out for:** Killing Teysa halves the drain, and killing the drain creatures matters more than killing tokens. Grave Pact punishes you for each creature it sacrifices, so do not overcommit. Several drains in play can be lethal; Pitiless Plunderer, Reassembling Skeleton and an altar together end the game.
- **Interaction:** Swords to Plowshares, Path to Exile, Anguished Unmaking, Despark, Mortify and Corrupted Conviction; Austere Command, Damn and Kaya's Wrath as sweepers. Teysa, Orzhov Scion exiles a creature for three white bodies; Bojuka Bog and Rite of Oblivion handle graveyards and permanents.

## Strategy

Make disposable bodies, sacrifice them, and let Teysa Karlov (and Drivnod) double every death trigger. Blood Artist, Zulaport, Cruel Celebrant and Bastion each drain twice per death with Teysa out; Black Market banks a counter per death into mana, and Grave Pact / Dictate of Erebos turn every sacrifice into opposing ones. Afterlife spirits are white and black, so they feed Teysa, Orzhov Scion's Spirits and her exile cost. Luminous Broodmoth brings each non-flyer back once. The one loop is Pitiless Plunderer + Reassembling Skeleton + an altar, three pieces and a drain payoff.

## Sim modeling notes

Model these first; if any is Partial or unmodeled, this deck plays below its real strength and its results should be flagged.

Teysa's and Drivnod's death-trigger doubling (also doubles token-creating death triggers); drain payoffs; afterlife and token colours (white and black Spirits for Teysa, Orzhov Scion); Grave Pact / Dictate of Erebos / Butcher of Malakir forced sacrifices; Skullclamp; Luminous Broodmoth returns; the Plunderer + Skeleton + altar loop (combo catalogue). All are modeled (impls/t12_tatyova_teysa.rs and the compiled patterns); Black Market's mana lasts the turn instead of the phase.

## Decklist by type (100)

**Commander (1).** Teysa Karlov

**Creatures (34).** Bartolomé del Presidio, Blood Artist, Butcher of Malakir, Carrion Feeder, Cruel Celebrant, Doomed Traveler, Drivnod, Carnage Dominus, Elas il-Kor, Sadistic Pilgrim, Elenda, the Dusk Rose, Grim Haruspex, Hallowed Spiritkeeper, Hunted Witness, Indebted Spirit, Infestation Sage, Luminous Broodmoth, Marionette Apprentice, Midnight Reaper, Ministrant of Obligation, Mirkwood Bats, Morbid Opportunist, Ophiomancer, Orzhov Enforcer, Pawn of Ulamog, Pitiless Plunderer, Priest of Forgotten Gods, Reassembling Skeleton, Sifter of Skulls, Solemn Simulacrum, Syr Konrad, the Grim, Teysa, Orzhov Scion, Vindictive Vampire, Viscera Seer, Yahenni, Undying Partisan, Zulaport Cutthroat

**Enchantments (7).** Bastion of Remembrance, Black Market, Dictate of Erebos, Grave Pact, Phyrexian Arena, The Meathook Massacre, Vampiric Rites

**Artifacts (8).** Arcane Signet, Ashnod's Altar, Mind Stone, Orzhov Signet, Phyrexian Altar, Skullclamp, Sol Ring, Talisman of Hierarchy

**Sorceries (6).** Austere Command, Damn, Diabolic Intent, Kaya's Wrath, Rite of Oblivion, Victimize

**Instants (8).** Anguished Unmaking, Corrupted Conviction, Deadly Dispute, Despark, Mortify, Path to Exile, Swords to Plowshares, Village Rites

**Lands (35).** Bojuka Bog, Caves of Koilos, Command Tower, Evolving Wilds, Fetid Heath, Godless Shrine, High Market, Isolated Chapel, Orzhov Basilica, Scoured Barrens, Shattered Sanctum, Shineshadow Snarl, Tainted Field, Temple of Silence, 9 Plains, 12 Swamp

**Planeswalkers (1).** Liliana, Dreadhorde General

## Import list (100)

```
1 Anguished Unmaking
1 Arcane Signet
1 Ashnod's Altar
1 Austere Command
1 Bartolomé del Presidio
1 Bastion of Remembrance
1 Black Market
1 Blood Artist
1 Bojuka Bog
1 Butcher of Malakir
1 Carrion Feeder
1 Caves of Koilos
1 Command Tower
1 Corrupted Conviction
1 Cruel Celebrant
1 Damn
1 Deadly Dispute
1 Despark
1 Diabolic Intent
1 Dictate of Erebos
1 Doomed Traveler
1 Drivnod, Carnage Dominus
1 Elas il-Kor, Sadistic Pilgrim
1 Elenda, the Dusk Rose
1 Evolving Wilds
1 Fetid Heath
1 Godless Shrine
1 Grave Pact
1 Grim Haruspex
1 Hallowed Spiritkeeper
1 High Market
1 Hunted Witness
1 Indebted Spirit
1 Infestation Sage
1 Isolated Chapel
1 Kaya's Wrath
1 Liliana, Dreadhorde General
1 Luminous Broodmoth
1 Marionette Apprentice
1 Midnight Reaper
1 Mind Stone
1 Ministrant of Obligation
1 Mirkwood Bats
1 Morbid Opportunist
1 Mortify
1 Ophiomancer
1 Orzhov Basilica
1 Orzhov Enforcer
1 Orzhov Signet
1 Path to Exile
1 Pawn of Ulamog
1 Phyrexian Altar
1 Phyrexian Arena
1 Pitiless Plunderer
1 Priest of Forgotten Gods
1 Reassembling Skeleton
1 Rite of Oblivion
1 Scoured Barrens
1 Shattered Sanctum
1 Shineshadow Snarl
1 Sifter of Skulls
1 Skullclamp
1 Sol Ring
1 Solemn Simulacrum
1 Swords to Plowshares
1 Syr Konrad, the Grim
1 Tainted Field
1 Talisman of Hierarchy
1 Temple of Silence
1 Teysa Karlov
1 Teysa, Orzhov Scion
1 The Meathook Massacre
1 Vampiric Rites
1 Victimize
1 Village Rites
1 Vindictive Vampire
1 Viscera Seer
1 Yahenni, Undying Partisan
1 Zulaport Cutthroat
9 Plains
12 Swamp
```
