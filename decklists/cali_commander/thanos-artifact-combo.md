# Kyodai, Soul of Kamigawa — Artifact Combo (Cali Commander)

A fast-mana artifact deck for Cali Commander: one opponent, 30 life, and Mana Crypt, Tinker and Primeval Titan legal.
Kyodai is a flash 3/3 flier that makes Deflecting Swat free, gives Jeska's Will and Akroma's Will both modes, and can
save a combo piece from a destroy effect, but the deck is built to win in a few turns without it. It has five ways to
end the game: **infinite colourless mana**
(Basalt Monolith + Rings of Brighthearth) into Walking Ballista, **infinite coloured mana** (Staff of Domination +
Faeburrow Elder, with Leyline of the Guildpact making every nonland permanent all five colours) into the whole library and
Thassa's Oracle or an unlimited Kyodai pump, **Tinker into Blightsteel Colossus** (one hit is lethal),
**Thassa's Oracle with Demonic Consultation or Tainted Pact** (the singleton library exiles itself), and **Underworld
Breach with Brain Freeze** as a storm line into the Oracle. Sneak Attack is a second way to put Blightsteel onto the
battlefield. It is weak to Null Rod and Collector Ouphe effects, to stax that stops free spells and rocks, and to a
hand that holds the answers (counters and removal) with no engine. Nothing in this document has been simulated: the
simulator has no Cali mode yet, so the plan and the keep rule are a pilot's guide, not measured.

| | |
|---|---|
| **Commander** | Kyodai, Soul of Kamigawa ({3}{W}, 3/3 flash flier; {W}{U}{B}{R}{G}: +5/+5); a white card with a five-colour identity |
| **Format** | Cali Commander: 1v1, 30 starting life, the player on the play skips their first draw, and the Cali ban list (below) |
| **Simulator** | Deck key `thanos`, played from `data/plans/thanos.toml`; findings in `audit/thanos/FINDINGS.md`, history in `decklists/CHANGELOG.md`. The engine has no 1v1, 30-life or Cali ban-list mode yet, Kyodai and Leyline of the Guildpact are modelled (Full) |
| **Buys** | None. Proxies: Leyline of the Guildpact, the original duals, the fetchlands, Mox Opal, Staff of Domination, Rhystic Study and Watery Grave |

## Strategy

### The commander

**Kyodai, Soul of Kamigawa** ({3}{W}, 3/3). *Flash, flying. When Kyodai enters, another target permanent gains
indestructible for as long as you control Kyodai. {W}{U}{B}{R}{G}: Kyodai gets +5/+5 until end of turn.*

**When to cast him.** Turn 2 off a fast rock, turn 4 off lands alone. He needs one white source and three more mana of
any kind, so Sol Ring, Mana Crypt, Mana Vault, Ancient Tomb and the other colourless rocks all help pay for him. He has
flash, so the best time is usually the opponent's end step, or in response to something. He turns on three spells:

- **Deflecting Swat** ({2}{R}, free while you control a commander): new targets for target spell or ability. Move
  removal off a combo piece or Blightsteel, or move a counterspell onto another spell on the stack. Flash him in
  response, then cast Swat for free in the same window.
- **Jeska's Will** ({2}{R}): with a commander out you get both modes: {R} for each card in the opponent's hand, *and*
  exile your top three cards and play them this turn.
- **Akroma's Will** ({3}{W}): with a commander out you get both modes: your creatures gain flying, vigilance and
  double strike, and lifelink, indestructible and protection from each colour.

All three are checked as you cast the spell. If Kyodai leaves the battlefield he can go back to the command zone and be
cast again for {2} more, and until then Swat costs full price.

**Protecting the combo.** His enter trigger makes another permanent indestructible while he stays on the battlefield.
Cast him in response to a destroy effect aimed at Rings of Brighthearth, Basalt Monolith or Staff of Domination. It does
nothing against exile, bounce, -X/-X, counterspells, Null Rod or Collector Ouphe, and it ends if Kyodai leaves.
Ephemerate on Kyodai lets him protect a new permanent when he returns, but the first one loses its protection.

