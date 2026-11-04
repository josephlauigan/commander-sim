# The Ur-Dragon — Five-Colour Dragon Copies

A five-colour Dragon deck that turns one Dragon into several. Miirym, Lathliss and token multipliers reward Dragons
entering, while Dragon Tempest, Scourge of Valkas and Terror of the Peaks turn those entries into damage. Astral Dragon
is the signature card: it turns noncreature permanents into pairs of flying 3/3 Dragon copies. The Ur-Dragon discounts
the creature suite from the command zone, then converts the resulting air force into cards and free permanents when it
finally attacks. The deck is weak to early combo, sweepers before its draw engines appear, and hands with expensive
Dragons but no acceleration.

| | |
|---|---|
| **Commander** | The Ur-Dragon ({4}{W}{U}{B}{R}{G}, 10/10, five colours) |
| **Bracket** | 4 (16 Game Changers; no intentional infinite combo) |
| **Simulator** | Deck key `urdragon`, Tier 5, played from `data/plans/urdragon.toml`; results in `audit/urdragon/FINDINGS.md`. |

## Strategy

### The commander

The Ur-Dragon's Eminence ability reduces every other Dragon spell by {1} while it is in the command zone or on the
battlefield. When one or more Dragons attack while The Ur-Dragon is on the battlefield, draw that many cards and put
one permanent from your hand onto the battlefield.

- Treat Eminence as dependable ramp from turn one. Do not wait to cast the commander before developing Dragons.
- Cast The Ur-Dragon when it can attack soon, when Lightning Greaves supplies haste, or when the table has exhausted
  its removal. Its first attack should refill the hand and put the best permanent drawn directly onto the battlefield.
- Dragon tokens count for the attack trigger. Utvara Hellkite's tokens arrive after attackers are declared, so they
  attack and count on later combats rather than the combat that created them.

### Game plan

**Opening hand.** Keep two or more usable mana sources plus acceleration. A hand with Dragon's Hoard, a mana creature,
Nature's Lore, Rampant Growth or fast mana is better than a hand full of six-drops. Keep an entry payoff when the mana
supports it; ship hands whose first meaningful play is turn five.

**Turns 1–3.** Fix colours and ramp. Dragon's Hoard is both fixing and later card draw. Dragon Tempest, Garruk's
Uprising and Temur Ascendancy are good setup plays when they do not delay necessary mana.

**Turns 4–6.** Establish one engine, then cast Dragons. Ganax pays for the next Dragon, Lathliss supplies bodies,
Miirym supplies copies, and Scourge of Valkas or Terror of the Peaks converts every new body into damage. Hold cheap
interaction for a spell that would end the game or erase the developing board.

**Late game.** Attack with the Dragon army. Utvara Hellkite multiplies attackers, Atarka gives Dragons double strike,
and The Ur-Dragon refills the hand and places another engine or Dragon onto the battlefield. Dracogenesis turns every
Dragon in hand into a free spell once eight mana is available.

### Entry and token engines

- **Miirym, Sentinel Wyrm:** each other nontoken Dragon entering creates a nonlegendary token copy. Copy Dragons with
  entry triggers first; both the original and token produce their own triggers.
- **Lathliss, Dragon Queen:** each other nontoken Dragon entering creates a 5/5 Dragon. The token does not retrigger
  Lathliss, but it does trigger Ganax, Dragon Tempest, Scourge and Terror.
- **Ganax, Astral Hunter:** every Dragon entering creates a Treasure, including tokens from Miirym, Lathliss, Astral
  Dragon and Utvara Hellkite.
- **Panharmonicon:** adds another applicable entry trigger from Miirym, Lathliss, Ganax, Astral Dragon, Dragon Tempest,
  Scourge of Valkas and Terror of the Peaks. It adds triggers; it does not copy the resulting tokens directly.
- **Maskwood Nexus:** makes every creature spell and creature a Dragon. Mana creatures and utility creatures then get
  the Eminence discount, trigger the entry engines and count for The Ur-Dragon's attack.

### Copies

**Astral Dragon** creates two token copies of a target noncreature permanent, except each is also a 3/3 flying Dragon.
Good targets include Rhystic Study, Smothering Tithe, Dragon Tempest, Kindred Discovery, Garruk's Uprising,
Panharmonicon, Dragon's Hoard and useful opposing permanents. It can copy a land when another mana source or utility
land is the best available target. Avoid copying your own legendary permanent unless losing the extras to the legend
rule is acceptable.

