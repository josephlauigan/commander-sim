# Sheoldred, the Apocalypse — Mono-Black Drain

A mono-black deck that makes the whole table draw cards while Sheoldred, the Apocalypse punishes every one of them. It
wins by draining: three opponents' draw steps cost them 6 life a round with Sheoldred out, more with Underworld Dreams
and Fate Unraveler beside her, and Exsanguinate, Peer into the Abyss and the Sanguine Bond drain combo close the game. It
is weak when Sheoldred is exiled or keeps getting removed, and when the symmetric draw cards are played with no
punisher out, because they then just refuel the table.

| | |
|---|---|
| **Commander** | Sheoldred, the Apocalypse ({2}{B}{B}, 4/5 deathtouch, black) |
| **Bracket** | 4 (4 Game Changers: Ad Nauseam, The One Ring, Demonic Tutor, Bolas's Citadel; Mikaeus + Triskelion infinite-damage combo; Exquisite Blood + Sanguine Bond or Vito infinite drain) |
| **Simulator** | deck key `sheoldred`; results in `decklists/pool/pool-results.md`, history in `decklists/CHANGELOG.md` |

## Strategy

### The commander

**Sheoldred, the Apocalypse** ({2}{B}{B}, 4/5). *Deathtouch. Whenever you draw a card, you gain 2 life. Whenever an
opponent draws a card, they lose 2 life.*

She is the deck's one big engine, and she does four jobs:

- **She drains the table.** Each of three opponents loses 2 life in their own draw step, so 6 a round from the draw steps
  alone. Every extra card an opponent draws (Howling Mine, Seizan, Memory Jar, Rhystic Study, Peer into the Abyss) is 2
  more.
- **She gains life.** Every card you draw is 2 life. That pays for the deck's life costs (The One Ring, Phyrexian Arena,
  Call of the Ring, Phyrexian Delver) and feeds the lifegain payoffs: Sanguine Bond, Vito, Foolish Fate, Leech Collector.
- **She blocks and attacks.** A 4/5 deathtouch body holds off nearly any attacker. Bonders' Enclave counts her as a
  creature with power 4 or more.
- **She is the thing the table will answer.** Everything in the deck that makes the table draw is only good with her (or
  Underworld Dreams and Fate Unraveler) out. Cast her first, protect her, then start the engines.

**Where she is.** She starts in the command zone, so no tutor can find her and she is never in your opening hand. Each
recast costs {2} more per previous cast. If she dies, leave her in the graveyard when you hold Reanimate, Animate Dead,
Phyrexian Delver or Yawgmoth's Will: Reanimate is {B} and 4 life, Animate Dead {1}{B} and no life, Delver {3}{B}{B} and 4 life, and Yawgmoth's Will ({2}{B}) lets you cast
her from the graveyard for {2}{B}{B} with no tax. With none of those, move her to the command zone and pay the tax.

**When to cast her.** Turn 3 with a rock and three lands (Arcane Signet, Fellwar Stone or Mind Stone), turn 4 off four
lands, a turn earlier with Dark Ritual. In the sims she resolves on turn 6 on average, so a rock plus three lands is a
real edge. Cast her before any symmetric draw card. If the table is holding up removal mana, cast her anyway: she makes
them spend it, and Underworld Dreams, Fate Unraveler and the drain pieces don't need her.

### K'rrik, Son of Yawgmoth

**K'rrik, Son of Yawgmoth** ({4}{B/P}{B/P}{B/P}, 2/2, legendary, lifelink). *For each {B} in a cost, you may pay 2 life
rather than pay that mana. Whenever you cast a black spell, put a +1/+1 counter on K'rrik.* Each {B/P} in its own cost
is {B} or 2 life, so it casts for {4} and 6 life: a four-mana play whatever your lands make. It is the deck's life-to-mana
converter, and the deck has the life for it (Sheoldred, Gray Merchant, Exsanguinate, lifelink).

Once it is on the battlefield, every {B} in any cost can be paid with 2 life, in spells and in activated abilities.
Generic mana, {X} and colourless costs are not reduced.

| Card | Cost | With K'rrik out (all {B} paid with life) |
|---|---|---|
| Dark Ritual | {B} | 2 life, and it still makes {B}{B}{B} |
| Entomb | {B} | 2 life |
| Reanimate | {B} | 2 life, plus the target's mana value |
| Demonic Tutor, Animate Dead | {1}{B} | {1} and 2 life |
| Yawgmoth's Will | {2}{B} | {2} and 2 life |
| Grim Tutor | {1}{B}{B} | {1} and 4 life, plus the 3 life it costs |
| Diabolic Tutor | {2}{B}{B} | {2} and 4 life |
| Sheoldred | {2}{B}{B} | {2} and 4 life |
| Underworld Dreams | {B}{B}{B} | 6 life, no mana |
| Grave Pact | {1}{B}{B}{B} | {1} and 6 life |
| Exquisite Blood | {4}{B} | {4} and 2 life |
| Sanguine Bond, Gray Merchant | {3}{B}{B} | {3} and 4 life |
| Grave Titan | {4}{B}{B} | {4} and 4 life |
| Massacre Wurm, Mikaeus, Bolas's Citadel | {3}{B}{B}{B} | {3} and 6 life |
| Peer into the Abyss | {4}{B}{B}{B} | {4} and 6 life |
| Exsanguinate, Torment of Hailfire | {X}{B}{B} | {X} and 4 life: X is 2 higher for the same mana |
| Invoke Despair | {1}{B}{B}{B}{B} | {1} and 8 life |
| Vito's lifelink ability, Yawgmoth's proliferate | {3}{B}{B}, {B}{B} | {3} and 4 life; 4 life |
| Barad-dûr's amass | {X}{X}{B} | {X}{X} and 2 life |

Not reduced: Howling Mine, Memory Jar, Triskelion, Skullclamp, The One Ring, Swiftfoot Boots and the equip cost,
Cabal Coffers' {2}.

- **What it does for the plan.** A hand of Gray Merchant, Sanguine Bond and Massacre Wurm that three lands can't cast
  becomes castable, a rock-less turn 4 can cast Sheoldred and a Tutor in the same turn, and the kill turn needs fewer
  lands: Exquisite Blood plus Sanguine Bond is 7 mana and 6 life instead of 10 mana, Exquisite Blood plus Vito is 6
  mana and 4 life. Mikaeus drops to 3 mana and 6 life, but Triskelion is colourless and stays at 6.
- **Three more jobs.** It is a lifelink body, and every point it gains is a Sanguine Bond or Vito trigger. It counts 3
  toward devotion for Gray Merchant (each Phyrexian {B/P} symbol is a black symbol). Mikaeus gives it +1/+1 and
  undying (it is not a Human), but undying returns it only if it has no +1/+1 counters, and each black spell you cast
  puts one on K'rrik itself. Barad-dûr enters untapped with it out.
- **Dark Ritual on turn 2.** With two lands, Dark Ritual makes {B}{B}{B}, the second land adds {B}, and K'rrik costs
  {4} and 6 life. Sheoldred then costs {2} and 4 life on turn 3.
- **When paying life is safe.** Sheoldred is out (2 back per card you draw), K'rrik or another lifelinker is attacking,
  the spell you pay for gains the life back (Gray Merchant, Exsanguinate, Sheoldred), your life is high (about 25 or
  more before paying), and the table has no burn, no go-wide aggro and no Exquisite Blood plus Sanguine Bond or Vito
  of its own.
- **When it is not.** Your life is low, the table is red or a fast aggro deck, Edgar's loop pieces are out, you are
  already paying for Phyrexian Arena, The One Ring or Grim Tutor with no Sheoldred, or the spell can be countered (life
  is paid on casting and does not come back). Peer into the Abyss halves the *target's* life, not yours, so its 6 life
  is a plain cost, but check your total after it. Ad Nauseam and Ambition's Cost also take life, so don't stack them
  on a K'rrik turn.
- **Which {B} to pay.** Pay life only for the pips you can't pay with mana. Count your lands first, then use life to
  close the gap, not as a default.

### How the deck wins

