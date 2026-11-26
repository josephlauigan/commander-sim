# The Ur-Dragon — Five-Colour Dragons (High Bracket 4)

Ramp on turns 1-3, a Dragon engine on turn 4 or 5, then win in combat. The Ur-Dragon draws a card for every attacking
Dragon and drops a permanent from your hand onto the battlefield free, so each attack refills your hand and puts a
seven-drop into play. Terror of the Peaks, Scourge of Valkas and Dragon Tempest turn every Dragon that enters into
damage, and Miirym and Lathliss double the Dragons. Beside those sits one four-card combo, **Garth One-Eye + Spark
Double + Twinflame + Doubling Season**, which makes mana and hasty tokens without limit. Around them sit the top-end
Dragons (Utvara Hellkite, Summon: Bahamut, Niv-Mizzet) and a spells package (Lightning Bolt, Fact or Fiction). A speed package
(Urza's Incubator, Dragonspeaker Shaman, Sapphire Medallion, the two Talismans, Sylvan Library and Jeska's Will) gets the commander and its
engine out earlier, and a protection and draw package (Mithril Coat, Giver of Runes, The Theorist and Tymna the Weaver) keeps the key permanent alive and refills the
hand; Rhythm of the Wild makes every creature spell uncounterable. The deck is weak to board wipes and mass bounce after it
has committed its Dragons, to decks that win before turn 5, and to Blood Moon effects (no basic land). Nothing in this document has been simulated yet.

| | |
|---|---|
| **Commander** | The Ur-Dragon ({4}{W}{U}{B}{R}{G}, 10/10 flier); all five colours. Eminence: other Dragon spells you cast cost {1} less while it is in the command zone or on the battlefield |
| **Bracket** | 4: 10 Game Changers and one four-card combo. Played against Tier 5 (High Bracket 4) |
| **Game Changers (10)** | Ancient Tomb, Demonic Tutor, Field of the Dead, Fierce Guardianship, Jeska's Will, Mana Vault, Rhystic Study, The One Ring, Vampiric Tutor, Worldly Tutor |
| **Simulator** | Deck key `urdragon`, tier T5; findings in `audit/urdragon/FINDINGS.md`, history in `decklists/CHANGELOG.md`; plan file `data/plans/urdragon.toml`. No sims yet |
| **Buys** | None: every card not owned is proxied |

## Strategy

### The commander

**The Ur-Dragon** ({4}{W}{U}{B}{R}{G}, 10/10). *Eminence — as long as it is in the command zone or on the
battlefield, other Dragon spells you cast cost {1} less. Flying. Whenever one or more Dragons you control attack, draw
that many cards, then you may put a permanent card from your hand onto the battlefield.*

It costs nine, so it is not the turn-4 play. It comes down on turn 5 or 6 off Mana Vault, dorks and Treasures (Mox Jasper once a Dragon is out), or
for {4} with **Morophon** out (Morophon naming Dragon takes {W}{U}{B}{R}{G} off every Dragon spell), or for {7} with
**Urza's Incubator** out (see below). While it sits in
the command zone it already discounts every Dragon you cast, so the deck can win without ever casting it.

What it does once it attacks:
- **Draws and cheats.** Each attacking Dragon is a card, and you may drop a permanent from the new hand onto the
  battlefield: a seven-drop or bigger (Old Gnawbone, Morophon, Utvara Hellkite, Summon: Bahamut), an engine, or Doubling Season.
  Tiamat only fetches Dragons if you cast it, so don't drop it.
- **Commander damage.** It is a 10/10 flier: two hits kill an opponent. With **Utvara Hellkite** out, every attacking
  Dragon makes a 6/6 flying Dragon, so the next attack is far bigger.
