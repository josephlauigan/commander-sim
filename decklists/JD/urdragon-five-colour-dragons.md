# The Ur-Dragon — Five-Colour Dragons (High Bracket 4)

Ramp on turns 1-3, a Dragon engine on turn 4 or 5, then win in combat. The Ur-Dragon draws a card for every attacking
Dragon and drops a permanent from your hand onto the battlefield free, so each attack refills your hand and puts a
seven-drop into play. Terror of the Peaks, Scourge of Valkas and Dragon Tempest turn every Dragon that enters into
damage, and Miirym and Lathliss double the Dragons. Beside those sits one four-card combo, **Garth One-Eye + Spark
Double + Twinflame + Doubling Season**, which makes mana and hasty tokens without limit. The deck is weak to board
wipes and mass bounce after it has committed its Dragons, to decks that win before turn 5, and to Blood Moon effects
(three basic lands). Nothing in this document has been simulated yet.

| | |
|---|---|
| **Commander** | The Ur-Dragon ({4}{W}{U}{B}{R}{G}, 10/10 flier); all five colours. Eminence: other Dragon spells you cast cost {1} less while it is in the command zone or on the battlefield |
| **Bracket** | 4: 13 Game Changers and one four-card combo. Played against Tier 5 (High Bracket 4) |
| **Game Changers (13)** | Ancient Tomb, Chrome Mox, Cyclonic Rift, Demonic Tutor, Enlightened Tutor, Fierce Guardianship, Mana Vault, Rhystic Study, Smothering Tithe, Teferi's Protection, The One Ring, Vampiric Tutor, Worldly Tutor |
| **Simulator** | Deck key `urdragon`, tier T5; findings in `audit/urdragon/FINDINGS.md`, history in `decklists/CHANGELOG.md`. No plan file and no sims yet |
| **Buys** | None: every card not owned is proxied |

## Strategy

### The commander

**The Ur-Dragon** ({4}{W}{U}{B}{R}{G}, 10/10). *Eminence — as long as it is in the command zone or on the
battlefield, other Dragon spells you cast cost {1} less. Flying. Whenever one or more Dragons you control attack, draw
that many cards, then you may put a permanent card from your hand onto the battlefield.*

It costs nine, so it is not the turn-4 play. It comes down on turn 5 or 6 off Mana Vault, Mox, dorks and Treasures, or
for {4} with **Morophon** out (Morophon naming Dragon takes {W}{U}{B}{R}{G} off every Dragon spell). While it sits in
the command zone it already discounts every Dragon you cast, so the deck can win without ever casting it.

What it does once it attacks:
- **Draws and cheats.** Each attacking Dragon is a card, and you may drop a permanent from the new hand onto the
  battlefield: a seven-drop (Klauth, Atarka, Old Gnawbone, Ancient Brass Dragon, Morophon), an engine, or Doubling Season.
  Tiamat only fetches Dragons if you cast it, so don't drop it.
- **Commander damage.** It is a 10/10 flier: two hits kill an opponent. With **Atarka** out, an attacking Dragon has
  double strike: one hit is twenty.
- **Protection** from the deck: **Scion of Draco** gives every creature of a colour vigilance, hexproof, lifelink, first
  strike or trample, and The Ur-Dragon is all five. **Lightning Greaves** and **Swiftfoot Boots** give hexproof and haste.
  **Fierce Guardianship** and **Deflecting Swat** are free while you control a commander. **Delighted Halfling** and
  **Cavern of Souls** (name Dragon) make the Dragons uncounterable.
- **A second cast costs {2} more per previous cast.** It is a nine-mana commander, so a recast is ten or more. Protect
  it, and do not rely on recasting.

### How the deck wins

#### Route 1: The Ur-Dragon in combat (main route)

Cast a hasty Dragon (Glorybringer, Thundermaw Hellkite, Goldspan Dragon, Smaug, Klauth, Hellkite Charger) the turn it
lands, or attack with what you have. Each attack with The Ur-Dragon or with Dragons draws a card per Dragon and puts a
permanent onto the battlefield, which triggers the damage engines below. **Hellkite Charger** pays {5}{R}{R} for an
extra combat, which is a second Ur-Dragon trigger. Plan the attack for the turn you have protection up.

