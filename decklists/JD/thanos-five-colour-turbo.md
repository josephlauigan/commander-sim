# Thanos, the Mad Titan — Five-Colour cEDH Turbo

The Scion v1 cEDH list (`decklists/JD/scion-ur-dragon-turbo.md`) with Thanos, the Mad Titan as the commander. The 96
cards that don't depend on the commander are unchanged: the fast mana and rituals, the tutors, the free counters and
both win lines. The three Dragons that were there only for Scion to copy are out, and three spells that are free while
you control your commander are in. JD's choice (10/30), not simulated; the Scion deck stays as the baseline.

- **Commander:** Thanos, the Mad Titan
- **Color identity:** White, Blue, Black, Red, Green (his power-up cost has all five colours)
- **Game Changers (21):** Ancient Tomb, Chrome Mox, Crop Rotation, Demonic Tutor, Enlightened Tutor, Fierce Guardianship, Force of Will, Gamble, Imperial Seal, Intuition, Lion's Eye Diamond, Mana Vault, Mox Diamond, Mystical Tutor, Necropotence, Orcish Bowmasters, Rhystic Study, Smothering Tithe, Thassa's Oracle, Underworld Breach, Vampiric Tutor
- **Simulator:** not wired in yet. Thanos's power-up needs a model and the deck needs a key; the three free spells are already in `data/cards.json`.

## Strategy

Fast mana and rituals into the cEDH finishes: Thassa's Oracle with Demonic Consultation or Tainted Pact, and
Underworld Breach with Lion's Eye Diamond and Brain Freeze. Free counters, Silence and Orim's Chant protect the win
turn. The commander is not the plan; he is the five-colour identity, a cheap body, and the switch for the free spells.

### Why Thanos instead of Scion

- **He is cheap.** Thanos costs {R}{W}{B}: a 4/4 with deathtouch and lifelink on turn 2-3 off a mana creature, and
  cheap to recast. Scion needs exactly {W}{U}{B}{R}{G}.
- **He makes three spells free.** Fierce Guardianship (counter a noncreature spell), Deflecting Swat (new targets for
  a spell or ability) and Deadly Rollick (exile a creature) cost nothing while you control your commander. A 3-mana
  commander is usually on the battlefield; Scion rarely was.
- **Scion's engine barely won.** In the goldfish, the Prismari line was 0.9% of Scion's kills and Thassa's Oracle 73%.
  Ancient Copper Dragon, Korvold and Prismari were copy targets for Scion and dead cards without him.
- **What Scion had that Thanos doesn't:** a backup win from the command zone (become Prismari for storm, or Ancient
  Copper Dragon for Treasures). If the Oracle and Breach lines are both stopped, Thanos has no third plan.

### The power-up

*Power-up — {C}{W}{U}{B}{R}{G}: put two +1/+1 counters on Thanos. Choose odd or even. Destroy each other creature with
mana value of the chosen quality.* Once per game; the cost drops by his mana cost ({R}{W}{B}) on the turn he enters,
to {C}{U}{G}. Zero is even.

- **{C} is colourless mana, not generic.** Six sources pay it: Sol Ring, Mana Vault, Ancient Tomb, Emergence Zone,
  Starting Town and Tarnished Citadel. City of Brass, the duals, Lotus Petal, Treasures, Command Tower and Arcane
  Signet can't. No Wastes needed.
- **Choose even, almost always.** Your mana creatures are odd and survive; the creature stax and most Tier 6
  commanders are even.

| | Your creatures | What it hits at the table |
|---|---|---|
| **Even** | Thassa's Oracle, Orcish Bowmasters, Faerie Mastermind, Lotho, Hexing Squelcher, Tataru Taru (2); Storm-Kiln Artist (4) | Grand Abolisher, Drannith Magistrate, Collector Ouphe, Dauthi Voidwalker (2); Kinnan, Tymna, Thrasios, Sisay (2), Rograkh (0) |
| **Odd** | Birds, Noble and Ignoble Hierarch, Deathrite Shaman, Tinder Wall, Ragavan, Esper Sentinel, Mockingbird (1); Ranger-Captain of Eos, The Cabbage Merchant, Valley Floodcaller (3) | Opponents' mana creatures and Ragavan (1); Silas Renn, Opposition Agent, Spellseeker (3); Kraum (5) |

Thassa's Oracle is usually cast on the win turn, so it is rarely on the battlefield when you wipe. It is a one-shot
answer to creature stax and creature commanders, not a win condition: six mana on the turn he enters is often enough
to just win instead.

## Bracket and Rule 0

