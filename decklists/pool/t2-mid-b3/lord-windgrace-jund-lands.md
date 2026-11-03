# Lord Windgrace — Jund Lands

Opponent pool deck — **Tier 2: Mid Bracket 3**. Axis: Lands / ramp value.

- **Commander:** Lord Windgrace
- **Color identity:** Black, Red, Green
- **Game Changers (1):** Field of the Dead
- **Combos:** Dark Depths + Thespian's Stage (a 20/20 flying, indestructible Marit Lage; two cards, not infinite). Nothing tutors for a half any more (round 3): Splendid Reclamation and the land recursion bring them back from the graveyard.
- **Lands:** 40 (27 nonbasic + 13 basic)

## Role in the pool

A resilient, graveyard-fed lands deck with a two-card threat. Tests graveyard hate from the other side, whether your decks can pressure a planeswalker, and whether they can answer a 20/20 indestructible flier (exile, bounce, edicts, chump blockers).

## At the table

- **Plan:** Ramps with Cultivate, Kodama's Reach, Farseek, Nature's Lore, Three Visits and Sakura-Tribe Elder while filling the graveyard with fetch lands. Exploration, Dryad of the Ilysian Grove, Icetill Explorer, Mina and Denn and Burgeoning add land drops; Crucible of Worlds, Ramunap Excavator, Icetill Explorer and Conduit of Worlds replay them from the graveyard. Lord Windgrace digs with his +2 and returns two lands with his -3.
- **Wins by:** Landfall: Avenger of Zendikar, Field of the Dead, Ob Nixilis and Tannuk turn each land into a body or damage, and Rampaging Baloths adds 4/4s. Marit Lage from Dark Depths and Thespian's Stage (or Depths counted down with spare mana) is the quick kill, but the deck must draw both halves.
- **Watch out for:** Windgrace and Nissa are planeswalkers: attack them. Dark Depths on the battlefield is half a 20/20; the Stage finishes it. Soul of Windgrace takes lands from any graveyard, and Constant Mists (with buyback) fogs your attacks again and again. Strip Mine hits your best land.
- **Interaction:** Assassin's Trophy, Putrefy, Beast Within, Windgrace's Judgment, Blasphemous Act and Toxic Deluge; Constant Mists against attacks, Sylvan Safekeeper and Boseiju protect. No counterspells.

## Strategy

Ramp, fill the graveyard with lands and replay them for landfall. Windgrace's +2 and Realms Uncharted find lands; Crucible, Ramunap, Life from the Loam, Splendid Reclamation and Soul of Windgrace bring them back. Avenger, Field of the Dead, Ob Nixilis, Tannuk and Tireless Provisioner turn land drops into a board, and Thespian's Stage with Dark Depths is the two-card Marit Lage kill when both halves are drawn. The ramp is plain (Cultivate, Skyshroud Claim, Burnished Hart, Solemn Simulacrum, Springbloom Druid) and the deck wins slowly.

## Sim modeling notes

Model these first; if any is Partial or unmodeled, this deck plays below its real strength and its results should be flagged.

Windgrace planeswalker loyalty and land recursion; landfall token makers and Field of the Dead; Life from the Loam dredge; Dark Depths + Thespian's Stage (impls/t12_windgrace.rs, chained in the combo catalogue); extra land drops and lands from the graveyard; Soul of Windgrace putting lands in; Spelunking's untapped lands. Left out: Thespian's Stage copies only Dark Depths; Constant Mists doesn't stop combat damage between creatures; Mina and Denn bounce a land only to replay it.

## Decklist by type (100)

**Commander (1).** Lord Windgrace

**Creatures (26).** Aftermath Analyst, Ancient Greenwarden, Avenger of Zendikar, Boreal Druid, Burnished Hart, Dryad of the Ilysian Grove, Elvish Rejuvenator, Eternal Witness, Icetill Explorer, Kura, the Boundless Sky, Mina and Denn, Wildborn, Noxious Newt, Ob Nixilis, the Fallen, Rampaging Baloths, Ramunap Excavator, Sakura-Tribe Elder, Satyr Wayfinder, Solemn Simulacrum, Soul of Windgrace, Springbloom Druid, Sylvan Safekeeper, Tannuk, Memorial Ensign, The Gitrog Monster, Tireless Provisioner, Tireless Tracker, World Shaper