#### Route 2: Dragons entering

- **Terror of the Peaks** ({3}{R}{R}): whenever another creature you control enters, it deals damage equal to that
  creature's power to any target. Spells that target it cost the opponent 3 life. Each Dragon you drop is a burst of damage.
- **Scourge of Valkas** ({2}{R}{R}{R}) and **Dragon Tempest** ({1}{R}): whenever a Dragon enters, it deals damage to
  any target equal to the number of Dragons you control.
- **Miirym, Sentinel Wyrm** ({3}{G}{U}{R}) copies every nontoken Dragon that enters, so a Terror, Scourge or Tempest
  trigger happens twice. **Lathliss, Dragon Queen** ({4}{R}{R}) makes a 5/5 flying Dragon for each nontoken Dragon,
  which feeds every one of those triggers again. **Doubling Season** doubles Lathliss's tokens.
- **Dragonspeaker Shaman** ({1}{R}{R}) takes {2} off every Dragon spell (three with Eminence), so a six-drop costs three.

#### Route 3: Garth One-Eye + Spark Double + Twinflame + Doubling Season

The loop was worked by hand from the Oracle text and also appears as a combo-catalogue line; it hasn't been simulated.

- **Garth One-Eye** ({W}{U}{B}{R}{G}): *{T}: Choose a card name that hasn't been chosen from among Disenchant,
  Braingeyser, Terror, Shivan Dragon, Regrowth and Black Lotus. Create a copy of the card with the chosen name. You may
  cast the copy.* The Black Lotus copy costs {0} and sacrifices for three mana. A copy made by Garth is not a card in
  the deck, so Black Lotus's ban doesn't matter.
- **Setup:** Garth untapped and not summoning sick; **Spark Double** entered as a copy of Garth (it isn't legendary, so
  tokens of it aren't either); **Doubling Season** on the battlefield; **Twinflame** ({1}{R}, strive {2}{R} per extra
  target) in hand; about three mana including {R}.
- **The loop:** Twinflame targets the Garths and, with Doubling Season, makes two hasty Garth tokens each. Each token
  taps for a Black Lotus copy (three mana apiece); one Garth makes Regrowth ({1}{G}) and returns Twinflame to hand. Each
  pass nets mana and leaves the token count growing, and every pass needs all four pieces: without Doubling Season it
  loses mana.
- **The win:** with unbounded mana, Garth makes **Braingeyser** copies (X is an opponent's library plus one; they lose
  at the next state-based check), or cast **Shivan Dragon** copies with Terror of the Peaks out, or attack with hasty
  tokens. Garth's other names: **Terror** and **Disenchant** are removal along the way.
- **Finding the pieces:** Demonic Tutor and Vampiric Tutor find any of them, Worldly Tutor finds Garth or Spark Double,
  Enlightened Tutor finds Doubling Season. **Twinflame is the weak point** (a lone sorcery with no protection of its
  own): hold Fierce Guardianship, or cast it on a turn **Dragonlord Dromoka** is out (opponents can't cast spells on
  your turn).

#### Fair wins and backups