Six lines, plus the slow grind. Lines 1 and 2 are the combos; 3 and 4 are one-turn kills; 5 and 6 are the creature and
graveyard packages. The deck has no counterspells, so every line is protected by timing and redundancy (see "Protecting
Sheoldred and the win turn").

#### Line 1: Exquisite Blood + Sanguine Bond or Vito

**Exquisite Blood** ({4}{B}): *Whenever an opponent loses life, you gain that much life.* **Sanguine Bond** ({3}{B}{B}):
*Whenever you gain life, target opponent loses that much life.* **Vito, Thorn of the Dusk Rose** ({2}{B}) has the same
trigger as Bond on a creature body.

1. Get Exquisite Blood and one of Bond or Vito onto the battlefield. **Blood is required.** Bond and Vito together
   don't loop: both only take life from the opponent, and nothing gives you life back.
2. Any life gain or opponent life loss now starts the loop. Blood gains you N, Bond or Vito takes N from an opponent,
   Blood gains N again. It repeats until every opponent is dead (each trigger picks a new target as they die).
3. Cheap starters: Sheoldred's trigger in anyone's draw step; cashing in Mind Stone with Sheoldred out (a draw, 2 life);
   Sephiroth's Intervention (gain 2); any drain creature (Blood Artist, Zulaport Cutthroat, Gray Merchant); Leech Collector's Bloodletting.

Mana: Blood 5, Bond 5, Vito 3. Vito plus Blood is 8 mana across two turns. With Cabal Coffers (see Line 4) both big
pieces come down in one turn.

- **Order.** Cast Bond first if you have it. Bond is useful alone: with Sheoldred out every card you draw drains an
  opponent for 2, and it is an enchantment, which most of the table's removal can't hit. Blood goes last, as the kill
  piece, on a turn when the table is tapped low. Vito is a creature and dies to cheap removal, so he comes down on the
  same turn as the piece that completes the pair.
- **One answer breaks it.** Destroy any piece while a trigger is on the stack and the loop ends after that trigger.
  With all three out, Blood is still the single point of failure (Blood + Bond, or Blood + Vito, is a pair).
- **Exsanguinate on top.** With Bond out, an Exsanguinate for X gains you 3X (one gain), and Bond turns that into 3X
  from one opponent.

#### Line 2: Mikaeus + Triskelion

**Mikaeus, the Unhallowed** ({3}{B}{B}{B}, 5/5): *Other non-Human creatures you control get +1/+1 and have undying.*
**Triskelion** ({6}, 1/1): *Enters with three +1/+1 counters. Remove a +1/+1 counter: it deals 1 damage to any
target.* Twelve mana across two turns.

1. Mikaeus out. Cast Triskelion (no tap symbol, so you can use it at once).
2. Remove all three counters. Aim two pings at Triskelion itself (with Mikaeus it is a 2/2 once the counters are gone)
   and one at an opponent. It dies with no +1/+1 counters, so undying returns it.
3. It comes back with three counters plus the undying counter: four. Remove all four: two at itself, two at the
   opponents. It dies, undying returns it again. Each cycle is 2 damage you choose, repeated without limit.
4. Stop when every opponent is dead. Every Triskelion death also drains with Blood Artist and Zulaport Cutthroat.

- **Cast Triskelion second.** If they answer Triskelion in response (Swords to Plowshares, Path to Exile), remove the
  counters in response and ping with them.
- **If Mikaeus is answered mid-loop,** Triskelion has no undying and the loop ends. Pings already dealt stay dealt.
- Demonic Tutor, Diabolic Tutor or Grim Tutor finds the missing half. Mikaeus has intimidate, and destroys any Human
  that deals damage to you.
- Humans don't get Mikaeus's bonus: Witch of the Moors, Zulaport Cutthroat, Vault Plunderer, Leech Collector, Sephiroth,
  Yawgmoth. Everything else does, Sheoldred included (see "Protecting Sheoldred").

#### Line 3: Peer into the Abyss

**Peer into the Abyss** ({4}{B}{B}{B}, sorcery): *Target player draws cards equal to half the number of cards in their
library and loses half their life. Round up each time.*

1. With Sheoldred out, cast it on one opponent. They draw about half their library, and each card is 2 life.
2. The maths: with N cards in the library and L life, they lose 2 for each of N/2 cards (about N) plus half of L. That is
   lethal when N + L/2 is at least L, so a library of half their life total or more. At 40 life, a library of 20
   or more. Mid-game libraries are 70 or more, so it kills any one opponent.
3. With Underworld Dreams alone it is 1 a card: lethal when the library is at least their life total. With Dreams and
   Fate Unraveler it is 2 a card, the same as Sheoldred.

Seven mana: Cabal Coffers, rocks and Dark Ritual get there. Demonic or Diabolic Tutor finds it. It is a sorcery, so
cast it when the table can't answer in response: a counterspell stops it, and Teferi's Protection (Aurelia, Edgar)
stops the life loss. Peer on one opponent leaves the other two untouched, so it is the first kill of three.

#### Line 4: Exsanguinate or Torment of Hailfire off Cabal Coffers

**Cabal Coffers** ({2}, {T}: add {B} for each Swamp you control). With 30 Swamps it is big mana by the mid-game.

| Lands in play (Coffers + Swamps) | Mana from lands | Exsanguinate X |
|---|---|---|
| 6 (5 Swamps) | 8 | 6 |
| 8 (7 Swamps) | 12 | 10 |
| 10 (9 Swamps) | 16 | 14 |

(Every other land is a Swamp, and Swamps pay the {2}; add rock mana on top. Barad-dûr taps for {B} but is not a Swamp,
so it adds one mana, not one more from Coffers.)

- **Exsanguinate** ({X}{B}{B}): *Each opponent loses X life. You gain life equal to the life lost this way.* X = 10 with
  three opponents is 30 life gained. Alone it is a big hit on the table; with Sanguine Bond it is 30 off one opponent;
  with Exquisite Blood it also starts Line 1.
- **Torment of Hailfire** ({X}{B}{B}): *Repeat X times: each opponent loses 3 life unless they sacrifice a nonland
  permanent or discard a card.* A big X either kills or strips the table of permanents and cards.
- **Gray Merchant of Asphodel** ({3}{B}{B}): each opponent loses life equal to your devotion to black, and you gain it.
  Sheoldred (2), Underworld Dreams (3), K'rrik (3), Phyrexian Arena (2), Seizan (2), Grave Pact (3) and the Merchant (2)
  all count, so with Sheoldred and Underworld Dreams out it is at least 7 from each, and 10 with K'rrik beside them.
- **K'rrik** pays the {X}{B}{B} of Exsanguinate and Torment of Hailfire with 4 life, so the same mana gives an X 2
  higher (8, 12 and 16 in the table above, with rocks on top).
- **Yawgmoth's Will** turns a spent Exsanguinate or Gray Merchant into a second one the same turn.

#### Line 5: the aristocrats package

Creatures dying drain the table; the sacrifice outlets make them die on demand; Mikaeus makes them come back.

- **Drain on death:** Blood Artist, Zulaport Cutthroat (each death, 1 each), **Sephiroth, Fabled
  SOLDIER** (1 for each other creature that dies; the fourth in a turn transforms him, and then every creature death
  drains for the rest of the game), Massacre Wurm (2 per opposing creature that dies).
- **Outlets:** Viscera Seer, Ashnod's Altar, Phyrexian Tower (a land: {T}, sacrifice a creature for {B}{B}), Yawgmoth,
  Deadly Dispute, Silence the Echo, Accursed Marauder, Sephiroth himself (sacrifice another creature on
  entering or attacking to draw).
- **Fodder:** Undercity Dire Rat (leaves a Treasure), the Zombie tokens from Grave Titan, the Army from Dreadhorde
  Invasion, a creature about to be killed anyway.
- **Grave Pact** ({1}{B}{B}{B}): each time a creature of yours dies, every opponent sacrifices a creature. Outlets
  plus Pact is an edict machine.
- **Yawgmoth + Mikaeus + Gray Merchant** (the three-card loop named under Bracket): you need a second nontoken non-Human
  creature (Blood Artist, Viscera Seer and Vito qualify). Pay 1 life and sacrifice the Merchant, putting a -1/-1 counter
  on the other creature and drawing a card. The Merchant dies and undying returns it, so it drains on entering again.
  The next activation does the same the other way round, so the -1/-1 counter on one creature cancels the +1/+1
  counter undying left on the other. Every second activation is a Gray Merchant drain; each draw is 2 life with
  Sheoldred out. Stop before you draw your library.
- **Barad-dûr** ({X}{X}{B}, {T}, only if a creature died this turn) turns any death into an Orc Army with X counters,
  a late use for spare mana.

#### Line 6: the graveyard start (Entomb, Reanimate, Animate Dead, Yawgmoth's Will)

- **Entomb + Animate Dead** is three mana for a six-drop, and no life. Entomb ({B}, instant) puts a card in the
  graveyard; Animate Dead ({1}{B}, an Aura) returns a creature card from any graveyard to the battlefield under your
  control. The creature gets -1/-0 and is sacrificed if the Aura leaves, so enchantment removal kills it too.
- **Entomb + Reanimate** is two mana: Reanimate ({B}, sorcery) puts a creature card from any graveyard onto the
  battlefield under your control, and you lose life equal to its mana value (6 for Grave Titan or Massacre Wurm, 5 for
  Gray Merchant, 4 for Sheoldred). No Aura to remove, and the creature keeps its full power. Check your life first:
  Sheoldred's 2 a draw pays it back. It also takes the best creature from an opponent's graveyard.
  - **Grave Titan**: a 6/6 deathtouch and two Zombies on turn 2, two more Zombies each attack. The default.
  - **Massacre Wurm**: the answer to a table of small creatures (Krenko, Edgar, Marwyn, Aurelia).
  - **Gray Merchant**: an early drain, and a second copy for later.
  - **Sheoldred herself** with Reanimate or Animate Dead when she is in a graveyard.
  - **K'rrik**: Animate Dead on it is {1}{B} and no life (it comes back a 1/2 with the Aura), so Entomb plus Animate Dead
    is a 3-mana K'rrik. Reanimate on it costs 7 life, and Phyrexian Delver 7 life: use those only at high life.
  - Entomb is an instant: use spare mana at the end of an opponent's turn, then Reanimate or Animate Dead on yours.
- **Yawgmoth's Will** ({2}{B}, sorcery): until end of turn you may play lands and cast spells from your graveyard; cards
  that would go to your graveyard are exiled. Replay the turn's rituals and tutors, a spent Exsanguinate or Gray Merchant,
  or Sheoldred.
- **Phyrexian Delver** is the creature version: it returns a creature card from your own graveyard only, and you lose
  life equal to its mana value.
- Graveyard hate (Bojuka Bog, Dauthi Voidwalker, Skullsnatcher, Farewell) turns this line off. The sims barely play
  Entomb and Yawgmoth's Will (stuck in hand 91-96% of the games they are drawn), so how well they win is untested.

#### The fair win: draw engines and drain

Sheoldred with Howling Mine, Seizan, Underworld Dreams and Fate Unraveler drains each opponent every round. Memory Jar
with Sheoldred out is 14 from each opponent in one turn. Grave Titan and its Zombies, Ob Nixilis (3 per land drop) and
the flying drain creatures keep the pressure on when the engines are answered. In an older sim run 76% of the
opponents the deck eliminated went by drain, and Memory Jar, Howling Mine and Underworld Dreams had the highest win rates
when cast.

### Game plan

**Turns 1-3.** A rock and cheap plays; Sheoldred on turn 3 or 4.

- Turn 1: Barad-dûr (it enters tapped unless you control a legendary creature, so play it when you have nothing to
  cast), Viscera Seer, or Dark Ritual only for a payoff you can cast.
- Turn 2: a rock (Arcane Signet, Fellwar Stone, Mind Stone), or a two-drop: Blood Artist, Zulaport Cutthroat, Undercity
  Dire Rat, Dreadhorde Invasion (an Army that grows every upkeep, 1 life each), Call of the Ring.
- Turn 3: Sheoldred off a rock, or Underworld Dreams (a good turn-3 play on its own: every card an opponent draws costs
  them 1), Vito, Phyrexian Arena, Gix.

**Turns 4-6.** Sheoldred first if she isn't out. Then, in order: equip Swiftfoot Boots, a punisher beside her
(Underworld Dreams, Fate Unraveler), then the draw engines (Howling Mine, Seizan, Memory Jar, the end-step lands), then
the drain pieces (Sanguine Bond, Exquisite Blood, Vito). Mikaeus, Grave Pact and Massacre Wurm arrive here too.
K'rrik fits here once Sheoldred is out and your life is high: cast it for {4} and 6 life, and the rest of the turn's
black spells cost life instead of mana.

**Late.** Run the engines, draw with The One Ring, Bolas's Citadel and Ad Nauseam, and take the kill that fits: Line 1
if both pieces are out, Peer into the Abyss if Sheoldred is safe, Exsanguinate off Cabal Coffers for the table. With
K'rrik out, every {B} in the kill pieces costs 2 life instead of a mana: spend the saved mana on the second piece.

**Hold up or go.**

- **Go** when the table is tapped low or holding one removal spell at most, when the kill is two pieces and you have
  both, or when someone else is about to win.
- **Hold** the symmetric draw cards (Howling Mine, Seizan, Memory Jar, Mikokoro, Geier Reach Sanitarium) until a punisher
  is out. Hold Memory Jar until she is safe; it gives every opponent seven cards for the turn.
- **Hold mana at the end of the opponent's turn** for Ad Nauseam, Mikokoro or the Sanitarium ({2} each), or Rooftop
  Assassin and instant removal. Ad Nauseam goes at the end of the opponent's turn before yours, with the mana untapped.

### The cards by role

#### Punish their draws

Sheoldred (2 life a card), **Underworld Dreams** and **Fate Unraveler** (1 damage a card each) stack: with all three
out, every card an opponent draws costs them 4. Underworld Dreams and Fate Unraveler work without Sheoldred, so they
keep the pressure on while she is dead or not yet cast. Fate Unraveler is also a 4-mana creature.

#### Make them draw

These turn the punishers on. Play the symmetric ones only once a punisher is out.

- **Howling Mine**: each player draws an extra card in their draw step, so each opponent draws two a turn (4 life with
  Sheoldred) while you draw two too (4 life back).
- **Mikokoro, Center of the Sea** and **Geier Reach Sanitarium** (lands): each player draws, at will. Mikokoro: {2},
  {T}, each player draws a card. The Sanitarium: {2}, {T}, each player draws then discards. Use them at an opponent's end
  step so you keep your mana.
- **Seizan, Perverter of Truth**: in each player's upkeep that player loses 2 life and draws two cards. With Sheoldred
  out each opponent loses 6 per upkeep, and you take 2 and draw two (net +2 life).
- **The wheel**: **Memory Jar** ({T}, sacrifice: each player exiles their hand and draws seven, then at the next end step
  discards and gets the old hand back) is 14 life from each opponent with Sheoldred out, and you gain 14. Cast it and
  use it in the same turn, in your second main phase.
- **Hand one opponent cards**: **Cost of Brilliance** (target player draws two and loses 2 life, and a +1/+1 counter
  on a creature) and **Vault Plunderer** (target player draws one and loses 1). With Sheoldred out these are 6 or 3 life
  against the chosen opponent. **Peer into the Abyss** is the big one (Line 3).

#### Draw your own

Each card you draw is 2 life with Sheoldred out, so the deck is full of draw: **Phyrexian Arena** (a card each upkeep,
1 life), **Ambition's Cost** (three cards, 3 life), **Syphon Mind** (each other player discards; you draw a card for
each), **Deadly Dispute** (sacrifice a creature or artifact, draw two, make a Treasure), **Skullclamp** (equip {1}: when
the creature dies, draw two), **Mind Stone** (cash in for a card), **Bonders' Enclave** ({3}, {T}: draw, with a creature
of power 4 or more, which Sheoldred is) and **Gix, Yawgmoth Praetor** (when a creature deals combat damage to an
opponent, its controller may pay 1 life to draw; that includes opposing creatures, so with Sheoldred out an attacker
drawing off Gix costs them 2 more).

