# Titania, Protector of Argoth — Mono-Green Lands to the Graveyard

A mono-green ramp deck built around one trick: **Titania, Protector of Argoth** makes a 5/3 Elemental every time a land
goes from the battlefield to the graveyard, and her enter trigger brings a land back so it can go again. The deck has
three lands that sacrifice themselves, a lot of ramp and card draw, and a top end of big green creatures to finish.
It wins late and through the board, not through a combo, and it is weak to graveyard hate and to fast decks that kill
it before the 5/3s pile up.

| | |
|---|---|
| **Commander** | Titania, Protector of Argoth ({3}{G}{G}, 5/3, green) |
| **Bracket** | 2 or 3 by the list (no Game Changers, no tutors, no infinite combo) |
| **Simulator** | deck key `titania`; results in `decklists/pool/pool-results.md`, history in `decklists/CHANGELOG.md` |

## Strategy

### The commander
**Titania, Protector of Argoth** reads: *When Titania enters, return target land card from your graveyard to the
battlefield. Whenever a land you control is put into a graveyard from the battlefield, create a 5/3 green Elemental
creature token.*

- Only a land that goes **from the battlefield** to the graveyard counts. Lands that are milled, discarded or put
  there from the library or hand do not make a 5/3, but they are what her enter trigger returns.
- The returned land comes back untapped, so a fetch land can be cracked again the same turn. One Titania enter
  trigger is therefore a land, a basic land and a 5/3.
- She is a 5/3 for five mana: easy to kill. The deck does not mind. Every recast returns another land, so removal on
  her buys you a second round of the engine.

