# Arzakon — Artifact Combo (Cali Commander)

A fast-mana artifact deck for Cali Commander: one opponent, 30 life, and Mana Crypt, Tinker and Primeval Titan legal.
Arzakon is a five-colour planeswalker that can be your commander: it pings for 3, makes a Black Lotus token and can wheel
both hands, but the deck is built to win in a few turns without it. Karn Liberated is a seven-mana planeswalker in the
99 that exiles any permanent. The deck has five ways to end the game: **infinite colourless mana**
(Basalt Monolith + Rings of Brighthearth) into Walking Ballista, **infinite coloured mana** (Staff of Domination +
Faeburrow Elder, with Leyline of the Guildpact making every nonland permanent all five colours) into the whole library and
Thassa's Oracle or a Walking Ballista with a huge X, **Tinker into Blightsteel Colossus** (one hit is lethal),
**Thassa's Oracle with Demonic Consultation or Tainted Pact** (the singleton library exiles itself), and **Underworld
Breach with Brain Freeze** as a storm line into the Oracle. Sneak Attack is a second way to put Blightsteel onto the
battlefield. It is weak to Null Rod and Collector Ouphe effects, to stax that stops free spells and rocks, and to a
hand that holds the answers (counters and removal) with no engine. Nothing in this document has been simulated at the
Cali format, so the plan and the keep rule are a pilot's guide, not measured.

| | |
|---|---|
| **Commander** | Arzakon ({W}{U}{B}{R}{G}, a legendary planeswalker with loyalty 4; "can be your commander"); a five-colour card |
| **Format** | Cali Commander: 1v1, 30 starting life, the player on the play skips their first draw, and the Cali ban list (below) |
| **Simulator** | Deck key `arzakon`, played from `data/plans/arzakon.toml`; findings in `audit/arzakon/FINDINGS.md`, history in `decklists/CHANGELOG.md`. The engine plays Cali Commander (1v1, 30 life, the Cali ban list). Karn Liberated, Dockside Extortionist and Leyline of the Guildpact are modelled (Full); Arzakon is modelled (Full: a hand-written walker; its +2 can hit any other player, creature, planeswalker or battle), and its loyalty choices are provisional rules that nothing has tested. The AI never copies loyalty abilities with Rings of Brighthearth; a person at a real table can. This list has not been simulated at the Cali format |
| **Buys** | None. Arzakon, Karn, the Great Creator, Voltaic Key, Pithing Needle, Dockside Extortionist, Karn Liberated and Sunpearl Kirin are owned. Proxies: Leyline of the Guildpact, the original duals, the fetchlands, Mox Opal, Staff of Domination and Watery Grave |

## Strategy

### The commander

