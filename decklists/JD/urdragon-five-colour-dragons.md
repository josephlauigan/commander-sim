# The Ur-Dragon — Five-Colour Dragons (High Bracket 4)

Ramp on turns 1-3, a Dragon engine on turn 4 or 5, then win in combat. The Ur-Dragon draws a card for every attacking
Dragon and drops a permanent from your hand onto the battlefield free, so each attack refills your hand and puts a
seven-drop into play. Terror of the Peaks, Scourge of Valkas and Dragon Tempest turn every Dragon that enters into
damage, and Miirym and Lathliss double the Dragons. Beside those sits one four-card combo, **Garth One-Eye + Spark
Double + Twinflame + Doubling Season**, which makes mana and hasty tokens without limit. Around them sit the top-end
Dragons (Utvara Hellkite, Steel Hellkite, Summon: Bahamut, Niv-Mizzet) and a spells package (Lightning Bolt, Decimate,
Fact or Fiction, Austere Command, Young Pyromancer) and Triple Triad. The deck is weak to board wipes and mass bounce after it
has committed its Dragons, to decks that win before turn 5, and to Blood Moon effects (two basic lands). Nothing in this document has been simulated yet.

| | |
|---|---|
| **Commander** | The Ur-Dragon ({4}{W}{U}{B}{R}{G}, 10/10 flier); all five colours. Eminence: other Dragon spells you cast cost {1} less while it is in the command zone or on the battlefield |
| **Bracket** | 4: 10 Game Changers and one four-card combo. Played against Tier 5 (High Bracket 4) |
| **Game Changers (10)** | Ancient Tomb, Cyclonic Rift, Demonic Tutor, Fierce Guardianship, Mana Vault, Rhystic Study, Teferi's Protection, The One Ring, Vampiric Tutor, Worldly Tutor |
| **Simulator** | Deck key `urdragon`, tier T5; findings in `audit/urdragon/FINDINGS.md`, history in `decklists/CHANGELOG.md`; plan file `data/plans/urdragon.toml`. No sims yet |
| **Buys** | None: every card not owned is proxied |

## Strategy

### The commander

**The Ur-Dragon** ({4}{W}{U}{B}{R}{G}, 10/10). *Eminence — as long as it is in the command zone or on the
battlefield, other Dragon spells you cast cost {1} less. Flying. Whenever one or more Dragons you control attack, draw
that many cards, then you may put a permanent card from your hand onto the battlefield.*

It costs nine, so it is not the turn-4 play. It comes down on turn 5 or 6 off Mana Vault, dorks and Treasures (Mox Jasper once a Dragon is out), or
for {4} with **Morophon** out (Morophon naming Dragon takes {W}{U}{B}{R}{G} off every Dragon spell). While it sits in
the command zone it already discounts every Dragon you cast, so the deck can win without ever casting it.

What it does once it attacks:
- **Draws and cheats.** Each attacking Dragon is a card, and you may drop a permanent from the new hand onto the
  battlefield: a seven-drop or bigger (Klauth, Old Gnawbone, Morophon, Utvara Hellkite, Summon: Bahamut), an engine, or Doubling Season.
  Tiamat only fetches Dragons if you cast it, so don't drop it.
- **Commander damage.** It is a 10/10 flier: two hits kill an opponent. With **Utvara Hellkite** out, every attacking
  Dragon makes a 6/6 flying Dragon, so the next attack is far bigger.
- **Protection** from the deck: **Scion of Draco** gives every creature of a colour vigilance, hexproof, lifelink, first
  strike or trample, and The Ur-Dragon is all five. **Dragonlord Ojutai** has hexproof while it is untapped.
  **Fierce Guardianship** is free while you control a commander. **Delighted Halfling** and
  **Cavern of Souls** (name Dragon) make the Dragons uncounterable.
- **A second cast costs {2} more per previous cast.** It is a nine-mana commander, so a recast is ten or more. Protect
  it, and do not rely on recasting.

### How the deck wins

#### Route 1: The Ur-Dragon in combat (main route)

Cast a hasty Dragon (Glorybringer, Thundermaw Hellkite, Goldspan Dragon, Smaug, Klauth) the turn it
lands, or attack with what you have. Each attack with The Ur-Dragon or with Dragons draws a card per Dragon and puts a
permanent onto the battlefield, which triggers the damage engines below. Plan the attack for the turn you have
protection up.

#### Route 2: Dragons entering