**Spark Double** copies your best creature without being legendary. Miirym is usually the highest ceiling, followed by
Lathliss, Ganax, Terror of the Peaks or The Ur-Dragon. **Cursed Mirror** is a mana rock that can be the best creature on
the battlefield for one turn. **Jaxis, the Troublemaker** and **Twinflame** make temporary copies; use them on Dragons
with entry triggers or on an engine that makes the next Dragon much stronger. **Sarkhan, Soul Aflame** discounts
Dragons and can become each Dragon that enters for the turn.

### Turning entries into a win

- **Dragon Tempest:** haste for the flying army and damage whenever a Dragon enters.
- **Scourge of Valkas:** another Dragon-entry damage engine on a flying body.
- **Terror of the Peaks:** each creature entering deals damage equal to its power, so Lathliss and Utvara tokens hit
  especially hard.
- **Utvara Hellkite:** creates a 6/6 Dragon for every Dragon that attacks.
- **Atarka, World Render:** gives attacking Dragons double strike and ends games without requiring another setup card.
- **Wrathful Red Dragon:** makes combat and damage-based sweepers dangerous for opponents by reflecting damage dealt to
  your Dragons.
- **Dragonborn Champion:** turns the deck's five-damage events into cards.

### Interaction and protection

Force of Will, Force of Negation, Pact of Negation and Mana Drain protect the board and stop faster combo decks.
Swords to Plowshares, Chaos Warp and Decimate answer permanents. Kindred Dominance names Dragon and normally leaves the
main board intact; Maskwood Nexus protects the utility creatures too. Teferi's Protection and Lightning Greaves cover
the turn when the deck commits its most important engine.

### Card flow and tutors

Kindred Discovery names Dragon and draws on both entries and attacks. Garruk's Uprising and Temur Ascendancy draw from
the large creatures, Dragon's Hoard spends its gold counters for cards, and Rhystic Study, Sylvan Library and The One
Ring keep the hand moving before the creature engines start.

Worldly Tutor finds the Dragon needed for the board: Ganax for mana, Miirym or Lathliss for multiplication, Scourge or
Terror for damage, Utvara for a developed attack, and Atarka for lethal combat. Enlightened Tutor finds Dragon Tempest,
Panharmonicon, Kindred Discovery, Maskwood Nexus, Dragon's Hoard or Dracogenesis. The unrestricted tutors can find
either half of that choice.

## Key lines

- **Need mana:** Ganax or Dragon's Hoard, then let later Dragon entries fund the turn.
- **Need bodies:** Lathliss first, then cast nontoken Dragons.
- **Need copies:** Miirym first; Spark Double can become a second Miirym.
- **Need damage:** Dragon Tempest, Scourge of Valkas or Terror of the Peaks before making tokens.
- **Ready to attack:** Utvara makes the army; Atarka doubles its combat damage; The Ur-Dragon turns the attack into
  cards and a free permanent.
- **Astral Dragon target:** copy a noncreature engine that remains useful in multiples. Rhystic Study and Smothering
  Tithe generate long-term value; Dragon Tempest and Panharmonicon accelerate a finishing turn.

## Mana and consistency

The deck has 35 lands, including Maelstrom of the Spirit Dragon as a five-colour Dragon source and late tutor. The
original duals, shock lands and fetch lands provide every colour; Nature's Lore can find any Forest dual or shock.
Ramp comes from Sol Ring, Mana Vault, Grim Monolith, Chrome Mox, Mox Diamond, Lotus Petal, Arcane Signet, Chromatic
Lantern, Cursed Mirror, Dragon's Hoard, three mana creatures and two land-ramp spells. Mox Jasper produces mana only
after a Dragon is on the battlefield, or any creature while Maskwood Nexus is out, so do not count it as an opening
source by itself.

## Bracket and Rule 0

Bracket 4 with 16 Game Changers: Ancient Tomb, Chrome Mox, Demonic Tutor, Enlightened Tutor, Force of Will, Gaea's
Cradle, Grim Monolith, Imperial Seal, Mana Vault, Mox Diamond, Rhystic Study, Smothering Tithe, Teferi's Protection,
The One Ring, Vampiric Tutor and Worldly Tutor. The rebuilt list has no intentional infinite combo. Disclose the fast
mana and tutors, and explain that the deck multiplies Dragon entry and attack triggers rather than assembling Godo and
Helm of the Host.

## Decklist by type (100)
**Commander (1).** The Ur-Dragon

**Creatures (23).** Astral Dragon, Atarka World Render, Birds of Paradise, Dragonborn Champion, Dragonlord Ojutai, Ganax Astral Hunter, Glorybringer, Goldspan Dragon, Jaxis the Troublemaker, Kura the Boundless Sky, Lathliss Dragon Queen, Llanowar Elves, Miirym Sentinel Wyrm, Nicol Bolas the Ravager, Paradise Druid, Sakura-Tribe Elder, Sarkhan Soul Aflame, Scourge of Valkas, Spark Double, Terror of the Peaks, Thundermaw Hellkite, Utvara Hellkite, Wrathful Red Dragon