**As a clock.** Each {W}{U}{B}{R}{G} gives him +5/+5. With infinite coloured mana (Route A2) the pump has no limit, so an
unblocked Kyodai is lethal. He has no trample, so a flying or reach blocker stops it. The deck has no sweeper of its
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
- **Hold the pieces until both can land in one turn** with Force of Will, Force of Negation or Swat behind them. An
  opponent who destroys Rings or the Monolith in response ends the loop.

#### Route A2: Staff of Domination + Faeburrow Elder (infinite coloured mana)

- **Pieces:** Staff of Domination ({3}; {1}: untap it; {3},{T}: untap target creature; {5},{T}: draw a card) and
  Faeburrow Elder ({1}{G}{W}; taps for one mana of each colour among your permanents) with all five colours among
  your permanents, and **Leyline of the Guildpact** ({G/W}{G/U}{B/G}{R/G}) to supply them: it makes every nonland
  permanent you control all colours, so with Leyline out the Elder alone taps for all five. Without Leyline, artifacts
  and lands are colourless, so the rocks and lands don't count (with it, every nonland permanent counts, rocks
  included). Without Leyline the permanents reach at most four colours (Kyodai is
  white, the Elder is green and white, Urza, Rhystic Study, Mystic Remora or Thassa's Oracle is blue, Ragavan, Sneak
  Attack or Underworld Breach is red) and nothing is black, so the loop makes no extra mana.
- **Casting Leyline:** each hybrid symbol needs coloured mana ({G}{G}{G}{G}, or {W}{U}{B}{R}, or a mix); the colourless
  rocks can't pay for it. It costs nothing when it is in your opening hand (you may begin with it on the battlefield),
  about one game in fourteen. Enlightened Tutor and Gifts Ungiven find it.
- **The loop:** tap the Elder for five coloured mana. Pay {3} and tap the Staff to untap the Elder. Pay {1} to untap
  the Staff. Four mana spent, five made: one extra coloured mana per loop. It takes five colours to be worth it, so
  check the colour count first. With infinite mana of every colour, draw the library with the Staff (stop at zero
  cards: drawing from an empty library loses) and cast Thassa's Oracle: an empty library wins. Or cast Walking
  Ballista for a huge X, or Blightsteel Colossus ({12}), whichever you hold or draw, or pump Kyodai if he is on the
  battlefield and unblocked.
- **What else Leyline does:** your lands become every basic land type as well, so Ancient Tomb, Strip Mine and Urza's
  Saga each tap for any one colour (that tap does not hurt you). It also makes your nonland permanents blue and
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

- **Kyodai** ({3}{W}): a flash 3/3 flier that blocks, attacks in the air and grows by +5/+5 for each {W}{U}{B}{R}{G}.
- **Primeval Titan** ({4}{G}{G}): fetches Urza's Saga and Strip Mine, or any two lands, and attacks.
- **Meteor Golem** ({7}): destroy any nonland permanent. **Hangarback Walker** ({X}{X}) is a mana sink that leaves
  fliers. **Walking Ballista** is a mana sink and a pinger. **Exsanguinate** ({X}{B}{B}) is a late finisher.

### The engines

- **The One Ring** ({4}): protection from everything until your next turn when you cast it, then {T}: a burden counter
  and that many cards. You lose 1 life per burden counter at every upkeep whether or not you keep drawing, so draw with it two or
  three times, then stop: the counters stay and the drain continues. It is indestructible; Mystic Confluence can bounce
  it to reset the counters, and Teferi's Protection freezes your life total for a turn. At 30 life you have less room
  for this than at 40.
- **Rhystic Study** (draw when the opponent casts a spell unless they pay {1}) and **Mystic Remora** (draw when the
  opponent casts a noncreature spell unless they pay {4}; cumulative upkeep). In a 1v1 game both watch one opponent.
