# Jodah, the Unifier — WUBRG Legends

2026-10-07
**Added 2026-10-07.** In the simulator as deck key `jodah` (`python3 -m commander_sim --deck jodah --pool t4`) and in practice mode.

## Strategy

Jodah, the Unifier ({W}{U}{B}{R}{G}, 5/5) reads: *Legendary creatures you control get +X/+X, where X is the number of legendary creatures you control. Whenever you cast a legendary spell from your hand, exile cards from the top of your library until you exile a legendary nonland card with lesser mana value. You may cast that card without paying its mana cost. Put the rest on the bottom in a random order.*

**Legend cascade.** 28 of the 63 spells are legendary, from Blackblade Reforged and Wrenn and Six at two mana up to Razia and Sisters of Stone Death at eight. With Jodah out, every legendary spell cast from hand brings a free legendary of lower mana value with it: Szadek (7) can bring Dragonlord Dromoka or Tolsimir, Lyra (5) can bring Elenda or the Kaldra equipment. The free card is cast, so Jodah triggers only on cards from hand — a cascaded legend doesn't cascade again.

**Legends get big.** Each legendary creature gets +1/+1 for every legendary creature you control, Jodah included. Three legends on the battlefield makes each of them +3/+3; Elenda, Lyra and Kura become threats on their own.