- **Protection** from the deck: **Scion of Draco** gives every creature of a colour vigilance, hexproof, lifelink, first
  strike or trample, and The Ur-Dragon is all five. **Dragonlord Ojutai** has hexproof while it is untapped.
  **Fierce Guardianship** is free while you control a commander. **Mithril Coat** makes a legendary creature
  indestructible and **Giver of Runes** gives one creature protection from a colour (both below). **Delighted Halfling** and
  **Cavern of Souls** (name Dragon) make the Dragons uncounterable, and **Rhythm of the Wild** does the same for every creature spell you cast.
- **Kyodai, Soul of Kamigawa** ({3}{W}, a 3/3 flash flier, a Dragon Spirit): when he enters, another target permanent gains
  indestructible for as long as you control him. Flash him in at an opponent's end step, or in response to a destroy effect
  or a destroy wipe aimed at one key permanent (Terror of the Peaks, Doubling Season, a Garth, the Ur-Dragon on the
  battlefield): that one survives, and the rest of the wipe still resolves. It does nothing against exile, bounce, -X/-X,
  counterspells or a wipe that sacrifices, and it ends if Kyodai leaves. {W}{U}{B}{R}{G}: he gets +5/+5 until end of turn
  (it needs all five colours at once). He is a Dragon, so Eminence takes {1} off him and
  Urza's Incubator naming Dragon {2}; he also triggers Terror of the Peaks, Scourge of Valkas and Dragon Tempest when he enters.
- **A second cast costs {2} more per previous cast.** It is a nine-mana commander, so a recast is ten or more. Protect
  it, and do not rely on recasting.

#### Getting The Ur-Dragon out earlier

The deck's weakness is reaching its commander, so a package of cards works on speed. The Ur-Dragon has to be cast (no card in the list puts it onto the battlefield any more), and cost reducers remove only generic mana
(its {4} is the only generic part of its cost); Eminence does not discount The Ur-Dragon itself.

- **Urza's Incubator** ({3} artifact, naming Dragon): Dragon creature spells cost {2} less. The Ur-Dragon
  ({4}{W}{U}{B}{R}{G}) goes from nine to seven, and every other Dragon costs two less on top of Eminence's one. With
  Morophon as well, The Ur-Dragon costs {2}. Cast it on turn 3 or 4, before the Dragons, and name Dragon.
- **Dragonspeaker Shaman** ({1}{R}{R}, a 2/2): Dragon spells you cast cost {2} less. It does the Incubator's job from a creature body, and the two stack: The Ur-Dragon
  with both costs {W}{U}{B}{R}{G} (five mana), and a five-drop Dragon with both and Eminence costs only its two red pips. It needs {R}{R} on turn 3, and a removal spell on it
  loses the discount; it is not a Dragon, so it triggers no Dragon payoff.
- **Talisman of Hierarchy** ({2} artifact): taps for {C}, or for {W} or {B} and deals 1 damage to you. **Talisman of Curiosity** ({2} artifact): taps for {C}, or for {G} or {U} and deals 1
  damage to you. A turn-2 rock that pays for a turn-3 four-drop and makes white, black, green or blue, the thinner colours in the list.
- **Sapphire Medallion** ({2} artifact): blue spells you cast cost {1} less. The Ur-Dragon, Rhystic Study, Fact or Fiction, Niv-Mizzet, Miirym, Dragonlord Ojutai, Garth, Tiamat and The Theorist (any
  spell that is blue, alone or with other colours) get cheaper, and the generic part of The Ur-Dragon's cost drops by one more. It makes no mana itself, so it is a turn-2 play only with another rock behind it.
- **Sylvan Library** ({1}{G}): at your draw step you may draw two extra cards, then for each of two cards drawn this turn
  pay 4 life or put it on top of the library. A Library on turn 2 draws up to two extra cards a turn, to find the ramp and the Dragons. At 40 life
  pay only for a card you will cast; the life adds to the fetches, the shocks and The One Ring.
- **Jeska's Will** ({2}{R}, sorcery): add {R} for each card in an opponent's hand, or exile your top three cards and play
  them this turn. With a commander on the battlefield you get both modes, a burst of mana and three extra cards on a turn
  when the commander is out.