**The One Ring** ({4}, indestructible) draws more every turn: tap it to add a burden counter, then draw a card per
counter. You lose 1 life per counter each upkeep, but with Sheoldred out each of those draws is 2 life back, so it
pays for itself. Cast, it also gives you protection from everything until your next turn. **Call of the Ring**
({1}{B}) has the Ring tempt you each upkeep, and each time you choose a Ring-bearer you may pay 2 life to draw a
card; Sheoldred gives the 2 back.

**Ad Nauseam** ({3}{B}{B} instant) turns spare life into a hand: reveal the top card, put it into your hand, lose life
equal to its mana value, repeat. Cast it at the end of the opponent's turn before yours, with your mana untapped,
and stop at a hand you can use next turn (about seven plus a card per three mana) or while your life is still well
above what the table could swing for. The cards are put into your hand, not drawn, so Sheoldred gains you nothing
from them. The cost stays moderate: the 63 nonland cards in the library average 3.4 mana value and 36 of the 99 cards
are lands (free), so a card costs about 2 life on average.

**Bolas's Citadel** ({3}{B}{B}{B}) lets you look at the top card of your library and play it, paying life equal to
its mana value instead of mana. The low curve keeps that cheap, and Sheoldred's lifegain keeps the life total up.
These cards are played, not drawn, so Sheoldred gains you nothing from them directly.

#### Gain life, use it

Sheoldred's lifegain turns on the payoffs: **Foolish Fate** (the target's controller loses 3 if you gained life this
turn), **Leech Collector // Bloodletting** (the first lifegain each turn makes Bloodletting available for {B}: each
opponent loses 2), **Sanguine Bond** and **Vito**. The life is also a resource: it pays for The One Ring's burden
counters, Call of the Ring, Bolas's Citadel, Ad Nauseam and Phyrexian Delver.

#### Drain creatures

**Blood Artist** (drain 1 on each death) keeps the pressure on when
Sheoldred is gone. **Ob Nixilis, the Fallen** ({3}{B}{B}, 3/3) makes a player lose 3 life each time a land enters under
your control and grows by three +1/+1 counters: one land drop a turn is 3 to an opponent and a bigger threat.
**Zulaport Cutthroat** drains each opponent 1 (and gains you 1) whenever a creature you control dies. Every point these
drain creatures gain you is also a Sanguine Bond or Vito trigger. **Dreadhorde Invasion** gives you an Army body that
grows every upkeep, a sacrifice-fodder token, and lifelink once it reaches power 6.

**Witch of the Moors** ({3}{B}{B}, 4/4 deathtouch): at the beginning of your end step, if you gained life this turn,
each opponent sacrifices a creature and you return up to one creature card from your graveyard to your hand. With
Sheoldred out you gain 2 on every draw, so it is a repeatable edict and a card back each turn; the drain creatures,
Exquisite Blood and Vito turn it on without her. It is a 5-drop that the sim leaves in hand in about half the games it
is drawn: cast it as soon as the mana allows, after Sheoldred.

#### Remove threats

Five removal spells, Massacre Wurm and Grave Pact keep the board manageable: **Massacre Wurm** (opposing creatures get
-2/-2, and each one that dies costs its controller 2 life: the answer to Krenko and other tokens), **Overkill**
(-0/-9999), **Foolish Fate**, **Sephiroth's Intervention** (destroy, gain 2 life), **Invoke
Despair** (edict for a creature, an enchantment and a planeswalker, with 2 life lost and a card for you each time they
have none; needs {B}{B}{B}{B}) and **Silence the Echo** (cheap with a creature to sacrifice, or pay {3}).
**Rooftop Assassin** (flash, flying, lifelink) destroys a creature that was dealt damage this turn, and **Accursed
Marauder** makes each player sacrifice a nontoken creature (it can sacrifice itself). **Grave Pact** makes each
opponent sacrifice a creature whenever one of yours dies, so every sacrifice for value is also removal. With K'rrik
out, Foolish Fate and Overkill cost {2} and 2 life, and Sephiroth's Intervention {3} and 2 life.