**Planeswalkers (1).** Wrenn and Six

**Enchantments (8).** Dracogenesis, Dragon Tempest, Garruk's Uprising, Kindred Discovery, Rhystic Study, Smothering Tithe, Sylvan Library, Temur Ascendancy

**Artifacts (15).** Arcane Signet, Chromatic Lantern, Chrome Mox, Cursed Mirror, Dragon's Hoard, Grim Monolith, Lightning Greaves, Lotus Petal, Mana Vault, Maskwood Nexus, Mox Diamond, Mox Jasper, Panharmonicon, Sol Ring, The One Ring

**Sorceries (7).** Decimate, Demonic Tutor, Imperial Seal, Kindred Dominance, Nature's Lore, Rampant Growth, Twinflame

**Instants (10).** Chaos Warp, Enlightened Tutor, Force of Negation, Force of Will, Mana Drain, Pact of Negation, Swords to Plowshares, Teferi's Protection, Vampiric Tutor, Worldly Tutor

**Lands (35).** Ancient Tomb, Badlands, Bayou, Blood Crypt, Bloodstained Mire, Breeding Pool, Cavern of Souls, City of Brass, Command Tower, Flooded Strand, Gaea's Cradle, Hallowed Fountain, Inventors' Fair, Maelstrom of the Spirit Dragon, Mana Confluence, Marsh Flats, Misty Rainforest, Overgrown Tomb, Plateau, Polluted Delta, Sacred Foundry, Scrubland, Steam Vents, Stomping Ground, Taiga, Temple Garden, Tropical Island, Tundra, Underground Sea, Verdant Catacombs, Volcanic Island, Watery Grave, Windswept Heath, Wooded Foothills, 1 Forest

## Import list (100)

```
1 Ancient Tomb
1 Arcane Signet
1 Astral Dragon
1 Atarka, World Render
1 Badlands
1 Bayou
1 Birds of Paradise
1 Blood Crypt
1 Bloodstained Mire
1 Breeding Pool
1 Cavern of Souls
1 Chaos Warp
1 Chromatic Lantern
1 Chrome Mox
1 City of Brass
1 Command Tower
1 Cursed Mirror
1 Decimate
1 Demonic Tutor
1 Dracogenesis
1 Dragon Tempest
1 Dragon's Hoard
1 Dragonborn Champion
1 Dragonlord Ojutai
1 Enlightened Tutor
1 Flooded Strand
1 Force of Negation
1 Force of Will
1 Gaea's Cradle
1 Ganax, Astral Hunter
1 Garruk's Uprising
1 Glorybringer
1 Goldspan Dragon
1 Grim Monolith
1 Hallowed Fountain
1 Imperial Seal
1 Inventors' Fair
1 Jaxis, the Troublemaker
1 Kindred Discovery
1 Kindred Dominance
1 Kura, the Boundless Sky
1 Lathliss, Dragon Queen
1 Lightning Greaves
1 Llanowar Elves
1 Lotus Petal
1 Maelstrom of the Spirit Dragon
1 Mana Confluence
1 Mana Drain
1 Mana Vault
1 Marsh Flats
1 Maskwood Nexus
1 Miirym, Sentinel Wyrm
1 Misty Rainforest
1 Mox Diamond
1 Mox Jasper
1 Nature's Lore
1 Nicol Bolas, the Ravager
1 Overgrown Tomb
1 Pact of Negation
1 Panharmonicon
1 Paradise Druid
1 Plateau
1 Polluted Delta
1 Rampant Growth
1 Rhystic Study
1 Sacred Foundry
1 Sakura-Tribe Elder
1 Sarkhan, Soul Aflame
1 Scourge of Valkas
1 Scrubland
1 Smothering Tithe
1 Sol Ring
1 Spark Double
1 Steam Vents
1 Stomping Ground
1 Swords to Plowshares
1 Sylvan Library
1 Taiga
1 Teferi's Protection
1 Temple Garden
1 Temur Ascendancy
1 Terror of the Peaks
1 The One Ring
1 The Ur-Dragon
1 Thundermaw Hellkite
1 Tropical Island
1 Tundra
1 Twinflame
1 Underground Sea
1 Utvara Hellkite
1 Vampiric Tutor
1 Verdant Catacombs
1 Volcanic Island
1 Watery Grave
1 Windswept Heath
1 Wooded Foothills
1 Worldly Tutor
1 Wrathful Red Dragon
1 Wrenn and Six
1 Forest
```
