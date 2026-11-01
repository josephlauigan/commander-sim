# Thanos, the Mad Titan — Five-Colour cEDH Turbo

JD's five-colour cEDH turbo deck with Thanos, the Mad Titan as the commander. It started on 10/30 as the Scion v1 list
(`decklists/pool/retired/scion-ur-dragon-turbo.md`, retired 10/30) minus Scion's copy targets; on 11/01 the list was
brought in line with JD's physical deck (22 cards each way, see the CHANGELOG). Not simulated since.

- **Commander:** Thanos, the Mad Titan
- **Color identity:** White, Blue, Black, Red, Green (his power-up cost has all five colours)
- **Game Changers (23):** Ad Nauseam, Ancient Tomb, Chrome Mox, Demonic Tutor, Enlightened Tutor, Fierce Guardianship, Force of Will, Gamble, Grim Monolith, Imperial Seal, Intuition, Jeska's Will, Lion's Eye Diamond, Mana Vault, Mox Diamond, Mystical Tutor, Necropotence, Orcish Bowmasters, Rhystic Study, Thassa's Oracle, Underworld Breach, Vampiric Tutor, Worldly Tutor
- **Simulator:** deck key `thanos`, Tier 6 (Scion's seat, 10/30). The power-up is card code (`impls/t6_thanos.rs`), the plan is `data/plans/thanos.toml`; results in `audit/thanos/FINDINGS.md`.

## Strategy

Fast mana and rituals into the cEDH finishes: Thassa's Oracle with Demonic Consultation or Tainted Pact, Underworld
Breach with Lion's Eye Diamond and Brain Freeze, and Ad Nauseam to dig for either. Free counters, Silence, Orim's
Chant, Grand Abolisher, Voice of Victory, Teferi, Time Raveler and Dragonlord Dromoka protect the win turn. The
commander is not the plan; he is the five-colour identity, a cheap body, and the switch for the free spells.

### Why Thanos instead of Scion

- **He is cheap.** Thanos costs {R}{W}{B}: a 4/4 with deathtouch and lifelink on turn 2-3 off a mana creature, and
  cheap to recast. Scion needs exactly {W}{U}{B}{R}{G}.
- **He makes two spells free.** Fierce Guardianship (counter a noncreature spell) and Deflecting Swat (new targets
  for a spell or ability) cost nothing while you control your commander. A 3-mana commander is usually on the
  battlefield; Scion rarely was.
- **Scion's engine barely won.** In the goldfish, the Prismari line was 0.9% of Scion's kills and Thassa's Oracle 73%.
  Ancient Copper Dragon, Korvold and Prismari were copy targets for Scion and dead cards without him.
- **What Scion had that Thanos doesn't:** a backup win from the command zone (become Prismari for storm, or Ancient
  Copper Dragon for Treasures). If the Oracle and Breach lines are both stopped, Thanos has no third plan.

### The power-up

*Power-up — {C}{W}{U}{B}{R}{G}: put two +1/+1 counters on Thanos. Choose odd or even. Destroy each other creature with
mana value of the chosen quality.* Once per game; the cost drops by his mana cost ({R}{W}{B}) on the turn he enters,
to {C}{U}{G}. Zero is even.

- **{C} is colourless mana, not generic.** Six sources pay it: Sol Ring, Mana Vault, Grim Monolith, Ancient Tomb,
  Cavern of Souls and Gemstone Caverns. Mana Confluence, the duals, Lotus Petal, the Moxen, Treasures and Command
  Tower can't. No Wastes needed.
- **Choose even, almost always.** Your mana creatures are odd and survive; the creature stax and most Tier 6
  commanders are even. Even also kills your own Grand Abolisher and Voice of Victory, so wipe on a turn you aren't
  relying on them.

| | Your creatures | What it hits at the table |
|---|---|---|
| **Even** | Thassa's Oracle, Orcish Bowmasters, Lotho, Hexing Squelcher, Grand Abolisher, Voice of Victory (2) | Grand Abolisher, Drannith Magistrate, Collector Ouphe, Dauthi Voidwalker (2); Kinnan, Tymna, Thrasios, Sisay (2), Rograkh (0) |
| **Odd** | Birds, Noble Hierarch, Ragavan, Esper Sentinel, Mockingbird (1); Ranger-Captain of Eos, Valley Floodcaller (3); Dragonlord Dromoka (7) | Opponents' mana creatures and Ragavan (1); Silas Renn, Opposition Agent, Spellseeker (3); Kraum (5) |

Thassa's Oracle is usually cast on the win turn, so it is rarely on the battlefield when you wipe. It is a one-shot
answer to creature stax and creature commanders, not a win condition: six mana on the turn he enters is often enough
to just win instead.

## Bracket and Rule 0

Bracket 5 (cEDH): 23 Game Changers, Thassa's Oracle and Underworld Breach combos, free counters, Silence effects.

## Decklist by type (100)
**Commander (1).** Thanos, the Mad Titan