- **Ancient Copper Dragon** (d20 Treasures), **Old Gnawbone** (a Treasure for each point of combat damage dealt by your
  creatures) and **Klauth** (mana equal to the attackers' total power) turn a connecting attack into the mana for the
  next play.
- **Hellkite Tyrant** takes every artifact a player controls when it connects. Against an artifact deck that is the
  game. (Twenty or more artifacts wins on your upkeep.)
- **Ancient Brass Dragon** puts creature cards from every graveyard onto the battlefield on combat damage (a d20 for the
  total mana value), which restocks the board after a wipe.
- **Dragonlord Silumgar** steals the best creature or planeswalker. **Tiamat** (cast it) fetches five Dragons to hand
  and refuels the Ur-Dragon trigger.
- **Crux of Fate** is a one-sided wrath: destroy all non-Dragon creatures. It also kills your own dorks, Shaman, Rivaz
  and Garth, so use it before the combo is set up or when the board is mostly Dragons. **Toxic Deluge** is the
  emergency button.

### The Dragons

| Cost | Dragons | Role |
|---|---|---|
| 4 | Smaug the Magnificent, Nicol Bolas the Ravager | Early hasty flier with a Treasure every upkeep; discard and a flip into a planeswalker |
| 5 | Terror of the Peaks, Goldspan Dragon, Glorybringer, Thundermaw Hellkite, Korvold, Scourge of Valkas | The core: damage, haste, removal, Treasures |
| 6 | Lathliss, Miirym, Dragonlord Dromoka, Dragonlord Silumgar, Ancient Copper Dragon, Hellkite Charger, Hellkite Tyrant, Shivan Dragon | Engines and finishers |
| 7 | Tiamat, Old Gnawbone, Klauth, Atarka, Ancient Brass Dragon, Morophon | The seven-drops The Ur-Dragon drops in free; Morophon is a Dragon (changeling) and makes the commander cost {4} |
| Varies | Scion of Draco | Costs {12} minus {2} per basic land type among your lands: about {2} with five types, {1} less than that with Eminence |

### The ramp and engines

- **Mana Vault, Chrome Mox, Sol Ring, Arcane Signet** (and Ancient Tomb): the fast start. Cast on turn 1 and left
  untapped, Mana Vault gives {C}{C}{C} on turn 2: two lands and the Vault pay for a five-drop. Chrome Mox imprints a spare gold card for its colours and is never worth a
  combo piece.
- **Birds of Paradise, Llanowar Elves, Elvish Mystic, Delighted Halfling:** one-mana dorks. The Halfling makes any colour
  for a legendary spell (The Ur-Dragon, Garth, Tiamat, Miirym, Korvold, Smaug, Scourge...) and that spell can't be countered.
- **Nature's Lore, Three Visits, Farseek:** each fetches an untapped dual land (Nature's Lore and Three Visits any Forest
  dual, Farseek any Plains, Island, Swamp or Mountain dual), which fixes colours and adds basic types for Scion of Draco.
- **Rivaz of the Claw** ({1}{B}{R}): taps for two mana for Dragon spells, and lets you cast one Dragon from the
  graveyard each turn.
- **Smothering Tithe, Rhystic Study, Sylvan Library, The One Ring:** the draw and tax engines. Rhystic and the Tithe
  make the table pay for casting into you; Sylvan Library picks the best cards; The One Ring protects you for a turn
  and draws more each turn (it costs 1 life per burden counter each upkeep).
- **Garruk's Uprising and Temur Ascendancy:** draw whenever a creature with power 4 or more enters (nearly every Dragon),
  and give haste or trample.

### Game plan

**Turns 1-2.** Land, a rock or a dork, Mana Vault or Chrome Mox if you hold them. Nature's Lore or Three Visits fetches
the dual that fixes your colours. A turn-1 dork or Sol Ring into a turn-2 four-drop (Smaug or a Shaman), or Mana Vault into a turn-2 five-drop, is the
strongest start.

**Turns 3-4.** The cheap Dragons (Smaug, Bolas, a Shaman-reduced Dragon) and an engine (Rhystic Study, Sylvan Library,
The One Ring). Keep mana for Counterspell or a free spell on the opponent's turn when you hold protection.

**Turns 5-7.** Land Terror of the Peaks or the key Dragon, then The Ur-Dragon (or Morophon first), then attack. Win
through the combat route or the Garth loop once the pieces are there.

**Hold up or go?** Go when the table has no sweeper in sight and you hold protection, or when an opponent is about to win.
Hold the Dragons in hand when an opponent has shown a board wipe, and rely on Teferi's Protection, Heroic Intervention
and the cheap haste Dragons.

### Mulligans

The first mulligan is free; after that it's a London mulligan (draw 7, bottom one card per mulligan after the free one).

| Hand | Keep when |
|---|---|
| 7 cards | 2-5 lands (1 land only with a Sol Ring, Mox, Vault or two dorks); at least two colours of source; 3+ mana by turn 3; and an action card: a ramp spell, a Dragon you can cast by turn 5, a tutor, or an engine |
| 6 or 5 cards | 1-5 lands, 2+ mana cards, and any Dragon or tutor |
| 4 cards | Keep |