#### Protecting and refilling

- **Rhythm of the Wild** ({1}{R}{G} enchantment): creature spells you cast can't be countered, and each nontoken creature you control has riot (it enters with your choice of a +1/+1 counter or haste). The Ur-Dragon, every Dragon, Garth and Spark Double resolve through a counterspell, and the haste choice lets any nontoken creature, a Dragon included, attack the turn it comes down. It does nothing by itself: Twinflame, Doubling Season and the tutors can still be countered, and it does not stop removal.
- **Mithril Coat** ({3}, flash Equipment, itself indestructible): when it enters, attach it to a legendary creature you control; the equipped creature has indestructible. Equip {3} to
  move it. Flash it in at an opponent's end step or in response to a destroy effect or a destroy wipe aimed at a legendary creature (The Ur-Dragon on the battlefield, Garth, Lathliss, Miirym,
  Niv-Mizzet, Old Gnawbone, either Smaug, Kura, Ganax, Kyodai, Tiamat, Tymna) and that creature survives. It does nothing against exile, bounce, -X/-X, a sacrifice or a counterspell, and Terror of the Peaks, Goldspan Dragon and
  the other non-legendary Dragons cannot be the target of its enter trigger (they take the {3} equip).
- **Giver of Runes** ({W}, a 1/2): {T}: another target creature you control gains protection from colorless or from a colour of your choice until end of turn. It needs a turn without summoning
  sickness first. It stops targeted removal and damage of one colour (and blockers of that colour) on the key Dragon, but not a wipe that does not target.
- **The Theorist, Jace Beleren** ({2}{U}{U} planeswalker, loyalty 3): at the beginning of each opponent's draw step you draw a card (about three extra cards a round at a four-player table, and each is a
  Niv-Mizzet ping). +1 makes a 1/1 Illusion to defend him; -2 returns up to one artifact or creature from each opponent to its owner's hand (a tempo play before an attack or against a combo piece); -6 draws three and grows the creatures. Opponents will attack him: keep a blocker.
- **Tymna the Weaver** ({1}{W}{B}, a 2/2 lifelink; here a plain creature in the 99, not a commander): at the beginning of each of your postcombat main phases you may pay X life, where X is the number of opponents dealt combat damage this turn,
  to draw X cards. When attackers connect she draws a card for each opponent hit, at 1 life per card. She is legendary, so Mithril Coat can protect her.

### How the deck wins

#### Route 1: The Ur-Dragon in combat (main route)

