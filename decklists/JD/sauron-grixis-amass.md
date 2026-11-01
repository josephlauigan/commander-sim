# Sauron, the Dark Lord — Grixis Amass

A Grixis (blue-black-red) deck that lets the whole table feed one enormous Orc Army. Sauron amasses every time an
opponent casts a spell, the deck multiplies and protects the counters, and a Grixis control shell of one-sided
removal and counters keeps the Army alive. The Army connects, the Ring tempts you, and you refill your hand;
Exsanguinate and Comet Storm finish the table. It is weak to a board of big creatures that outclasses the Army, to
spot removal on the Army before it grows, and to fast decks that kill before Sauron lands.

| | |
|---|---|
| **Commander** | Sauron, the Dark Lord ({3}{U}{B}{R}, 7/6, Grixis) |
| **Bracket** | 3 (3 Game Changers: Cyclonic Rift, Orcish Bowmasters, Force of Will) |
| **Simulator** | deck key `sauron`; results in `decklists/pool/pool-results.md`, history in `decklists/CHANGELOG.md` |

## Strategy

### The commander
Sauron does the work for free. He is a 7/6 with ward (sacrifice a legendary artifact or legendary creature, which most
opponents simply cannot pay), and:

- Whenever an opponent casts a spell, amass Orcs 1.
- Whenever an Army you control deals combat damage to a player, the Ring tempts you.
- Whenever the Ring tempts you, you may discard your hand and draw four cards.

Amass stacks. If you control no Army it makes a single 0/0 Orc Army token, then every later amass piles more +1/+1
counters onto that same creature. Three opponents casting two spells a turn means six counters a turn cycle that cost
you nothing. The deck is built around one big Army, so protect that single creature and do not spread counters
around.

### Game plan
- **Turns 1 to 3.** Start an Army before Sauron lands: **Dreadhorde Invasion** and **Lazotep Reaver** make one, and a
  rock (eleven in the deck) pushes you toward six mana. Hold up **Orcish Bowmasters** (flash) for an opponent's
  extra draw, and keep cheap removal for early threats. Cast **Call of the Ring** when you have nothing better.
- **Turns 4 to 6.** Land Sauron, then The Ozolith, the Army's insurance, before you commit a big Army. Every
  opponent spell after Sauron lands is a free counter. Develop Talrand, Iron Man, Mauhúr and Sheoldred, and keep
  counters up.
- **Late game.** The Army connects, the Ring tempts you, and you refill. Use the sweepers and X spells to clear the
  way and finish.

### Grow it
Orcish Bowmasters adds counters on its own and again on every extra opponent draw. Iron Man adds a counter every time
you draw. Vision grows on off-turn casts. **Barad-dûr** is a late-game mana sink: it taps for black, and for
{X}{X}{B} it amasses Orcs X whenever a creature has died that turn, which in a deck with Orcish Bowmasters, the
spot removal, and Massacre Wurm, Ral, Inferno Titan and Comet Storm killing opposing creatures is almost always.

### Start it early
**Dreadhorde Invasion** ({1}{B} enchantment) loses you 1 life and amasses Zombies 1 every upkeep, and **Lazotep
Reaver** ({1}{B}) amasses Zombies 1 when it enters. Both put counters on an Army you control (it is also a Zombie) or
make one if you have none, so the Army exists before Sauron lands and every later amass piles onto it. A Zombie token
Army with power 6 or greater gains lifelink when it attacks. The early Army also makes the first hit come about a
turn sooner.

### Jace's Archivist and Bowmasters
**Jace's Archivist** turns Bowmasters into an Army engine. {U}, {T}: each player discards their hand and draws that
many (the greatest number anyone discarded). Every card an opponent draws off it triggers Orcish Bowmasters: 1 damage
and amass Orcs 1. Three opponents redrawing five cards each is about fifteen pings and fifteen counters, thirty with
Mauhúr out. Iron Man adds a counter for each card *you* draw, and refilled opponents cast more spells into Sauron and
Kaervek. Without Bowmasters it's just a symmetrical wheel that refuels the table, so hold the activation until a
payoff is on board.