**Enchantments (5).** Burgeoning, Exploration, Retreat to Hagra, Spelunking, Valakut Exploration

**Artifacts (4).** Conduit of Worlds, Crucible of Worlds, Sol Ring, Zuran Orb

**Sorceries (15).** Blasphemous Act, Cartographer's Survey, Cultivate, Explore, Farseek, Grow from the Ashes, Kodama's Reach, Life from the Loam, Mulch, Nature's Lore, Skyshroud Claim, Splendid Reclamation, Three Visits, Torment of Hailfire, Toxic Deluge

**Instants (7).** Assassin's Trophy, Beast Within, Constant Mists, Harrow, Putrefy, Realms Uncharted, Windgrace's Judgment

**Lands (40).** Blood Crypt, Bloodstained Mire, Bojuka Bog, Boseiju, Who Endures, Cinder Glade, Command Tower, Commercial District, Dark Depths, Evolving Wilds, Fabled Passage, Field of the Dead, Golgari Rot Farm, Gruul Turf, Lotus Field, Overgrown Tomb, Rakdos Carnarium, Smoldering Marsh, Stomping Ground, Strip Mine, Terramorphic Expanse, Thespian's Stage, Underground Mortuary, Urborg, Tomb of Yawgmoth, Verdant Catacombs, Wooded Foothills, Yavimaya, Cradle of Growth, Ziatora's Proving Ground, 6 Forest, 3 Swamp, 3 Mountain, 1 Snow-Covered Forest

**Planeswalkers (2).** Nissa of Shadowed Boughs, Wrenn and Six

## Import list (100)

```
1 Aftermath Analyst
1 Ancient Greenwarden
1 Assassin's Trophy
1 Avenger of Zendikar
1 Beast Within
1 Blasphemous Act
1 Blood Crypt
1 Bloodstained Mire
1 Bojuka Bog
1 Boreal Druid
1 Boseiju, Who Endures
1 Burgeoning
1 Burnished Hart
1 Cartographer's Survey
1 Cinder Glade
1 Command Tower
1 Commercial District
1 Conduit of Worlds
1 Constant Mists
1 Crucible of Worlds
1 Cultivate
1 Dark Depths
1 Dryad of the Ilysian Grove
1 Elvish Rejuvenator
1 Eternal Witness
1 Evolving Wilds
1 Exploration
1 Explore
1 Fabled Passage
1 Farseek
1 Field of the Dead
1 Golgari Rot Farm
1 Grow from the Ashes
1 Gruul Turf
1 Harrow
1 Icetill Explorer
1 Kodama's Reach
1 Kura, the Boundless Sky
1 Life from the Loam
1 Lord Windgrace
1 Lotus Field
1 Mina and Denn, Wildborn
1 Mulch
1 Nature's Lore
1 Nissa of Shadowed Boughs
1 Noxious Newt
1 Ob Nixilis, the Fallen
1 Overgrown Tomb
1 Putrefy
1 Rakdos Carnarium
1 Rampaging Baloths
1 Ramunap Excavator
1 Realms Uncharted
1 Retreat to Hagra
1 Sakura-Tribe Elder
1 Satyr Wayfinder
1 Skyshroud Claim
1 Smoldering Marsh
1 Sol Ring
1 Solemn Simulacrum
1 Soul of Windgrace
1 Spelunking
1 Splendid Reclamation
1 Springbloom Druid
1 Stomping Ground
1 Strip Mine
1 Sylvan Safekeeper
1 Tannuk, Memorial Ensign
1 Terramorphic Expanse
1 The Gitrog Monster
1 Thespian's Stage
1 Three Visits
1 Tireless Provisioner
1 Tireless Tracker
1 Torment of Hailfire
1 Toxic Deluge
1 Underground Mortuary
1 Urborg, Tomb of Yawgmoth
1 Valakut Exploration
1 Verdant Catacombs
1 Windgrace's Judgment
1 Wooded Foothills
1 World Shaper
1 Wrenn and Six
1 Yavimaya, Cradle of Growth
1 Ziatora's Proving Ground
1 Zuran Orb
6 Forest
3 Swamp
3 Mountain
1 Snow-Covered Forest
```