### Game plan
- **Turns 1-3.** Play a mana creature (Llanowar Elves) or a ramp spell (Rampant Growth, Nature's Lore) and fill the
  graveyard with the cheap fillers. Land drops come first: with 38 lands you rarely miss them.
- **Turns 4-6.** Cast Titania with fetch lands on the battlefield or in the graveyard. Crack every fetch land you
  control for a 5/3 each, let her enter trigger return one, and crack it again. Add the card draw (Garruk's Uprising,
  Harmonize, Tireless Tracker) as you go.
- **Late game.** The 5/3s and the big green creatures win the game. Turn the pile into lethal with Overwhelming
  Stampede, End-Raze Forerunners or Greenbelt Radical, or keep removing blockers with the bite and fight spells.
  The wins come late, so protect the engine and keep drawing cards rather than racing.

### Lands that go to the graveyard
**Evolving Wilds**, **Terramorphic Expanse** and **Fabled Passage** are the only lands in the deck that put
themselves in the graveyard, and each one is a 5/3 as soon as Titania is on the battlefield. They also fetch a basic
Forest, so cracking them ramps you and thins the deck. The basic enters tapped, except that Fabled Passage untaps
it once you control four or more lands.

- **Fetch lands in the graveyard are Titania's fuel.** Leave them there. Don't take them back to hand with Pothole
  Mole before Titania is on the battlefield, and when her enter trigger asks for a target, pick a fetch land, not a
  Forest.
- **Hold a fetch land when Titania is a turn or two away.** Cracking every one in the same turn after she lands is a
  5/3 each. If you need the mana now, crack anyway: it goes to the graveyard, where she can return it later.
- **Opponents' land destruction** and any other way to lose a land also pay: a destroyed fetch land is a 5/3 too.

### Fill the graveyard
Satyr Wayfinder, Malevolent Rumble, Something Worth Saving, Scout the City and Pothole Mole put cards from your
library into the graveyard. Each is also a card-selection spell: take the best permanent (or land) and let the rest
fill the graveyard with returnable lands. **Pothole Mole** also returns a land card from the graveyard to your hand,
which gives you an extra land drop (and a landfall trigger) when you replay it.

### Ramp and land drops
- **Mana creatures:** Llanowar Elves, Noxious Newt (a deathtouch blocker), Hardbristle Bandit (any colour; it untaps
  when you commit a crime, which targeting an opponent or their stuff does, so a bite spell gives you the mana back),
  Topiary Lecturer (taps for more mana as it grows), Silhana Starfletcher (reach), Greenhouse Propagator (gains
  1 life for each creature that enters) and Bugenhagen, Wise Elder (taps for any colour; draws a card on your upkeep
  if you control a creature with power 7 or more, such as Ghalta or End-Raze Forerunners).
- **Spells:** Rampant Growth, Nature's Lore (a Forest straight onto the battlefield, untapped), Cultivate and
  Skyshroud Claim (two Forests, untapped), plus Thirsting Roots (a basic to hand, or proliferate).
- **Creatures that find land:** Environmental Scientist, Studious First-Year (enters prepared, so you can cast its
  *{1}{G}* land-fetching sorcery side from the battlefield) and Spider-Man, Brooklyn Visionary. Spider-Man's
  web-slinging costs {2}{G} if you return a tapped creature to your hand: a tapped mana creature, or, late in the
  game, Titania herself after she attacks, so you can recast her and return another land. Vinelasher Adept can be discarded for basic
  landcycling {2} when you need a land.
- **Cartographer's Survey** puts two lands from the top seven onto the battlefield (tapped); Track Down, Boughside
  Wanderers and Follow the Lumarets find lands and creatures from the top of the library.

### Landfall and counters
- **Tireless Tracker** makes a Clue for every land that enters, including each land Titania returns and each basic a
  fetch land finds, and grows when you sacrifice a Clue. It is the best card-draw engine in the deck.
- **Boughside Wanderers** gets +2/+2 until end of turn each time a land enters (a 6/6 on a land turn) and digs four cards deep when it enters.
- **Kura, the Boundless Sky** is a 4/4 flying deathtouch creature. When she dies, either fetch three lands to your
  hand (fetch lands for Titania, or lands for your next three turns) or, late, leave an X/X Spirit for the number of
  lands you control.
- **Vorinclex, Monstrous Raider** doubles the counters you put on things: Tireless Tracker, Topiary Lecturer's
  Increment, Vinelasher Adept's three counters (six), Genesis Hydra's X counters, Knockout Maneuver, Greenbelt
  Radical, and Thirsting Roots' proliferate. It also halves the counters opponents put on their things. It has
  trample and haste, so it is a threat the turn it lands.

### Card draw
- **Garruk's Uprising** is the card that turns the engine into cards: it draws when it enters if you control a
  creature with power 4 or more, draws again for every creature with power 4 or more that enters afterwards (every 5/3
  Elemental counts), and gives your creatures trample.
- **Harmonize** (three cards), **Shamanic Revelation** (a card for each creature, plus 4 life for each creature with
  power 4 or more) and **Lifestream's Blessing** (a card for the greatest power you control; foretell it for {2}
  early, and its later cast also gains life).
- **Tenured Concocter** (draws if an opponent targets it, and gets +2/+0 when you gained life this turn),
  **Rending Vines** and **Glorious Decay** (an artifact or enchantment answer or a graveyard exile that also draws a
  card, so they never feel dead). Life gain also turns on Follow the Lumarets' second reveal and Tenured Concocter:
  Greenhouse Propagator, Scout the City and Something Worth Saving all gain life.

### Big creatures
- **Annoyed Altisaur** (cascade, reach, trample) and **Hornet Queen** (five flying deathtouch bodies) are the best
  topdecks.
- **Ghalta the Unstoppable** costs {X} less, where X is your greatest power: with a single 5/3 on the battlefield she
  costs {3}{G}. She also gives the team trample.