- **Urza, Lord High Artificer** ({2}{U}{U}): a Construct token, then every untapped artifact taps for {U}. Pays for
  Thassa's Oracle, and his {5} ability plays the top card free.
- **Crucible of Worlds** ({3}): play lands from the graveyard: Strip Mine every turn, Urza's Saga again after it
  sacrifices itself.
- **Sculpting Steel** ({3}): enters as a copy of any artifact on the battlefield, yours or the opponent's, or stays a
  plain artifact if you decline. It is a mid-game card that makes the best artifact you already have a second time, not
  a route and not a tutor, and it does nothing on an empty board. What to copy, best first:
  1. **Rings of Brighthearth** in Route A (two Rings, two extra mana per loop; see Route A).
  2. **Blightsteel Colossus** after Tinker or Sneak Attack, as a second attacker (see Route E).
  3. **A colourless rock** (Sol Ring, Mana Vault, Basalt Monolith, Mind Stone) when you need mana for a big turn.
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
  Sol Ring, Mana Crypt, Mana Vault, Lotus Petal, Mox Opal, Chrome Mox, Esper Sentinel, and Walking Ballista or Hangarback Walker (mana
  value 0 in the library). Use him to fetch the Ballista or a free rock. His -7 emblem (three +1/+1 counters on an
  artifact each combat) is rarely reached.
- **Mox Opal** ({0}, legendary artifact): taps for one mana of any colour while you control three or more artifacts, and
  it counts itself. It needs two other artifacts to work, which is possible on turn 1 with free rocks (Mana Crypt and Lotus Petal) and
  usual from turn 2 (the deck has 25 artifacts with the artifact creatures). **Chrome Mox** ({0}, a Game Changer): when it enters, exile a nonartifact, nonland card
  from your hand; it taps for one mana of any colour of that card. Imprint the card whose colour you are missing (a
  multicolour card makes it tap for any of its colours), not a combo piece; it costs a card, so it is best on a hand
  with a spare.
