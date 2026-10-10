# Jodah, the Unifier — WUBRG Legends

**Updated 2026-10-09.** Out: Evolving Wilds, Terramorphic Expanse. In: Unclaimed Territory, Vivid Creek. JD's land switch (Unclaimed Territory names Human: Jodah and 10 other creatures are Humans). Measured flat: +0.2 ± 0.5 points (13,000 paired games, adaptive AI; audit/jodah/README.md).
**Updated 2026-10-09.** Out: Blackblade Reforged, Desertion, Dismiss, Dissipate, Helm of Kaldra, Invoke Despair, Kaervek's Purge, Pillage, Shield of Kaldra, Sisay's Ring, Sword of Kaldra, Szadek, Lord of Secrets. In: Arcane Signet, Birds of Paradise, Chromatic Lantern, Fellwar Stone, Gix, Yawgmoth Praetor, Paradise Druid, Shalai, Voice of Plenty, Talisman of Hierarchy. The Jodah rework (audit/jodah, packages B-D): the Kaldra pieces cost 3-4 mana, did little alone and blocked a quarter of cascades, so three basics take their slots (+5 on their own); Birds of Paradise, Paradise Druid, Fellwar Stone, Talisman of Hierarchy, Chromatic Lantern and Arcane Signet get Jodah out a turn sooner and fix colours; Shalai gives Jodah hexproof and soaks up removal; Gix draws off combat damage. The recommended package measured +6.5 with the look-ahead AI. Off-theme picks (not Donato Giancola / Richard Kane Ferguson) were allowed for this measured gain.
**Updated 2026-10-09.** Out: Shalai, Voice of Plenty. In: Tymna the Weaver. JD's choice: Tymna draws a card for each opponent your creatures hit (paying that much life) after combat. In the Shalai slot she measured -0.7 ±0.8 against Shalai, within noise (adaptive, paired, 1,000 games per tier).
**Updated 2026-10-09.** Out: Experimental Augury, Genesis Hydra, Lagrella, the Magpie, Sisters of Stone Death. In: Garland, Royal Kidnapper, Helm of Kaldra, Shield of Kaldra, Sword of Kaldra. JD's choice: the Kaldra pieces back (the deck's art theme) plus Garland, Royal Kidnapper, keeping the mana base and Razia. Measured -1.4 ±1.2 against the list before (adaptive, paired, 1,000 games per tier); the least costly of five ways to fit them. Garland alone measured +1.4.

2026-10-07
**Added 2026-10-07.** In the simulator as deck key `jodah` (`python3 -m commander_sim --deck jodah --pool t4`) and in practice mode.

## Strategy

Jodah, the Unifier ({W}{U}{B}{R}{G}, 5/5) reads: *Legendary creatures you control get +X/+X, where X is the number of legendary creatures you control. Whenever you cast a legendary spell from your hand, exile cards from the top of your library until you exile a legendary nonland card with lesser mana value. You may cast that card without paying its mana cost. Put the rest on the bottom in a random order.*

**Legend cascade.** 27 of the 59 spells are legendary, from Wrenn and Six at two mana up to Razia at eight; the Kaldra pieces count (each is a legendary artifact). With Jodah out, every legendary spell cast from hand brings a free legendary of lower mana value with it: Tolsimir (6) can bring Dragonlord Dromoka, Lyra (5) can bring Elenda, Tymna or Gix. The free card is cast, so Jodah triggers again only for spells cast from hand.

**Legends get big.** Each legendary creature gets +1/+1 for every legendary creature you control, Jodah included. Three legends on the battlefield makes each of them +3/+3; Elenda, Lyra and Kura become threats on their own.