- **Bottom first:** lands past the fourth, the seven-drops, then situational answers.
- **A hand of only seven-drops goes back,** even with mana. The deck needs the early game.
- **A hand with a tutor and two mana sources is a keep:** Demonic Tutor, Vampiric Tutor, Worldly Tutor and Enlightened
  Tutor each find the piece you need.

| # | Hand | Call | Why |
|---|---|---|---|
| 1 | Taiga, Windswept Heath, Birds of Paradise, Sol Ring, Terror of the Peaks, Counterspell, Swords to Plowshares | Keep | Turn 2: Sol Ring and Birds; turn 3 Terror with a counter or removal behind it |
| 2 | Ancient Tomb, Mana Vault, Chrome Mox, Old Gnawbone, Klauth, Atarka, Rivaz of the Claw | Ship | One land that only makes colourless, and nothing to cast before turn 5 |
| 3 | Tundra, Bayou, Elvish Mystic, Farseek, Three Visits, Lathliss, Doubling Season | Keep (close) | Two duals plus Farseek and Three Visits fetch two more duals, and the Mystic adds mana: Lathliss and Doubling Season by turn 5 |
| 4 | Demonic Tutor, Vampiric Tutor, Command Tower, Sol Ring, Garth One-Eye, Spark Double, Twinflame | Keep (close) | Tutors and Sol Ring find the rest, but it needs a second land and Doubling Season to be a combo |

### What to tutor for

| Situation | Find | With |
|---|---|---|
| Garth and Spark Double in hand | Doubling Season, then Twinflame | Enlightened Tutor for Doubling Season; Demonic or Vampiric for Twinflame |
| Doubling Season on the battlefield, no Garth | Garth One-Eye | Worldly Tutor, then the loop with Spark Double and Twinflame |
| Seven mana and no Dragon | Terror of the Peaks or Goldspan Dragon | Demonic Tutor or Vampiric Tutor (to the top) |
| Opponent about to wipe | Teferi's Protection or Heroic Intervention | Vampiric Tutor, hold both mana and the card |
| The Ur-Dragon is about to attack | A seven-drop to put in free | Tutor for Klauth or Atarka and leave it in hand |
| A creature with the table's best card | Swords, Path, Assassin's Trophy or Dragonlord Silumgar | Any tutor; Silumgar steals it |

### Protecting the key turn

- **Free spells:** Fierce Guardianship (counter a noncreature spell) and Deflecting Swat (redirect) cost nothing while you
  control a commander, which is any turn after The Ur-Dragon lands.
- **Heroic Intervention** and **Teferi's Protection** answer a wipe or removal. Teferi's Protection also protects you from
  a lethal attack for a turn.
- **Greaves and Boots** on The Ur-Dragon or a combo piece give it hexproof.
- **Cavern of Souls** (name Dragon) and **Delighted Halfling** make the key Dragon or Garth uncounterable.
- **Counterspell** ({U}{U}) holds the stack for a creature or spell you can't let through; blue is 18 land sources, so
  keep {U}{U} up before you spend the rest of your mana.
- **Dromoka** stops every spell on your turn: cast it before the combo or the attack.

### Interaction

- **Removal:** Swords to Plowshares, Path to Exile, Assassin's Trophy (any permanent), Anguished Unmaking (exile any
  nonland permanent, 3 life), Chaos Warp (any permanent, random replacement), Cyclonic Rift (overloaded at {6}{U}: bounces
  every nonland permanent opponents control, a one-sided reset).
- **On bodies:** Glorybringer (4 damage each exert), Dragonlord Silumgar (steals a creature or planeswalker), Terror of
  the Peaks and the Dragon-enters pingers (damage to any target), Thundermaw Hellkite (taps and damages fliers), Garth's
  Terror and Disenchant copies.
- **Wipes:** Crux of Fate (non-Dragon mode), Toxic Deluge (pay X life), Cyclonic Rift overloaded.

### Matchups (Tier 5)

The matchup notes are expectations, not measured.