**Nevinyrral's Disk** ({4}, enters tapped; {1}, {T}: destroy all artifacts, creatures and enchantments) is the deck's reset button for a board Massacre Wurm can't handle. It destroys Sheoldred, Mikaeus, K'rrik and every drain enchantment too, so activate it only when you are behind on board, with Animate Dead, Phyrexian Delver or the command zone ready to bring Sheoldred back, or when the engines are already dead. It is colourless, so K'rrik can't pay for any of it.

#### Recursion

**Phyrexian Delver** returns a creature card from your graveyard to the battlefield (you lose life equal to its mana
value): recover Sheoldred, or return Gray Merchant for another drain. **Reanimate** and **Animate Dead** put a creature card from
any graveyard onto the battlefield (Reanimate for {B} and life equal to its mana value; Animate Dead is an Aura: if it
leaves, so does the creature); **Entomb** puts the one you want there
first. **Yawgmoth's Will** lets you replay a
turn from the graveyard (Line 6).

#### Ramp

Three rocks, Arcane Signet, Fellwar Stone and Mind Stone (it can cash in for a card), plus **Dark Ritual** (Sheoldred a
turn early), Ashnod's Altar and Undercity Dire Rat (a Treasure when it dies). Memory Jar, The One Ring and Bolas's
Citadel are the bigger artifacts you cast once the rocks are out. **Cabal Coffers** is the late-game ramp, and
**K'rrik, Son of Yawgmoth** turns life into mana: each {B} in a cost can be paid with 2 life.

### Mulligans

The first mulligan is free (you see two 7-card hands). After that it is a London mulligan: draw 7, put one card on the
bottom for each mulligan after the free one. Sheoldred is in the command zone, so she is never in the hand.