- **Paladin of Predation** (toxic 6, can't be blocked by power 2 or less), **End-Raze Forerunners**, **Genesis Hydra**
  (put a permanent from the top X cards onto the battlefield when you cast it), **Lashwhip Predator** (costs {2} less
  against three or more creatures, reach), **Symbiotic Beast** (four Insects when it dies), **Infestation Expert**
  (Insect tokens each time it enters or attacks; two at night), **Thrashing Brontodon** (a body that can destroy an
  artifact or enchantment) and **Bestial Incursion** (a 4/4 trampler, with flashback) round out the pile.
- **Tumbleweed Rising** makes a token as big as your greatest power: a 5/5 with a 5/3, an 8/8 with Ghalta.
  **Compel Brutality**, **Bite Down** and the other bite spells use the same big power.

### Protection and removal
- **Protect Titania.** Blossoming Defense (+2/+2, hexproof) and Royal Treatment (hexproof, plus a Royal Role that
  gives +1/+1 and ward {1}) in response to removal. If she dies anyway, recast her.
- **Arachnogenesis** creates a 1/2 reach Spider for each creature attacking you and prevents all combat damage from
  non-Spider creatures that turn, including yours. Cast it on their turn when a big attack is coming.
- **Bite and fight spells:** Bite Down, Compel Brutality, Hard-Hitting Question, Diplomatic Relations, Knockout
  Maneuver and Chelonian Tackle (+0/+10, then it fights). Use your biggest creature as the source. Hard-Hitting
  Question costs {G}.
- **Artifact, enchantment and flier answers:** Carnivorous Canopy, Crushing Canopy, Pick Your Poison, Rending Vines,
  Glorious Decay and Scout the City (Bring Down). Silhana Starfletcher, Lashwhip Predator, Vinelasher Adept and
  Bugenhagen have reach.

### How it wins
- **Elemental swarm.** Four or five 5/3s in one turn plus Overwhelming Stampede (all creatures gain trample and
  +X/+X, where X is your greatest power) or End-Raze Forerunners (+2/+2, vigilance and trample for your other
  creatures) is lethal against most tables. Garruk's Uprising and Ghalta give trample to the whole team.
- **Paladin of Predation with Vorinclex.** Toxic 6 doubled is twelve poison counters, and ten is lethal. Paladin can't
  be blocked by creatures with power 2 or less.
- **Big bodies and recursion.** When the engine is slow, Hornet Queen, Annoyed Altisaur and Ghalta carry the game,
  and Titania's recasts keep making 5/3s.

### Weak spots and how to play around them
- **Graveyard hate** (Rest in Peace, Bojuka Bog, Relic of Progenitus and the like) turns the engine off, because
  there is nothing for her to return. Keep an answer for the artifact or enchantment version: Thrashing Brontodon,
  Carnivorous Canopy, Crushing Canopy, Rending Vines and Glorious Decay. A one-shot graveyard exile such as Bojuka
  Bog can't be answered, so against a deck that might have one, keep a fetch land on the battlefield rather than
  putting every one in the graveyard before Titania lands.
- **Sweepers and fast combo.** There is no counter-magic. Hold Arachnogenesis for combat, keep Blossoming Defense
  up for Titania when you can, and don't overextend into a sweeper when you already have enough power.
- **Slow starts.** The deck wins late. It needs the ramp: mulligan hands without a land or a mana creature.

## Key lines

- **Turns 1-4:** ramp (Llanowar Elves, Rampant Growth, Nature's Lore, Cultivate) and graveyard fillers. Play a fetch
  land but hold it uncracked when Titania is a turn or two away.
- **Titania lands:** crack the held fetch lands (a 5/3 each); her enter trigger returns a cracked one to crack again.
- **Removal on her:** Blossoming Defense or Royal Treatment in response; if she dies anyway, recast her: each recast
  returns a fetch land.
- **Big attack coming:** Arachnogenesis when an attack would deal a third of your life or more.
- **Ghalta turn:** once you control a 5/3, Ghalta costs {3}{G}; Tumbleweed Rising then makes an 8/8.

## Mana and consistency

38 lands: 35 Forest plus Evolving Wilds, Terramorphic Expanse and Fabled Passage. The deck plays more fetch lands than
a plain mono-colour deck would, and basics found by them enter tapped, but each sacrifice is a 5/3, the deck's
engine. Ramp: Llanowar Elves, Noxious Newt, Hardbristle Bandit, Topiary Lecturer, Silhana Starfletcher, Greenhouse
Propagator, Bugenhagen, plus Rampant Growth, Nature's Lore, Cultivate, Skyshroud Claim and Spider-Man. Environmental Scientist, Studious
First-Year, Thirsting Roots, Cartographer's Survey and Vinelasher Adept (landcycling) find more. Card draw: Harmonize, Shamanic Revelation, Garruk's Uprising, Tireless Tracker, Track
Down and Lifestream's Blessing. Interaction: thirteen bite, fight and artifact, enchantment and flier answers.

## Bracket and Rule 0

No Game Changers, no tutors and no infinite combo, so the list fits Bracket 2 or 3. Tell the table before the game
that:

- the deck is a land-and-token deck with big green creatures, not a combo deck;
- the poison line (Paladin of Predation with Vorinclex) kills in one hit;
- graveyard hate is the answer that hurts it most.

## Decklist by type (100)

**Commander (1).** Titania, Protector of Argoth

**Creatures (29).** Annoyed Altisaur, Boughside Wanderers, Bugenhagen Wise Elder, End-Raze Forerunners, Environmental Scientist, Genesis Hydra, Ghalta the Unstoppable, Greenbelt Radical, Greenhouse Propagator, Hardbristle Bandit, Hornet Queen, Infestation Expert, Kura the Boundless Sky, Lashwhip Predator, Llanowar Elves, Noxious Newt, Paladin of Predation, Pothole Mole, Satyr Wayfinder, Silhana Starfletcher, Spider-Man Brooklyn Visionary, Studious First-Year, Symbiotic Beast, Tenured Concocter, Thrashing Brontodon, Tireless Tracker, Topiary Lecturer, Vinelasher Adept, Vorinclex Monstrous Raider

**Enchantments (1).** Garruk's Uprising

**Instants (11).** Arachnogenesis, Bite Down, Blossoming Defense, Compel Brutality, Crushing Canopy, Diplomatic Relations, Glorious Decay, Lifestream's Blessing, Rending Vines, Royal Treatment, Something Worth Saving

**Sorceries (20).** Bestial Incursion, Carnivorous Canopy, Cartographer's Survey, Chelonian Tackle, Cultivate, Follow the Lumarets, Hard-Hitting Question, Harmonize, Knockout Maneuver, Malevolent Rumble, Nature's Lore, Overwhelming Stampede, Pick Your Poison, Rampant Growth, Scout the City, Shamanic Revelation, Skyshroud Claim, Thirsting Roots, Track Down, Tumbleweed Rising

**Lands (38).** Evolving Wilds, Fabled Passage, Terramorphic Expanse, 35 Forest

## Import list (100)

```
1 Annoyed Altisaur
1 Arachnogenesis
1 Bestial Incursion
1 Bite Down
1 Blossoming Defense
1 Boughside Wanderers
1 Bugenhagen, Wise Elder
1 Carnivorous Canopy
1 Cartographer's Survey
1 Chelonian Tackle
1 Compel Brutality
1 Crushing Canopy
1 Cultivate
1 Diplomatic Relations
1 End-Raze Forerunners
1 Environmental Scientist
1 Evolving Wilds
1 Fabled Passage
1 Follow the Lumarets
1 Garruk's Uprising
1 Genesis Hydra
1 Ghalta the Unstoppable
1 Glorious Decay
1 Greenbelt Radical
1 Greenhouse Propagator
1 Hard-Hitting Question
1 Hardbristle Bandit
1 Harmonize
1 Hornet Queen
1 Infestation Expert
1 Knockout Maneuver
1 Kura, the Boundless Sky
1 Lashwhip Predator
1 Lifestream's Blessing
1 Llanowar Elves
1 Malevolent Rumble
1 Nature's Lore
1 Noxious Newt
1 Overwhelming Stampede
1 Paladin of Predation
1 Pick Your Poison
1 Pothole Mole
1 Rampant Growth
1 Rending Vines
1 Royal Treatment
1 Satyr Wayfinder
1 Scout the City
1 Shamanic Revelation
1 Silhana Starfletcher
1 Skyshroud Claim
1 Something Worth Saving
1 Spider-Man, Brooklyn Visionary
1 Studious First-Year
1 Symbiotic Beast
1 Tenured Concocter
1 Terramorphic Expanse
1 Thirsting Roots
1 Thrashing Brontodon
1 Tireless Tracker
1 Titania, Protector of Argoth
1 Topiary Lecturer
1 Track Down
1 Tumbleweed Rising
1 Vinelasher Adept
1 Vorinclex, Monstrous Raider
35 Forest
```