- **Terror of the Peaks** ({3}{R}{R}): whenever another creature you control enters, it deals damage equal to that
  creature's power to any target. Spells that target it cost the opponent 3 life. Each Dragon you drop is a burst of damage.
- **Scourge of Valkas** ({2}{R}{R}{R}) and **Dragon Tempest** ({1}{R}): whenever a Dragon enters, it deals damage to
  any target equal to the number of Dragons you control.
- **Miirym, Sentinel Wyrm** ({3}{G}{U}{R}) copies every nontoken Dragon that enters, so a Terror, Scourge or Tempest
  trigger happens twice. **Lathliss, Dragon Queen** ({4}{R}{R}) makes a 5/5 flying Dragon for each nontoken Dragon,
  which feeds every one of those triggers again. **Doubling Season** doubles Lathliss's tokens.
- **Ganax, Astral Hunter** ({4}{R}) makes a Treasure whenever it or another Dragon enters. **Niv-Mizzet, the Firemind**
  ({2}{U}{U}{R}{R}) pings any target for every card you draw (every Ur-Dragon attack, Rhystic Study, The One Ring, Relic of Sauron's loot,
  its own {T}: draw).
  **Utvara Hellkite** ({6}{R}{R}) makes a 6/6 flying Dragon for each Dragon that attacks, and each of those Dragons
  feeds the enters triggers above.

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
- **Finding the pieces:** Demonic Tutor and Vampiric Tutor find any of them, Worldly Tutor finds Garth or Spark Double.
  **Twinflame is the weak point** (a lone sorcery with no protection of its
  own): hold Fierce Guardianship, or cast it on a turn **Dragonlord Dromoka** is out (opponents can't cast spells on
  your turn).

#### Fair wins and backups