Cast a hasty Dragon (Glorybringer, Thundermaw Hellkite, Goldspan Dragon, Smaug) the turn it
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
  ({2}{U}{U}{R}{R}) pings any target for every card you draw (every Ur-Dragon attack, Rhystic Study, The One Ring, The Theorist, Tymna's postcombat draw, Relic of Sauron's loot,
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

- **Old Gnawbone** (a Treasure for each point of combat damage dealt by your
  creatures) turns a connecting attack into the mana for the next play.
- **Hellkite Tyrant** takes every artifact a player controls when it connects. Against an artifact deck that is the
  game. (Twenty or more artifacts wins on your upkeep.)
- **Summon: Bahamut** ({9}, a flying Saga Dragon) destroys up to one nonland permanent in each of its first two
  chapters, draws two cards in the third, and in its last chapter deals damage to each opponent equal to the total mana
  value of your other permanents, then it is sacrificed.
- **Tiamat** (cast it) fetches five Dragons to hand
  and refuels the Ur-Dragon trigger.
- **Crux of Fate** is a one-sided wrath: destroy all non-Dragon creatures. It also kills your own dorks, Rivaz,
  Garth, Dragonspeaker Shaman, Giver of Runes, Tymna and Spark Double (unless it copies a Dragon), so use it before the combo is set up or when the board is mostly Dragons. Mithril Coat on a legendary creature
  saves that one creature from it.

### The Dragons

| Cost | Dragons | Role |
|---|---|---|
| 4 | Smaug the Magnificent, Nicol Bolas the Ravager, Kyodai, Soul of Kamigawa | Early hasty flier with a Treasure every upkeep; discard and a flip into a planeswalker; a flash flier that protects one permanent (see Protection) |
| 5 | Terror of the Peaks, Goldspan Dragon, Glorybringer, Thundermaw Hellkite, Scourge of Valkas, Ganax, Kura, Dragonlord Ojutai | The core: damage, haste, removal, Treasures, card selection |
| 6 | Lathliss, Miirym, Dragonlord Dromoka, Hellkite Tyrant, Shivan Dragon, Niv-Mizzet | Engines and finishers |
| 7 | Tiamat, Old Gnawbone, Smaug the Impenetrable, Morophon | The seven-drops The Ur-Dragon drops in free; Morophon is a Dragon (changeling) and makes the commander cost {4}; Smaug the Impenetrable is hasty and indestructible, so it survives a destroy wipe, and turns noncombat damage dealt to it into Treasures |
| 8-9 | Utvara Hellkite (8), Summon: Bahamut (9) | The finishers: a Dragon-attack engine and a slow four-chapter removal, card draw and burn |
| Varies | Scion of Draco | Costs {12} minus {2} per basic land type among your lands: about {2} with five types, {1} less than that with Eminence |

### The ramp and engines

- **Mana Vault, Sol Ring, Arcane Signet** (and Ancient Tomb): the fast start. Cast on turn 1 and left
  untapped, Mana Vault gives {C}{C}{C} on turn 2: two lands and the Vault pay for a five-drop. **Mox Jasper** makes any
  colour once you control a Dragon, so it is a dead card before one lands. **Talisman of Hierarchy** ({2}) taps for {C}, or for {W} or {B}
  at 1 damage; **Talisman of Curiosity** ({2}) taps for {C}, or for {G} or {U} at 1 damage. **Sapphire Medallion** ({2}) makes no mana but takes {1} off every blue spell.
- **Relic of Sauron** ({4}): taps for two mana in any mix of blue, black and red, which pays for a seven-drop and fixes
  the blue and black the elves don't make. Later, {3} and a tap draws two cards and discards one. Its own mana can't pay
  for the loot.
- **Birds of Paradise, Llanowar Elves, Avacyn's Pilgrim, Delighted Halfling:** one-mana dorks. Llanowar makes {G}; Avacyn's Pilgrim costs {G} and makes {W}, one of the two thinnest colours here. The Halfling makes any colour
  for a legendary spell (The Ur-Dragon, Garth, Tiamat, Miirym, Smaug...) and that spell can't be countered.
- **Nature's Lore, Three Visits:** each fetches a Forest dual (a shock land costs 2 life to enter untapped; the deck has no basic Forest), which fixes colours and adds
  basic types for Scion of Draco.
- **Rivaz of the Claw** ({1}{B}{R}): taps for two mana for Dragon spells, and lets you cast one Dragon from the
  graveyard each turn.
- **Rhystic Study, Sylvan Library, The One Ring, Fact or Fiction, The Theorist, Tymna the Weaver:** the draw engines. The Theorist draws a card in each opponent's draw step and Tymna one per opponent hit by combat damage, and both feed Niv-Mizzet's pings. Rhystic makes the table pay for casting into you;
  Sylvan Library lets you draw up to two extra cards in the draw step for 4 life each (or put them back);
  The One Ring protects you for a turn and draws more each turn (it costs 1 life per burden counter each upkeep); Fact
  or Fiction is a four-mana instant for a pile of cards.
- **Lightning Bolt, Fact or Fiction:** the spells package. Bolt is cheap removal or reach; Fact or Fiction is the four-mana instant for a pile of cards.
- **Dragonspeaker Shaman, Mithril Coat, Giver of Runes, Rhythm of the Wild:** the bodies and tools of the speed and protection packages (see Getting The Ur-Dragon out earlier and Protecting and refilling).

### Game plan

**Turns 1-2.** Land, a rock or a dork, Mana Vault if you hold it. Nature's Lore or Three Visits fetches
the dual that fixes your colours. A turn-1 dork or Sol Ring into a turn-2 four-drop (Smaug), or Mana Vault into a turn-2 five-drop, is the
strongest start. Sylvan Library is a good turn-2 play when the hand has no ramp.

**Turns 3-4.** Dragonspeaker Shaman or Urza's Incubator (naming Dragon) first, before the big Dragons and The Ur-Dragon, then the cheap Dragons (Smaug the Magnificent, Bolas, Ganax)
and an engine (Rhystic Study, The One Ring, The Theorist). Keep
mana for Fierce Guardianship or a free spell on the opponent's turn when you hold protection.

**Turns 5-7.** Land Terror of the Peaks or the key Dragon, then The Ur-Dragon (or Morophon first), then attack. Win
through the combat route or the Garth loop once the pieces are there.

**Hold up or go?** Go when the table has no sweeper in sight and you hold protection, or when an opponent is about to win.
Hold the Dragons in hand when an opponent has shown a board wipe, and rely on Fierce Guardianship (it counters a
noncreature wipe), Smaug the Impenetrable (indestructible, so it survives a destroy wipe), Mithril Coat flashed onto a legendary creature and the cheap haste
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
| 2 | Ancient Tomb, Mana Vault, Mox Jasper, Old Gnawbone, Tiamat, Utvara Hellkite, Rivaz of the Claw | Ship | One land that only makes colourless, and nothing to cast before turn 5 |
| 3 | Tundra, Bayou, Avacyn's Pilgrim, Nature's Lore, Three Visits, Lathliss, Doubling Season | Keep (close) | Two duals plus Nature's Lore and Three Visits fetch two more duals, and the Pilgrim adds mana: Lathliss and Doubling Season by turn 5 |
| 4 | Demonic Tutor, Vampiric Tutor, Command Tower, Sol Ring, Garth One-Eye, Spark Double, Twinflame | Keep (close) | Tutors and Sol Ring find the rest, but it needs a second land and Doubling Season to be a combo |

### What to tutor for

| Situation | Find | With |
|---|---|---|
| Garth and Spark Double in hand | Doubling Season, then Twinflame | Demonic or Vampiric Tutor for Doubling Season, then Twinflame |
| Doubling Season on the battlefield, no Garth | Garth One-Eye | Worldly Tutor, then the loop with Spark Double and Twinflame |
| Seven mana and no Dragon | Terror of the Peaks or Goldspan Dragon | Demonic Tutor or Vampiric Tutor (to the top) |
| Opponent about to wipe | Fierce Guardianship | Vampiric Tutor, hold the mana (it is free only with a commander out) and the card |
| The Ur-Dragon is about to attack | A seven-drop to put in free | Tutor for Old Gnawbone or Utvara Hellkite and leave it in hand |
| A creature with the table's best card | Swords, Path or Chaos Warp | Any tutor |

### Protecting the key turn

- **Free spells:** Fierce Guardianship (counter a noncreature spell) costs nothing while you control a commander, which
  is any turn after The Ur-Dragon lands.
- **Mithril Coat** (flash) makes a legendary creature indestructible: in response to a destroy effect or a destroy wipe, or at an opponent's end step. It does not stop exile, bounce, -X/-X or
  a sacrifice. Answer a wipe that does not destroy with Fierce Guardianship, or do not commit the Dragons.
- **Giver of Runes** ({T}) gives another creature protection from a colour until end of turn: held up on an opponent's turn it answers a targeted removal spell or a block. A wipe that does not target goes through it.
- **Dragonlord Ojutai** has hexproof while it is untapped (itself only). **Scion of Draco** gives your blue creatures
  hexproof, and The Ur-Dragon is all five colours.
- **Cavern of Souls** (name Dragon) and **Delighted Halfling** make the key Dragon or Garth uncounterable.
- **Dromoka** stops every spell on your turn: cast it before the combo or the attack.

### Interaction

- **Removal:** Swords to Plowshares, Path to Exile, Lightning Bolt (3 damage to any target), Chaos Warp (any permanent, random replacement).
- **On bodies:** Glorybringer (4 damage each exert), Terror of
  the Peaks and the Dragon-enters pingers (damage to any target), Thundermaw Hellkite (taps and damages fliers), Garth's
  Terror and Disenchant copies.
- **Wipes:** Crux of Fate (non-Dragon mode), Nibelheim Aflame.
- **Nibelheim Aflame** ({2}{R}{R} sorcery): choose a creature you control; it deals damage equal to its power to each
  other creature, yours included, and takes none itself. It is not a Dragon-safe wipe like Crux of Fate: every Dragon
  whose toughness is at or below the chosen creature's power dies too. Cast it when the table's board is the problem
  and yours is small, or when the chosen creature is your only one. The game picks your highest-power creature, which
  at 10 power for The Ur-Dragon kills every other creature you have. Smaug the Impenetrable (indestructible)
  survives it as a bystander and turns the damage into Treasures. Its flashback ({5}{R}{R}) discards your hand and draws four: use it late, when the hand is empty or only
  holds cards you can't cast.

### Matchups (Tier 5)

The matchup notes are expectations, not measured.

| Opponent type | What to do |
|---|---|
| Fast combo (Kinnan, Yawgmoth, Zur) | Hold Fierce Guardianship; kill the commander with Swords or Path; go over the top with a hasty Dragon on turn 5 |
| Board wipes | Keep Dragons in hand and hold Fierce Guardianship; Smaug the Impenetrable survives a destroy wipe, and Kyodai, flashed in, saves one other permanent from it; Rivaz and Tiamat rebuild; Mithril Coat, flashed in on a legendary creature, saves that one from a destroy wipe |
| Creature-heavy midrange | Glorybringer, Thundermaw and the Dragon-enters damage clear blockers; Crux of Fate (non-Dragon mode) clears their board but not yours |
| Artifact decks (Thanos, Urza, Kinnan) | Hellkite Tyrant takes their artifacts; Chaos Warp answers the key one |
| Stax / Blood Moon | Under Blood Moon every land here makes only {R} (there is no basic), so lean on the rocks and dorks for the other colours; cast Dromoka to dodge counterspells; Cavern of Souls names Dragon |

### Life management

You start at 40 and pay life for the mana base (seven fetches, five shocks, City of Brass, Sulfurous Springs, Ancient Tomb), Mana Vault,
Vampiric Tutor, The One Ring, the Talismans (1 damage per coloured tap), Tymna the Weaver (1 life per card drawn) and Sylvan Library (4 life per card kept in hand). Dromoka gives
lifelink, and Scion of Draco gives it to The Ur-Dragon and every other black creature. Don't fetch for a colour you
already have.

### Common mistakes

- Casting Crux of Fate in non-Dragon mode with your dorks and Garth on the board.
- Dropping Tiamat with The Ur-Dragon (it must be cast to fetch Dragons).
- Casting Twinflame into open blue mana without Fierce Guardianship or Dromoka.
- Paying for The Ur-Dragon (nine) when Morophon and Rivaz would let you cast it and the Dragons you hold in the
  same turn for less.
- Fetching a land that adds a colour you already have: a fetch finds any land with one of its two basic types, so pick
  the dual (or shock) that adds the colour you lack.
- Going to combat into a Settle the Wreckage or Aetherize pattern with no Fierce Guardianship up.

## Mana

33 lands, all nonbasic (the deck has no basic land). Ten original duals (Tundra, Underground Sea, Badlands,
Taiga, Savannah, Scrubland, Volcanic Island, Bayou, Plateau, Tropical Island), seven fetches, five shocks (Blood Crypt,
Steam Vents, Stomping Ground, Overgrown Tomb, Sacred Foundry), Command Tower, City of Brass, Path of Ancestry (enters
tapped, any colour, scries when it pays for a Dragon creature spell), Maelstrom of the Spirit Dragon (any colour for
Dragon spells; {4}, {T}, sacrifice it to fetch a Dragon), Sokenzan, Crucible of Defiance ({R}; channel it for two hasty
1/1s), Sulfurous Springs ({B} or {R} for 1 damage), Exotic Orchard (any colour an opponent's land could produce),
Minas Tirith (a legendary land that taps for {W}; it enters tapped unless you control a legendary creature, and {1}{W}, {T}: draw a card, only if you attacked with two or more creatures this turn),
Cavern of Souls, Ancient Tomb and Field of the Dead (enters tapped, makes {C}, and creates a 2/2 Zombie whenever a land
enters while you control seven or more lands with different names; almost every land here has its own name). A fetch finds any dual or shock that shares a type, so every colour is reachable
(Springs, Orchard, Minas Tirith and the other lands without a land type cannot be fetched): land sources by colour are white 16,
blue 15, black 17, red 20 and green 16, counting the fetches and the three any-colour lands, with
Maelstrom adding one more of each for Dragon spells and Orchard adding what the table's lands make. Red is the heaviest
colour (RR on most Dragons, RRR on Scourge of Valkas), then green and black; white and blue are the thinnest, for
Niv-Mizzet and The Theorist ({U}{U}), Fact or Fiction, Tymna and Giver of Runes. Add the mana creatures and rocks (Talisman of Hierarchy makes {W} or {B}, Talisman of Curiosity {G} or {U}): Arcane Signet, Birds, Mox Jasper (with a
Dragon out) and Delighted Halfling make any colour, and Treasures from Goldspan, Ganax, Smaug and
Old Gnawbone fix the rest. The deck has 17 legendary creatures besides the commander, so Minas Tirith
enters untapped once one is out. With no basic land, a Blood Moon turns every land in the deck into a Mountain that makes only {R}: red is the
only colour the lands make, and the other four come from the rocks, the mana creatures and Treasures.

## Bracket and Rule 0

Bracket 4 (High): 10 Game Changers and one four-card combo (Garth One-Eye + Spark Double + Twinflame + Doubling Season,
unbounded mana and tokens at sorcery speed). No mass land destruction, no extra turns, no fast-mana turbo (no Mana Crypt
or Jeweled Lotus). Tell the table about the combo before the game.

## Decklist by type (100)

**Commander (1).** The Ur-Dragon

**Creatures (34).** Avacyn's Pilgrim, Birds of Paradise, Delighted Halfling, Dragonlord Dromoka, Dragonlord Ojutai, Dragonspeaker Shaman, Ganax, Astral Hunter, Garth One-Eye, Giver of Runes, Glorybringer, Goldspan Dragon, Hellkite Tyrant, Kura, the Boundless Sky, Kyodai, Soul of Kamigawa, Lathliss, Dragon Queen, Llanowar Elves, Miirym, Sentinel Wyrm, Morophon, the Boundless, Nicol Bolas, the Ravager, Niv-Mizzet, the Firemind, Old Gnawbone, Rivaz of the Claw, Scion of Draco, Scourge of Valkas, Shivan Dragon, Smaug the Impenetrable, Smaug the Magnificent, Spark Double, Summon: Bahamut, Terror of the Peaks, Thundermaw Hellkite, Tiamat, Tymna the Weaver, Utvara Hellkite

**Artifacts (11).** Arcane Signet, Mana Vault, Mithril Coat, Mox Jasper, Relic of Sauron, Sapphire Medallion, Sol Ring, Talisman of Curiosity, Talisman of Hierarchy, The One Ring, Urza's Incubator

**Enchantments (5).** Doubling Season, Dragon Tempest, Rhystic Study, Rhythm of the Wild, Sylvan Library

**Instants (8).** Chaos Warp, Fact or Fiction, Fierce Guardianship, Lightning Bolt, Path to Exile, Swords to Plowshares, Vampiric Tutor, Worldly Tutor

**Sorceries (7).** Crux of Fate, Demonic Tutor, Jeska's Will, Nature's Lore, Nibelheim Aflame, Three Visits, Twinflame

**Lands (33).** Ancient Tomb, Arid Mesa, Badlands, Bayou, Blood Crypt, Cavern of Souls, City of Brass, Command Tower, Exotic Orchard, Field of the Dead, Flooded Strand, Maelstrom of the Spirit Dragon, Marsh Flats, Minas Tirith, Overgrown Tomb, Path of Ancestry, Plateau, Polluted Delta, Sacred Foundry, Savannah, Scalding Tarn, Scrubland, Sokenzan, Crucible of Defiance, Steam Vents, Stomping Ground, Sulfurous Springs, Taiga, Tropical Island, Tundra, Underground Sea, Volcanic Island, Windswept Heath, Wooded Foothills

**Planeswalkers (1).** The Theorist, Jace Beleren

## Import list (100)

```
1 Ancient Tomb
1 Arcane Signet
1 Arid Mesa
1 Avacyn's Pilgrim
1 Badlands
1 Bayou
1 Birds of Paradise
1 Blood Crypt
1 Cavern of Souls
1 Chaos Warp
1 City of Brass
1 Command Tower
1 Crux of Fate
1 Delighted Halfling
1 Demonic Tutor
1 Doubling Season
1 Dragon Tempest
1 Dragonlord Dromoka
1 Dragonlord Ojutai
1 Dragonspeaker Shaman
1 Exotic Orchard
1 Fact or Fiction
1 Field of the Dead
1 Fierce Guardianship
1 Flooded Strand
1 Ganax, Astral Hunter
1 Garth One-Eye
1 Giver of Runes
1 Glorybringer
1 Goldspan Dragon
1 Hellkite Tyrant
1 Jeska's Will
1 Kura, the Boundless Sky
1 Kyodai, Soul of Kamigawa
1 Lathliss, Dragon Queen
1 Lightning Bolt
1 Llanowar Elves
1 Maelstrom of the Spirit Dragon
1 Mana Vault
1 Marsh Flats
1 Miirym, Sentinel Wyrm
1 Minas Tirith
1 Mithril Coat
1 Morophon, the Boundless
1 Mox Jasper
1 Nature's Lore
1 Nibelheim Aflame
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
1 Rhythm of the Wild
1 Rivaz of the Claw
1 Sacred Foundry
1 Sapphire Medallion
1 Savannah
1 Scalding Tarn
1 Scion of Draco
1 Scourge of Valkas
1 Scrubland
1 Shivan Dragon
1 Smaug the Impenetrable
1 Smaug the Magnificent
1 Sokenzan, Crucible of Defiance
1 Sol Ring
1 Spark Double
1 Steam Vents
1 Stomping Ground
1 Sulfurous Springs
1 Summon: Bahamut
1 Swords to Plowshares
1 Sylvan Library
1 Taiga
1 Talisman of Curiosity
1 Talisman of Hierarchy
1 Terror of the Peaks
1 The One Ring
1 The Theorist, Jace Beleren
1 The Ur-Dragon
1 Three Visits
1 Thundermaw Hellkite
1 Tiamat
1 Tropical Island
1 Tundra
1 Twinflame
1 Tymna the Weaver
1 Underground Sea
1 Urza's Incubator
1 Utvara Hellkite
1 Vampiric Tutor
1 Volcanic Island
1 Windswept Heath
1 Wooded Foothills
1 Worldly Tutor
```