**Sheoldred, the Apocalypse** ({2}{B}{B}, 4/5 deathtouch) gains you 2 life whenever you draw and costs each opponent
2 life whenever they draw. With Jace's Archivist's wheel and Orcish Bowmasters every opponent draw now costs life:
three opponents redrawing five cards each lose 10 life apiece to Sheoldred on top of the Bowmasters pings, and your
own draws (Iron Man, Night's Whisper, Fact or Fiction) pay you back.

### Multiply it
**Mauhúr, Uruk-hai Captain** is a Hardened Scales for your Army:
every counter placement on an Army, Orc, or Goblin you control gets one more.

### Fliers and bodies
**Talrand, Sky Summoner** makes a 2/2 flying Drake whenever you cast an instant or sorcery. The deck runs
twenty-seven of them, so Talrand turns counterspells, removal and draw spells into a second army of fliers: blockers
against the creature decks that cause most losses, and attackers that don't depend on the Orc Army. **Iron Man**
({3}{U}, flying) also gives other attacking modified creatures (a creature with a +1/+1 counter, so your Army)
flying, and **War Machine** gives modified attackers double strike. **Vision, Synthezoid Avenger** is a colourless
flier that grows or phases out whenever a spell is cast outside its caster's own turn. **Scarlet Witch, Chaotic Avenger**
flies and, when she connects, lets you cast a noncreature spell (or a Hero) from the top two cards for free.
**Sauron, the Necromancer** (menace) attacks with a free, temporary 3/3 Wraith copy of a creature card in your
graveyard (exiled at the next end step unless he is your Ring-bearer). A copy of Grave Titan, Massacre Wurm, Inferno
Titan or Noxious Gearhulk brings its enter trigger with it. **Grave Titan** ({4}{B}{B}, deathtouch) makes two 2/2
Zombies when it enters and attacks, which is also food for Diabolic Intent.

### Keep it
**The Ozolith** is insurance for the one-creature plan. Whenever a creature you control leaves the battlefield with
counters on it, those counters move onto The Ozolith, and at the beginning of combat on your turn you can put them all
onto a target creature. A spot-removed, bounced, or swept Army no longer resets you to zero: the next amass makes a
fresh Army and The Ozolith hands it the old pile (or put it straight onto Sauron). It also catches the counters if one
of your own sweeps (Nibelheim Aflame, Chandra's Ignition) takes a counter-carrying creature with it.

### Get it through
Whispersilk Cloak (can't be blocked, shroud) and Rogue's Passage ({4}, {T}: target creature can't be blocked) push the
Army through. Shroud also stops your own targeted abilities, so equip the Cloak when you do not need The Ozolith or
Iron Man to target it that turn. **Lightning Greaves** gives haste and shroud for free, which lets a freshly cast
Grave Titan or Inferno Titan attack and protects a key creature from spot removal (with the same caveat).
Every connection tempts the Ring: **Call of the Ring** tempts you every upkeep, and **Ringsight**
tempts on the way to a tutor, so with Sauron out each can mean discarding your hand for four new cards.

### Clear the table, keep the Army
Every sweeper here is one-sided or Army-friendly. **Massacre Wurm** ({3}{B}{B}{B}) gives opponents' creatures −2/−2
when it enters, and each opponent loses 2 life whenever one of their creatures dies. **Ral and the Implicit Maze**
({3}{R}{R} Saga) deals 2 damage to each creature and planeswalker your opponents control (chapter I), lets you discard
a card to exile the top two and play them through your next turn (II), and makes a Spellgorger Weird (III). **Inferno
Titan** deals 3 damage divided among up to three targets when it enters and whenever it attacks. **Chandra's
Ignition** ({3}{R}{R}) has a creature you control, the Army, deal damage equal to its power to each other creature and
each opponent: it also hits your own other creatures, like Nibelheim Aflame, so cast it when the Army is the only body
that matters. **Nibelheim Aflame** ({2}{R}{R}) does the same to creatures only and, from the graveyard with flashback
({5}{R}{R}), also discards your hand for four new cards. **Comet Storm** ({X}{R}{R} instant, multikicker {1}) deals X to
any target and to one more for each kick: removal at instant speed or a spread-out finisher. **Cyclonic Rift** bounces
one nonland permanent, or, overloaded for {6}{U}, every nonland permanent you don't control, at instant
speed. **Vandalblast** ({R}) destroys an artifact you don't control, or each of them for {4}{R}.
**Noxious Gearhulk** destroys another creature on entering and gains you life equal to its toughness. **Baleful
Mastery** ({3}{B}, instant) exiles a creature or planeswalker; pay {1}{B} instead and an opponent draws a card, which
Orcish Bowmasters pings and grows the Army for and Sheoldred charges 2 life for.

### Finishers
**Exsanguinate** ({X}{B}{B} sorcery) makes each opponent lose X life and you gain that much: with Sauron's mana and
rocks, it finishes a table that the Army has already thinned or that is racing you. **Torment of Hailfire**
({X}{B}{B}) repeats X times: each opponent loses 3 life unless they sacrifice a nonland permanent or discard a card. A
big X either kills or strips their boards and hands, and it needs no combat, so it wins when the Army is walled or
answered.

**Two punishers on the same trigger.** Sauron amasses and **Kaervek the Merciless** throws a burn or removal bolt
equal to the spell's mana value: both fire off the same event, an opponent casting a spell.

**Sephiroth, Planet's Heir** ({4}{U}{B}, 4/4 vigilance): when he enters, opponents' creatures get −2/−2 until end of
turn, which clears tokens, mana creatures and small blockers in front of the Army, and he gets a +1/+1 counter
whenever a creature an opponent controls dies, so the turn he lands he is usually a 6/6 or bigger. Vigilance lets
him attack and still guard Sauron.

### Card flow, mana and interaction
Card draw: **Fact or Fiction** (an instant-speed dig: cast it at the end of an opponent's turn for two of the five
cards), **Night's Whisper**, **Bloodsoaked Insight // Sanguine Morass** (an exile-and-play spell that is cheaper for
each life your opponents have lost this turn, or a land that taps for {B} or {R}) and Iron Man.
**Faerie Mastermind** ({1}{U}, flash flier) draws you a card whenever an opponent draws their second card in a turn:
the Archivist wheel, Bowmasters' forced draws and Baleful Mastery's cheap mode all trigger it.
Rocks: Sol Ring, Arcane Signet, Mind Stone, Fellwar Stone, Dimir Signet, Rakdos Signet,
Talismans of Creativity, Dominance and Indulgence, and Chromatic Lantern. **Urza, Lord High Artificer** brings a
Construct that grows with every artifact you control and taps your non-mana artifacts (The Ozolith, Lightning Greaves,
Whispersilk Cloak, the Construct itself) for {U}; the rocks already make mana, so he adds little there. Late, {5}
exiles the top card of your library, which you may play free this turn. Spot removal: **Bitter Triumph**,
**Bloodchief's Thirst**, **Baleful Mastery**, **Infernal Grasp**, **Terminate** and **Chaos
Warp**. Counters: **Mana Drain** (the countered spell's mana value comes back as {C} on your next main phase: put it
toward Sauron, a Titan or a big X spell), **Counterspell**, **An Offer You Can't Refuse** ({U}: counter a noncreature spell; its controller gets two Treasures, which
help them cast more spells, and every spell they cast grows the Army),
**Mystic Confluence** (three modes in any mix: counter unless they pay {3}, bounce a creature, draw a card) and **Force of
Will** (counter anything: exile a blue card from hand and pay 1 life, or pay {3}{U}{U}; the deck has plenty of blue cards
to pitch, but it costs two cards, so keep it for a spell that would end the game or wreck the Army).
**Jin-Gitaxias, Progress Tyrant** ({5}{U}{U}) copies your first artifact, instant or sorcery
each turn and counters each opponent's first one, which slows the combo tables. **Propaganda** taxes every attacker {2}.
Protection for Sauron and the Army: **Not of This World**, **Deflecting Swat** (free while you control your commander:
redirect a removal spell or counter) and **Slip Out the Back** ({U}: phase it out with a +1/+1 counter; the counters
stay).
**Not of This World** ({7}) counters a spell or ability that targets one of your permanents; it costs only {0}
when it protects a creature of yours with power 7 or more (a grown Army or Sauron).

### How it wins
The Army wins by size, by getting through, and by the X-spell finishers; the deck has no combo. A big Army that
connects tempts the Ring, which refills your hand while the table keeps feeding it. Sheoldred and the Archivist wheel
drain the table, Exsanguinate, Comet Storm and Torment of Hailfire finish a table that is low, and Massacre
Wurm, Ral or Inferno Titan clear blockers first.

### Weak spots and how to play around them
- **Big creature decks.** The deck loses mostly to combat. Land Propaganda, Talrand's Drakes and the one-sided
  sweepers early, and keep Counterspell and Mystic Confluence for the threats.
- **Spot removal on the Army.** Play The Ozolith first and do not overcommit counters to one body; Not of This World, Slip
  Out the Back, Deflecting Swat, Lightning Greaves and Whispersilk Cloak can protect it.
- **Your own sweepers.** Nibelheim Aflame and Chandra's Ignition hit your other creatures, so use them when the
  Army is the only body that matters.
- **Scavenger Grounds** ({2}, {T}, sacrifice a Desert: exile all graveyards) hits your own graveyard too, including
  the creature that Sauron, the Necromancer needs. Fire it deliberately.
- **Force of Will** costs you a second card (or {3}{U}{U}) and 1 life when pitched; hold it for the spell that would
  stop the Army or end the game, not for the first thing that is cast.

## Key lines

**Torment of Hailfire is the combat-free finish.** When the Army is walled or answered, cast it for a big X: each repeat makes every opponent lose 3 life, sacrifice a nonland permanent or discard. With Sauron's mana and the rocks, X of 8 or more usually kills a table the Army has softened, or leaves it with no board or hand.

**Archivist wheel with Bowmasters** (see Strategy) is the grind engine and, late, a finisher: three opponents redrawing five cards each take about 5 pings apiece from Bowmasters, and the Army grows by about 15. Hold it for the turn it kills or the counters matter.

**The early Army.** Dreadhorde Invasion on turn two, or Lazotep Reaver, with a rock to follow (eleven in the deck), builds an Army before Sauron arrives. Then the first hit comes sooner, and Massacre Wurm, Ral or Inferno Titan clear the way for it.

**Finish with X.** Once the Army has connected or opponents are low, Exsanguinate drains every opponent for X, and Comet Storm spreads X damage over as many targets as you kick. Both are better after a sweeper has already thinned the table.

**Sequencing.** Get Sauron down early, since every opponent spell after he lands is a free counter. Land Dreadhorde Invasion early, and The Ozolith before you commit a big Army, so removal only costs you tempo. Bowmasters pays off from the turn it lands, so cast it before a wheel, not after. Hold the counterspells for the turn you swing.

## Mana and consistency

The list is exactly 100 cards. 35 lands (18 nonbasic, 5 Island, 6 Mountain, 6 Swamp); Bloodsoaked Insight // Sanguine Morass is counted as a spell, so the deck plays as 36 when you need the land half. Ten rocks: Sol Ring, Arcane Signet, Mind Stone, Fellwar Stone (any colour an opponent's land could produce), Dimir Signet, Rakdos Signet, Talisman of Creativity, Talisman of Dominance, Talisman of Indulgence, and Chromatic Lantern. Urza, Lord High Artificer adds {U} from the non-mana artifacts.

| Measure | Count | Note |
| --- | --- | --- |
| Blue lands | 16 | 5 Island plus the U duals, Command Tower, Exotic Orchard, Path of Ancestry, Treno |
| Black lands | 18 | 6 Swamp plus Barad-dûr, Blood Crypt, Drowned Catacomb, Underground Sea, Watery Grave, Underground River, Foreboding Ruins, Haunted Ridge, Treno, and the any-colour lands |
| Red lands | 16 | 6 Mountain, Sokenzan, plus the R duals, Haunted Ridge, and the any-colour lands |
| Colorless-only lands | 2 | Rogue's Passage, Scavenger Grounds |

Dual lands: the original dual Underground Sea; the shocklands Blood Crypt, Steam Vents and Watery Grave; the check lands Drowned Catacomb and Sulfur Falls; the painlands Shivan Reef and Underground River; Foreboding Ruins (reveal a matching land or enter tapped); Haunted Ridge; and the tapped lands Treno, Dark City and Path of Ancestry. **Sokenzan, Crucible of Defiance** taps for {R} like a Mountain; late, channel it ({3}{R}, discard it) for two hasty 1/1 Spirits at instant speed.

**Interaction.** Six pieces of spot removal (Bitter Triumph, Bloodchief's Thirst, Baleful Mastery, Infernal Grasp, Terminate and Chaos Warp) plus Noxious Gearhulk; five counters (Mana Drain, Counterspell, An Offer You Can't Refuse, Force of Will and **Mystic Confluence**) plus Jin-Gitaxias; Not of This World, Deflecting Swat and Slip Out the Back to protect Sauron and the Army; Propaganda against attackers; two symmetric resets in Nibelheim Aflame and Cyclonic Rift; five one-sided sweepers or sweeper-like cards (Massacre Wurm, Ral and the Implicit Maze, Inferno Titan, Chandra's Ignition, Comet Storm); and Vandalblast for artifact-heavy tables.

## Tutor targets by board state

Three tutors: **Diabolic Tutor** (any card to hand), **Diabolic Intent** (any card to hand for {1}{B}, plus a creature sacrificed as a cost), and **Ringsight** (a card that shares a colour with a legendary creature you control, tempting the Ring on the way: with Sauron out that is any blue, black or red card). Ringsight reaches Massacre Wurm, Inferno Titan, Exsanguinate, Comet Storm, Torment of Hailfire and most of the rest of the deck; the colourless pieces (The Ozolith, Not of This World, the rocks) need a Diabolic. Diabolic Intent only wants spare bodies: a Zombie from Grave Titan, a Lazotep Reaver that has already amassed, a creature about to die anyway. Never feed it the Army. If every creature you have is working, hold it.

- **Army is huge, need to break through → the unblockable enabler.** Diabolic Tutor for Whispersilk Cloak or Rogue's Passage. If you already have evasion, Ringsight → Exsanguinate or Comet Storm for the kill.
- **Army walled or answered, table low → Ringsight for Torment of Hailfire.**
- **Opponents holding removal for the Army → Diabolic Tutor for The Ozolith** before you commit.
- **Bowmasters on board, hand running dry → Ringsight for Jace's Archivist**, then wheel the table.
- **Need to close the game → Ringsight for Exsanguinate or Comet Storm** once the Army is connecting.
- **Need protection for the swing → Diabolic Tutor for Not of This World** (it needs a colourless-capable tutor; Ringsight can't find it).
- **One problem creature → Ringsight for Infernal Grasp** (or Terminate for {B}{R} when the target is a creature and Grasp is not in hand).
- **Pod is casting into you → Ringsight for Kaervek**.
- **Behind on board / need the reset → Ringsight for Massacre Wurm or Inferno Titan**, or Ral and the Implicit Maze when you can wait a turn for the Army to stay safe; Cyclonic Rift if it is the whole table. The Ozolith out helps if you cast Nibelheim Aflame or Chandra's Ignition.
- **Slow start, no mana → Diabolic Tutor for Sol Ring.**
- **Flooded and grinding → Diabolic Tutor for Barad-dûr.**

Rule of thumb: Ringsight first for anything blue/black/red, and reserve Diabolic Tutor and Diabolic Intent for the colourless pieces nothing else can find (The Ozolith, Not of This World, the rocks).

## Bracket and Rule 0

Bracket 3 by Game Changers, with **three**, the cap for the bracket: **Cyclonic Rift**, **Orcish Bowmasters** and **Force of Will**. No other card in the deck is on the list.

**The deck has no two-card combo.** Exsanguinate, Comet Storm and Torment of Hailfire are X-spell finishers, not combos.

Disclose before the game:

- **Cyclonic Rift**, **Orcish Bowmasters** and **Force of Will** (the three Game Changers).
- **Sauron amasses off every spell an opponent casts**, and **Kaervek** punishes the same trigger.
- **Jace's Archivist** is a repeatable wheel; with Orcish Bowmasters out it hurts for every card they draw.

Re-check the official list at https://commanderbrackets.com/faq before an event.

## Decklist by type (100)

**Commander (1).** Sauron, the Dark Lord

**Creatures (20).** Faerie Mastermind, Grave Titan, Inferno Titan, Iron Man Armored Avenger, Jace's Archivist, Jin-Gitaxias Progress Tyrant, Kaervek the Merciless, Lazotep Reaver, Massacre Wurm, Mauhúr Uruk-hai Captain, Noxious Gearhulk, Orcish Bowmasters, Sauron the Necromancer, Scarlet Witch Chaotic Avenger, Sephiroth Planet's Heir, Sheoldred the Apocalypse, Talrand Sky Summoner, Urza Lord High Artificer, Vision Synthezoid Avenger, War Machine Avenging Arsenal

**Enchantments (4).** Call of the Ring, Dreadhorde Invasion, Propaganda, Ral and the Implicit Maze

**Artifacts (13).** Arcane Signet, Chromatic Lantern, Dimir Signet, Fellwar Stone, Lightning Greaves, Mind Stone, Rakdos Signet, Sol Ring, Talisman of Creativity, Talisman of Dominance, Talisman of Indulgence, The Ozolith, Whispersilk Cloak

**Instants and sorceries (27).** An Offer You Can't Refuse, Baleful Mastery, Bitter Triumph, Bloodchief's Thirst, Bloodsoaked Insight // Sanguine Morass, Chandra's Ignition, Chaos Warp, Comet Storm, Counterspell, Cyclonic Rift, Deflecting Swat, Diabolic Intent, Diabolic Tutor, Exsanguinate, Fact or Fiction, Force of Will, Infernal Grasp, Mana Drain, Mystic Confluence, Nibelheim Aflame, Night's Whisper, Not of This World, Ringsight, Slip Out the Back, Terminate, Torment of Hailfire, Vandalblast

**Lands (35).** Barad-dûr, Blood Crypt, Command Tower, Drowned Catacomb, Exotic Orchard, Foreboding Ruins, Haunted Ridge, Path of Ancestry, Rogue's Passage, Scavenger Grounds, Shivan Reef, Sokenzan Crucible of Defiance, Steam Vents, Sulfur Falls, Treno Dark City, Underground River, Underground Sea, Watery Grave, 5 Island, 6 Mountain, 6 Swamp

Sanguine Morass is counted above as a spell, so the deck plays as 36 lands when you need the land half.

## Import list (100)

```
1 An Offer You Can't Refuse
1 Arcane Signet
1 Baleful Mastery
1 Barad-dûr
1 Bitter Triumph
1 Blood Crypt
1 Bloodchief's Thirst
1 Bloodsoaked Insight // Sanguine Morass
1 Call of the Ring
1 Chandra's Ignition
1 Chaos Warp
1 Chromatic Lantern
1 Comet Storm
1 Command Tower
1 Counterspell
1 Cyclonic Rift
1 Deflecting Swat
1 Diabolic Intent
1 Diabolic Tutor
1 Dimir Signet
1 Dreadhorde Invasion
1 Drowned Catacomb
1 Exotic Orchard
1 Exsanguinate
1 Fact or Fiction
1 Faerie Mastermind
1 Fellwar Stone
1 Force of Will
1 Foreboding Ruins
1 Grave Titan
1 Haunted Ridge
1 Infernal Grasp
1 Inferno Titan
1 Iron Man, Armored Avenger
1 Jace's Archivist
1 Jin-Gitaxias, Progress Tyrant
1 Kaervek the Merciless
1 Lazotep Reaver
1 Lightning Greaves
1 Mana Drain
1 Massacre Wurm
1 Mauhúr, Uruk-hai Captain
1 Mind Stone
1 Mystic Confluence
1 Nibelheim Aflame
1 Night's Whisper
1 Not of This World
1 Noxious Gearhulk
1 Orcish Bowmasters
1 Path of Ancestry
1 Propaganda
1 Rakdos Signet
1 Ral and the Implicit Maze
1 Ringsight
1 Rogue's Passage
1 Sauron, the Dark Lord
1 Sauron, the Necromancer
1 Scarlet Witch, Chaotic Avenger
1 Scavenger Grounds
1 Sephiroth, Planet's Heir
1 Sheoldred, the Apocalypse
1 Shivan Reef
1 Slip Out the Back
1 Sokenzan, Crucible of Defiance
1 Sol Ring
1 Steam Vents
1 Sulfur Falls
1 Talisman of Creativity
1 Talisman of Dominance
1 Talisman of Indulgence
1 Talrand, Sky Summoner
1 Terminate
1 The Ozolith
1 Torment of Hailfire
1 Treno, Dark City
1 Underground River
1 Underground Sea
1 Urza, Lord High Artificer
1 Vandalblast
1 Vision, Synthezoid Avenger
1 War Machine, Avenging Arsenal
1 Watery Grave
1 Whispersilk Cloak
5 Island
6 Mountain
6 Swamp
```