- **Esper Sentinel** ({W}) draws you a card when the opponent casts their first noncreature spell each turn and doesn't
  pay {X}, where X is its power (1, or more with a +1/+1 counter from Tezzeret's 0). **Mystic Confluence** ({3}{U}{U}) is a flexible counter, bounce or draw.

### Game plan

**Turns 1-2.** Land and fast mana: Mana Crypt, Sol Ring, Mana Vault, Lotus Petal, Dark Ritual, Chrome Mox, Mox Opal (with three
artifacts), Arcane Signet, Mind Stone, a Talisman, a dork. A turn-1 Mana Crypt or Sol Ring plus a white source puts
Kyodai out on turn 2, or flashes him in at the end of the opponent's turn. On the play you skip your first
draw, so an opening seven with a tutor and two mana sources is better than a clunky seven with five lands.

**Turns 3-4.** Assemble a route. Count which routes your hand supports (A: Basalt + Rings; A2: Staff + Elder + Leyline
of the Guildpact; E: Tinker, a sacrifice rock, Greaves, and Blightsteel in the library; O: Oracle plus Consultation or Pact) and
tutor for the missing piece. Keep a free counter (Force of Will, Force of Negation) or {U}{U} available.

**Turn 4-5.** Go with protection in hand, or hold one more turn. A fast opponent's win on their own turn 4 is the
clock you are racing: do not slow-roll the combo into open mana and a hand of counterspells.

**Hold up or go?** Go when you have a route and a free counter behind it, or when you will lose next turn to a combo
you cannot stop. Hold when the opponent has Force of Will, Pact-style free counters and two cards in hand, and you
only have one route: bait with a cheaper spell (Brain Freeze, Mystic Confluence), then go.

### Mulligans

The first mulligan is free; after that it is a London mulligan (draw 7, bottom one card per mulligan after the free
one). The player on the play skips their first draw.

| Hand | Keep when |
|---|---|
| 7 cards | 2-4 lands, or 1 land and 2+ mana rocks or Moxen-style pieces (Crypt, Petal, Ritual); at least one coloured source; and an action card: a tutor, a route piece, an engine, or Kyodai castable by turn 3 (a white source and three more mana from lands and rocks) |
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

- **Free counters:** Force of Will, Force of Negation (free on the opponent's turn), then Counterspell, Dovin's Veto
  (can't be countered, for noncreature spells), An Offer You Can't Refuse (counter a noncreature spell for {U}; it gives
  the opponent two Treasures, so use it on the combo turn), Mystic Confluence, Silence (stops the opponent casting
  during your turn), Grand Abolisher (no spells on your turn), Teferi's Protection (phases out everything, ends in a
  stalled game), Heroic Intervention, Akroma's Will (with Kyodai out, both modes: flying, vigilance, double strike, lifelink,
  indestructible and protection from each colour for your creatures), Deflecting Swat (free with Kyodai), and Kyodai himself, flashed in in response to a
  destroy effect on a combo piece.
- **Order:** bait with the less important spell first. Cast Brain Freeze or a rock before the key spell, keep the free
  counter for the one that matters.
- **The Oracle line is the hardest to stop:** it is two cheap spells and no permanents to destroy. Silence or
  Grand Abolisher first, then Consultation, then the Oracle.

### Interaction

- **Spot removal:** Swords to Plowshares (a creature, including an artifact creature) and Chaos Warp (any permanent),
  Cyclonic Rift (overloaded at {6}{U}: bounce all their nonland permanents, which is a reset and a clock in a 1v1 game),
  and Meteor Golem (any nonland permanent).
- **Hate pieces to expect:** Null Rod and Collector Ouphe hit the artifacts, Stony Silence, Pithing Needle and Cursed
  Totem hit abilities, Rule of Law and Eidolon of the Great Revel hit spells. Route E (a creature, but without Greaves it has no haste) and Route O (two spells and a
  creature) mostly dodge artifact hate.
- **Remove the hate piece before the combo:** Chaos Warp or Cyclonic Rift (Swords only if it is a creature, such as Collector
  Ouphe). Ephemerate saves a creature from
  removal and reuses an enter trigger.

### Matchups (Cali 1v1)

The matchup notes are expectations, not measured.

| Opponent type | What to do |
|---|---|
| Fast combo | Race it with Route E or Route O. Hold Force of Will and Silence. You are the beatdown on turn 3-4 |
| Stax (Null Rod, Collector Ouphe, Stony Silence, Cursed Totem) | Route O (spells and a creature) and Route E (Blightsteel is a creature, but Greaves' equip is stopped, so it attacks a turn later). Remove the stax piece first if you must |
| Creature-heavy midrange | Remove the blocker (Swords, Chaos Warp, Meteor Golem) or reset the board with Cyclonic Rift, then a Blightsteel attack or Ballista. Kyodai flashes in to block |
| Control (counters and removal) | Bait with Brain Freeze and Mystic Confluence, then go with a free counter behind it. The One Ring protects you for a turn |

### After a stopped attempt

If a counterspell or removal ends the first try, count what is left before the second: Blightsteel shuffles back
into the library, so Tinker still finds it. An Oracle in the graveyard is out of reach except through Underworld Breach (escape {U}{U} plus three other cards), so check
it first. Keep a Force of Will for the retry, and rebuild mana with the rocks you did not sacrifice.

### Life management

You start at 30. Mana Crypt costs 3 life on a lost coin flip each upkeep (about 1.5 life per turn on average), The One
Ring costs 1 per burden counter, Ancient Tomb costs 2 per tap, each fetchland costs 1 (a fetched shockland costs 2
more life to enter untapped, so fetch an original), Talismans cost 1 per coloured tap, Diabolic Intent is a creature sacrifice, and
Exsanguinate gains life for the X it costs. If you are on the draw against a fast deck, hold off on Mana Crypt and
Ring until you know the matchup. Exsanguinate pays some of it back. The original duals and Command Tower cost
nothing to tap.

### Common mistakes

- Casting Consultation or Pact before the Oracle is in your hand, or naming a card that is in the library.
- Casting Tinker with no artifact to sacrifice, or sacrificing the Greaves you needed.
- Tinkering Blightsteel into a board with a blocker you can't remove: the blocker absorbs the hit unless trample
  carries ten damage over, so clear it first.
- Playing the two Route A pieces into open mana and a hand of counters instead of waiting for both to resolve at once.
- Casting Kyodai in your own main phase when you could flash him in at the opponent's end step or in response to a
  destroy effect on a combo piece.
- Cracking a fetchland after Vampiric, Mystical, Enlightened or Worldly Tutor: the shuffle loses the card you put on top.
- Holding Underworld Breach for a perfect turn: it is sacrificed at the end step, so cast it only when the whole line
  fits in the turn.

## Mana

27 lands, 3 of them colourless (Ancient Tomb, Strip Mine, Urza's Saga): Command Tower (any colour, painless), the ten
original duals (Tundra, Underground Sea, Volcanic Island, Tropical Island, Bayou, Badlands, Savannah, Scrubland,
Plateau, Taiga), ten fetchlands (Flooded Strand, Polluted Delta, Windswept Heath, Verdant Catacombs, Scalding Tarn,
Wooded Foothills, Bloodstained Mire, Marsh Flats, Arid Mesa, Misty Rainforest) and three shocklands (Hallowed Fountain,
Steam Vents, Watery Grave). Coloured sources among the lands, counting each fetchland as any colour: blue 18, white 16,
black 16, red 16, green 15. The deck also has 5 mana dorks (Birds of Paradise, Llanowar Elves, Paradise Druid,
Avacyn's Pilgrim, Faeburrow Elder), 15 rocks and rituals (Mana Crypt, Sol Ring, Mana Vault, Lotus Petal, Dark Ritual,
Chrome Mox, Mox Opal, Arcane Signet, Coalition Relic, Fellwar Stone, Mind Stone, Talisman of Hierarchy, Talisman of
Progress, The Mightstone and Weakstone, Basalt Monolith). Blue is the heaviest colour ({U}{U} for Counterspell, Thassa's
Oracle, Urza and Mystic Confluence), white needs {W}{W} for Grand Abolisher and one {W} for Kyodai, Silence and several other spells, and green
matters most for the three one-mana dorks (Birds of Paradise, Llanowar Elves, Avacyn's Pilgrim). Red is the lightest: only Chaos Warp, Deflecting Swat, Jeska's Will, Gamble,
Underworld Breach, Sneak Attack and Ragavan need it.

**Fetchlands in this list.** The list has no basic lands, so a fetchland finds an original dual or a shockland by its
two land types. Every pair of colours has an original dual, so each fetchland works as a five-colour land for 1 life,
and there are 13 lands to find for 10 fetchlands. Fetch an original dual, not a shockland (a fetched shockland costs 2 more
life to enter untapped). Prismatic Vista would find nothing. Crucible of Worlds lets you replay a cracked fetchland from the graveyard as your land drop each turn, and each
crack thins the library a little before the Oracle line. Leyline of the Guildpact makes every land every basic type,
but it does not affect cards in the library, which keep their printed land types, so fetchlands still find only the
real duals and shocklands.

## Format and Rule 0

Cali Commander: 1v1 and 30 life. Banned: Ancestral Recall, Balance, Black Lotus, Channel, Erayo, Fastbond, Karakas, the
five Moxen (Emerald, Jet, Pearl, Ruby, Sapphire), Nadu, Time Vault, Time Walk, Tolarian Academy, Tabernacle, Yawgmoth's
Bargain. Mana Crypt, Tinker, Primeval Titan, Paradox Engine, Jeweled Lotus, Dockside Extortionist and Upheaval are
legal. The deck's two combos and its Oracle line are all infinite or library-win kills, so tell the opponent before the
game. Check with the organiser whether proxies are allowed.

## Decklist by type (100)

**Commander (1).** Kyodai, Soul of Kamigawa

**Creatures (15).** Avacyn's Pilgrim, Birds of Paradise, Blightsteel Colossus, Esper Sentinel, Faeburrow Elder, Grand Abolisher, Hangarback Walker, Llanowar Elves, Meteor Golem, Paradise Druid, Primeval Titan, Ragavan, Nimble Pilferer, Thassa's Oracle, Urza, Lord High Artificer, Walking Ballista

**Artifacts (20).** Arcane Signet, Basalt Monolith, Chrome Mox, Coalition Relic, Crucible of Worlds, Fellwar Stone, Lightning Greaves, Lotus Petal, Mana Crypt, Mana Vault, Mind Stone, Mox Opal, Rings of Brighthearth, Sculpting Steel, Sol Ring, Staff of Domination, Talisman of Hierarchy, Talisman of Progress, The Mightstone and Weakstone, The One Ring

**Enchantments (5).** Leyline of the Guildpact, Mystic Remora, Rhystic Study, Sneak Attack, Underworld Breach

**Planeswalkers (1).** Tezzeret, Cruel Captain

**Instants (24).** Akroma's Will, An Offer You Can't Refuse, Brain Freeze, Chaos Warp, Counterspell, Cyclonic Rift, Dark Ritual, Deflecting Swat, Demonic Consultation, Dovin's Veto, Enlightened Tutor, Ephemerate, Force of Negation, Force of Will, Gifts Ungiven, Heroic Intervention, Mystic Confluence, Mystical Tutor, Silence, Swords to Plowshares, Tainted Pact, Teferi's Protection, Vampiric Tutor, Worldly Tutor

**Sorceries (7).** Demonic Tutor, Diabolic Intent, Exsanguinate, Gamble, Grim Tutor, Jeska's Will, Tinker

**Lands (27).** Ancient Tomb, Arid Mesa, Badlands, Bayou, Bloodstained Mire, Command Tower, Flooded Strand, Hallowed Fountain, Marsh Flats, Misty Rainforest, Plateau, Polluted Delta, Savannah, Scalding Tarn, Scrubland, Steam Vents, Strip Mine, Taiga, Tropical Island, Tundra, Underground Sea, Urza's Saga, Verdant Catacombs, Volcanic Island, Watery Grave, Windswept Heath, Wooded Foothills

## Import list (100)

```
1 Akroma's Will
1 An Offer You Can't Refuse
1 Ancient Tomb
1 Arcane Signet
1 Arid Mesa
1 Avacyn's Pilgrim
1 Badlands
1 Basalt Monolith
1 Bayou
1 Birds of Paradise
1 Blightsteel Colossus
1 Bloodstained Mire
1 Brain Freeze
1 Chaos Warp
1 Chrome Mox
1 Coalition Relic
1 Command Tower
1 Counterspell
1 Crucible of Worlds
1 Cyclonic Rift
1 Dark Ritual
1 Deflecting Swat
1 Demonic Consultation
1 Demonic Tutor
1 Diabolic Intent
1 Dovin's Veto
1 Enlightened Tutor
1 Ephemerate
1 Esper Sentinel
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
1 Jeska's Will
1 Kyodai, Soul of Kamigawa
1 Leyline of the Guildpact
1 Lightning Greaves
1 Llanowar Elves
1 Lotus Petal
1 Mana Crypt
1 Mana Vault
1 Marsh Flats
1 Meteor Golem
1 Mind Stone
1 Misty Rainforest
1 Mox Opal
1 Mystic Confluence
1 Mystic Remora
1 Mystical Tutor
1 Paradise Druid
1 Plateau
1 Polluted Delta
1 Primeval Titan
1 Ragavan, Nimble Pilferer
1 Rhystic Study
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
1 Swords to Plowshares
1 Taiga
1 Tainted Pact
1 Talisman of Hierarchy
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
1 Urza's Saga
1 Urza, Lord High Artificer
1 Vampiric Tutor
1 Verdant Catacombs
1 Volcanic Island
1 Walking Ballista
1 Watery Grave
1 Windswept Heath
1 Wooded Foothills
1 Worldly Tutor
```
