# Thanos, the Mad Titan — Artifact Combo (Cali Commander)

A fast-mana artifact deck for Cali Commander: one opponent, 30 life, and Mana Crypt, Tinker and Primeval Titan legal.
Thanos is a cheap 4/4 deathtouch lifelinker who makes Deflecting Swat free and gives Jeska's Will both modes, but the
deck is built to win in a few turns without him. It has five ways to end the game: **infinite colourless mana**
(Basalt Monolith + Rings of Brighthearth) into Walking Ballista, **infinite coloured mana** (Staff of Domination +
Faeburrow Elder) into the whole library and Thassa's Oracle, **Tinker into Blightsteel Colossus** (one hit is lethal),
**Thassa's Oracle with Demonic Consultation or Tainted Pact** (the singleton library exiles itself), and **Underworld
Breach with Brain Freeze** as a storm line into the Oracle. Sneak Attack is a second way to put Blightsteel onto the
battlefield. It is weak to Null Rod and Collector Ouphe effects, to stax that stops free spells and rocks, and to a
hand that holds the answers (counters and removal) with no engine. Nothing in this document has been simulated: the
simulator has no Cali mode yet, so the plan and the keep rule are a pilot's guide, not measured.

| | |
|---|---|
| **Commander** | Thanos, the Mad Titan ({R}{W}{B}, 4/4 deathtouch, lifelink); five colours (his power-up costs all five) |
| **Format** | Cali Commander: 1v1, 30 starting life, the player on the play skips their first draw, and the Cali ban list (below) |
| **Simulator** | Deck key `thanos`, played from `data/plans/thanos.toml`; findings in `audit/thanos/FINDINGS.md`, history in `decklists/CHANGELOG.md`. The engine has no 1v1, 30-life or Cali ban-list mode yet |
| **Buys** | None |

## Strategy

### The commander

**Thanos, the Mad Titan** ({R}{W}{B}, 4/4). *Deathtouch, lifelink. Power-up — {C}{W}{U}{B}{R}{G}: Put two +1/+1
counters on Thanos. Choose odd or even. Destroy each other creature with mana value of the chosen quality. (Activate
each power-up ability only once. Reduce the cost by his mana cost if he entered this turn. Zero is even.)*

**When to cast him.** Turn 2 off Birds of Paradise or two rocks, turn 3 otherwise. He needs exactly {R}{W}{B}: Sol Ring,
Mana Crypt, Mana Vault and Ancient Tomb can't pay for him, but Arcane Signet, Coalition Relic, Birds and the duals can.
He is the cheapest threat, a good blocker against the decks that attack, and he turns on two free spells:

- **Deflecting Swat** ({2}{R}, free while you control a commander): new targets for target spell or ability. Move
  removal off a combo piece or Blightsteel, or move a counterspell onto another spell on the stack.
- **Jeska's Will** ({2}{R}): with a commander out you get both modes: {R} for each card in the opponent's hand, *and*
  exile your top three cards and play them this turn.

Both are checked as you cast the spell. If Thanos dies, Swat costs full price again.

**The power-up** is a one-sided wipe in a deck that has few creatures of its own: six mana, or {C}{U}{G} the turn he
enters. {C} means colourless mana, not generic: Sol Ring, Mana Crypt, Mana Vault, Basalt Monolith, Mind Stone, The
Mightstone and Weakstone (for abilities), Thought Vessel, Ancient Tomb, Strip Mine and the painlands can pay it. Coloured-only sources (the duals, Command Tower, Signet, Birds) can't. It is an ability, so Force of Will
effects and Silence don't stop it. Rings of Brighthearth can copy it for {2}: choose odd for one and even for the other
and every other creature dies. In a 1v1 game it is mostly for clearing a blocker or a Dragon before a Blightsteel
attack, or when the opponent has built a board and you have no combo yet.

### How the deck wins

#### Route A: Basalt Monolith + Rings of Brighthearth (infinite {C})