**The pieces.**
- **Card flow:** Fact or Fiction, Memory Jar, Mordenkainen, Tireless Tracker, Urza, Powerstone Prodigy, Court of Ardenvale (the monarch).
- **Mana:** Birds of Paradise, Paradise Druid, Fyndhorn Elder, Arcane Signet, Fellwar Stone, Talisman of Hierarchy, Chromatic Lantern, Coalition Relic, Moss Diamond, Star Compass, Solemn Simulacrum, Cartographer's Survey, Plaza of Heroes (any colour for legendary spells). The simulations found the deck short of mana: the ramp gets Jodah out by turn 4 about twice as often.
- **Tutors:** Demonic, Vampiric, Enlightened, Mystical, Worldly and Profane Tutor find the legend or the answer the game needs.
- **Protection:** Privileged Position (hexproof for your other permanents), Grand Abolisher and Dragonlord Dromoka (opponents can't cast spells during your turn).
- **Interaction:** Force of Will and Arcane Denial; Bitter Triumph, Darksteel Mutation, Skyclave Apparition, Disenchant; Toxic Deluge; Garland, Royal Kidnapper (makes an opponent the monarch and steals their best creature while they keep it).
- **Kaldra:** Sword, Shield and Helm of Kaldra assemble Kaldra, a 4/4 legendary Avatar wearing all three (+5/+5, first strike, trample, haste, indestructible). The simulations found the pieces cost about 1.5 points as kept here (no mana cut for them); they stay for the deck's art theme.
- **Card draw from combat:** Gix, Yawgmoth Praetor draws a card (for 1 life) whenever one of your creatures deals combat damage to an opponent; Tymna the Weaver draws a card (for 1 life each) for every opponent your creatures hit, after combat.

## Key lines

- **Turns 1–4: mana.** Coalition Relic, Star Compass, Fyndhorn Elder or Moss Diamond, then Jodah on turn 4 or 5.
- **Jodah, then legends from hand.** Cast the expensive legends first: a seven- or eight-drop cascades into almost any other legend in the deck.
- **Protect the board.** Privileged Position or Grand Abolisher before the big turn; hold Force of Will for the wipe.

**Mulligan:** keep three lands that make at least three colours, or two lands and a rock. The mana base has 36 lands: 10 painlands, 10 tri-lands (they enter tapped), 2 slow lands (Overgrown Farmland, Shattered Sanctum), Command Tower, Exotic Orchard, Path of Ancestry, Plaza of Heroes, Vivid Creek ({U}, or any colour twice), Unclaimed Territory (any colour for Human creature spells, Jodah included) and 8 basics.

## What the simulator found

First read with the fast AI (loose profile, 300 games per tier; 25% is an even share): 13.7% against Tier 1, 17.3% against Tier 2, 20.3% against Tier 3, 20.0% against Tier 4 and 22.0% against Tier 5. A look-ahead run would give the fuller picture. In 20 traced games against Tier 3, Jodah cascaded into a free legend 26 times.

Modeling: every card is modeled; four are approximate, each missing a minor activated ability: Cromat (the blocker-destroy and top-of-library abilities), King Darien (the sacrifice for protection), Razia (the damage redirect) and Wrenn and Six (the -7 emblem is recorded but not used).

## Bracket and Rule 0

**Bracket 4,** with six Game Changers: **Force of Will**, **Demonic Tutor**, **Vampiric Tutor**, **Mystical Tutor**, **Enlightened Tutor** and **Worldly Tutor**. No two-card infinite combos, no mass land destruction, no extra turns.

Disclose before the game: the six tutors, Force of Will, and Grand Abolisher and Dragonlord Dromoka (no spells during your turn).

Re-check the official list at https://commanderbrackets.com/faq before an event.

## Decklist by type (100)

**Commander (1).** Jodah, the Unifier

**Creatures (27).** Birds of Paradise, Caparocti Sunborn, Carth the Lion, Cromat, Dakkon Blackblade, Dragonlord Dromoka, Elenda, the Dusk Rose, Fyndhorn Elder, Garland, Royal Kidnapper, Gix, Yawgmoth Praetor, Grand Abolisher, King Darien XLVIII, Korlash, Heir to Blackblade, Kura, the Boundless Sky, Lyra Dawnbringer, Marchesa, the Black Rose, Mirri, Weatherlight Duelist, Paradise Druid, Razia, Boros Archangel, Skyclave Apparition, Sol'kanar the Swamp King, Solemn Simulacrum, Terror of the Peaks, Tireless Tracker, Tolsimir Wolfblood, Tymna the Weaver, Urza, Powerstone Prodigy

**Planeswalkers (4).** Dakkon, Shadow Slayer, Mordenkainen, The Aetherspark, Wrenn and Six

**Artifacts (12).** Arcane Signet, Chromatic Lantern, Coalition Relic, Fellwar Stone, Helm of Kaldra, Memory Jar, Mirari, Moss Diamond, Shield of Kaldra, Star Compass, Sword of Kaldra, Talisman of Hierarchy

**Enchantments (3).** Court of Ardenvale, Darksteel Mutation, Privileged Position

**Instants (9).** Arcane Denial, Bitter Triumph, Disenchant, Enlightened Tutor, Fact or Fiction, Force of Will, Mystical Tutor, Vampiric Tutor, Worldly Tutor

**Sorceries (4).** Cartographer's Survey, Demonic Tutor, Profane Tutor, Toxic Deluge

**Lands (40).** Adarkar Wastes, Arcane Sanctum, Battlefield Forge, Brushland, Caves of Koilos, Command Tower, Crumbling Necropolis, Exotic Orchard, Frontier Bivouac, Jungle Shrine, Karplusan Forest, Llanowar Wastes, Mystic Monastery, Nomad Outpost, Opulent Palace, Overgrown Farmland, Path of Ancestry, Plaza of Heroes, Sandsteppe Citadel, Savage Lands, Seaside Citadel, Shattered Sanctum, Shivan Reef, Sulfurous Springs, Unclaimed Territory, Underground River, Vivid Creek, Yavimaya Coast, 3 Forest, 2 Island, 1 Mountain, 3 Plains, 3 Swamp

## Import list (100)

```
1 Adarkar Wastes
1 Arcane Denial
1 Arcane Sanctum
1 Arcane Signet
1 Battlefield Forge
1 Birds of Paradise
1 Bitter Triumph
1 Brushland
1 Caparocti Sunborn
1 Carth the Lion
1 Cartographer's Survey
1 Caves of Koilos
1 Chromatic Lantern
1 Coalition Relic
1 Command Tower
1 Court of Ardenvale
1 Cromat
1 Crumbling Necropolis
1 Dakkon Blackblade
1 Dakkon, Shadow Slayer
1 Darksteel Mutation
1 Demonic Tutor
1 Disenchant
1 Dragonlord Dromoka
1 Elenda, the Dusk Rose
1 Enlightened Tutor
1 Exotic Orchard
1 Fact or Fiction
1 Fellwar Stone
1 Force of Will
1 Frontier Bivouac
1 Fyndhorn Elder
1 Garland, Royal Kidnapper
1 Gix, Yawgmoth Praetor
1 Grand Abolisher
1 Helm of Kaldra
1 Jodah, the Unifier
1 Jungle Shrine
1 Karplusan Forest
1 King Darien XLVIII
1 Korlash, Heir to Blackblade
1 Kura, the Boundless Sky
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
1 Paradise Druid
1 Path of Ancestry
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
1 Skyclave Apparition
1 Sol'kanar the Swamp King
1 Solemn Simulacrum
1 Star Compass
1 Sulfurous Springs
1 Sword of Kaldra
1 Talisman of Hierarchy
1 Terror of the Peaks
1 The Aetherspark
1 Tireless Tracker
1 Tolsimir Wolfblood
1 Toxic Deluge
1 Tymna the Weaver
1 Unclaimed Territory
1 Underground River
1 Urza, Powerstone Prodigy
1 Vampiric Tutor
1 Vivid Creek
1 Worldly Tutor
1 Wrenn and Six
1 Yavimaya Coast
3 Forest
2 Island
1 Mountain
3 Plains
3 Swamp
```