- **Ancient Copper Dragon** (d20 Treasures), **Old Gnawbone** (a Treasure for each point of combat damage dealt by your
  creatures) and **Klauth** (mana equal to the attackers' total power) turn a connecting attack into the mana for the
  next play.
- **Hellkite Tyrant** takes every artifact a player controls when it connects. Against an artifact deck that is the
  game. (Twenty or more artifacts wins on your upkeep.)
- **Steel Hellkite** ({6}) pays {X} after it connects to destroy each nonland permanent with mana value X whose controller
  it hit. **Summon: Bahamut** ({9}, a flying Saga Dragon) destroys up to one nonland permanent in each of its first two
  chapters, draws two cards in the third, and in its last chapter deals damage to each opponent equal to the total mana
  value of your other permanents, then it is sacrificed.
- **Dragonlord Silumgar** steals the best creature or planeswalker. **Tiamat** (cast it) fetches five Dragons to hand
  and refuels the Ur-Dragon trigger.
- **Crux of Fate** is a one-sided wrath: destroy all non-Dragon creatures. It also kills your own dorks, Rivaz,
  Garth, Young Pyromancer and Spark Double (unless it copies a Dragon), so use it before the combo is set up or when the board is mostly Dragons. **Austere Command** ({4}{W}{W}) picks two modes: only
  "creatures with mana value 3 or less" leaves every Dragon alone (it still kills your dorks, Rivaz and Young
  Pyromancer); the artifact mode also kills your rocks, The One Ring, Steel Hellkite and Scion of Draco, and the
  enchantment mode kills Doubling Season, Rhystic Study and Summon: Bahamut.

### The Dragons

| Cost | Dragons | Role |
|---|---|---|
| 4 | Smaug the Magnificent, Nicol Bolas the Ravager | Early hasty flier with a Treasure every upkeep; discard and a flip into a planeswalker |
| 5 | Terror of the Peaks, Goldspan Dragon, Glorybringer, Thundermaw Hellkite, Korvold, Scourge of Valkas, Ganax, Kura, Dragonlord Ojutai, Scion of the Ur-Dragon | The core: damage, haste, removal, Treasures, card selection |
| 6 | Lathliss, Miirym, Dragonlord Dromoka, Dragonlord Silumgar, Ancient Copper Dragon, Hellkite Tyrant, Shivan Dragon, Steel Hellkite, Niv-Mizzet | Engines and finishers |
| 7 | Tiamat, Old Gnawbone, Klauth, Smaug the Impenetrable, Morophon | The seven-drops The Ur-Dragon drops in free; Morophon is a Dragon (changeling) and makes the commander cost {4}; Smaug the Impenetrable is hasty and indestructible, so it survives a destroy wipe, and turns noncombat damage dealt to it into Treasures |
| 8-9 | Utvara Hellkite (8), Summon: Bahamut (9) | The finishers: a Dragon-attack engine and a slow four-chapter removal, card draw and burn |
| Varies | Scion of Draco | Costs {12} minus {2} per basic land type among your lands: about {2} with five types, {1} less than that with Eminence |

### The ramp and engines

- **Mana Vault, Sol Ring, Arcane Signet** (and Ancient Tomb): the fast start. Cast on turn 1 and left
  untapped, Mana Vault gives {C}{C}{C} on turn 2: two lands and the Vault pay for a five-drop. **Mox Jasper** makes any
  colour once you control a Dragon, so it is a dead card before one lands. **Cursed Mirror** ({2}{R}) taps for {R}, or
  enters as a hasty copy of the best creature on the table for a turn.
- **Relic of Sauron** ({4}): taps for two mana in any mix of blue, black and red, which pays for a seven-drop and fixes
  the blue and black the elves don't make. Later, {3} and a tap draws two cards and discards one. Its own mana can't pay
  for the loot.
- **Birds of Paradise, Llanowar Elves, Elvish Mystic, Delighted Halfling:** one-mana dorks. The Halfling makes any colour
  for a legendary spell (The Ur-Dragon, Garth, Tiamat, Miirym, Korvold, Smaug...) and that spell can't be countered.
- **Nature's Lore, Three Visits:** each fetches a Forest dual (a shock land costs 2 life to enter untapped; a basic Forest is also a legal find), which fixes colours and adds
  basic types for Scion of Draco.
- **Rivaz of the Claw** ({1}{B}{R}): taps for two mana for Dragon spells, and lets you cast one Dragon from the
  graveyard each turn.
- **Rhystic Study, The One Ring, Fact or Fiction:** the draw engines. Rhystic makes the table pay for casting into you;
  The One Ring protects you for a turn and draws more each turn (it costs 1 life per burden counter each upkeep); Fact
  or Fiction is a four-mana instant for a pile of cards.
- **Lightning Bolt, Decimate, Austere Command, Young Pyromancer:** the spells package. Bolt is
  cheap removal or reach; Decimate destroys an artifact, a creature, an enchantment and a land at once (all four need
  a legal target); Young Pyromancer turns each instant or sorcery into a 1/1 Elemental, but the
  deck holds only seventeen of them (ten instants and seven sorceries).
- **Triple Triad** ({3}{R}{R}{R}, enchantment): at the beginning of your upkeep each player exiles the top card of their
  library, and this turn you may play your own exiled card and every other exiled card with a lower mana value without
  paying their mana costs. About a third of the 99 are lands (a land flip gives no free spell, but you may play it as your
  land drop) and 29 are Dragons, mostly costing 5 to 7, so a flip is a free Dragon about three times in ten, and the
  higher the card you flip, the more of the opponents' exiled cards qualify. It needs three red sources.
- **Temur Ascendancy:** gives your creatures haste, and you may draw a card whenever a creature with power 4 or more
  enters under your control (nearly every Dragon, including the ones The Ur-Dragon puts in).

### Game plan

**Turns 1-2.** Land, a rock or a dork, Mana Vault if you hold it. Nature's Lore or Three Visits fetches
the dual that fixes your colours. A turn-1 dork or Sol Ring into a turn-2 four-drop (Smaug), or Mana Vault into a turn-2 five-drop, is the
strongest start.

**Turns 3-4.** The cheap Dragons (Smaug the Magnificent, Bolas, Ganax) and an engine (Rhystic Study, The One Ring). Keep
mana for Fierce Guardianship or a free spell on the opponent's turn when you hold protection.

**Turns 5-7.** Land Terror of the Peaks or the key Dragon, then The Ur-Dragon (or Morophon first), then attack. Win
through the combat route or the Garth loop once the pieces are there.

**Hold up or go?** Go when the table has no sweeper in sight and you hold protection, or when an opponent is about to win.
Hold the Dragons in hand when an opponent has shown a board wipe, and rely on Teferi's Protection and the cheap haste
Dragons.

### Mulligans

The first mulligan is free; after that it's a London mulligan (draw 7, bottom one card per mulligan after the free one).

| Hand | Keep when |
|---|---|
| 7 cards | 2-5 lands (1 land only with a Sol Ring, Mana Vault or two dorks); at least two colours of source; 3+ mana by turn 3; and an action card: a ramp spell, a Dragon you can cast by turn 5, a tutor, or an engine |
| 6 or 5 cards | 1-5 lands, 2+ mana cards, and any Dragon or tutor |
| 4 cards | Keep |

- **Bottom first:** lands past the fourth, the seven-drops, then situational answers.
- **A hand of only seven-drops goes back,** even with mana. The deck needs the early game.
- **A hand with a tutor and two mana sources is a keep:** Demonic Tutor, Vampiric Tutor and Worldly Tutor each find the
  piece you need.

| # | Hand | Call | Why |
|---|---|---|---|
| 1 | Taiga, Windswept Heath, Birds of Paradise, Sol Ring, Terror of the Peaks, Lightning Bolt, Swords to Plowshares | Keep | Turn 2: Sol Ring and Birds; turn 3 Terror with removal behind it |
| 2 | Ancient Tomb, Mana Vault, Mox Jasper, Old Gnawbone, Klauth, Utvara Hellkite, Rivaz of the Claw | Ship | One land that only makes colourless, and nothing to cast before turn 5 |
| 3 | Tundra, Bayou, Elvish Mystic, Nature's Lore, Three Visits, Lathliss, Doubling Season | Keep (close) | Two duals plus Nature's Lore and Three Visits fetch two more duals, and the Mystic adds mana: Lathliss and Doubling Season by turn 5 |
| 4 | Demonic Tutor, Vampiric Tutor, Command Tower, Sol Ring, Garth One-Eye, Spark Double, Twinflame | Keep (close) | Tutors and Sol Ring find the rest, but it needs a second land and Doubling Season to be a combo |

### What to tutor for

| Situation | Find | With |
|---|---|---|
| Garth and Spark Double in hand | Doubling Season, then Twinflame | Demonic or Vampiric Tutor for Doubling Season, then Twinflame |
| Doubling Season on the battlefield, no Garth | Garth One-Eye | Worldly Tutor, then the loop with Spark Double and Twinflame |
| Seven mana and no Dragon | Terror of the Peaks or Goldspan Dragon | Demonic Tutor or Vampiric Tutor (to the top) |
| Opponent about to wipe | Teferi's Protection | Vampiric Tutor, hold both mana and the card |
| The Ur-Dragon is about to attack | A seven-drop to put in free | Tutor for Klauth or Utvara Hellkite and leave it in hand |
| A creature with the table's best card | Swords, Path, Decimate or Dragonlord Silumgar | Any tutor; Silumgar steals it |

### Protecting the key turn

- **Free spells:** Fierce Guardianship (counter a noncreature spell) costs nothing while you control a commander, which
  is any turn after The Ur-Dragon lands.
- **Teferi's Protection** answers a wipe or removal. It also protects you from a lethal attack for a turn.
- **Dragonlord Ojutai** has hexproof while it is untapped (itself only). **Scion of Draco** gives your blue creatures
  hexproof, and The Ur-Dragon is all five colours.
- **Cavern of Souls** (name Dragon) and **Delighted Halfling** make the key Dragon or Garth uncounterable.
- **Dromoka** stops every spell on your turn: cast it before the combo or the attack.

### Interaction

- **Removal:** Swords to Plowshares, Path to Exile, Lightning Bolt (3 damage to any target), Decimate (an artifact, a
  creature, an enchantment and a land), Chaos Warp (any permanent, random replacement), Cyclonic Rift (overloaded at {6}{U}: bounces
  every nonland permanent opponents control, a one-sided reset).
- **On bodies:** Glorybringer (4 damage each exert), Dragonlord Silumgar (steals a creature or planeswalker), Terror of
  the Peaks and the Dragon-enters pingers (damage to any target), Thundermaw Hellkite (taps and damages fliers), Garth's
  Terror and Disenchant copies.
- **Wipes:** Crux of Fate (non-Dragon mode), Austere Command, Cyclonic Rift overloaded, Steel Hellkite.

### Matchups (Tier 5)

The matchup notes are expectations, not measured.

| Opponent type | What to do |
|---|---|
| Fast combo (Kinnan, Yawgmoth, Zur) | Hold Fierce Guardianship; kill the commander with Swords or Path; go over the top with a hasty Dragon on turn 5 |
| Board wipes | Keep Dragons in hand until you hold Teferi's Protection; Rivaz and Tiamat rebuild |
| Creature-heavy midrange | Glorybringer, Thundermaw and the Dragon-enters damage clear blockers; Crux of Fate (non-Dragon mode) clears their board but not yours |
| Artifact decks (Thanos, Urza, Kinnan) | Hellkite Tyrant takes their artifacts; Decimate, Austere Command and Chaos Warp answer the key one |
| Stax / Blood Moon | Fetch the basics (Forest, Swamp) first; cast Dromoka to dodge counterspells; Cavern of Souls names Dragon |

### Life management

You start at 40 and pay life for the mana base (seven fetches, five shocks, City of Brass, Sulfurous Springs, Ancient Tomb), Mana Vault,
Vampiric Tutor and The One Ring. Dromoka gives
lifelink, and Scion of Draco gives it to The Ur-Dragon and every other black creature. Don't fetch for a colour you
already have.

### Common mistakes

- Casting Crux of Fate in non-Dragon mode with your dorks and Garth on the board.
- Dropping Tiamat with The Ur-Dragon (it must be cast to fetch Dragons).
- Casting Twinflame into open blue mana without Fierce Guardianship or Dromoka.
- Paying for The Ur-Dragon (nine) when Morophon and Rivaz would let you cast it and the Dragons you hold in the
  same turn for less.
- Fetching a basic when a dual would fix more colours: a fetch finds any land with one of its two basic types, so pick
  the dual (or shock) that adds the colour you lack.
- Going to combat into a Settle the Wreckage or Aetherize pattern with no Teferi's Protection.

## Mana

33 lands: 31 nonbasic and 2 basics (Forest, Swamp). Ten original duals (Tundra, Underground Sea, Badlands,
Taiga, Savannah, Scrubland, Volcanic Island, Bayou, Plateau, Tropical Island), seven fetches, five shocks (Blood Crypt,
Steam Vents, Stomping Ground, Overgrown Tomb, Sacred Foundry), Command Tower, City of Brass, Path of Ancestry (enters
tapped, any colour, scries when it pays for a Dragon creature spell), Maelstrom of the Spirit Dragon (any colour for
Dragon spells; {4}, {T}, sacrifice it to fetch a Dragon), Sokenzan, Crucible of Defiance ({R}; channel it for two hasty
1/1s), Sulfurous Springs ({B} or {R} for 1 damage), Exotic Orchard (any colour an opponent's land could produce),
Cavern of Souls and Ancient Tomb. A fetch finds any dual or basic that shares a type, so every colour is reachable
(Springs, Orchard and the other nonbasics without a land type cannot be fetched): land sources by colour are white 15,
blue 15, black 18, red 20 and green 17, counting the fetches, the three any-colour lands and the basics, with
Maelstrom adding one more of each for Dragon spells and Orchard adding what the table's lands make. Red is the heaviest
colour (RR on most Dragons, RRR on Scourge of Valkas and Triple Triad), then green and black; white and blue are the thinnest, for
Niv-Mizzet ({U}{U}), Fact or Fiction and Austere Command ({W}{W}). Add the mana creatures and rocks: Arcane Signet, Birds, Mox Jasper (with a
Dragon out) and Delighted Halfling make any colour, and Treasures from Goldspan, Ganax, Smaug, Ancient Copper Dragon and
Old Gnawbone fix the rest. The fetches also count as sacrifices for Korvold. The two basics are the answer to Blood
Moon (fetch them first).

## Bracket and Rule 0

Bracket 4 (High): 10 Game Changers and one four-card combo (Garth One-Eye + Spark Double + Twinflame + Doubling Season,
unbounded mana and tokens at sorcery speed). No mass land destruction, no extra turns, no fast-mana turbo (no Mana Crypt
or Jeweled Lotus). Tell the table about the combo before the game.

## Decklist by type (100)

**Commander (1).** The Ur-Dragon

**Creatures (37).** Ancient Copper Dragon, Birds of Paradise, Delighted Halfling, Dragonlord Dromoka, Dragonlord Ojutai, Dragonlord Silumgar, Elvish Mystic, Ganax, Astral Hunter, Garth One-Eye, Glorybringer, Goldspan Dragon, Hellkite Tyrant, Klauth, Unrivaled Ancient, Korvold, Fae-Cursed King, Kura, the Boundless Sky, Lathliss, Dragon Queen, Llanowar Elves, Miirym, Sentinel Wyrm, Morophon, the Boundless, Nicol Bolas, the Ravager, Niv-Mizzet, the Firemind, Old Gnawbone, Rivaz of the Claw, Scion of Draco, Scion of the Ur-Dragon, Scourge of Valkas, Shivan Dragon, Smaug the Impenetrable, Smaug the Magnificent, Spark Double, Steel Hellkite, Summon: Bahamut, Terror of the Peaks, Thundermaw Hellkite, Tiamat, Utvara Hellkite, Young Pyromancer

**Artifacts (7).** Arcane Signet, Cursed Mirror, Mana Vault, Mox Jasper, Relic of Sauron, Sol Ring, The One Ring

**Enchantments (5).** Doubling Season, Dragon Tempest, Rhystic Study, Temur Ascendancy, Triple Triad

**Instants (10).** Chaos Warp, Cyclonic Rift, Fact or Fiction, Fierce Guardianship, Lightning Bolt, Path to Exile, Swords to Plowshares, Teferi's Protection, Vampiric Tutor, Worldly Tutor

**Sorceries (7).** Austere Command, Crux of Fate, Decimate, Demonic Tutor, Nature's Lore, Three Visits, Twinflame

**Lands (33).** Ancient Tomb, Arid Mesa, Badlands, Bayou, Blood Crypt, Cavern of Souls, City of Brass, Command Tower, Exotic Orchard, Flooded Strand, Maelstrom of the Spirit Dragon, Marsh Flats, Overgrown Tomb, Path of Ancestry, Plateau, Polluted Delta, Sacred Foundry, Savannah, Scalding Tarn, Scrubland, Sokenzan, Crucible of Defiance, Steam Vents, Stomping Ground, Sulfurous Springs, Taiga, Tropical Island, Tundra, Underground Sea, Volcanic Island, Windswept Heath, Wooded Foothills, 1 Forest, 1 Swamp

## Import list (100)

```
1 Ancient Copper Dragon
1 Ancient Tomb
1 Arcane Signet
1 Arid Mesa
1 Austere Command
1 Badlands
1 Bayou
1 Birds of Paradise
1 Blood Crypt
1 Cavern of Souls
1 Chaos Warp
1 City of Brass
1 Command Tower
1 Crux of Fate
1 Cursed Mirror
1 Cyclonic Rift
1 Decimate
1 Delighted Halfling
1 Demonic Tutor
1 Doubling Season
1 Dragon Tempest
1 Dragonlord Dromoka
1 Dragonlord Ojutai
1 Dragonlord Silumgar
1 Elvish Mystic
1 Exotic Orchard
1 Fact or Fiction
1 Fierce Guardianship
1 Flooded Strand
1 Ganax, Astral Hunter
1 Garth One-Eye
1 Glorybringer
1 Goldspan Dragon
1 Hellkite Tyrant
1 Klauth, Unrivaled Ancient
1 Korvold, Fae-Cursed King
1 Kura, the Boundless Sky
1 Lathliss, Dragon Queen
1 Lightning Bolt
1 Llanowar Elves
1 Maelstrom of the Spirit Dragon
1 Mana Vault
1 Marsh Flats
1 Miirym, Sentinel Wyrm
1 Morophon, the Boundless
1 Mox Jasper
1 Nature's Lore
1 Nicol Bolas, the Ravager
1 Niv-Mizzet, the Firemind
1 Old Gnawbone
1 Overgrown Tomb
1 Path of Ancestry
1 Path to Exile
1 Plateau
1 Polluted Delta
1 Relic of Sauron
1 Rhystic Study
1 Rivaz of the Claw
1 Sacred Foundry
1 Savannah
1 Scalding Tarn
1 Scion of Draco
1 Scion of the Ur-Dragon
1 Scourge of Valkas
1 Scrubland
1 Shivan Dragon
1 Smaug the Impenetrable
1 Smaug the Magnificent
1 Sokenzan, Crucible of Defiance
1 Sol Ring
1 Spark Double
1 Steam Vents
1 Steel Hellkite
1 Stomping Ground
1 Sulfurous Springs
1 Summon: Bahamut
1 Swords to Plowshares
1 Taiga
1 Teferi's Protection
1 Temur Ascendancy
1 Terror of the Peaks
1 The One Ring
1 The Ur-Dragon
1 Three Visits
1 Thundermaw Hellkite
1 Tiamat
1 Triple Triad
1 Tropical Island
1 Tundra
1 Twinflame
1 Underground Sea
1 Utvara Hellkite
1 Vampiric Tutor
1 Volcanic Island
1 Windswept Heath
1 Wooded Foothills
1 Worldly Tutor
1 Young Pyromancer
1 Forest
1 Swamp
```