**Arzakon** ({W}{U}{B}{R}{G}, legendary planeswalker, loyalty 4). *+2: Arzakon deals 3 damage to any other target. -3:
Create a Black Lotus token. (It's a {0} artifact with "{T}, Sacrifice this token: Add three mana of any one color.")
-7: Each player shuffles their hand and graveyard into their library, then draws seven cards. Arzakon can be your
commander.*

**When to cast it.** It costs one mana of each colour and nothing else, so the colourless rocks (Sol Ring, Mana Crypt, Mana
Vault, Basalt Monolith, Ancient Tomb) pay nothing toward it. The sources that can pay a colour are the dual lands and
fetchlands, Command Tower, Arcane Signet, Coalition Relic, Birds of Paradise, Paradise Druid, Lotus Petal, Fellwar Stone (only a colour an
opponent's land can make), Chrome Mox (one colour), Mox Opal (with three artifacts) and Leyline of the Guildpact, which turns every land into every
basic land type so that each land taps for any colour. Expect it on turn 4 or 5, not turn 2 or 3. If it leaves the
battlefield you may return it to the command zone and cast it again for {2} more.

**What each ability is for.** One ability a turn, and the loyalty is the cost of using it.

- **+2: 3 damage to any other target.** Use it first. It kills Collector Ouphe, a mana dork, Ragavan or another creature
  with toughness 3 or less, or a planeswalker at 3 loyalty or less; with nothing worth killing, 3 to the face is a
  ten-turn clock at 30 life. It goes to 6 loyalty.
- **-3: a Black Lotus token** (a {0} artifact; {T}, sacrifice: three mana of one colour). Use it when three extra mana of
  one colour wins the turn or pays for a key spell, or when Mox Opal needs a third artifact. From 4 loyalty it leaves
  Arzakon at 1, so on the turn it lands it is a choice between the Lotus and keeping a walker that can survive an attack.
  The token is an artifact too, so it also gives Tezzeret, Cruel Captain a loyalty counter and can be Tinker's sacrifice.
- **-7: each player shuffles hand and graveyard into the library and draws seven.** It needs 7 loyalty, so two +2 first
  (4 to 8). Use it only when your hand is small and the table's is large, and when the table's draw punishers are not
  out: Narset, Sheoldred, the Apocalypse, Orcish Bowmasters and Notion Thief all turn a symmetric draw against you. Do
  not use it while your graveyard is the fuel for an Underworld Breach line.
- **Rings of Brighthearth copies loyalty abilities** (they are not mana abilities): for {2}, a copied -3 is two Lotus
  tokens and a copied +2 is 6 damage.

**The Black Lotus token and the ban list.** Black Lotus is banned in Cali Commander. The token Arzakon makes is not a card
in the deck, so it is legal, the same way Garth One-Eye's copy in the Ur-Dragon deck is. Tell the opponent before the game.

**What Arzakon does not do.** It deals no commander damage and has no flying clock (the damage is the +2 ping). It is a
4-loyalty planeswalker, so a creature or two can attack it down in a 1v1 game, and it does not turn on any card: no spell
in the 99 needs a commander on the battlefield.

**Protecting the combo.** Arzakon does not protect it: the protection is the free counters, Silence, Grand Abolisher, Teferi's
Protection and Heroic Intervention (see Protecting the win turn). The deck has no sweeper of its
own besides Cyclonic Rift (overloaded), so when the combo is stopped it falls back on the fair wins
below.

### How the deck wins

#### Route A: Basalt Monolith + Rings of Brighthearth (infinite {C})

- **Pieces:** Basalt Monolith ({3}, {T}: add {C}{C}{C}; {3}: untap) and Rings of Brighthearth ({3}; whenever you
  activate an ability that isn't a mana ability, you may pay {2} to copy it), both on the battlefield.
- **The loop:** tap the Monolith for {C}{C}{C}. Activate its untap ({3}) and pay {2} to copy it. The copy resolves
  first and untaps the Monolith: tap it again for three with the original still on the stack. The original then untaps
  it once more. That is {6} for {5}: one extra colourless mana per loop, as much as you like. The first loop needs five
  mana: the Monolith's own three and two more from elsewhere.
- **Sculpting Steel as a second Rings.** With the Monolith and one Rings out, Steel ({3}) enters as a copy of Rings.
  Each activation now triggers two Rings, so you may pay {2} for each copy: {7} spent, three taps for {9}, two extra
  mana per loop instead of one. You may pay for only one copy, so the first loop still starts at five mana. Steel
  does not find a missing piece: it needs one of the two already on the battlefield. With Rings in play and the
  Monolith stuck in the library, Steel is only a mana rock, so tutor for the Monolith instead.
- **Finishers (all colourless):** **Walking Ballista** ({X}{X}) from hand with a huge X, then remove counters at the
  opponent (30 life needs X=30). If Ballista is still in the library, **Staff of Domination's** {5},{T}: draw a card,
  untapped for {1} each time, draws to it. **Urza, Lord High Artificer** ({5}: shuffle, then exile the top card and play it free this turn)
  digs too, but a Ballista played free has X=0 and dies, and Urza cannot cast it for mana: use the Staff to draw the
  Ballista into your hand, and use Urza for the other pieces.
- **Hold the pieces until both can land in one turn** with Force of Will or Force of Negation behind them. An
  opponent who destroys Rings or the Monolith in response ends the loop.

#### Route A2: Staff of Domination + Faeburrow Elder (infinite coloured mana)

- **Pieces:** Staff of Domination ({3}; {1}: untap it; {3},{T}: untap target creature; {5},{T}: draw a card) and
  Faeburrow Elder ({1}{G}{W}; taps for one mana of each colour among your permanents) with all five colours among
  your permanents, and **Leyline of the Guildpact** ({G/W}{G/U}{B/G}{R/G}) to supply them: it makes every nonland
  permanent you control all colours, so with Leyline out the Elder alone taps for all five. Without Leyline, artifacts
  and lands are colourless, so the rocks and lands don't count (with it, every nonland permanent counts, rocks
  included). Without Leyline the permanents reach at most four colours (the Elder is
  green and white, Grand Abolisher or Sunpearl Kirin is white, Urza, Mystic Remora or Thassa's Oracle is blue, Ragavan, Sneak
  Attack or Underworld Breach is red) and nothing is black, so the loop makes no extra mana, unless Arzakon is on the
  battlefield: it is all five colours itself.
- **Casting Leyline:** each hybrid symbol needs coloured mana ({G}{G}{G}{G}, or {W}{U}{B}{R}, or a mix); the colourless
  rocks can't pay for it. It costs nothing when it is in your opening hand (you may begin with it on the battlefield),
  about one game in fourteen. Enlightened Tutor and Gifts Ungiven find it.
- **The loop:** tap the Elder for five coloured mana. Pay {3} and tap the Staff to untap the Elder. Pay {1} to untap
  the Staff. Four mana spent, five made: one extra coloured mana per loop. It takes five colours to be worth it, so
  check the colour count first. With infinite mana of every colour, draw the library with the Staff (stop at zero
  cards: drawing from an empty library loses) and cast Thassa's Oracle: an empty library wins. Or cast Walking
  Ballista for a huge X, or Blightsteel Colossus ({12}), whichever you hold or draw.
- **What else Leyline does:** your lands become every basic land type as well, so Ancient Tomb, Strip Mine, Urza's
  Saga and the Urza lands each tap for any one colour (that tap does not hurt you). It also makes your nonland permanents blue and
  red, so Pyroblast, Red Elemental Blast and Hydroblast can destroy any of them (your lands stay colourless); Heroic
  Intervention is the answer.

#### Route E: Tinker into Blightsteel Colossus

- **Pieces:** Tinker ({2}{U}, sorcery; as an additional cost sacrifice an artifact; search for an artifact and put it
  onto the battlefield, so any cheap rock is the sacrifice), Blightsteel Colossus (indestructible, infect, trample,
  11/11) and **Lightning Greaves** to give it haste and shroud ({0} equip).
- **The kill:** Tinker, Greaves, attack: 11 infect damage is 11 poison counters, lethal (ten) in one hit. Blightsteel
  can't be destroyed, and if it would go to the graveyard it is shuffled into the library instead, so Tinker can find
  it again. Exile effects (Swords to Plowshares) remove it for good.
- **It mostly dodges artifact hate.** Null Rod and Collector Ouphe don't stop a creature from attacking, but they stop
  Greaves' equip, so under them Blightsteel has no haste or shroud and attacks a turn later (Sneak Attack gives haste
  without equipping). Greaves' shroud stops targeted removal. The opponent's answers are a blocker (trample carries excess damage over, but ten of Blightsteel's eleven
  damage must reach them), a fog, a counterspell on Tinker, or a bounce or exile effect before the attack.
- **If Blightsteel is in your hand, Tinker can't find it** (it searches the library): cast it from hand, or use Sneak
  Attack.
- **Sneak Attack** ({3}{R}, then {R}): put a creature card from your hand onto the battlefield with haste. Blightsteel
  from hand, attack, and it is sacrificed at the end step and shuffles back. It needs {3}{R} up front and {R} each
  activation, so it is a backup when Blightsteel is drawn, and it also puts Meteor Golem (destroy a permanent) or
  Primeval Titan in free.
- **Sculpting Steel is a second Blightsteel.** Blightsteel Colossus is an artifact creature, so once it is on the
  battlefield, Steel ({3}) enters as a copy of it: another 11/11 indestructible infect trampler, summoning sick, with
  Lightning Greaves ({0} to move) as the way to give it haste. Use it when the first Blightsteel was blocked or fogged
  and the opponent is still alive, or to attack with two so that one blocker or one removal spell is not
  enough. It is insurance, not a way to win on the turn you Tinker: the first Blightsteel is already lethal alone.

#### Route O: Thassa's Oracle with Demonic Consultation or Tainted Pact

- **Pieces:** Thassa's Oracle ({U}{U}) and either Demonic Consultation ({B}) or Tainted Pact ({1}{B}).
- **The line:** with the Oracle in hand and {U}{U} open, cast Consultation and name a card that is not in your
  library. It exiles the top six, then reveals the rest of the library looking for the name and exiles it all, so you
  have no library. Cast the Oracle: its trigger sees that X (devotion to blue, at least two from the Oracle itself) is
  at least the zero cards left, and you win. Tainted Pact does the same: the deck is singleton, so no two exiled cards
  share a name, and you decline every card until the library is gone.
- **Cast the Oracle second.** Consultation or Pact first, so the library is empty when the Oracle's trigger counts it.
  If the Oracle isn't in hand yet, find it first with a tutor.
- **Underworld Breach line:** Breach ({1}{R}) gives every nonland card in your graveyard escape (its mana cost plus
  exiling three other cards). The deck's rituals and free spells (Lotus Petal, Dark Ritual, Brain Freeze) cast from the
  graveyard, and **Brain Freeze** with storm mills your own library; the Oracle then wins on the empty library. Breach
  is sacrificed at the beginning of the end step, so the whole line happens in one turn.

#### Fair wins

If the combo is stopped, the deck still has threats and a way to grind:

- **Arzakon** ({W}{U}{B}{R}{G}): the +2 is 3 damage every turn, a ten-turn clock at 30 life, and it clears Collector Ouphe,
  dorks and small blockers.
- **Primeval Titan** ({4}{G}{G}): fetches Urza's Saga and Strip Mine, or any two lands (two of the Urza lands, say), and attacks.
- **Meteor Golem** ({7}): destroy any nonland permanent. **Hangarback Walker** ({X}{X}) is a mana sink that leaves
  fliers. **Walking Ballista** is a mana sink and a pinger. **Exsanguinate** ({X}{B}{B}) is a late finisher.

### The engines

- **The One Ring** ({4}): protection from everything until your next turn when you cast it, then {T}: a burden counter
  and that many cards. You lose 1 life per burden counter at every upkeep whether or not you keep drawing, so draw with it two or
  three times, then stop: the counters stay and the drain continues. It is indestructible; Mystic Confluence can bounce
  it to reset the counters, and Teferi's Protection freezes your life total for a turn. At 30 life you have less room
  for this than at 40.
- **Mystic Remora** (draw when the opponent casts a noncreature spell unless they pay {4}; cumulative upkeep). In a 1v1
  game it watches one opponent.
- **Urza, Lord High Artificer** ({2}{U}{U}): a Construct token, then every untapped artifact taps for {U}. Pays for
  Thassa's Oracle, and his {5} ability plays the top card free.
- **Crucible of Worlds** ({3}): play lands from the graveyard: Strip Mine every turn, Urza's Saga again after it
  sacrifices itself.
- **Sculpting Steel** ({3}): enters as a copy of any artifact on the battlefield, yours or the opponent's, or stays a
  plain artifact if you decline. It is a mid-game card that makes the best artifact you already have a second time, not
  a route and not a tutor, and it does nothing on an empty board. What to copy, best first:
  1. **Rings of Brighthearth** in Route A (two Rings, two extra mana per loop; see Route A).
  2. **Blightsteel Colossus** after Tinker or Sneak Attack, as a second attacker (see Route E).
  3. **A colourless rock** (Sol Ring, Mana Vault, Basalt Monolith) when you need mana for a big turn.
     A second Monolith taps for {C}{C}{C} again but costs {3} to untap like the first.
  4. **An opponent's rock** (Sol Ring, Mana Vault, a Signet) when you have none of your own.
  It is also a spare artifact to sacrifice to Tinker and an extra {U} under Urza. **Do not copy** The One Ring or The
  Mightstone and Weakstone (legendary: the copy and the original cannot both stay), Walking Ballista or Hangarback
  Walker (they enter with no counters and die), Mox Opal (legendary), Chrome Mox (a copy may imprint its own card, but that costs a
  card for one mana), Mana Crypt (the extra coin flip costs about 1.5 life a turn more, so
  only when you win that turn), or an opponent's Null Rod.
- **Tezzeret, Cruel Captain** ({3}, loyalty 4): gets a loyalty counter whenever an artifact you control enters, which
  this deck does all the time. **0:** untap target artifact or creature (an artifact creature also gets a +1/+1
  counter): untap the Basalt Monolith for another {C}{C}{C}, or a rock you have spent; with Rings of Brighthearth, pay
  {2} to copy it and untap twice. **-3:** search for an artifact with mana value 1 or less and put it into your hand:
  Sol Ring, Mana Crypt, Mana Vault, Lotus Petal, Mox Opal, Chrome Mox, and Walking Ballista or Hangarback Walker (mana
  value 0 in the library). Use him to fetch the Ballista or a free rock. His -7 emblem (three +1/+1 counters on an
  artifact each combat) is rarely reached.
- **Mox Opal** ({0}, legendary artifact): taps for one mana of any colour while you control three or more artifacts, and
  it counts itself. It needs two other artifacts to work, which is possible on turn 1 with free rocks (Mana Crypt and Lotus Petal) and
  usual from turn 2 (the deck has 24 artifacts with the artifact creatures). **Chrome Mox** ({0}, a Game Changer): when it enters, exile a nonartifact, nonland card
  from your hand; it taps for one mana of any colour of that card. Imprint the card whose colour you are missing (a
  multicolour card makes it tap for any of its colours), not a combo piece; it costs a card, so it is best on a hand
  with a spare.
- **Dockside Extortionist** ({1}{R}, a 1/2 Goblin Pirate): when it enters, it creates X Treasure tokens, where X is the number of
  artifacts and enchantments the opponent controls. Cali has one opponent, so it makes 0 to 2 Treasures on turn 2 and more as
  that player's board grows; into an empty board it is a 1/2 body and nothing else, so hold it until the opponent has artifacts or
  enchantments to count. A Treasure is an artifact that taps (and sacrifices itself) for one mana of any colour, so the Treasures
  pay Arzakon's five colours, count toward Mox Opal's three artifacts, are fodder for Tinker, give Urza blue mana, and each one that
  enters adds a loyalty counter to Tezzeret. It needs a red source for its {R}, the deck's lightest colour. It is banned in
  Commander and legal in Cali.
- **Mystic Confluence** ({3}{U}{U}) is a flexible counter, bounce or draw.
- **Karn Liberated** ({7}, colourless legendary planeswalker, loyalty 6): **+4:** target player exiles a card from their
  hand. **-3:** exile target permanent. **-14:** restart the game. It is the deck's seven-mana answer to any permanent that
  is not a land, and it exiles rather than destroys, so it also beats an indestructible permanent or one that returns from the
  graveyard. At {7} it comes down after the mana is built (a Monolith or Mana Vault turn). From 6 loyalty the -3 exiles one
  permanent and leaves it at 3; the -14 needs two turns of +4 first (6 to 14) and leaves Karn at 0, so it is not a plan. It is a
  planeswalker, so Sneak Attack cannot put it onto the battlefield. Karn Liberated and Karn, the Great Creator have different
  names, so both can be on the battlefield.
- **Karn, the Great Creator** ({4}, loyalty 5): *activated abilities of artifacts your opponents control can't be
  activated*, mana abilities included, so an opponent's Sol Ring, Mana Vault or Basalt Monolith stops tapping and
  untapping while he is out. It is the deck's answer to an artifact combo, and it does nothing to a creature or enchantment
  combo. **+1:** until your next turn, up to one target noncreature artifact becomes an artifact creature with power and
  toughness equal to its mana value (a Basalt Monolith becomes a 3/3). **-2:** reveal an artifact card you own from
  outside the game or choose a face-up artifact card you own in exile, and put it into your hand. The deck has no
  sideboard, so the -2 only reaches an artifact that is in exile (one exiled by Swords to Plowshares or by an escape
  cost).
- **Voltaic Key** ({1}; {1}, {T}: untap target artifact): untaps a rock you have already used. The net is two mana off
  Mana Vault or Basalt Monolith ({C}{C}{C} for {1}; the Monolith's own untap costs {3}) and one off Sol Ring. It is a mana
  card, not a combo piece: Rings of Brighthearth and the Monolith loop without it.
- **Pithing Needle** ({1}): as it enters, name a card; the activated abilities of sources with that name can't be
  activated unless they are mana abilities. Name the engine the opponent is winning with: Yawgmoth, Thran Physician, a
  planeswalker, or an artifact or creature the combo runs through. It stops nothing static and no mana ability.

### Game plan

**Turns 1-2.** Land and fast mana: Mana Crypt, Sol Ring, Mana Vault, Lotus Petal, Dark Ritual, Chrome Mox, Mox Opal (with three
artifacts), Arcane Signet, Fellwar Stone, Talisman of Progress, a dork; Dockside Extortionist when the opponent already has an artifact or an enchantment.
Arzakon is not a turn-2 play (it needs all five colours). On the play you skip your first
draw, so an opening seven with a tutor and two mana sources is better than a clunky seven with five lands.

**Turns 3-4.** Assemble a route. Count which routes your hand supports (A: Basalt + Rings; A2: Staff + Elder + Leyline
of the Guildpact; E: Tinker, a sacrifice rock, Greaves, and Blightsteel in the library; O: Oracle plus Consultation or Pact) and
tutor for the missing piece. Keep a free counter (Force of Will, Force of Negation) or {U}{U} available.

**Turn 4-5.** Arzakon arrives once all five colours are available (Leyline of the Guildpact makes every land any colour).
Use the +2 the turn it lands unless the -3 wins the turn (see The commander). Go with protection in hand, or hold one more turn. A fast opponent's win on their own turn 4 is the
clock you are racing: do not slow-roll the combo into open mana and a hand of counterspells.

**Hold up or go?** Go when you have a route and a free counter behind it, or when you will lose next turn to a combo
you cannot stop. Hold when the opponent has Force of Will, Pact-style free counters and two cards in hand, and you
only have one route: bait with a cheaper spell (Brain Freeze, Mystic Confluence), then go.

### Mulligans

The first mulligan is free; after that it is a London mulligan (draw 7, bottom one card per mulligan after the free
one). The player on the play skips their first draw.

| Hand | Keep when |
|---|---|
| 7 cards | 2-4 lands, or 1 land and 2+ mana rocks or Moxen-style pieces (Crypt, Petal, Ritual); at least one coloured source; and an action card: a tutor, a route piece, an engine, or a way to cast Arzakon by turn 4 or 5 (all five colours reachable from lands and any-colour rocks; colourless rocks do not pay for it) |
| 6 or 5 cards | 1-4 lands plus mana, and any tutor or route piece |
| 4 cards | Keep |

- **Bottom first:** lands past the fourth, the top end (Meteor Golem, Blightsteel Colossus if Tinker isn't in hand),
  then situational answers.
- **Blightsteel in the opening hand is a tutor that can't win by itself.** It needs {12} or a Sneak Attack, so treat it
  as a dead card unless you hold Sneak Attack or a rock-heavy hand.
- **Tutor hands are strong here:** the deck has Demonic, Vampiric, Grim, Enlightened, Mystical, Worldly, Gamble, Tinker,
  Diabolic Intent and, at instant speed, Gifts Ungiven. Keep a hand with two mana sources and a tutor, even with one land.

### What to tutor for

| You hold | Tutor for | Then |
|---|---|---|
| Rings of Brighthearth | Basalt Monolith (Demonic Tutor, Enlightened Tutor, Tinker) | Loop, then Ballista or Staff |
| Basalt Monolith | Rings of Brighthearth (Enlightened Tutor, Demonic Tutor) | Same |
| Tinker, rock, Greaves | Blightsteel Colossus (Tinker finds it itself) | Tinker, equip, attack |
| Thassa's Oracle | Demonic Consultation or Tainted Pact (Demonic Tutor, Vampiric Tutor, Grim Tutor, Diabolic Intent) | Pact or Consultation, then Oracle |
| Consultation or Pact | Thassa's Oracle (a tutor puts it in hand; do not use Consultation to find it) | Cast it with library empty |
| Staff, Elder, Leyline | Walking Ballista or Thassa's Oracle (Staff draws to it) | Loop and draw |
| Staff and Elder, no Leyline | Leyline of the Guildpact (Enlightened Tutor, Gifts Ungiven) | Cast it, then loop |
| Nothing yet, three or more mana | Tinker (Demonic Tutor), then Blightsteel | Route E |
| Opponent has a lot of hate | The route that doesn't need artifacts: Oracle with Consultation or Pact | Win with a creature spell |

Gamble discards a random card after it finds, so with Tinker or the piece you fetched in a small hand it can
discard what you wanted: use it with three or more spare cards, or as a one-shot for Tinker when you hold lands.
Mystical Tutor finds Silence, a counterspell or Brain Freeze; Worldly Tutor finds Blightsteel Colossus, Primeval Titan or
Walking Ballista; Enlightened Tutor finds any artifact or enchantment (Tinker's partner pieces, Staff, Rings,
Underworld Breach).

**Gifts Ungiven** ({3}{U}, instant) finds four cards and the opponent chooses two of them for your graveyard, so a pile
has to work for every split. Each pile that works needs one piece already in hand, so Gifts adds redundancy at instant
speed, not a new route; cast it at the end of the opponent's turn:

- **Thassa's Oracle in hand:** Demonic Consultation, Tainted Pact, Underworld Breach and Demonic Tutor. Five of the six
  splits leave an exile half (Consultation or Pact) in your hand. In the sixth the opponent bins both, and you hold
  Breach and Demonic Tutor: cast Breach and escape Consultation ({B} plus exiling three other graveyard cards; Pact and
  Gifts are already there, so one more, such as Demonic Tutor once cast), 5 to 7 mana in one turn.
- **Underworld Breach in hand:** Thassa's Oracle and Demonic Consultation (with only these two found, both go to the
  graveyard, and Breach escapes them next turn; you need about four other cards in the graveyard besides Gifts for the two escapes).
- **Do not** ask for Tinker with Blightsteel Colossus, for the Oracle with Consultation and nothing in hand, or for the
  Basalt-route pieces with only two in hand (the opponent bins the Monolith, and no tutor finds it in the graveyard): the
  opponent gives you the two cards that don't win.

### Protecting the win turn

- **Sunpearl Kirin** ({1}{W}, a 2/1 flier with flash): when it enters, return up to one other target nonland permanent you
  control to its owner's hand; if it was a token, draw a card. In response to a spell or ability that targets one of your
  permanents (Rings of Brighthearth, Basalt Monolith, Staff of Domination, Dockside Extortionist), flash it in and return that
  permanent: the removal has no target left, and you cast the piece again later. It does nothing against a counterspell on a
  combo spell or a removal spell that does not target. It can also return Dockside Extortionist to cast it again for more
  Treasures, and returning a token (a Treasure, a Hangarback Walker's Thopter) draws a card. The AI has no rule that casts it
  in response to removal (it is not one of its protector cards); a person at a real table can.
- **Free counters:** Force of Will, Force of Negation (free on the opponent's turn), then Counterspell, Dovin's Veto
  (can't be countered, for noncreature spells), An Offer You Can't Refuse (counter a noncreature spell for {U}; it gives
  the opponent two Treasures, so use it on the combo turn), Mystic Confluence, Silence (stops the opponent casting
  during your turn), Grand Abolisher (no spells on your turn), Teferi's Protection (phases out everything, ends in a
  stalled game), and Heroic Intervention.
- **Order:** bait with the less important spell first. Cast Brain Freeze or a rock before the key spell, keep the free
  counter for the one that matters.
- **The Oracle line is the hardest to stop:** it is two cheap spells and no permanents to destroy. Silence or
  Grand Abolisher first, then Consultation, then the Oracle.

### Interaction

- **Spot removal:** Swords to Plowshares (a creature, including an artifact creature), Reality Shift ({1}{U} instant: exile
  target creature, and its controller manifests the top card of their library, so a face-down 2/2 takes its place) and Chaos Warp (any permanent),
  Cyclonic Rift (overloaded at {6}{U}: bounce all their nonland permanents, which is a reset and a clock in a 1v1 game),
  Meteor Golem (destroy any nonland permanent) and Karn Liberated (-3 exiles any permanent).
- **Hate pieces to expect:** Null Rod and Collector Ouphe hit the artifacts, Stony Silence, Pithing Needle and Cursed
  Totem hit abilities (a +2 from Arzakon kills Collector Ouphe), Rule of Law and Eidolon of the Great Revel hit spells. Route E (a creature, but without Greaves it has no haste) and Route O (two spells and a
  creature) mostly dodge artifact hate.
- **Remove the hate piece before the combo:** Chaos Warp or Cyclonic Rift (Swords or Reality Shift only if it is a creature, such as Collector
  Ouphe). Ephemerate saves a creature from
  removal and reuses an enter trigger.

### Matchups (Cali 1v1)

The matchup notes are expectations, not measured.

| Opponent type | What to do |
|---|---|
| Fast combo | Race it with Route E or Route O. Hold Force of Will and Silence. You are the beatdown on turn 3-4 |
| Stax (Null Rod, Collector Ouphe, Stony Silence, Cursed Totem) | Route O (spells and a creature) and Route E (Blightsteel is a creature, but Greaves' equip is stopped, so it attacks a turn later). Remove the stax piece first if you must |
| Creature-heavy midrange | Remove the blocker (Swords, Chaos Warp, Meteor Golem, Karn Liberated) or reset the board with Cyclonic Rift, then a Blightsteel attack or Ballista |
| Artifact combo (Urza, Kinnan) | Karn, the Great Creator stops every activated ability of their artifacts, mana abilities included; Pithing Needle names one card for the abilities that are not mana abilities |
| Control (counters and removal) | Bait with Brain Freeze and Mystic Confluence, then go with a free counter behind it. The One Ring protects you for a turn |

### After a stopped attempt

If a counterspell or removal ends the first try, count what is left before the second: Blightsteel shuffles back
into the library, so Tinker still finds it. An Oracle in the graveyard is out of reach except through Underworld Breach (escape {U}{U} plus three other cards), so check
it first. Keep a Force of Will for the retry, and rebuild mana with the rocks you did not sacrifice.

### Life management

You start at 30. Mana Crypt costs 3 life on a lost coin flip each upkeep (about 1.5 life per turn on average), The One
Ring costs 1 per burden counter, Ancient Tomb costs 2 per tap, each fetchland costs 1 (a fetched shockland costs 2
more life to enter untapped, so fetch an original), Talisman of Progress costs 1 per coloured tap, Diabolic Intent is a creature sacrifice, and
Exsanguinate gains life for the X it costs. If you are on the draw against a fast deck, hold off on Mana Crypt and
Ring until you know the matchup. Exsanguinate pays some of it back. The original duals and Command Tower cost
nothing to tap.

### Common mistakes

- Casting Consultation or Pact before the Oracle is in your hand, or naming a card that is in the library.
- Casting Tinker with no artifact to sacrifice, or sacrificing the Greaves you needed.
- Tinkering Blightsteel into a board with a blocker you can't remove: the blocker absorbs the hit unless trample
  carries ten damage over, so clear it first.
- Playing the two Route A pieces into open mana and a hand of counters instead of waiting for both to resolve at once.
- Using Arzakon's -3 with nothing for the three mana to do, or its -7 into Narset, Sheoldred, Orcish Bowmasters or Notion
  Thief, or when your hand is the larger one.
- Cracking a fetchland after Vampiric, Mystical, Enlightened or Worldly Tutor: the shuffle loses the card you put on top.
- Holding Underworld Breach for a perfect turn: it is sacrificed at the end step, so cast it only when the whole line
  fits in the turn.

## Mana

27 lands, 6 of them colourless (Ancient Tomb, Strip Mine, Urza's Saga and the three Urza lands, Urza's Tower, Urza's Mine
and Urza's Power Plant): Command Tower (any colour, painless), the ten
original duals (Tundra, Underground Sea, Volcanic Island, Tropical Island, Bayou, Badlands, Savannah, Scrubland,
Plateau, Taiga), seven fetchlands (Flooded Strand, Polluted Delta, Windswept Heath, Scalding Tarn,
Wooded Foothills, Marsh Flats, Arid Mesa) and three shocklands (Hallowed Fountain,
Steam Vents, Watery Grave). Coloured sources among the lands, counting each fetchland as any colour: blue 15, white 13,
black 13, red 13, green 12. The deck also has 5 mana dorks (Birds of Paradise, Llanowar Elves, Paradise Druid,
Avacyn's Pilgrim, Faeburrow Elder), 13 rocks and rituals (Mana Crypt, Sol Ring, Mana Vault, Lotus Petal, Dark Ritual,
Chrome Mox, Mox Opal, Arcane Signet, Coalition Relic, Fellwar Stone, Talisman of Progress,
The Mightstone and Weakstone, Basalt Monolith) and Voltaic Key to untap them, and Dockside Extortionist, which makes Treasures from the
opponent's artifacts and enchantments. Blue is the heaviest colour ({U}{U} for Counterspell, Thassa's
Oracle, Urza and Mystic Confluence), white needs {W}{W} for Grand Abolisher and one {W} for Silence, Sunpearl Kirin and several other spells, and green
matters most for the three one-mana dorks (Birds of Paradise, Llanowar Elves, Avacyn's Pilgrim). Red is the lightest: only Chaos Warp, Gamble,
Underworld Breach, Sneak Attack, Ragavan and Dockside Extortionist need it, besides Arzakon. Arzakon needs all five at once, which the colourless sources
(the six colourless lands and the colourless rocks) cannot help with.

**Fetchlands in this list.** The list has no basic lands, so a fetchland finds an original dual or a shockland by its
two land types. Every pair of colours has an original dual, so each fetchland works as a five-colour land for 1 life,
and there are 13 lands to find for 7 fetchlands. Fetch an original dual, not a shockland (a fetched shockland costs 2 more
life to enter untapped). Prismatic Vista would find nothing. Crucible of Worlds lets you replay a cracked fetchland from the graveyard as your land drop each turn, and each
crack thins the library a little before the Oracle line. Leyline of the Guildpact makes every land every basic type,
but it does not affect cards in the library, which keep their printed land types, so fetchlands still find only the
real duals and shocklands.

**The Urza lands.** Urza's Tower, Urza's Mine and Urza's Power Plant each tap for {C}; with all three on the battlefield
the Tower taps for {C}{C}{C} and the Mine and the Power-Plant for {C}{C} each, seven colourless mana from three lands.
Seven is exactly Karn Liberated or Meteor Golem, and most of Blightsteel Colossus's {12}. With one or two of them out each
is only a colourless land, and, without Leyline, none of them pays a colour toward Arzakon, so a hand with just the Urza lands as its
mana (and no Leyline) has no coloured source. Primeval Titan can fetch two of them at once.

## Format and Rule 0

Cali Commander: 1v1 and 30 life. Arzakon can be your commander (its own text). Banned: Ancestral Recall, Balance, Black Lotus, Channel, Erayo, Fastbond, Karakas, the
five Moxen (Emerald, Jet, Pearl, Ruby, Sapphire), Nadu, Time Vault, Time Walk, Tolarian Academy, Tabernacle, Yawgmoth's
Bargain. Mana Crypt, Tinker, Primeval Titan, Paradox Engine, Jeweled Lotus, Dockside Extortionist (banned in Commander) and Upheaval are
legal. The Black Lotus token Arzakon's -3 makes is not a card in the deck, so the ban does not apply to it. The deck's two combos and its Oracle line are all infinite or library-win kills, so tell the opponent before the
game. Check with the organiser whether proxies are allowed.

## Decklist by type (100)

**Commander (1).** Arzakon

**Creatures (16).** Avacyn's Pilgrim, Birds of Paradise, Blightsteel Colossus, Dockside Extortionist, Faeburrow Elder, Grand Abolisher, Hangarback Walker, Llanowar Elves, Meteor Golem, Paradise Druid, Primeval Titan, Ragavan, Nimble Pilferer, Sunpearl Kirin, Thassa's Oracle, Urza, Lord High Artificer, Walking Ballista

**Artifacts (20).** Arcane Signet, Basalt Monolith, Chrome Mox, Coalition Relic, Crucible of Worlds, Fellwar Stone, Lightning Greaves, Lotus Petal, Mana Crypt, Mana Vault, Mox Opal, Pithing Needle, Rings of Brighthearth, Sculpting Steel, Sol Ring, Staff of Domination, Talisman of Progress, The Mightstone and Weakstone, The One Ring, Voltaic Key

**Enchantments (4).** Leyline of the Guildpact, Mystic Remora, Sneak Attack, Underworld Breach

**Planeswalkers (3).** Karn Liberated, Karn, the Great Creator, Tezzeret, Cruel Captain

**Instants (23).** An Offer You Can't Refuse, Brain Freeze, Chaos Warp, Counterspell, Cyclonic Rift, Dark Ritual, Demonic Consultation, Dovin's Veto, Enlightened Tutor, Ephemerate, Force of Negation, Force of Will, Gifts Ungiven, Heroic Intervention, Mystic Confluence, Mystical Tutor, Reality Shift, Silence, Swords to Plowshares, Tainted Pact, Teferi's Protection, Vampiric Tutor, Worldly Tutor

**Sorceries (6).** Demonic Tutor, Diabolic Intent, Exsanguinate, Gamble, Grim Tutor, Tinker

**Lands (27).** Ancient Tomb, Arid Mesa, Badlands, Bayou, Command Tower, Flooded Strand, Hallowed Fountain, Marsh Flats, Plateau, Polluted Delta, Savannah, Scalding Tarn, Scrubland, Steam Vents, Strip Mine, Taiga, Tropical Island, Tundra, Underground Sea, Urza's Mine, Urza's Power Plant, Urza's Saga, Urza's Tower, Volcanic Island, Watery Grave, Windswept Heath, Wooded Foothills

## Import list (100)

```
1 An Offer You Can't Refuse
1 Ancient Tomb
1 Arcane Signet
1 Arid Mesa
1 Arzakon
1 Avacyn's Pilgrim
1 Badlands
1 Basalt Monolith
1 Bayou
1 Birds of Paradise
1 Blightsteel Colossus
1 Brain Freeze
1 Chaos Warp
1 Chrome Mox
1 Coalition Relic
1 Command Tower
1 Counterspell
1 Crucible of Worlds
1 Cyclonic Rift
1 Dark Ritual
1 Demonic Consultation
1 Demonic Tutor
1 Diabolic Intent
1 Dockside Extortionist
1 Dovin's Veto
1 Enlightened Tutor
1 Ephemerate
1 Exsanguinate
1 Faeburrow Elder
1 Fellwar Stone
1 Flooded Strand
1 Force of Negation
1 Force of Will
1 Gamble
1 Gifts Ungiven
1 Grand Abolisher
1 Grim Tutor
1 Hallowed Fountain
1 Hangarback Walker
1 Heroic Intervention
1 Karn Liberated
1 Karn, the Great Creator
1 Leyline of the Guildpact
1 Lightning Greaves
1 Llanowar Elves
1 Lotus Petal
1 Mana Crypt
1 Mana Vault
1 Marsh Flats
1 Meteor Golem
1 Mox Opal
1 Mystic Confluence
1 Mystic Remora
1 Mystical Tutor
1 Paradise Druid
1 Pithing Needle
1 Plateau
1 Polluted Delta
1 Primeval Titan
1 Ragavan, Nimble Pilferer
1 Reality Shift
1 Rings of Brighthearth
1 Savannah
1 Scalding Tarn
1 Scrubland
1 Sculpting Steel
1 Silence
1 Sneak Attack
1 Sol Ring
1 Staff of Domination
1 Steam Vents
1 Strip Mine
1 Sunpearl Kirin
1 Swords to Plowshares
1 Taiga
1 Tainted Pact
1 Talisman of Progress
1 Teferi's Protection
1 Tezzeret, Cruel Captain
1 Thassa's Oracle
1 The Mightstone and Weakstone
1 The One Ring
1 Tinker
1 Tropical Island
1 Tundra
1 Underground Sea
1 Underworld Breach
1 Urza's Mine
1 Urza's Power Plant
1 Urza's Saga
1 Urza's Tower
1 Urza, Lord High Artificer
1 Vampiric Tutor
1 Volcanic Island
1 Voltaic Key
1 Walking Ballista
1 Watery Grave
1 Windswept Heath
1 Wooded Foothills
1 Worldly Tutor
```