**Creatures (16).** Birds of Paradise, Dragonlord Dromoka, Elvish Spirit Guide, Esper Sentinel, Grand Abolisher, Hexing Squelcher, Lotho Corrupt Shirriff, Mockingbird, Noble Hierarch, Orcish Bowmasters, Ragavan Nimble Pilferer, Ranger-Captain of Eos, Simian Spirit Guide, Thassa's Oracle, Valley Floodcaller, Voice of Victory

**Enchantments (5).** Mystic Remora, Necropotence, Rhystic Study, Sylvan Library, Underworld Breach

**Artifacts (7).** Chrome Mox, Grim Monolith, Lion's Eye Diamond, Lotus Petal, Mana Vault, Mox Diamond, Sol Ring

**Sorceries (10).** Beseech the Mirror, Demonic Tutor, Diabolic Intent, Gamble, Gitaxian Probe, Grim Tutor, Imperial Seal, Jeska's Will, Merchant Scroll, Rite of Flame

**Instants (33).** Ad Nauseam, Borne Upon a Wind, Brain Freeze, Cabal Ritual, Chain of Vapor, Dark Ritual, Deflecting Swat, Demonic Consultation, Desperate Ritual, Eladamri's Call, Enlightened Tutor, Fierce Guardianship, Flusterstorm, Force of Negation, Force of Will, Into the Flood Maw, Intuition, Manamorphose, Mental Misstep, Mindbreak Trap, Mystical Tutor, Noxious Revival, Orim's Chant, Pact of Negation, Red Elemental Blast, Seething Song, Silence, Snapback, Swan Song, Tainted Pact, Vampiric Tutor, Veil of Summer, Worldly Tutor

**Lands (27).** Ancient Tomb, Arid Mesa, Badlands, Bayou, Bloodstained Mire, Boseiju Who Endures, Cavern of Souls, Command Tower, Flooded Strand, Gemstone Caverns, Mana Confluence, Marsh Flats, Misty Rainforest, Otawara Soaring City, Plateau, Polluted Delta, Savannah, Scalding Tarn, Scrubland, Taiga, Tropical Island, Tundra, Underground Sea, Verdant Catacombs, Volcanic Island, Windswept Heath, Wooded Foothills

**Planeswalkers (1).** Teferi Time Raveler

## Import list (100)

```
1 Ad Nauseam
1 Ancient Tomb
1 Arid Mesa
1 Badlands
1 Bayou
1 Beseech the Mirror
1 Birds of Paradise
1 Bloodstained Mire
1 Borne Upon a Wind
1 Boseiju, Who Endures
1 Brain Freeze
1 Cabal Ritual
1 Cavern of Souls
1 Chain of Vapor
1 Chrome Mox
1 Command Tower
1 Dark Ritual
1 Deflecting Swat
1 Demonic Consultation
1 Demonic Tutor
1 Desperate Ritual
1 Diabolic Intent
1 Dragonlord Dromoka
1 Eladamri's Call
1 Elvish Spirit Guide
1 Enlightened Tutor
1 Esper Sentinel
1 Fierce Guardianship
1 Flooded Strand
1 Flusterstorm
1 Force of Negation
1 Force of Will
1 Gamble
1 Gemstone Caverns
1 Gitaxian Probe
1 Grand Abolisher
1 Grim Monolith
1 Grim Tutor
1 Hexing Squelcher
1 Imperial Seal
1 Into the Flood Maw
1 Intuition
1 Jeska's Will
1 Lion's Eye Diamond
1 Lotho, Corrupt Shirriff
1 Lotus Petal
1 Mana Confluence
1 Mana Vault
1 Manamorphose
1 Marsh Flats
1 Mental Misstep
1 Merchant Scroll
1 Mindbreak Trap
1 Misty Rainforest
1 Mockingbird
1 Mox Diamond
1 Mystic Remora
1 Mystical Tutor
1 Necropotence
1 Noble Hierarch
1 Noxious Revival
1 Orcish Bowmasters
1 Orim's Chant
1 Otawara, Soaring City
1 Pact of Negation
1 Plateau
1 Polluted Delta
1 Ragavan, Nimble Pilferer
1 Ranger-Captain of Eos
1 Red Elemental Blast
1 Rhystic Study
1 Rite of Flame
1 Savannah
1 Scalding Tarn
1 Scrubland
1 Seething Song
1 Silence
1 Simian Spirit Guide
1 Snapback
1 Sol Ring
1 Swan Song
1 Sylvan Library
1 Taiga
1 Tainted Pact
1 Teferi, Time Raveler
1 Thanos, the Mad Titan
1 Thassa's Oracle
1 Tropical Island
1 Tundra
1 Underground Sea
1 Underworld Breach
1 Valley Floodcaller
1 Vampiric Tutor
1 Veil of Summer
1 Verdant Catacombs
1 Voice of Victory
1 Volcanic Island
1 Windswept Heath
1 Wooded Foothills
1 Worldly Tutor
```
