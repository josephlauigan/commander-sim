# Heliod, Sun-Crowned — Mono-White Stax

Opponent pool deck — **Tier 4: Low Bracket 4**. Axis: Life gain engine with light stax and a compact combo.

- **Commander:** Heliod, Sun-Crowned
- **Color identity:** White
- **Game Changers (7):** Ancient Tomb, Drannith Magistrate, Enlightened Tutor, Farewell, Smothering Tithe, Teferi's Protection, The One Ring
- **Combos:** Heliod, Sun-Crowned + Walking Ballista, and Heliod + Triskelion (Heliod gives the pinger lifelink; every ping gains life, which puts the counter back: unbounded damage). The commander is half of both; Recruiter of the Guard and Ranger-Captain of Eos find them. Felidar Sovereign (40 life at your upkeep) and Aetherflux Reservoir (pay 50 life) are alternate kills the life gain feeds.
- **Lands:** 33 (9 nonbasic + 24 basic)

## Role in the pool

A life-gain engine. Small incidental gains (Soul Warden, Suture Priest, Daxos, Haliya) are turned into counters, cards and a fast Felidar Sovereign or Aetherflux kill, while a few stax pieces (Drannith Magistrate, Deafening Silence, Authority of the Consuls, Blind Obedience, Ghostly Prison) slow the table. The commander is half of a two-card combo.

## At the table

- **Plan:** Gains life every time a creature enters (Soul Warden, Soul's Attendant, Suture Priest, Daxos, Haliya, Authority of the Consuls) or attacks (Linden), and multiplies it with Cleric Class and The Wind Crystal. Heliod, Ajani's Pridemate, Archangel of Thune, Voice of the Blessed, Exemplar of Light and Cleric Class turn each gain into +1/+1 counters; Well of Lost Dreams, Cosmos Elixir, Archivist of Oghma and The One Ring turn it into cards.
- **Wins by:** Heliod with Walking Ballista or Triskelion (unbounded pings). Felidar Sovereign wins at its controller's upkeep with 40 or more life, the starting total; Aetherflux Reservoir fires once the life is above 50. Otherwise Serra Ascendant, Archangel of Thune, Speaker of the Heavens and the grown lifelinkers beat down.
- **Watch out for:** Felidar Sovereign on the battlefield (it wins on the next upkeep unless the life drops under 40), Heliod with Ballista or Triskelion, and Aetherflux Reservoir at high life. Authority of the Consuls and Blind Obedience make your creatures enter tapped; Drannith Magistrate shuts off commanders, Deafening Silence caps noncreature spells.
- **Interaction:** Swords to Plowshares, Path to Exile, Generous Gift, Stroke of Midnight and Heliod's Intervention (X artifacts and enchantments); sweepers Wrath of God, Fumigate, Austere Command and Farewell. Teferi's Protection, Flawless Maneuver, Mother of Runes, Giver of Runes, Reprieve and Silence protect.

## Strategy

The deck lives on small, repeated life gains: every creature entering under Soul Warden, Soul's Attendant, Suture Priest, Daxos or Haliya, every attack under Linden, every Ajani +1. Cleric Class adds one to each gain and The Wind Crystal doubles it, so a single Soul Warden trigger can be four life. The gains feed counters (Heliod, Ajani's Pridemate, Archangel of Thune, Voice of the Blessed, Cleric Class level 2), cards (Well of Lost Dreams, Cosmos Elixir, Archivist of Oghma off every opposing tutor) and the alternate kills: Felidar Sovereign at 40, Aetherflux Reservoir past 50, Ajani's 0 at 55. Heliod with Walking Ballista or Triskelion ends the game on the spot. Smothering Tithe, Land Tax, Esper Sentinel and The One Ring keep the cards coming; the sweepers reset a board the deck rebuilds quickly.

Built as Low Bracket 4: Enlightened Tutor and the creature tutors (Recruiter of the Guard, Ranger-Captain of Eos, Ranger of Eos) rather than a full tutor suite.

## Sim modeling notes

Model these first; if any is Partial or unmodeled, this deck plays below its real strength and its results should be flagged.

Life gain events and their "whenever you gain life" triggers (compiled ones fire from engine/life.rs); the gain replacements (Cleric Class plus 1, The Wind Crystal doubling); Heliod + Walking Ballista and Heliod + Triskelion; Felidar Sovereign's upkeep win; Aetherflux Reservoir; Well of Lost Dreams draws; Authority of the Consuls / Blind Obedience enter-tapped replacements; Drannith Magistrate and Deafening Silence. Archivist of Oghma and Haliya are Approximate (search detection by shuffle; Haliya's warp left out).