- **Pieces:** Basalt Monolith ({3}, {T}: add {C}{C}{C}; {3}: untap) and Rings of Brighthearth ({3}; whenever you
  activate an ability that isn't a mana ability, you may pay {2} to copy it), both on the battlefield.
- **The loop:** tap the Monolith for {C}{C}{C}. Activate its untap ({3}) and pay {2} to copy it. The copy resolves
  first and untaps the Monolith: tap it again for three with the original still on the stack. The original then untaps
  it once more. That is {6} for {5}: one extra colourless mana per loop, as much as you like. The first loop needs five
  mana: the Monolith's own three and two more from elsewhere.
- **Finishers (all colourless):** **Walking Ballista** ({X}{X}) from hand with a huge X, then remove counters at the
  opponent (30 life needs X=30). If Ballista is still in the library, **Staff of Domination's** {5},{T}: draw a card,
  untapped for {1} each time, draws to it. **Urza, Lord High Artificer** ({5}: shuffle, then play the top card free)
  digs too, but a Ballista played free has X=0, so cast the Ballista you find with real mana.
- **Hold the pieces until both can land in one turn** with Force of Will, Force of Negation or Swat behind them. An
  opponent who destroys Rings or the Monolith in response ends the loop.

#### Route A2: Staff of Domination + Faeburrow Elder (infinite coloured mana)

- **Pieces:** Staff of Domination ({3}; {1}: untap it; {3},{T}: untap target creature; {5},{T}: draw a card) and
  Faeburrow Elder ({1}{G}{W}; taps for one mana of each colour among your permanents) with all five colours among
  your permanents. Thanos is R, W and B, the Elder is G and W, and one blue permanent (Urza, Mystic Remora or
  Thassa's Oracle) adds U. Artifacts and lands are colourless, so the rocks and lands don't count.
- **The loop:** tap the Elder for five coloured mana. Pay {3} and tap the Staff to untap the Elder. Pay {1} to untap
  the Staff. Four mana spent, five made: one extra coloured mana per loop. It takes five colours to be worth it, so
  check the colour count first. With infinite mana of every colour, draw the library with the Staff (stop at zero
  cards: drawing from an empty library loses) and cast Thassa's Oracle: an empty library wins. Or cast Walking
  Ballista for a huge X, or Blightsteel Colossus ({12}), whichever you hold or draw.

#### Route E: Tinker into Blightsteel Colossus

- **Pieces:** Tinker ({2}{U}, sorcery; as an additional cost sacrifice an artifact; search for an artifact and put it
  onto the battlefield, so any cheap rock is the sacrifice), Blightsteel Colossus (indestructible, infect, trample,
  11/11) and **Lightning Greaves** to give it haste and shroud ({0} equip).
- **The kill:** Tinker, Greaves, attack: 11 infect damage is 11 poison counters, lethal (ten) in one hit. Blightsteel
  can't be destroyed, and if it would go to the graveyard it is shuffled into the library instead, so Tinker can find
  it again. Exile effects (Swords to Plowshares) remove it for good.
- **It dodges artifact hate.** Null Rod and Collector Ouphe don't stop a creature, and Greaves' shroud stops targeted
  removal. The opponent's answers are a blocker (trample carries excess damage over, but ten of Blightsteel's eleven
  damage must reach them), a fog, a counterspell on Tinker, or a bounce or exile effect before the attack.
- **If Blightsteel is in your hand, Tinker can't find it** (it searches the library): cast it from hand, or use Sneak
  Attack.
- **Sneak Attack** ({3}{R}, then {R}): put a creature card from your hand onto the battlefield with haste. Blightsteel
  from hand, attack, and it is sacrificed at the end step and shuffles back. It needs {3}{R} up front and {R} each
  activation, so it is a backup when Blightsteel is drawn, and it also puts Elesh Norn, Meteor Golem (destroy a
  permanent) or Primeval Titan in free.

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

- **Thanos's power-up** clears the opponent's creatures and leaves a 6/6 deathtouch lifelinker.
- **Elesh Norn, Mother of Machines** ({4}{W}): your enter-the-battlefield triggers happen twice (Urza, Meteor Golem,
  Primeval Titan, Mightstone) and the opponent's don't happen at all.
- **Primeval Titan** ({4}{G}{G}): fetches Urza's Saga and Strip Mine, or any two lands, and attacks.
- **Windborn Muse** ({3}{W}): a 2/3 flier that taxes every attacker {2}, a good blocker against an aggressive deck.
- **Meteor Golem** ({7}): destroy any nonland permanent. **Hangarback Walker** ({X}{X}) is a mana sink that leaves
  fliers. **Walking Ballista** is a mana sink and a pinger. **Exsanguinate** ({X}{B}{B}) is a late finisher.

### The engines

- **The One Ring** ({4}): protection from everything until your next turn when you cast it, then {T}: a burden counter
  and that many cards. You lose 1 life per burden counter each upkeep, so draw with it two or three times, then let it
  sit or bounce it. 30 life pays for this more easily than 40 did.
- **Rhystic Study** (draw when the opponent casts a spell unless they pay {1}) and **Mystic Remora** (draw when the
  opponent casts a noncreature spell unless they pay {4}; cumulative upkeep). In a 1v1 game both watch one opponent.
- **Urza, Lord High Artificer** ({2}{U}{U}): a Construct token, then every untapped artifact taps for {U}. Pays for
  Thassa's Oracle, and his {5} ability plays the top card free.
- **Crucible of Worlds** ({3}): play lands from the graveyard: Strip Mine every turn, Urza's Saga again after it
  sacrifices itself.
- **Triple Triad** ({3}{R}{R}{R}) and **Fiery Emancipation** ({3}{R}{R}{R}) are six-mana red enchantments that come
  down on a big-mana turn, not before: Triple Triad lets you play your top card and any lower-cost exiled card free
  each upkeep, Emancipation triples damage from your sources (Thanos, Ballista).
- **Esper Sentinel** ({W}) draws you a card when the opponent casts their first noncreature spell each turn and doesn't
  pay {1}. **Mystic Confluence** ({3}{U}{U}) is a flexible counter, bounce or draw.

### Game plan

**Turns 1-2.** Land and fast mana: Mana Crypt, Sol Ring, Mana Vault, Lotus Petal, Dark Ritual, Arcane Signet, Mind
Stone, a Talisman, a dork. A turn-1 rock plus a turn-2 Thanos is the strongest start. On the play you skip your first
draw, so an opening seven with a tutor and two mana sources is better than a clunky seven with five lands.

**Turns 3-4.** Assemble a route. Count which routes your hand supports (A: Basalt + Rings; A2: Staff + Elder + five
colours; E: Tinker, a sacrifice rock, Greaves, and Blightsteel in the library; O: Oracle plus Consultation or Pact) and
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
| 7 cards | 2-4 lands, or 1 land and 2+ mana rocks or Moxen-style pieces (Crypt, Petal, Ritual); at least one coloured source; and an action card: a tutor, a route piece, an engine, or Thanos castable by turn 3 |
| 6 or 5 cards | 1-4 lands plus mana, and any tutor or route piece |
| 4 cards | Keep |

- **Bottom first:** lands past the fourth, the top end (Meteor Golem, Blightsteel Colossus if Tinker isn't in hand),
  then situational answers.
- **Blightsteel in the opening hand is a tutor that can't win by itself.** It needs {12} or a Sneak Attack, so treat it
  as a dead card unless you hold Sneak Attack or a rock-heavy hand.
- **Tutor hands are strong here:** the deck has Demonic, Vampiric, Grim, Enlightened, Mystical, Worldly, Gamble, Tinker,
  Diabolic Intent. Keep a hand with two mana sources and a tutor, even with one land.

### What to tutor for

| You hold | Tutor for | Then |
|---|---|---|
| Rings of Brighthearth | Basalt Monolith (Demonic Tutor, Enlightened Tutor, Tinker) | Loop, then Ballista or Staff |
| Basalt Monolith | Rings of Brighthearth (Enlightened Tutor, Demonic Tutor) | Same |
| Tinker, rock, Greaves | Blightsteel Colossus (Tinker finds it itself) | Tinker, equip, attack |
| Thassa's Oracle | Demonic Consultation or Tainted Pact (Demonic Tutor, Vampiric Tutor, Grim Tutor, Diabolic Intent) | Pact or Consultation, then Oracle |
| Consultation or Pact | Thassa's Oracle (a tutor puts it in hand; do not use Consultation to find it) | Cast it with library empty |
| Staff, Elder, five colours | Walking Ballista or Thassa's Oracle (Staff draws to it) | Loop and draw |
| Nothing yet, three or more mana | Tinker (Demonic Tutor), then Blightsteel | Route E |
| Opponent has a lot of hate | The route that doesn't need artifacts: Oracle with Consultation or Pact | Win with a creature spell |

Gamble discards a random card after it finds, so with Tinker or the piece you fetched in a small hand it can
discard what you wanted: use it with three or more spare cards, or as a one-shot for Tinker when you hold lands.
Mystical Tutor finds Silence, a counterspell or Brain Freeze; Worldly Tutor finds Blightsteel Colossus, Primeval Titan or
Walking Ballista; Enlightened Tutor finds any artifact or enchantment (Tinker's partner pieces, Staff, Rings,
Underworld Breach).

### Protecting the win turn

- **Free counters:** Force of Will, Force of Negation (free on the opponent's turn), then Counterspell, Dovin's Veto
  (can't be countered, for noncreature spells), An Offer You Can't Refuse (counter a noncreature spell for {U}; it gives
  the opponent two Treasures, so use it on the combo turn), Mystic Confluence, Silence (stops the opponent casting
  during your turn), Grand Abolisher (no spells on your turn), Teferi's Protection (phases out everything, ends in a
  stalled game), Heroic Intervention, Akroma's Will (with Thanos out, both modes: double strike, indestructible and protection
  from a colour for your creatures), Deflecting Swat (free with Thanos).
- **Order:** bait with the less important spell first. Cast Brain Freeze or a rock before the key spell, keep the free
  counter for the one that matters.
- **The Oracle line is the hardest to stop:** it is two cheap spells and no permanents to destroy. Silence or
  Grand Abolisher first, then Consultation, then the Oracle.

### Interaction

- **Spot removal:** Swords to Plowshares and Chaos Warp (both answer a creature or artifact blocking or attacking),
  Cyclonic Rift (overloaded at {6}: bounce all their nonland permanents, which is a reset and a clock in a 1v1 game),
  Meteor Golem and Thanos's power-up (creatures).
- **Hate pieces to expect:** Null Rod and Collector Ouphe hit the artifacts, Stony Silence, Pithing Needle and Cursed
  Totem hit abilities, Rule of Law and Eidolon of the Great Revel hit spells. Route E (a creature) and Route O (two spells and a
  creature) both dodge artifact hate.
- **Remove the hate piece before the combo:** Swords, Chaos Warp or Cyclonic Rift. Ephemerate saves a creature from
  removal and reuses an enter trigger.

### Matchups (Cali 1v1)

The matchup notes are expectations, not measured.

| Opponent type | What to do |
|---|---|
| Fast combo | Race it with Route E or Route O. Hold Force of Will and Silence. You are the beatdown on turn 3-4 |
| Stax (Null Rod, Collector Ouphe, Stony Silence, Cursed Totem) | Route O (spells and a creature) and Route E (Blightsteel is a creature). Remove the stax piece first if you must |
| Creature-heavy midrange | Thanos's power-up, then a Blightsteel attack or Ballista. Windborn Muse blocks |
| Control (counters and removal) | Bait with Brain Freeze and Mystic Confluence, then go with a free counter behind it. The One Ring protects you for a turn |

### After a stopped attempt

If a counterspell or removal ends the first try, count what is left before the second: Blightsteel shuffles back
into the library, so Tinker still finds it. The Oracle in the graveyard (not in the library) is gone, so check it
first. Keep a Force of Will for the retry, and rebuild mana with the rocks you did not sacrifice.

### Life management

You start at 30. Mana Crypt costs 3 life on a lost coin flip each upkeep (about 1.5 life per turn on average), The One
Ring costs 1 per burden counter, Talismans and painlands cost 1 per tap, Diabolic Intent is a creature sacrifice, and
Exsanguinate gains life for the X it costs. If you are on the draw against a fast deck, hold off on Mana Crypt and
Ring until you know the matchup. Thanos's lifelink and Exsanguinate pay some of it back.

### Common mistakes

- Casting Consultation or Pact before the Oracle is in your hand, or naming a card that is in the library.
- Casting Tinker with no artifact to sacrifice, or sacrificing the Greaves you needed.
- Tinkering Blightsteel into a board with a blocker you can't remove: the blocker absorbs the hit unless trample
  carries ten damage over, so clear it first.
- Playing the two Route A pieces into open mana and a hand of counters instead of waiting for both to resolve at once.
- Casting Thanos before an engine when the opponent has a sweeper and you hold the combo.
- Holding Underworld Breach for a perfect turn: it is sacrificed at the end step, so cast it only when the whole line
  fits in the turn.

## Mana

27 lands, 3 of them colourless (Ancient Tomb, Strip Mine, Urza's Saga). Coloured sources among the lands: blue 14,
black 12, red 12, white 11, green 10, counting Command Tower, Path of Ancestry, Exotic Orchard and Plaza of Heroes
(Plaza makes coloured mana only for legendary spells, so Thanos). The deck also has 5 mana dorks (Birds of Paradise,
Llanowar Elves, Paradise Druid, Avacyn's Pilgrim, Faeburrow Elder), 14 rocks and rituals (Mana Crypt, Sol Ring, Mana
Vault, Lotus Petal, Dark Ritual, Arcane Signet, Coalition Relic, Fellwar Stone, Mind Stone, Talisman of Hierarchy,
Talisman of Progress, Thought Vessel, The Mightstone and Weakstone, Basalt Monolith). Red and green are the thin
colours: only Thanos, Faeburrow Elder, Chaos Warp, Deflecting Swat, Jeska's Will, Gamble, Underworld Breach and Sneak
Attack need red, and only the green dorks, Faeburrow Elder and Primeval Titan need green, so a fetch-free mana base is
enough. The painlands (Adarkar Wastes, Battlefield Forge, Brushland, Caves of Koilos, Shivan Reef, Underground River,
Yavimaya Coast) and the shocklands (Blood Crypt, Hallowed Fountain, Overgrown Tomb, Steam Vents, Temple Garden, Watery
Grave) enter untapped on any turn; Spirebluff Canal is untapped on turns 1-3; Sunpetal Grove, Sulfur Falls and
Woodland Cemetery need a matching basic land type; Haunted Ridge and Stormcarved Coast need two other lands.

## Format and Rule 0

Cali Commander: 1v1 and 30 life. Banned: Ancestral Recall, Balance, Black Lotus, Channel, Erayo, Fastbond, Karakas, the
five Moxen (Emerald, Jet, Pearl, Ruby, Sapphire), Nadu, Time Vault, Time Walk, Tolarian Academy, Tabernacle, Yawgmoth's
Bargain. Mana Crypt, Tinker, Primeval Titan, Paradox Engine, Jeweled Lotus, Dockside Extortionist and Upheaval are
legal. The deck's two combos and its Oracle line are all infinite or library-win kills, so tell the opponent before the
game. Check with the organiser whether proxies are allowed.

## Decklist by type (100)

**Commander (1).** Thanos, the Mad Titan

**Creatures (17).** Avacyn's Pilgrim, Birds of Paradise, Blightsteel Colossus, Elesh Norn, Mother of Machines, Esper Sentinel, Faeburrow Elder, Grand Abolisher, Hangarback Walker, Llanowar Elves, Meteor Golem, Paradise Druid, Primeval Titan, Ragavan, Nimble Pilferer, Thassa's Oracle, Urza, Lord High Artificer, Walking Ballista, Windborn Muse

**Artifacts (18).** Arcane Signet, Basalt Monolith, Coalition Relic, Crucible of Worlds, Fellwar Stone, Lightning Greaves, Lotus Petal, Mana Crypt, Mana Vault, Mind Stone, Rings of Brighthearth, Sol Ring, Staff of Domination, Talisman of Hierarchy, Talisman of Progress, The Mightstone and Weakstone, The One Ring, Thought Vessel

**Enchantments (6).** Fiery Emancipation, Mystic Remora, Rhystic Study, Sneak Attack, Triple Triad, Underworld Breach

**Instants (23).** Akroma's Will, An Offer You Can't Refuse, Brain Freeze, Chaos Warp, Counterspell, Cyclonic Rift, Dark Ritual, Deflecting Swat, Demonic Consultation, Dovin's Veto, Enlightened Tutor, Ephemerate, Force of Negation, Force of Will, Heroic Intervention, Mystic Confluence, Mystical Tutor, Silence, Swords to Plowshares, Tainted Pact, Teferi's Protection, Vampiric Tutor, Worldly Tutor

**Sorceries (8).** Demonic Tutor, Diabolic Intent, Exsanguinate, Gamble, Grim Tutor, Jeska's Will, Nibelheim Aflame, Tinker

**Lands (27).** Adarkar Wastes, Ancient Tomb, Battlefield Forge, Blood Crypt, Brushland, Caves of Koilos, Command Tower, Exotic Orchard, Hallowed Fountain, Haunted Ridge, Overgrown Tomb, Path of Ancestry, Plaza of Heroes, Shivan Reef, Spirebluff Canal, Steam Vents, Stormcarved Coast, Strip Mine, Sulfur Falls, Sunpetal Grove, Temple Garden, Underground River, Urborg, Tomb of Yawgmoth, Urza's Saga, Watery Grave, Woodland Cemetery, Yavimaya Coast

## Import list (100)

```
1 Adarkar Wastes
1 Akroma's Will
1 An Offer You Can't Refuse
1 Ancient Tomb
1 Arcane Signet
1 Avacyn's Pilgrim
1 Basalt Monolith
1 Battlefield Forge
1 Birds of Paradise
1 Blightsteel Colossus
1 Blood Crypt
1 Brain Freeze
1 Brushland
1 Caves of Koilos
1 Chaos Warp
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
1 Elesh Norn, Mother of Machines
1 Enlightened Tutor
1 Ephemerate
1 Esper Sentinel
1 Exotic Orchard
1 Exsanguinate
1 Faeburrow Elder
1 Fellwar Stone
1 Fiery Emancipation
1 Force of Negation
1 Force of Will
1 Gamble
1 Grand Abolisher
1 Grim Tutor
1 Hallowed Fountain
1 Hangarback Walker
1 Haunted Ridge
1 Heroic Intervention
1 Jeska's Will
1 Lightning Greaves
1 Llanowar Elves
1 Lotus Petal
1 Mana Crypt
1 Mana Vault
1 Meteor Golem
1 Mind Stone
1 Mystic Confluence
1 Mystic Remora
1 Mystical Tutor
1 Nibelheim Aflame
1 Overgrown Tomb
1 Paradise Druid
1 Path of Ancestry
1 Plaza of Heroes
1 Primeval Titan
1 Ragavan, Nimble Pilferer
1 Rhystic Study
1 Rings of Brighthearth
1 Shivan Reef
1 Silence
1 Sneak Attack
1 Sol Ring
1 Spirebluff Canal
1 Staff of Domination
1 Steam Vents
1 Stormcarved Coast
1 Strip Mine
1 Sulfur Falls
1 Sunpetal Grove
1 Swords to Plowshares
1 Tainted Pact
1 Talisman of Hierarchy
1 Talisman of Progress
1 Teferi's Protection
1 Temple Garden
1 Thanos, the Mad Titan
1 Thassa's Oracle
1 The Mightstone and Weakstone
1 The One Ring
1 Thought Vessel
1 Tinker
1 Triple Triad
1 Underground River
1 Underworld Breach
1 Urborg, Tomb of Yawgmoth
1 Urza's Saga
1 Urza, Lord High Artificer
1 Vampiric Tutor
1 Walking Ballista
1 Watery Grave
1 Windborn Muse
1 Woodland Cemetery
1 Worldly Tutor
1 Yavimaya Coast
```