**The pieces.**
- **Card flow:** Fact or Fiction, Memory Jar, Mordenkainen, Experimental Augury, Tireless Tracker, Urza, Powerstone Prodigy, Court of Ardenvale (the monarch).
- **Mana:** Coalition Relic, Moss Diamond, Star Compass, Sisay's Ring, Fyndhorn Elder, Solemn Simulacrum, Cartographer's Survey, Plaza of Heroes (any colour for legendary spells).
- **Tutors:** Demonic, Vampiric, Enlightened, Mystical, Worldly and Profane Tutor find the legend or the answer the game needs.
- **Protection:** Privileged Position (hexproof for your other permanents), Grand Abolisher and Dragonlord Dromoka (opponents can't cast spells during your turn), Shield of Kaldra.
- **Interaction:** Force of Will, Dismiss, Dissipate, Desertion and Arcane Denial; Bitter Triumph, Darksteel Mutation, Skyclave Apparition, Kaervek's Purge, Lagrella, Disenchant, Pillage; Toxic Deluge and Invoke Despair.
- **Kaldra:** Sword, Shield and Helm of Kaldra assemble Kaldra, a 4/4 legendary Avatar wearing all three (+5/+5, first strike, trample, haste, indestructible). Each piece is legendary, so each one triggers Jodah when cast.

## Key lines

- **Turns 1–4: mana.** Coalition Relic, Star Compass, Fyndhorn Elder or Moss Diamond, then Jodah on turn 4 or 5.
- **Jodah, then legends from hand.** Cast the expensive legends first: a seven- or eight-drop cascades into almost any other legend in the deck.
- **Protect the board.** Privileged Position or Grand Abolisher before the big turn; hold Force of Will for the wipe.

**Mulligan:** keep three lands that make at least three colours, or two lands and a rock. The mana base has 36 lands: 10 painlands, 10 tri-lands (they enter tapped), 2 slow lands (Overgrown Farmland, Shattered Sanctum), Command Tower, Exotic Orchard, Path of Ancestry, Plaza of Heroes, two fetches for basics (Evolving Wilds, Terramorphic Expanse) and 8 basics.

## What the simulator found

First read with the fast AI (loose profile, 300 games per tier; 25% is an even share): 13.7% against Tier 1, 17.3% against Tier 2, 20.3% against Tier 3, 20.0% against Tier 4 and 22.0% against Tier 5. A look-ahead run would give the fuller picture. In 20 traced games against Tier 3, Jodah cascaded into a free legend 26 times.

Modeling: 92 of the 97 unique cards are modeled in full. Five are approximate, each missing a minor activated ability: Cromat (the blocker-destroy and top-of-library abilities), King Darien (the sacrifice for protection), Razia (the damage redirect), Sisters of Stone Death (the lure and the reanimation of exiled creatures) and Wrenn and Six (the -7 emblem is recorded but not used).

## Bracket and Rule 0

**Bracket 4,** with six Game Changers: **Force of Will**, **Demonic Tutor**, **Vampiric Tutor**, **Mystical Tutor**, **Enlightened Tutor** and **Worldly Tutor**. No two-card infinite combos, no mass land destruction (Pillage is one land), no extra turns.

Disclose before the game: the six tutors, Force of Will, Grand Abolisher and Dragonlord Dromoka (no spells during your turn), and Desertion (it steals the creature or artifact it counters).

Re-check the official list at https://commanderbrackets.com/faq before an event.

## Decklist by type (100)

**Commander (1).** Jodah, the Unifier

**Creatures (26).** Caparocti Sunborn, Carth the Lion, Cromat, Dakkon Blackblade, Dragonlord Dromoka, Elenda, the Dusk Rose, Fyndhorn Elder, Genesis Hydra, Grand Abolisher, King Darien XLVIII, Korlash, Heir to Blackblade, Kura, the Boundless Sky, Lagrella, the Magpie, Lyra Dawnbringer, Marchesa, the Black Rose, Mirri, Weatherlight Duelist, Razia, Boros Archangel, Sisters of Stone Death, Skyclave Apparition, Sol'kanar the Swamp King, Solemn Simulacrum, Szadek, Lord of Secrets, Terror of the Peaks, Tireless Tracker, Tolsimir Wolfblood, Urza, Powerstone Prodigy

**Planeswalkers (4).** Dakkon, Shadow Slayer, Mordenkainen, The Aetherspark, Wrenn and Six

**Artifacts (10).** Blackblade Reforged, Coalition Relic, Helm of Kaldra, Memory Jar, Mirari, Moss Diamond, Shield of Kaldra, Sisay's Ring, Star Compass, Sword of Kaldra

**Enchantments (3).** Court of Ardenvale, Darksteel Mutation, Privileged Position

**Instants (13).** Arcane Denial, Bitter Triumph, Desertion, Disenchant, Dismiss, Dissipate, Enlightened Tutor, Experimental Augury, Fact or Fiction, Force of Will, Mystical Tutor, Vampiric Tutor, Worldly Tutor

**Sorceries (7).** Cartographer's Survey, Demonic Tutor, Invoke Despair, Kaervek's Purge, Pillage, Profane Tutor, Toxic Deluge

**Lands (36).** Adarkar Wastes, Arcane Sanctum, Battlefield Forge, Brushland, Caves of Koilos, Command Tower, Crumbling Necropolis, Evolving Wilds, Exotic Orchard, Frontier Bivouac, Jungle Shrine, Karplusan Forest, Llanowar Wastes, Mystic Monastery, Nomad Outpost, Opulent Palace, Overgrown Farmland, Path of Ancestry, Plaza of Heroes, Sandsteppe Citadel, Savage Lands, Seaside Citadel, Shattered Sanctum, Shivan Reef, Sulfurous Springs, Terramorphic Expanse, Underground River, Yavimaya Coast, 2 Forest, 1 Island, 1 Mountain, 2 Plains, 2 Swamp

## Import list (100)

```
1 Jodah, the Unifier
1 Adarkar Wastes
1 Arcane Denial
1 Arcane Sanctum
1 Battlefield Forge
1 Bitter Triumph
1 Blackblade Reforged
1 Brushland
1 Caparocti Sunborn
1 Carth the Lion
1 Cartographer's Survey
1 Caves of Koilos
1 Coalition Relic
1 Command Tower
1 Court of Ardenvale
1 Cromat
1 Crumbling Necropolis
1 Dakkon Blackblade
1 Dakkon, Shadow Slayer
1 Darksteel Mutation
1 Demonic Tutor
1 Desertion
1 Disenchant
1 Dismiss
1 Dissipate
1 Dragonlord Dromoka
1 Elenda, the Dusk Rose
1 Enlightened Tutor
1 Evolving Wilds
1 Exotic Orchard
1 Experimental Augury
1 Fact or Fiction
1 Force of Will
1 Frontier Bivouac
1 Fyndhorn Elder
1 Genesis Hydra
1 Grand Abolisher
1 Helm of Kaldra
1 Invoke Despair
1 Jungle Shrine
1 Kaervek's Purge
1 Karplusan Forest
1 King Darien XLVIII
1 Korlash, Heir to Blackblade
1 Kura, the Boundless Sky
1 Lagrella, the Magpie
1 Llanowar Wastes
1 Lyra Dawnbringer
1 Marchesa, the Black Rose
1 Memory Jar
1 Mirari
1 Mirri, Weatherlight Duelist
1 Mordenkainen
1 Moss Diamond
1 Mystic Monastery
1 Mystical Tutor
1 Nomad Outpost
1 Opulent Palace
1 Overgrown Farmland
1 Path of Ancestry
1 Pillage
1 Plaza of Heroes
1 Privileged Position
1 Profane Tutor
1 Razia, Boros Archangel
1 Sandsteppe Citadel
1 Savage Lands
1 Seaside Citadel
1 Shattered Sanctum
1 Shield of Kaldra
1 Shivan Reef
1 Sisay's Ring
1 Sisters of Stone Death
1 Skyclave Apparition
1 Sol'kanar the Swamp King
1 Solemn Simulacrum
1 Star Compass
1 Sulfurous Springs
1 Sword of Kaldra
1 Szadek, Lord of Secrets
1 Terramorphic Expanse
1 Terror of the Peaks
1 The Aetherspark
1 Tireless Tracker
1 Tolsimir Wolfblood
1 Toxic Deluge
1 Underground River
1 Urza, Powerstone Prodigy
1 Vampiric Tutor
1 Worldly Tutor
1 Wrenn and Six
1 Yavimaya Coast
2 Forest
1 Island
1 Mountain
2 Plains
2 Swamp
```