Bracket 5 (cEDH): 21 Game Changers, Thassa's Oracle and Underworld Breach combos, free counters, Silence effects.

## Decklist by type (100)
**Commander (1).** Thanos, the Mad Titan

**Creatures (20).** Birds of Paradise, Deathrite Shaman, Elvish Spirit Guide, Esper Sentinel, Faerie Mastermind, Hexing Squelcher, Ignoble Hierarch, Lotho Corrupt Shirriff, Mockingbird, Noble Hierarch, Orcish Bowmasters, Ragavan Nimble Pilferer, Ranger-Captain of Eos, Simian Spirit Guide, Storm-Kiln Artist, Tataru Taru, Thassa's Oracle, The Cabbage Merchant, Tinder Wall, Valley Floodcaller

**Enchantments (5).** Mystic Remora, Necropotence, Rhystic Study, Smothering Tithe, Underworld Breach

**Artifacts (7).** Arcane Signet, Chrome Mox, Lion's Eye Diamond, Lotus Petal, Mana Vault, Mox Diamond, Sol Ring

**Sorceries (8).** Beseech the Mirror, Demonic Tutor, Diabolic Intent, Flash Photography, Gamble, Imperial Seal, Rite of Flame, Sevinne's Reclamation

**Instants (31).** An Offer You Can't Refuse, Borne Upon a Wind, Brain Freeze, Cabal Ritual, Chain of Vapor, Crop Rotation, Dark Ritual, Deadly Rollick, Deflecting Swat, Demonic Consultation, Enlightened Tutor, Fierce Guardianship, Final Fortune, Flusterstorm, Force of Negation, Force of Will, Intuition, Manamorphose, Mental Misstep, Mindbreak Trap, Misdirection, Mystical Tutor, Noxious Revival, Orim's Chant, Pact of Negation, Redirect Lightning, Silence, Swan Song, Tainted Pact, Vampiric Tutor, Veil of Summer

**Lands (28).** Ancient Tomb, Arid Mesa, Badlands, Bayou, Bloodstained Mire, Boseiju Who Endures, City of Brass, Command Tower, Emergence Zone, Flooded Strand, Gemstone Caverns, Mana Confluence, Marsh Flats, Misty Rainforest, Plateau, Polluted Delta, Savannah, Scalding Tarn, Scrubland, Shifting Woodland, Starting Town, Tarnished Citadel, Tropical Island, Tundra, Underground Sea, Verdant Catacombs, Volcanic Island, Windswept Heath

## Import list (100)

```
1 An Offer You Can't Refuse
1 Ancient Tomb
1 Arcane Signet
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
1 Chain of Vapor
1 Chrome Mox
1 City of Brass
1 Command Tower
1 Crop Rotation
1 Dark Ritual
1 Deadly Rollick
1 Deathrite Shaman
1 Deflecting Swat
1 Demonic Consultation
1 Demonic Tutor
1 Diabolic Intent
1 Elvish Spirit Guide
1 Emergence Zone
1 Enlightened Tutor
1 Esper Sentinel
1 Faerie Mastermind
1 Fierce Guardianship
1 Final Fortune
1 Flash Photography
1 Flooded Strand
1 Flusterstorm
1 Force of Negation
1 Force of Will
1 Gamble
1 Gemstone Caverns
1 Hexing Squelcher
1 Ignoble Hierarch
1 Imperial Seal
1 Intuition
1 Lion's Eye Diamond
1 Lotho, Corrupt Shirriff
1 Lotus Petal
1 Mana Confluence
1 Mana Vault
1 Manamorphose
1 Marsh Flats
1 Mental Misstep
1 Mindbreak Trap
1 Misdirection
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
1 Pact of Negation
1 Plateau
1 Polluted Delta
1 Ragavan, Nimble Pilferer
1 Ranger-Captain of Eos
1 Redirect Lightning
1 Rhystic Study
1 Rite of Flame
1 Savannah
1 Scalding Tarn
1 Scrubland
1 Sevinne's Reclamation
1 Shifting Woodland
1 Silence
1 Simian Spirit Guide
1 Smothering Tithe
1 Sol Ring
1 Starting Town
1 Storm-Kiln Artist
1 Swan Song
1 Tainted Pact
1 Tarnished Citadel
1 Tataru Taru
1 Thanos, the Mad Titan
1 Thassa's Oracle
1 The Cabbage Merchant
1 Tinder Wall
1 Tropical Island
1 Tundra
1 Underground Sea
1 Underworld Breach
1 Valley Floodcaller
1 Vampiric Tutor
1 Veil of Summer
1 Verdant Catacombs
1 Volcanic Island
1 Windswept Heath
```