## Decklist by type (100)

**Commander (1).** Heliod, Sun-Crowned

**Creatures (30).** Ajani's Pridemate, Archangel of Thune, Archivist of Oghma, Auriok Champion, Aven Mindcensor, Daxos, Blessed by the Sun, Drannith Magistrate, Esper Sentinel, Exemplar of Light, Felidar Sovereign, Giver of Runes, Grand Abolisher, Haliya, Guided by Light, Linden, the Steadfast Queen, Mangara, the Diplomat, Mother of Runes, Ranger of Eos, Ranger-Captain of Eos, Recruiter of the Guard, Serra Ascendant, Soul Warden, Soul's Attendant, Speaker of the Heavens, Sun Titan, Sunscorch Regent, Suture Priest, Triskelion, Voice of the Blessed, Walking Ballista, Weathered Wayfarer

**Planeswalkers (1).** Ajani, Strength of the Pride

**Enchantments (9).** Ajani's Welcome, Authority of the Consuls, Blind Obedience, Cleric Class, Deafening Silence, Ghostly Prison, Land Tax, Smothering Tithe, Trouble in Pairs

**Artifacts (12).** Aetherflux Reservoir, Altar of the Pantheon, Arcane Signet, Cosmos Elixir, Mind Stone, Oketra's Monument, Pearl Medallion, Pristine Talisman, Sol Ring, The One Ring, The Wind Crystal, Well of Lost Dreams

**Sorceries (4).** Austere Command, Farewell, Fumigate, Wrath of God

**Instants (10).** Enlightened Tutor, Flawless Maneuver, Generous Gift, Heliod's Intervention, Path to Exile, Reprieve, Silence, Stroke of Midnight, Swords to Plowshares, Teferi's Protection

**Lands (33).** Ancient Tomb, Buried Ruin, Castle Ardenvale, Eiganjo, Seat of the Empire, Emeria, the Sky Ruin, Hall of Heliod's Generosity, Nykthos, Shrine to Nyx, Radiant Fountain, War Room, 24 Plains

## Import list (100)

```
1 Aetherflux Reservoir
1 Ajani's Pridemate
1 Ajani's Welcome
1 Ajani, Strength of the Pride
1 Altar of the Pantheon
1 Ancient Tomb
1 Arcane Signet
1 Archangel of Thune
1 Archivist of Oghma
1 Auriok Champion
1 Austere Command
1 Authority of the Consuls
1 Aven Mindcensor
1 Blind Obedience
1 Buried Ruin
1 Castle Ardenvale
1 Cleric Class
1 Cosmos Elixir
1 Daxos, Blessed by the Sun
1 Deafening Silence
1 Drannith Magistrate
1 Eiganjo, Seat of the Empire
1 Emeria, the Sky Ruin
1 Enlightened Tutor
1 Esper Sentinel
1 Exemplar of Light
1 Farewell
1 Felidar Sovereign
1 Flawless Maneuver
1 Fumigate
1 Generous Gift
1 Ghostly Prison
1 Giver of Runes
1 Grand Abolisher
1 Haliya, Guided by Light
1 Hall of Heliod's Generosity
1 Heliod's Intervention
1 Heliod, Sun-Crowned
1 Land Tax
1 Linden, the Steadfast Queen
1 Mangara, the Diplomat
1 Mind Stone
1 Mother of Runes
1 Nykthos, Shrine to Nyx
1 Oketra's Monument
1 Path to Exile
1 Pearl Medallion
1 Pristine Talisman
1 Radiant Fountain
1 Ranger of Eos
1 Ranger-Captain of Eos
1 Recruiter of the Guard
1 Reprieve
1 Serra Ascendant
1 Silence
1 Smothering Tithe
1 Sol Ring
1 Soul Warden
1 Soul's Attendant
1 Speaker of the Heavens
1 Stroke of Midnight
1 Sun Titan
1 Sunscorch Regent
1 Suture Priest
1 Swords to Plowshares
1 Teferi's Protection
1 The One Ring
1 The Wind Crystal
1 Triskelion
1 Trouble in Pairs
1 Voice of the Blessed
1 Walking Ballista
1 War Room
1 Weathered Wayfarer
1 Well of Lost Dreams
1 Wrath of God
24 Plains
```