| Opponent type | What to do |
|---|---|
| Fast combo (Kinnan, Yawgmoth, Zur) | Hold Fierce Guardianship, Counterspell or Swat; kill the commander with Swords or Path; go over the top with a hasty Dragon on turn 5 |
| Board wipes | Keep Dragons in hand until you hold Teferi's Protection or Heroic Intervention; Brass Dragon, Rivaz and Tiamat rebuild |
| Creature-heavy midrange | Glorybringer, Thundermaw and the Dragon-enters damage clear blockers; Crux of Fate (non-Dragon mode) clears their board but not yours |
| Artifact decks (Thanos, Urza, Kinnan) | Hellkite Tyrant takes their artifacts; Assassin's Trophy and Chaos Warp answer the key one |
| Stax / Blood Moon | Fetch the three basics first; cast Dromoka to dodge counterspells; Cavern of Souls names Dragon |

### Life management

You start at 40 and pay life for the mana base (ten fetches, five shocks, City of Brass, Mana Confluence, Ancient Tomb),
Mana Vault, Sylvan Library, Vampiric Tutor, Anguished Unmaking, Toxic Deluge and The One Ring. Dromoka gives
lifelink, and Scion of Draco gives it to The Ur-Dragon and every other black creature. Don't fetch for a colour you
already have, and don't pay 4 life per Sylvan Library card in a game you're racing.

### Common mistakes

- Casting Crux of Fate in non-Dragon mode with your dorks and Garth on the board.
- Dropping Tiamat with The Ur-Dragon (it must be cast to fetch Dragons).
- Casting Twinflame into open blue mana without Fierce Guardianship or Dromoka.
- Paying for The Ur-Dragon (nine) when Morophon, Shaman and Rivaz would let you cast it and the Dragons you hold in the
  same turn for less.
- Fetching a basic when a dual would fix more colours: a fetch finds any land with one of its two basic types, so pick
  the dual (or shock) that adds the colour you lack.
- Going to combat into a Settle the Wreckage or Aetherize pattern with no Teferi's Protection or Heroic Intervention.

## Mana

33 lands: 30 nonbasic and 3 basics (Mountain, Forest, Swamp). Ten original duals (Tundra, Underground Sea, Badlands,
Taiga, Savannah, Scrubland, Volcanic Island, Bayou, Plateau, Tropical Island), ten fetches, five shocks (Blood Crypt,
Steam Vents, Stomping Ground, Overgrown Tomb, Sacred Foundry), Command Tower, City of Brass, Mana Confluence, Cavern of
Souls and Ancient Tomb. A fetch finds any dual or basic that shares a type, so every colour is reachable: land sources
by colour are white 18, blue 18, black 20, red 22 and green 20, counting the fetches, the three any-colour lands and
the basics. Red is the heaviest colour (RR on most Dragons, RRR on Scourge of Valkas), then green and black. Add
the mana creatures and rocks: Arcane Signet, Birds, Chrome Mox and Delighted Halfling make any colour, and Treasures
from Goldspan, Smaug, Ancient Copper Dragon and Old Gnawbone fix the rest. The fetches also count as sacrifices for
Korvold. The three basics are the answer to Blood Moon (fetch them first).

## Bracket and Rule 0

Bracket 4 (High): 13 Game Changers and one four-card combo (Garth One-Eye + Spark Double + Twinflame + Doubling Season,
unbounded mana and tokens at sorcery speed). No mass land destruction, no extra turns, no fast-mana turbo (no Mana Crypt
or Jeweled Lotus). Tell the table about the combo before the game.

## Decklist by type (100)

**Commander (1).** The Ur-Dragon

**Creatures (31).** Ancient Brass Dragon, Ancient Copper Dragon, Atarka, World Render, Birds of Paradise, Delighted Halfling, Dragonlord Dromoka, Dragonlord Silumgar, Dragonspeaker Shaman, Elvish Mystic, Garth One-Eye, Glorybringer, Goldspan Dragon, Hellkite Charger, Hellkite Tyrant, Klauth, Unrivaled Ancient, Korvold, Fae-Cursed King, Lathliss, Dragon Queen, Llanowar Elves, Miirym, Sentinel Wyrm, Morophon, the Boundless, Nicol Bolas, the Ravager, Old Gnawbone, Rivaz of the Claw, Scion of Draco, Scourge of Valkas, Shivan Dragon, Smaug the Magnificent, Spark Double, Terror of the Peaks, Thundermaw Hellkite, Tiamat