| Hand | Keep when |
|---|---|
| 7 cards | 2-5 lands including a source of {B} (Swamp or Barad-dûr); a play on turns 2-4 (a rock, Dark Ritual, a two-drop, a punisher or Sheoldred's four mana by turn 4); and one reason to go: a tutor, a punisher (Underworld Dreams, Fate Unraveler), a draw engine, a drain piece or Entomb with Animate Dead. A two-lander needs a rock or Dark Ritual and cheap plays. Five lands need two real spells. 0-1 lands and 6+ lands go back |
| 6 or 5 cards | 2-4 lands including a {B} source, and two plays that cost 3 or less |
| 4 cards | Keep |

- **No black source goes back.** A hand with only Bonders' Enclave, Mikokoro, the Sanitarium, Phyrexian Tower or Cabal
  Coffers (which needs Swamps) can't cast Sheoldred.
- **Count turn 3-4 mana.** Three lands plus a rock is Sheoldred on turn 3; four lands is turn 4. A hand with only
  6-7 mana spells (Peer into the Abyss, Bolas's Citadel, Invoke Despair) is a ship, whatever the lands. K'rrik counts as
  a 4-mana spell ({4} and 6 life), not a 7.
- **Bottom first:** lands past the fourth, Peer into the Abyss, Bolas's Citadel, Memory Jar. Keep rocks, Dark Ritual,
  tutors and the cheap punishers.
- **Symmetric draw cards** (Howling Mine, Memory Jar) are not a reason to keep: they are dead until a punisher is out.

| # | Hand | Call | Why |
|---|---|---|---|
| 1 | Swamp, Swamp, Barad-dûr, Arcane Signet, Underworld Dreams, Blood Artist, Phyrexian Arena | Keep | Barad-dûr on turn 1 (tapped, costs nothing), Signet on turn 2, Sheoldred on turn 3 from the command zone; Dreams, Arena and Blood Artist follow |
| 2 | Swamp, Swamp, Swamp, Mikokoro, Center of the Sea, Demonic Tutor, Howling Mine, Underworld Dreams | Keep | Dreams on turn 3 (BBB), Sheoldred on turn 4, then Howling Mine is safe to play; the Tutor finds a drain piece |
| 3 | Swamp, Cabal Coffers, Phyrexian Tower, Bonders' Enclave, Peer into the Abyss, Memory Jar, Exsanguinate | Ship | One Swamp and three lands that can't make {B} without help, no play before turn 5, and Memory Jar and Peer are dead without a punisher |
| 4 | Swamp, Swamp, Mind Stone, Dark Ritual, Viscera Seer, Zulaport Cutthroat, Gray Merchant of Asphodel | Keep (close) | Two lands, but Seer on turn 1 and Mind Stone on turn 2 mean turn 3 is two Swamps, Mind Stone and Dark Ritual: 5 mana for Sheoldred with no third land |
| 5 | Swamp, Swamp, Swamp, Swamp, Swamp, Mikokoro, Center of the Sea, Bolas's Citadel | Ship | Six lands go back, and the one spell costs 6 |
| 6 | Swamp, Swamp, Swamp, Barad-dûr, Vito, Thorn of the Dusk Rose, Exquisite Blood, Grim Tutor | Keep | Vito on turn 3, Sheoldred on turn 4, Blood on turn 5 (needs a fifth land): the first card an opponent draws starts the loop. Grim Tutor finds Sanguine Bond or an answer |
| 7 | Swamp, Swamp, Swamp, Entomb, Animate Dead, Demonic Tutor, Howling Mine | Keep | Entomb into Animate Dead is Grave Titan on turn 2 (no life), Sheoldred when the fourth land comes. The Tutor finds the next piece; the Mine waits for a punisher |
| 8 | Swamp, Swamp, Ad Nauseam, Bolas's Citadel, Invoke Despair, Peer into the Abyss, Ambition's Cost | Ship | Two lands, no rock, and every spell costs 4 or more |

### What to tutor for

Sheoldred is in the command zone, so no tutor can find her: the tutors find everything around her. Every tutor except
Entomb is a sorcery, and the found card goes to your hand, not through a draw, so Sheoldred gains you nothing from it.
Every tutor can find any card in the deck, K'rrik included. Before you cast one, name the card you will cast next and
check that the mana for it is there this turn or the next.

#### Every card that finds a card

| Card | Cost | Card goes to | Can find | Speed | The catch |
|---|---|---|---|---|---|
| Demonic Tutor | {1}{B} | Hand | Any card | Sorcery | Game Changer. The cheapest clean tutor, so it is the one to chain into the found card in the same turn |
| Diabolic Tutor | {2}{B}{B} | Hand | Any card | Sorcery | Four mana: on turn 4 it competes with Sheoldred for the mana. Best from turn 6 on, with the found card castable next turn |
| Grim Tutor | {1}{B}{B} | Hand | Any card | Sorcery | Costs 3 life and Sheoldred gives none back. Off at low life, without Sheoldred out, and against Exquisite Blood plus Sanguine Bond or Vito on the table |
| Entomb | {B} | Graveyard | Any card | Instant | Useless without Reanimate, Animate Dead, Phyrexian Delver or Yawgmoth's Will to use the card. Dead to graveyard hate. It is the only tutor you can use on an opponent's turn |

Other cards that find cards, none of them a choice: **Call of the Ring** (draws when you choose a Ring-bearer, 2 life
each, and Sheoldred gives the 2 back), **Ad Nauseam** (top cards into your hand for life) and **Bolas's Citadel**
(plays the top card for life). They do not replace a tutor on a turn that needs one specific card.

#### Which card, by board state

Count mana before you pick: lands, rocks and Cabal Coffers (it needs {2} and then makes {B} for each Swamp, so Coffers
plus 5 Swamps is 8 mana from 6 lands). Cast the tutor first, then check what is left for the found card. With K'rrik
out, each {B} in the tutor and the target can be paid with 2 life (see "K'rrik, Son of Yawgmoth").

**Sheoldred not yet cast (turns 1-4)**

| Your mana and hand | Opponents | Tutor | Target | Why, and what to watch |
|---|---|---|---|---|
| Turn 2, two lands, no rock, nothing to do on turn 3 | Slow or empty boards | Demonic Tutor | Dark Ritual | Turn 3: three lands plus Dark Ritual ({B} for {B}{B}{B}) is 5 mana, Sheoldred plus 1 spare. Use it when the hand has no better play. A rock (Arcane Signet, Mind Stone) is turn-4 Sheoldred, which the lands already give |
| Turn 3 or 4, Sheoldred castable this turn, your hand has no protection | White or blue-black with open mana, or cheap removal shown | Demonic Tutor if there is mana left, else next turn | Swiftfoot Boots | Cast Sheoldred first, Boots and equip ({2} plus {1}) the turn after. Boots stop targeted removal only, not wipes, edicts or Council's Judgment |
| Turn 3, three Swamps, hand holds Howling Mine, Seizan or Memory Jar and no punisher | Anything | Demonic Tutor | Underworld Dreams | {B}{B}{B} turns the symmetric cards on and works with Sheoldred dead or not yet cast. Sheoldred still comes first if she is castable |
| Turn 3, a creature wipe is likely (Damn, Toxic Deluge, Blasphemous Act, Farewell shown) | Wipe deck | Any tutor | Underworld Dreams, Sanguine Bond or Exquisite Blood | Enchantments survive a creature wipe. Hold creatures in hand and do not tutor for Mikaeus, K'rrik or the Merchant into it |
| Turn 2-3, Entomb plus Animate Dead or Reanimate in hand, life 30 or more | No Bojuka Bog, Voidwalker, Skullsnatcher or Farewell shown | Entomb, at the end of their turn | Grave Titan (default), Massacre Wurm against small creatures, K'rrik when the hand is full of black spells | Titan is a 6/6 deathtouch (5/6 under Animate Dead) and two Zombies on turn 2. K'rrik is the pick when Sheoldred and the rest of the hand are then cast for life instead of mana. Reanimate pays life equal to the mana value (K'rrik 7), Animate Dead pays none |
| Turn 1-3, an opponent is set up to win with a creature engine (Chulane with Aluren, Krenko, Marwyn's elves, Yuriko) and you hold no answer | Engine on the table | Demonic Tutor, then hold the mana | Foolish Fate or Overkill | Both are {2}{B} instants: tutor on your turn, keep three mana, kill the engine creature on their turn or in response to the activation. Overkill beats regeneration and indestructible |

**Sheoldred out, engines running (turns 4-6)**

| Your mana and hand | Opponents | Tutor | Target | Why, and what to watch |
|---|---|---|---|---|
| 5 or 6 mana, Sheoldred safe, no Line 1 piece out | Tapped low or open removal | Demonic Tutor | Sanguine Bond | Bond works alone (every card you draw takes 2 from an opponent) and is an enchantment, which creature removal misses. Exquisite Blood goes last |
| Exquisite Blood out, no Bond or Vito | Any | Demonic Tutor (5 mana for tutor plus Vito) | Vito, Thorn of the Dusk Rose | Cheapest second piece (3 mana) and the loop starts the same turn. Pick Bond instead when the table has open creature removal |
| Bond or Vito out, no Blood | Tapped out, or one removal spell at most | Any tutor | Exquisite Blood | Blood is the loop piece (5 mana). Cast it only when they cannot respond: one answer to a piece ends it. Starters: a draw with Sheoldred out, K'rrik's lifelink, any drain creature |
| Mikaeus out, no Triskelion, 8 mana for tutor plus Triskelion | Table tapped, or Swords or Path open | Demonic Tutor, else tutor now and cast next turn | Triskelion | Line 2 needs both. Cast Triskelion second, and if they answer it, remove the counters in response. Triskelion is colourless, so K'rrik does not reduce it |
| Triskelion out, no Mikaeus | Table tapped | Any tutor | Mikaeus, the Unhallowed | He is a creature: bring him down when the table cannot answer. 6 mana, or 3 and 6 life with K'rrik |
| Mikaeus and Gray Merchant out, no Yawgmoth, and you control a second nontoken non-Human creature | Any | Any tutor | Yawgmoth, Thran Physician | The three-card loop (Line 5): a Merchant drain on every second activation. Yawgmoth has protection from Humans. Stop before you draw your library |
| Sheoldred out and an opponent draws a lot (Chulane, Korvold, Rhystic Study, Brainstorm) | Any | Any tutor | Underworld Dreams, then Fate Unraveler | They lose 2 for each draw already, and each punisher adds 1: three punishers are 4 a card. Dreams is {B}{B}{B} |
| Sheoldred out, table with removal mana up, no way to keep going if she dies | Open blue or white mana | Any tutor | Underworld Dreams, Fate Unraveler or Phyrexian Arena | A second source of damage and draws that does not depend on her |
| A go-wide board (Krenko, Edgar Markov, Aurelia, Marwyn, Chatterfang) | Many X/2 creatures | Any tutor, cast it next turn if the mana is short | Massacre Wurm | Kills the X/2s and each opponent loses 2 per creature that dies. Cast it before the swing, not after. 6 mana, or 3 and 6 life with K'rrik |
| An opponent has Opposition Agent | Agent on the board | Do not tutor | Kill the Agent first (Foolish Fate, Overkill, Massacre Wurm) | It takes whatever you search for |
| Tergrid or Liliana of the Veil out | Discard or edicts | Demonic Tutor, cast the card the same turn | The card you can cast now | So the card is not discarded or taken in hand. No Sanitarium or Dispute |

**Sheoldred removed, or the table is ahead**

| Situation | Tutor | Target | Why, and what to watch |
|---|---|---|---|
| Sheoldred in the graveyard, no Animate Dead in hand | Demonic Tutor | Animate Dead | {1}{B}, no life. Reanimate pays 4 life and Phyrexian Delver is 5 mana and 4 life. Yawgmoth's Will casts her for {2}{B}{B} with no tax. If she is in the graveyard and you hold one of these, leave her there |
| Sheoldred exiled, bounced or stolen | Any tutor | Underworld Dreams or Fate Unraveler | Recast her from the command zone ({2} more per earlier cast). The punishers hold the damage meanwhile. If she was stolen, kill the thief's creature or sacrifice her in response (Viscera Seer, Ashnod's Altar, Yawgmoth, Deadly Dispute) |
| A wipe just resolved, Sheoldred was the only creature | Any tutor | Animate Dead, then Sheoldred | Enchantments (Dreams, Arena, Bond, Blood) are still out. Rebuild with Phyrexian Delver or Perigee Beckoner |
| An opponent has a combo that wins next turn | Demonic Tutor, then hold three mana (four for Sephiroth's Intervention) | Foolish Fate, Overkill or Sephiroth's Intervention (all instants) | You tutor at sorcery speed, so you need the answer in hand and the mana up at the right time. Rooftop Assassin only kills a creature that has been dealt damage |
| An opponent has graveyard hate out (Bojuka Bog, Dauthi Voidwalker, Skullsnatcher, Farewell) | Any tutor except Entomb | The card itself, hard-cast | Line 6 is off, Entomb finds nothing you can use |
| Life 15 or less, Sheoldred out | Demonic Tutor only | Gray Merchant, Exsanguinate or Rooftop Assassin | Grim Tutor, Ad Nauseam and K'rrik's life payments are off. Merchant and Exsanguinate gain it back |
| Life 15 or less, Sheoldred not out | Demonic Tutor | Gray Merchant or Exsanguinate | You cannot gain from draws yet. Do not pay life for a tutor or for K'rrik |

**The kill**

| Your board and mana | Opponents | Tutor | Target | Why, and what to watch |
|---|---|---|---|---|
| Sheoldred or Underworld Dreams out, 7 mana (or 4 and 6 life with K'rrik), one opponent is the threat | That player's library at least half their life total, no Teferi's Protection or counterspell mana | Demonic or Diabolic Tutor | Peer into the Abyss | Lethal on one opponent (Line 3). A sorcery that leaves the other two untouched, so it is the first kill of three |
| Cabal Coffers plus 5 or more Swamps, table not yet at lethal | Tapped low | Demonic or Diabolic Tutor | Exsanguinate | X is mana minus 2 (6 at 6 lands, 10 at 8, 14 at 10), 2 higher with K'rrik. 3X life for you against three opponents. With Bond out it is a second drain, with Blood it starts the loop |
| The same Coffers mana, opponents low on cards and permanents (or you want to strip a combo piece) | Few cards in hand, bare boards | Demonic or Diabolic Tutor | Torment of Hailfire | Each repeat takes 3 life or a card or a permanent, their choice. Worse than Exsanguinate against full hands and boards, and it gains you nothing |
| Devotion (count it: Sheoldred 2, Dreams 3, K'rrik 3, Arena 2, Seizan 2, Pact 3, Mikaeus 3, Massacre Wurm 3, Grave Titan 2, Bond 2, Gix 2, plus the Merchant's 2) reaches the lowest opponent's life, or Bond is out | Any | Any tutor | Gray Merchant | Five mana and a creature, so it dodges counterspells that only hit noncreature spells. With Bond or Blood out it is a Line 1 starter |
| Sheoldred out and every opponent at 14 or less (21 with Underworld Dreams also out) | Libraries of 7 or more | Demonic Tutor if 7 mana | Memory Jar | Cast it and tap it the same turn: every opponent draws seven, 14 life each, and you gain 14. Never with no punisher out |

**K'rrik, Son of Yawgmoth: when to find it**

| Situation | Tutor | Target | Why, and what to watch |
|---|---|---|---|
| Turn 4-6, 5-6 lands or rocks, hand holds two or more black spells with three or more {B} pips between them (Gray Merchant, Grave Titan, Sanguine Bond, Massacre Wurm, Mikaeus), life 25 or more, Sheoldred out or lifelink available | Demonic Tutor (2 mana, K'rrik 4 and 6 life) | K'rrik | Turns the hand into mana: the next turn's spells cost 2 life per {B}. Check your life for the whole turn first |
| Exquisite Blood and Sanguine Bond (or Vito) in hand, mana is the only thing missing, life 25 or more | Demonic Tutor | K'rrik | 7 mana and 6 life for both pieces (6 mana and 4 life with Vito). The kill is then a turn earlier. Do it only if the table cannot punish the life |
| Entomb plus Animate Dead in hand, Sheoldred in the command zone, hand full of {B} spells | Entomb | K'rrik instead of Grave Titan | Animate Dead is 3 mana in all for K'rrik, and Sheoldred is {2} and 4 life next turn. It loses 1 power to the Aura and dies if the Aura goes |
| Cabal Coffers is online (about 10 mana) | Any tutor | The kill (Exsanguinate, Peer, Gray Merchant), not K'rrik | You already have the mana, and K'rrik is a 7-mana-value creature that dies to removal while the life is still gone |
| Life under 20, or a red or aggro table, or Edgar's loop pieces out | Any tutor | Not K'rrik | Each {B} would cost 2 life you cannot afford to lose, and a drain that starts the table's loop is worse than the mana |
| A removal-heavy table with open mana | Any tutor | K'rrik only after Boots is on Sheoldred | Removal will meet K'rrik first. The life paid on casting is not refunded |

#### Priority order for the common cases

1. **Sheoldred cannot land on turn 4 (no rock, three lands).** A mana source that makes turn 3 or 4 work (Dark Ritual, Arcane Signet, Mind Stone).
2. **An opponent wins next turn.** The instant answer (Foolish Fate, Overkill) with mana kept up, before anything else.
3. **Sheoldred is out, the table has removal.** Swiftfoot Boots, then a second punisher (Underworld Dreams, Fate Unraveler).
4. **Sheoldred is out and safe.** Sanguine Bond (works alone), then the missing Line 1 piece (Blood last), or Mikaeus and Triskelion, whichever half you lack.
5. **Mana is the problem and life is high.** K'rrik, cast the same turn if the tutor leaves 4 mana.
6. **The kill is ready.** Peer into the Abyss, Exsanguinate or Gray Merchant for the table, whichever the opponents' life totals and your mana allow.
7. **Sheoldred is dead.** Animate Dead, then a replacement punisher.

#### Tutor mistakes

- **Tutoring for a card you cannot cast soon.** A tutor that leaves no mana for the found card, with removal on the table,
  gives the table a turn to answer or stop you. Tutor with the mana left over.
- **Tutoring for Exquisite Blood or Vito into untapped removal.** One answer ends the loop. Find Bond first, Blood last.
- **Grim Tutor at low life, without Sheoldred out, or against Edgar's Exquisite Blood pieces.** The 3 life is not given back.
- **Entomb with nothing to reanimate it, or into graveyard hate.** The card is simply lost.
- **Tutoring under Tergrid or Liliana of the Veil** without casting the card the same turn, so it is discarded.
- **Tutoring while an opponent controls Opposition Agent.**
- **Paying K'rrik life for a spell that gets countered, or at low life.** The life does not come back.
- **Peer into the Abyss at a library that is small compared to the life total,** or into Teferi's Protection or open counters.
- **Tutoring for a creature piece (Mikaeus, K'rrik, Vito, Merchant) into a wipe** you have seen coming. Hold it, or find
  an enchantment.

### Protecting Sheoldred and the win turn

The deck has no counterspells. Protection is timing, tokens, a few cards and a second engine.

| Threat to Sheoldred | Answer in the deck |
|---|---|
| Targeted removal (Swords to Plowshares, Path to Exile, Go for the Throat, Terminate) | **Swiftfoot Boots**: hexproof, equip {1}, sorcery speed, so equip it as soon as she lands. Boots don't stop wipes, edicts, or Council's Judgment (it is a vote, not a target) |
| Removal or combat danger this turn | **Perigee Beckoner** (another creature gets +2/+0 and returns to the battlefield tapped if it dies this turn; warp {1}{B} reuses it). Cast it on her before she attacks or blocks into a risk |
| Any destroy or damage, while Mikaeus is out | Sheoldred is a non-Human, so Mikaeus gives her undying: she returns with a +1/+1 counter, a 6/7 with Mikaeus's bonus |
| Edicts (Sheoldred's Edict, Plaguecrafter, Fleshbag Marauder, Liliana of the Veil) | Keep tokens and cheap bodies out: Zombie tokens, the Army, Undercity Dire Rat, Viscera Seer. Sheoldred's Edict can choose nontoken creatures, so keep a cheap nontoken creature too |
| Wipes (Damn, Toxic Deluge, Blasphemous Act, Innocent Blood) | Don't overextend while she is your only engine. Hold creatures in hand. Mikaeus's undying brings your non-Humans back |
| Exile, bounce, Farewell, Cyclonic Rift | Recast her from the command zone. Keep Underworld Dreams, Fate Unraveler, Phyrexian Arena and the drain enchantments as engines that don't need her |
| Steal (New Blood) or turn into an Elk (Oko) | Sacrifice in response with Viscera Seer, Ashnod's Altar, Yawgmoth or Deadly Dispute; after Oko, sacrifice the Elk and return her with Animate Dead |
| She is dead | See "After a stopped attempt" |

**The One Ring** ({4}) gives you, not her, protection from everything until your next turn: no damage and no targeted
spells for you, a good turn to cast a draw engine. It does not save a creature from a wipe.

**For the kill turn:** the deck's lines are mostly enchantments and sorceries, so
- cast the kill when the blue and white players are tapped low (Yuriko, Chulane, Atraxa, Aurelia);
- make your last piece a creature against counterspell decks (Fierce Guardianship, Dovin's Veto and Swan Song can't
  hit creatures; Gray Merchant, Mikaeus, Vito and Triskelion are creatures);
- keep a second line: Bond plus Blood plus Vito, Peer into the Abyss as the backup, Exsanguinate for the table;
- cast Ad Nauseam at the end of an opponent's turn, when they have spent their mana.

### Interaction on opponents' turns

The deck's instant-speed answers. Everything else is a sorcery.

| Card | Cost, speed | What to use it on |
|---|---|---|
| Foolish Fate | {2}{B} instant | A creature that is attacking, or the combo creature (Krenko, Yuriko, Korvold). Extra 3 life loss if you gained life this turn: cast it after a draw |
| Overkill | {2}{B} instant | The biggest threat, or a creature with regeneration or indestructible (it shrinks, it doesn't destroy). Not a hexproof creature |
| Sephiroth's Intervention | {3}{B} instant | A creature; the 2 life gained also starts Line 1 if the pieces are out |
| Rooftop Assassin | {3}{B} flash | After blocks and damage: it destroys an opposing creature that was dealt damage this turn and survived. Also a flying lifelink blocker |
| Deadly Dispute | {1}{B} instant | In response to removal, a steal or an edict aimed at your creature: sacrifice it, draw two and make a Treasure (4 life with Sheoldred out) |
| Viscera Seer, Ashnod's Altar, Phyrexian Tower, Yawgmoth | Free or cheap | In response to a steal effect on one of your creatures, or to get value from a creature that is about to die |
| Ad Nauseam | {3}{B}{B} instant | The end of the opponent's turn before yours |
| Mikokoro, Geier Reach Sanitarium | {2}, {T} | An opponent's end step: each player draws, and Sheoldred drains the opponents |
| Entomb, Dark Ritual | {B} instant | Entomb at their end step to set up Animate Dead; Dark Ritual to pay for an instant above |

- **Hold {3}{B} (four mana) on the opponent's turn** when Rooftop Assassin or a removal spell would change a fight.
- **Don't fight what you can't answer.** The deck has no enchantment or artifact removal (Invoke Despair is a
  choice they make). Against Aluren, Intruder Alarm and Smothering Tithe, race and kill the creatures that use them.

### Matchups (Tiers 3 and 4)

Your tiers are t3 (Atraxa, Aurelia, Korvold, Marwyn, Tergrid) and t4 (Chulane, Edgar Markov, Krenko, Prosper, Yuriko).
Where numbers appear they come from older sim runs on earlier lists (`audit/sheoldred/FINDINGS.md`).

**Types of deck that are bad for you:**
- **Decks that stop draws or punish the symmetric cards.** If a deck is built around card draw, or has a combo that
  wants cards, hold the Mine, Seizan, Mikokoro, the Sanitarium and the wheels until a punisher is out, and kill that deck
  first.
- **Big creature decks.** Use Sheoldred's deathtouch, Grave Pact and the removal; save Overkill for the biggest threat.
- **Lifegain decks.** The drain is slower against them; push Exsanguinate and the drain creatures for the last points.

| Opponent | Their threat | Your plan | Cards that matter |
|---|---|---|---|
| Atraxa, Praetors' Voice (t3, WUBG infect and planeswalkers) | Poison: Blighted Agent (unblockable), Plague Stinger and Skithiryx (fliers), then a proliferate every turn. Ten poison kills at any life total, so your life gain doesn't help. Farewell exiles creatures, enchantments, artifacts and graveyards; Oko turns Sheoldred into an Elk; their Narset caps you at one draw a turn (the Mine and Seizan give you nothing extra; their draws still cost them). The sims show it ends about 6-8% of games | Kill the poison creatures on sight and race. Fliers and blockers: Rooftop Assassin. With Sheoldred out, don't pay Rhystic Study: their draw costs them 2. Keep a creature in hand and a second engine for after Farewell (it exiles the graveyard too, so no Animate Dead after it) | Massacre Wurm, Foolish Fate, Overkill, Swiftfoot Boots (against Swords, Path, Assassin's Trophy), Underworld Dreams |
| Aurelia, the Warleader (t3, Boros extra combats) | Hasty flying attackers (Aurelia, Hero of Bladehold, Goblin Rabblemaster, Legion Warboss) with Embercleave and extra combats; Mother of Runes and Giver of Runes protect them; Blasphemous Act. Smothering Tithe makes a Treasure for them for each card you draw unless you pay {2} | Kill Mother of Runes and Giver of Runes first. Block with Sheoldred (deathtouch). Massacre Wurm kills their small creatures. Mikaeus destroys any Human that damages you (Hero of Bladehold, Adeline, Combat Celebrant). With Tithe out, draw fewer extra cards | Massacre Wurm, Grave Pact, Mikaeus, Foolish Fate, Accursed Marauder; Boots don't stop Council's Judgment |
| Korvold, Fae-Cursed King (t3, Jund sacrifice) | Korvold draws and grows off every sacrifice; Blood Artist, Zulaport Cutthroat, Mayhem Devil, Goblin Bombardment, Impact Tremors; Chatterfang + Pitiless Plunderer; Terror of the Peaks; Toxic Deluge and cheap removal | Their deck draws a lot (Korvold, Skullclamp), so punishers hit hard: Underworld Dreams, Fate Unraveler, then Sheoldred. Kill Korvold. Your life gain out-races their pings. Massacre Wurm clears the tokens | Underworld Dreams, Fate Unraveler, Massacre Wurm, Foolish Fate, Overkill, Boots |
| Marwyn, the Nurturer (t3, mono-green elves) | A board of mana elves, then Craterhoof Behemoth once Gaea's Cradle and the elves are out (Natural Order, Green Sun's Zenith, Chord of Calling, Finale of Devastation find it). Killing Craterhoof after it enters doesn't stop its trigger | Kill the mana elves early and sweep before the swing. They draw little, so the punishers are weaker here: lean on the creatures and Exsanguinate. Fellwar Stone may only make green mana, so count it as colorless | Massacre Wurm (elves have toughness 2 or less, and each costs 2 life), Foolish Fate, Silence the Echo, Grave Pact, Sheoldred as a blocker (Craterhoof tramples over it) |
| Tergrid, God of Fright (t3, mono-black discard and edicts) | Tergrid takes every nontoken permanent you sacrifice or discard. Edicts (Sheoldred's Edict, Plaguecrafter, Fleshbag Marauder, Innocent Blood), Liliana of the Veil, Waste Not (their gain when you discard), Painful Quandary. Their deck has its own Sheoldred (your draws cost you 2), Whispering One, Gray Merchant, Phyrexian Obliterator. Dauthi Voidwalker and Bojuka Bog hate the graveyard | Never sacrifice or discard a nontoken permanent while Tergrid is out (Deadly Dispute, Silence the Echo, Phyrexian Tower, Geier Reach Sanitarium, Mind Stone). Edicts take your choice of creature, so keep tokens and cheap bodies (Sheoldred's Edict can force a nontoken one). Play out your hand before Liliana. Kill their Sheoldred (Animate Dead takes it from a graveyard). Keep your hand to 7 | Zombie tokens (Grave Titan, Dreadhorde Invasion), Undercity Dire Rat, Foolish Fate, Overkill, Animate Dead, Underworld Dreams |
| Chulane, Teller of Tales (t4, Bant value and Aluren) | Chulane draws and ramps off every creature spell. Aluren or Intruder Alarm with Shrieking Drake or Whitemane Lion loops creatures, draws the deck and wins with Thassa's Oracle, Laboratory Maniac or Craterhoof. Counterspell, Dovin's Veto, Swan Song, Fierce Guardianship; Cyclonic Rift; Drannith Magistrate (you can't cast Sheoldred from the command zone or the graveyard) | A punisher is your answer to the loop: with Sheoldred out each card they draw costs 2, so drawing the deck kills them. Cast creature spells into counterspell decks (Veto, Swan Song and Fierce can't hit creatures). Kill Chulane, the bouncers and Drannith Magistrate. You have almost no answer to their enchantments: race | Sheoldred, Underworld Dreams, Fate Unraveler, Foolish Fate, Overkill, Massacre Wurm (Magistrate and the small creatures), creature finishers (Gray Merchant, Mikaeus) |
| Edgar Markov (t4, Mardu Vampires) | Eminence makes a 1/1 Vampire for every Vampire spell cast, from the command zone. Wide first-strike attacks (Edgar, Drana). Their own Sanguine Bond, Vito and Exquisite Blood loops. Cheap removal (Swords, Path, Bolt, Terminate, Fatal Push, Anguished Unmaking), Damn, New Blood, Teferi's Protection | Massacre Wurm costs them 2 per Vampire token. Sheoldred and Grave Pact hold the board. Do not pay life (Phyrexian Arena, The One Ring, Ad Nauseam, Grim Tutor, K'rrik's 2 life per {B}) when their Exquisite Blood or Bloodthirsty Conqueror is out beside Sanguine Bond, Vito or Marauding Blight-Priest: your life loss starts their loop. Kill Vito or Blight-Priest first | Massacre Wurm, Grave Pact, Accursed Marauder, Overkill, Foolish Fate, Mikaeus (undying against their wipes), Boots |
| Krenko, Mob Boss (t4, mono-red goblins) | Krenko doubles the Goblins each activation; Impact Tremors, Goblin Bombardment, Purphoros, Coat of Arms; haste from Lightning Greaves and Swiftfoot Boots. Thornbite Staff + an outlet is infinite. Blood Moon turns your nonbasic lands into Mountains. The sims show Krenko ended about 12% of games | Kill Krenko before he untaps (Foolish Fate and Overkill are instants). Massacre Wurm answers the whole Goblin board. Keep Sheoldred home as a blocker. Under Blood Moon only the 30 Swamps make {B} (Cabal Coffers and Barad-dûr don't) | Massacre Wurm, Foolish Fate, Overkill, Grave Pact, Accursed Marauder, Mikaeus (undying against Bombardment) |
| Prosper, Tome-Bound (t4, Rakdos exile and Treasure) | Exiles cards to play them and makes a Treasure each time; Opposition Agent (steals your tutors); Bolas's Citadel + Sensei's Divining Top + Aetherflux Reservoir; Underworld Breach; Toxic Deluge, Blasphemous Act, Delayed Blast Fireball. The sims show the combo ended about 9% of games | They draw less than most, so the punishers are weaker: lean on the drain creatures and Exsanguinate. Kill Opposition Agent on sight and don't tutor under it. Kill Prosper and the Treasure drainers (Marionette Master). Dauthi Voidwalker and Bojuka Bog shut off Line 6 | Foolish Fate, Overkill, Massacre Wurm (kills the Agent), Gray Merchant, Exsanguinate |
| Yuriko, the Tiger's Shadow (t4, Dimir ninjas) | Cheap evasive attackers (Ornithopter, Faerie Seer, Spectral Sailor), then ninjutsu Yuriko or another Ninja: each Ninja hit reveals a card and you lose life equal to its mana value (Shadow of Mortality 15, Temporal Trespass 11). Counterspell, Force of Will, Fierce Guardianship, Flare of Denial, Misdirection; Cyclonic Rift; Rhystic Study. The sims show Yuriko ended about 7% of games | Keep blockers back: their evasive creatures are small, and Cover of Darkness gives fear, which black creatures like yours can block. Massacre Wurm kills the 1/1 attackers. They draw a lot (Brainstorm, Rhystic Study, Top), so punishers hit hard. Cast creatures into the counters. Pay for Rhystic Study only on the spell that matters | Massacre Wurm, Sheoldred and Rooftop Assassin as blockers, Underworld Dreams, Fate Unraveler, Foolish Fate |

### After a stopped attempt

| What happened | Rebuild |
|---|---|
| Sheoldred destroyed | Animate Dead ({1}{B}, no life) or Phyrexian Delver, or Yawgmoth's Will, if she is in the graveyard. Otherwise recast her from the command zone for {2} more per previous cast |
| Sheoldred exiled or bounced, or a wipe | Recast her from the command zone (or from hand). Meanwhile Underworld Dreams, Fate Unraveler, Phyrexian Arena and the drain creatures carry the damage; hold the symmetric draw cards |
| Mikaeus or Triskelion answered | Tutor for the replacement. Mikaeus's undying still has to be on the battlefield for the loop. If Triskelion was exiled, Mikaeus alone still gives undying to the rest |
| Exquisite Blood, Bond or Vito answered | Tutor for the missing piece. Vito is the cheapest (3 mana). Cast Blood last, once the table is tapped |
| Peer into the Abyss countered or fizzled | Exsanguinate, or Gray Merchant off Animate Dead, as the table-wide finish. Peer is a sorcery: wait until the table is tapped out |
| Board wiped | Animate Dead (Grave Titan from the graveyard, if you used Entomb), Phyrexian Delver, Perigee Beckoner. Underworld Dreams, Arena, Bond and Blood survive a creature wipe |
| Graveyard hate (Bojuka Bog, Voidwalker, Farewell) | Switch to hard casting: tutors, Rooftop Assassin, Massacre Wurm; Line 6 is off |
| Counterspell on the key piece | Hold the second piece. Cast a creature next (the counters are mostly noncreature), or wait for a turn when they tap out |
| K'rrik answered | Costs already paid stay paid. Count your mana again before the next black spell: the {B} pips are back to mana. Recast it for {4} and 6 life if your life allows, or bring it back with Animate Dead ({1}{B}) |
| Life too low to run the engines | Stop The One Ring, Phyrexian Arena, Ad Nauseam and the tutors that cost life. Draw with Sheoldred first, or gain with Exsanguinate or Gray Merchant |

- A failed attempt shows the table your plan. The next try needs a second line ready.

### Life management

| Source | Cost | What gains it back |
|---|---|---|
| Phyrexian Arena | 1 per upkeep | Its card (2 life with Sheoldred) |
| Dreadhorde Invasion | 1 per upkeep | Nothing; it makes an Army body |
| The One Ring | 1 per burden counter, each upkeep | Each draw is 2 life with Sheoldred |
| Call of the Ring | 2 per Ring-bearer chosen, to draw | The card (2 with Sheoldred) |
| Seizan | 2 in your upkeep | The 2 cards (4 with Sheoldred) |
| Cost of Brilliance (aimed at you) | 2 | The 2 cards (4 with Sheoldred) |
| Ambition's Cost | 3 | The 3 cards (6 with Sheoldred) |
| Grim Tutor | 3 | Nothing: the card is put into your hand, not drawn |
| Ad Nauseam | Each card's mana value (about 2 a card on average) | Nothing: put into your hand, not drawn |
| Bolas's Citadel | The mana value of each spell played from the top | Nothing |
| Phyrexian Delver | The mana value of the creature (Grave Titan 6, Sheoldred 4) | Sheoldred, Gray Merchant or Exsanguinate |
| K'rrik, Son of Yawgmoth | 6 to cast it, then 2 per {B} paid with life | Lifelink on K'rrik, Gray Merchant, Exsanguinate, Sheoldred's draws |
| Yawgmoth, Thran Physician | 1 per activation | The card drawn (2 with Sheoldred) |
| Gix | 1 per card drawn off combat damage | The card (2 with Sheoldred) |

Sources of life: Sheoldred (2 per draw), Exsanguinate, Gray Merchant, Blood Artist, Zulaport Cutthroat,
Sephiroth, Sephiroth's Intervention (2), Rooftop Assassin (lifelink), Vito's lifelink ability
({3}{B}{B}), Exquisite Blood, and the lifelink on a grown Dreadhorde Army.

**Rules of thumb.** Without Sheoldred, stop The One Ring, Ad Nauseam and Grim Tutor, and keep your life well above what
the table can swing in one round (the sims show the t3 table can kill from about 12). Check Phyrexian Delver's cost before you
cast it: Grave Titan is 6. Against Edgar's loop pieces, do not pay life at all. With K'rrik out, add up the life for the
whole turn before the first spell (each {B} is 2) and leave yourself at least 15 unless the turn wins.

### Common mistakes

1. Playing Howling Mine, Seizan, Memory Jar, Mikokoro or the Sanitarium before a punisher is out. They refuel the table.
2. Tapping out for the engines while Sheoldred is your only engine and the table has removal or a wipe. Boots first.
3. Treating Bond + Vito as a combo. The loop needs Exquisite Blood.
4. Casting Exquisite Blood or Vito into untapped removal. One answer breaks the loop; go when the table is tapped, and
   keep a second line.
5. Paying life (Arena, Ad Nauseam, Grim Tutor, The One Ring) without Sheoldred, or against Edgar's Exquisite Blood pieces.
6. Tutoring while an opponent controls Opposition Agent.
7. Sacrificing or discarding a nontoken permanent while Tergrid is out (Deadly Dispute, the Sanitarium),
   or ending the turn with 8 cards.
8. Putting Sheoldred in the command zone when you hold Animate Dead, Phyrexian Delver or Yawgmoth's Will, which bring her back
   cheaper, and paying the tax when she is already in the graveyard.
9. Playing Barad-dûr late: it enters tapped unless you control a legendary creature. Play it on turn 1 or 2.
10. Counting on Mikaeus's undying for Humans (Zulaport Cutthroat, Witch of the Moors, Vault Plunderer, Leech Collector,
    Sephiroth, Yawgmoth), or casting Triskelion before Mikaeus is safe.
11. Paying life with K'rrik at low life, against a burn or aggro table, or for a spell that gets countered. The life does
    not come back.

## Mana

36 lands: 30 Swamp, Barad-dûr, Cabal Coffers, Phyrexian Tower, Bonders' Enclave, Mikokoro, Center of the Sea and Geier
Reach Sanitarium. Thirty-one lands make {B} outright (30 Swamps and Barad-dûr, which enters tapped unless you control a
legendary creature). **Cabal Coffers** ({2}, {T}: add {B} for each Swamp you control) needs no help from other land
types: with 30 Swamps, six Swamps out make it net {B}{B}{B}{B}, more every turn, for Exsanguinate and the big turns.
Phyrexian Tower makes {C}, or {B}{B} by sacrificing a creature. Bonders' Enclave, Mikokoro and the Sanitarium make {C}
and draw cards.

Other mana: three rocks (Arcane Signet, which makes {B}; Fellwar Stone, which makes a colour an opposing land could
produce, so it may not make {B}; Mind Stone), Dark Ritual, Ashnod's Altar, and the Treasures from Undercity Dire Rat and
Deadly Dispute, and K'rrik, which pays each {B} with 2 life. Tutors: Demonic Tutor, Diabolic Tutor and
Grim Tutor, plus Entomb. Interaction: five removal spells, Nevinyrral's Disk, Massacre Wurm, Accursed Marauder, Rooftop Assassin and Grave Pact; protection: Swiftfoot Boots and
Perigee Beckoner (and Mikaeus's undying).

## Bracket and Rule 0

Bracket 4: four Game Changers (Ad Nauseam, The One Ring, Demonic Tutor, Bolas's Citadel) and the combos named here.
There are no extra turns and no mass land destruction, but there are infinite combos: Mikaeus + Triskelion is a
two-card infinite-damage combo (Commander Spellbook lists it), Mikaeus + Yawgmoth + Gray Merchant of Asphodel is a
three-card loop (it needs a second nontoken non-Human creature with toughness 2 or more), Exquisite Blood + Sanguine
Bond (or + Vito, Thorn of the Dusk Rose) is a two-card infinite drain, and Sheoldred + Peer into the
Abyss (or Underworld Dreams + Peer into the Abyss) is a near-instant kill on an opponent whose library is large
compared with their life. Name these before the game. Tell the table about the four Game
Changers, and tell it that Howling Mine, Memory Jar and the other draw cards are symmetric on purpose: they make everyone
draw, and Sheoldred, Underworld Dreams and Fate Unraveler are what make that cost the opponents.

## Decklist by type (100)

**Commander (1).** Sheoldred, the Apocalypse

**Lands (36).** Barad-dûr, Bonders' Enclave, Cabal Coffers, Geier Reach Sanitarium, Mikokoro Center of the Sea, Phyrexian Tower, 30 Swamp

**Creatures (24).** Accursed Marauder, Blood Artist, Fate Unraveler, Gix Yawgmoth Praetor, Grave Titan, Gray Merchant of Asphodel, K'rrik Son of Yawgmoth, Leech Collector // Bloodletting, Massacre Wurm, Mikaeus the Unhallowed, Ob Nixilis the Fallen, Perigee Beckoner, Phyrexian Delver, Rooftop Assassin, Seizan Perverter of Truth, Sephiroth Fabled SOLDIER // Sephiroth One-Winged Angel, Triskelion, Undercity Dire Rat, Vault Plunderer, Viscera Seer, Vito Thorn of the Dusk Rose, Witch of the Moors, Yawgmoth Thran Physician, Zulaport Cutthroat

**Enchantments (8).** Animate Dead, Call of the Ring, Dreadhorde Invasion, Exquisite Blood, Grave Pact, Phyrexian Arena, Sanguine Bond, Underworld Dreams

**Artifacts (11).** Arcane Signet, Ashnod's Altar, Bolas's Citadel, Fellwar Stone, Howling Mine, Memory Jar, Mind Stone, Nevinyrral's Disk, Skullclamp, Swiftfoot Boots, The One Ring

**Sorceries (13).** Ambition's Cost, Cost of Brilliance, Demonic Tutor, Diabolic Tutor, Exsanguinate, Grim Tutor, Invoke Despair, Peer into the Abyss, Reanimate, Silence the Echo, Syphon Mind, Torment of Hailfire, Yawgmoth's Will

**Instants (7).** Ad Nauseam, Dark Ritual, Deadly Dispute, Entomb, Foolish Fate, Overkill, Sephiroth's Intervention

## Import list (100)

```
1 Accursed Marauder
1 Ad Nauseam
1 Ambition's Cost
1 Animate Dead
1 Arcane Signet
1 Ashnod's Altar
1 Barad-dûr
1 Blood Artist
1 Bolas's Citadel
1 Bonders' Enclave
1 Cabal Coffers
1 Call of the Ring
1 Cost of Brilliance
1 Dark Ritual
1 Deadly Dispute
1 Demonic Tutor
1 Diabolic Tutor
1 Dreadhorde Invasion
1 Entomb
1 Exquisite Blood
1 Exsanguinate
1 Fate Unraveler
1 Fellwar Stone
1 Foolish Fate
1 Geier Reach Sanitarium
1 Gix, Yawgmoth Praetor
1 Grave Pact
1 Grave Titan
1 Gray Merchant of Asphodel
1 Grim Tutor
1 Howling Mine
1 Invoke Despair
1 K'rrik, Son of Yawgmoth
1 Leech Collector // Bloodletting
1 Massacre Wurm
1 Memory Jar
1 Mikaeus, the Unhallowed
1 Mikokoro, Center of the Sea
1 Mind Stone
1 Nevinyrral's Disk
1 Ob Nixilis, the Fallen
1 Overkill
1 Peer into the Abyss
1 Perigee Beckoner
1 Phyrexian Arena
1 Phyrexian Delver
1 Phyrexian Tower
1 Reanimate
1 Rooftop Assassin
1 Sanguine Bond
1 Seizan, Perverter of Truth
1 Sephiroth's Intervention
1 Sephiroth, Fabled SOLDIER // Sephiroth, One-Winged Angel
1 Sheoldred, the Apocalypse
1 Silence the Echo
1 Skullclamp
1 Swiftfoot Boots
1 Syphon Mind
1 The One Ring
1 Torment of Hailfire
1 Triskelion
1 Undercity Dire Rat
1 Underworld Dreams
1 Vault Plunderer
1 Viscera Seer
1 Vito, Thorn of the Dusk Rose
1 Witch of the Moors
1 Yawgmoth's Will
1 Yawgmoth, Thran Physician
1 Zulaport Cutthroat
30 Swamp
```
