# Isshin — Mardu Attack Triggers

Opponent pool deck — **Tier 1: High Bracket 2 / Low Bracket 3**. Axis: Aggro / combat pressure.

- **Commander:** Isshin, Two Heavens as One
- **Color identity:** White, Black, Red
- **Game Changers (0):** none
- **Combos:** None. No two-card infinite; the extra combats (Aurelia, the Warleader, Karlach, Raiyuu) are finite, even doubled by Isshin.
- **Lands:** 36 (19 nonbasic + 17 basic)

## Role in the pool

Early combat pressure that turns every attack into damage off the board. Whether your decks can block, sweep, or race a table that loses life each time this deck declares attackers.

## At the table

- **Plan:** Cheap creatures whose attack triggers Isshin, Two Heavens as One doubles: Adeline, Resplendent Cathar, Hero of Bladehold, Otharri, Suns' Glory, Voice of Victory and Zurgo Stormrender make attacking tokens; Captain Lannery Storm makes Treasure; Hammers of Moradin taps blockers. Windcrag Siege doubles them again.
- **Wins by:** Damage on attack, not only in combat: Hellrider, Brutal Hordechief, Mishra, Claimed by Gix, Commissar Severina Raine, Ingris Stingerquill and Tectonic Giant drain or burn each opponent as the team swings. Fervent Charge and Adriana's melee pump the attackers and Iroas, God of Victory shields them; Aurelia, the Warleader, Karlach and Raiyuu add extra combats.
- **Watch out for:** Isshin with a wide board: each attack triggers twice, so a turn-four swing can take ten or more life from every opponent before blocks. Kill Isshin first, then the drain creatures (Hellrider, Mishra, Hordechief). Sweepers set it back, but Alesha and Otharri come back from the graveyard.
- **Interaction:** Swords to Plowshares, Path to Exile, Anguished Unmaking, Despark, Generous Gift, Chaos Warp, Boros Charm, and the sweepers Blasphemous Act, Damn and Ruinous Ultimatum. Lightning Greaves, Swiftfoot Boots, Boros Charm and Unbreakable Formation protect its team.

## Strategy

Curve out with attackers that do something when they swing, land Isshin, and attack every turn. Token makers (Adeline, Hero of Bladehold, Anim Pakal, Otharri, Krenko, Tin Street Kingpin, Voice of Victory, Zurgo) widen the board mid-combat, and each attacker then feeds the per-attacker payoffs: Hellrider and Brutal Hordechief per creature, Mishra and Severina by the number of attackers, Ingris per creature to each opponent. Curse of Opulence and Captain Lannery Storm fund the top end (Aurelia, the Warleader, Etali, Sun Titan); Painful Truths, Read the Bones and Breena, the Demagogue refill the hand.

## Sim modeling notes

Model these first; if any is Partial or unmodeled, this deck plays below its real strength and its results should be flagged.

Isshin's trigger doubling (attack triggers only, not ETB; every hand-written and compiled attack trigger goes through it); attacking-token makers; per-attacker damage (Hellrider, Brutal Hordechief, Ingris, Mishra and Severina, whose X counts the attackers); extra combats (Aurelia, the Warleader, Karlach, Raiyuu). Approximate: Curse of Opulence's Gold tokens are Treasures; Reconnaissance is vigilance on your turn; Tectonic Giant always takes the burn mode; Firebender Ascension counts and copies only compiled and hand-written attack triggers (impls/t12_isshin.rs).

## Decklist by type (100)

**Commander (1).** Isshin, Two Heavens as One

**Creatures (30).** Adeline, Resplendent Cathar, Adriana, Captain of the Guard, Alesha, Who Laughs at Fate, Anim Pakal, Thousandth Moon, Audacious Thief, Aurelia, the Law Above, Aurelia, the Warleader, Breena, the Demagogue, Brutal Hordechief, Caesar, Legion's Emperor, Captain Lannery Storm, Commissar Severina Raine, Etali, Primal Storm, Hammers of Moradin, Hanweir Garrison, Hellrider, Hero of Bladehold, Ingris Stingerquill, Iroas, God of Victory, Karlach, Fury of Avernus, Kazuul, Tyrant of the Cliffs, Krenko, Tin Street Kingpin, Mishra, Claimed by Gix, Otharri, Suns' Glory, Raiyuu, Storm's Edge, Skyknight Vanguard, Sun Titan, Tectonic Giant, Voice of Victory, Zurgo Stormrender