**Artifacts (7).** Arcane Signet, Chrome Mox, Lightning Greaves, Mana Vault, Sol Ring, Swiftfoot Boots, The One Ring

**Enchantments (7).** Doubling Season, Dragon Tempest, Garruk's Uprising, Rhystic Study, Smothering Tithe, Sylvan Library, Temur Ascendancy

**Instants (14).** Anguished Unmaking, Assassin's Trophy, Chaos Warp, Counterspell, Cyclonic Rift, Deflecting Swat, Enlightened Tutor, Fierce Guardianship, Heroic Intervention, Path to Exile, Swords to Plowshares, Teferi's Protection, Vampiric Tutor, Worldly Tutor

**Sorceries (7).** Crux of Fate, Demonic Tutor, Farseek, Nature's Lore, Three Visits, Toxic Deluge, Twinflame

**Lands (33).** Ancient Tomb, Arid Mesa, Badlands, Bayou, Blood Crypt, Bloodstained Mire, Cavern of Souls, City of Brass, Command Tower, Flooded Strand, Mana Confluence, Marsh Flats, Misty Rainforest, Overgrown Tomb, Plateau, Polluted Delta, Sacred Foundry, Savannah, Scalding Tarn, Scrubland, Steam Vents, Stomping Ground, Taiga, Tropical Island, Tundra, Underground Sea, Verdant Catacombs, Volcanic Island, Windswept Heath, Wooded Foothills, 1 Forest, 1 Mountain, 1 Swamp

## Import list (100)

```
1 Ancient Brass Dragon
1 Ancient Copper Dragon
1 Ancient Tomb
1 Anguished Unmaking
1 Arcane Signet
1 Arid Mesa
1 Assassin's Trophy
1 Atarka, World Render
1 Badlands
1 Bayou
1 Birds of Paradise
1 Blood Crypt
1 Bloodstained Mire
1 Cavern of Souls
1 Chaos Warp
1 Chrome Mox
1 City of Brass
1 Command Tower
1 Counterspell
1 Crux of Fate
1 Cyclonic Rift
1 Deflecting Swat
1 Delighted Halfling
1 Demonic Tutor
1 Doubling Season
1 Dragon Tempest
1 Dragonlord Dromoka
1 Dragonlord Silumgar
1 Dragonspeaker Shaman
1 Elvish Mystic
1 Enlightened Tutor
1 Farseek
1 Fierce Guardianship
1 Flooded Strand
1 Garruk's Uprising
1 Garth One-Eye
1 Glorybringer
1 Goldspan Dragon
1 Hellkite Charger
1 Hellkite Tyrant
1 Heroic Intervention
1 Klauth, Unrivaled Ancient
1 Korvold, Fae-Cursed King
1 Lathliss, Dragon Queen
1 Lightning Greaves
1 Llanowar Elves
1 Mana Confluence
1 Mana Vault
1 Marsh Flats
1 Miirym, Sentinel Wyrm
1 Misty Rainforest
1 Morophon, the Boundless
1 Nature's Lore
1 Nicol Bolas, the Ravager
1 Old Gnawbone
1 Overgrown Tomb
1 Path to Exile
1 Plateau
1 Polluted Delta
1 Rhystic Study
1 Rivaz of the Claw
1 Sacred Foundry
1 Savannah
1 Scalding Tarn
1 Scion of Draco
1 Scourge of Valkas
1 Scrubland
1 Shivan Dragon
1 Smaug the Magnificent
1 Smothering Tithe
1 Sol Ring
1 Spark Double
1 Steam Vents
1 Stomping Ground
1 Swiftfoot Boots
1 Swords to Plowshares
1 Sylvan Library
1 Taiga
1 Teferi's Protection
1 Temur Ascendancy
1 Terror of the Peaks
1 The One Ring
1 The Ur-Dragon
1 Three Visits
1 Thundermaw Hellkite
1 Tiamat
1 Toxic Deluge
1 Tropical Island
1 Tundra
1 Twinflame
1 Underground Sea
1 Vampiric Tutor
1 Verdant Catacombs
1 Volcanic Island
1 Windswept Heath
1 Wooded Foothills
1 Worldly Tutor
1 Forest
1 Mountain
1 Swamp
```