**Enchantments (8).** Curse of Opulence, Duelist's Heritage, Fervent Charge, Firebender Ascension, Mardu Ascendancy, Reconnaissance, Rising of the Day, Windcrag Siege

**Artifacts (12).** Arcane Signet, Boros Signet, Fellwar Stone, Lightning Greaves, Orzhov Signet, Rakdos Signet, Sol Ring, Swiftfoot Boots, Sword of the Animist, Talisman of Conviction, Talisman of Hierarchy, Talisman of Indulgence

**Sorceries (5).** Blasphemous Act, Damn, Painful Truths, Read the Bones, Ruinous Ultimatum

**Instants (8).** Anguished Unmaking, Boros Charm, Chaos Warp, Despark, Generous Gift, Path to Exile, Swords to Plowshares, Unbreakable Formation

**Lands (36).** Battlefield Forge, Blood Crypt, Caves of Koilos, Clifftop Retreat, Command Tower, Dragonskull Summit, Evolving Wilds, Exotic Orchard, Godless Shrine, Isolated Chapel, Nomad Outpost, Path of Ancestry, Rogue's Passage, Sacred Foundry, Savai Triome, Smoldering Marsh, Spectator Seating, Temple of Triumph, Terramorphic Expanse, 6 Plains, 6 Mountain, 5 Swamp

## Import list (100)

```
1 Adeline, Resplendent Cathar
1 Adriana, Captain of the Guard
1 Alesha, Who Laughs at Fate
1 Anguished Unmaking
1 Anim Pakal, Thousandth Moon
1 Arcane Signet
1 Audacious Thief
1 Aurelia, the Law Above
1 Aurelia, the Warleader
1 Battlefield Forge
1 Blasphemous Act
1 Blood Crypt
1 Boros Charm
1 Boros Signet
1 Breena, the Demagogue
1 Brutal Hordechief
1 Caesar, Legion's Emperor
1 Captain Lannery Storm
1 Caves of Koilos
1 Chaos Warp
1 Clifftop Retreat
1 Command Tower
1 Commissar Severina Raine
1 Curse of Opulence
1 Damn
1 Despark
1 Dragonskull Summit
1 Duelist's Heritage
1 Etali, Primal Storm
1 Evolving Wilds
1 Exotic Orchard
1 Fellwar Stone
1 Fervent Charge
1 Firebender Ascension
1 Generous Gift
1 Godless Shrine
1 Hammers of Moradin
1 Hanweir Garrison
1 Hellrider
1 Hero of Bladehold
1 Ingris Stingerquill
1 Iroas, God of Victory
1 Isolated Chapel
1 Isshin, Two Heavens as One
1 Karlach, Fury of Avernus
1 Kazuul, Tyrant of the Cliffs
1 Krenko, Tin Street Kingpin
1 Lightning Greaves
1 Mardu Ascendancy
1 Mishra, Claimed by Gix
1 Nomad Outpost
1 Orzhov Signet
1 Otharri, Suns' Glory
1 Painful Truths
1 Path of Ancestry
1 Path to Exile
1 Raiyuu, Storm's Edge
1 Rakdos Signet
1 Read the Bones
1 Reconnaissance
1 Rising of the Day
1 Rogue's Passage
1 Ruinous Ultimatum
1 Sacred Foundry
1 Savai Triome
1 Skyknight Vanguard
1 Smoldering Marsh
1 Sol Ring
1 Spectator Seating
1 Sun Titan
1 Swiftfoot Boots
1 Sword of the Animist
1 Swords to Plowshares
1 Talisman of Conviction
1 Talisman of Hierarchy
1 Talisman of Indulgence
1 Tectonic Giant
1 Temple of Triumph
1 Terramorphic Expanse
1 Unbreakable Formation
1 Voice of Victory
1 Windcrag Siege
1 Zurgo Stormrender
6 Plains
6 Mountain
5 Swamp
```
