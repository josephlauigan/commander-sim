# Sheoldred, the Apocalypse — Mono-Black Drain

A mono-black deck that makes the whole table draw cards while Sheoldred, the Apocalypse punishes every one of them. It
wins by draining: three opponents' draw steps cost them 6 life a round with Sheoldred out, more with Underworld Dreams,
Fate Unraveler, Orcish Bowmasters and Psychosis Crawler beside her, and Exsanguinate, Peer into the Abyss, Torment of
Hailfire and the Exquisite Blood drain combo close the game. It is built in five packages: punish and make them draw; own
draw and life; drain finishers; mana, tutors and K'rrik; interaction. It is weak when Sheoldred is exiled or keeps
getting removed, when fast combat decks get going before the engines do, and when the symmetric draw cards are played
with no punisher out, because they then just refuel the table.

| | |
|---|---|
| **Commander** | Sheoldred, the Apocalypse ({2}{B}{B}, 4/5 deathtouch, black) |
| **Bracket** | 4 (4 Game Changers: The One Ring, Demonic Tutor, Orcish Bowmasters, Vampiric Tutor; Exquisite Blood or Bloodthirsty Conqueror with Sanguine Bond, Enduring Tenacity, Vito or Marauding Blight-Priest is an infinite drain) |
| **Simulator** | deck key `sheoldred`; results in `decklists/pool/pool-results.md`, history in `decklists/CHANGELOG.md` |

## Strategy

### The commander

**Sheoldred, the Apocalypse** ({2}{B}{B}, 4/5). *Deathtouch. Whenever you draw a card, you gain 2 life. Whenever an
opponent draws a card, they lose 2 life.*

She is the deck's one big engine, and she does four jobs:

- **She drains the table.** Each of three opponents loses 2 life in their own draw step, so 6 a round from the draw steps
  alone. Every extra card an opponent draws (Howling Mine, Seizan, Temple Bell, Memory Jar, Painful Lesson, Rhystic Study,
  Peer into the Abyss) is 2 more.
- **She gains life.** Every card you draw is 2 life. That pays for the deck's life costs (The One Ring, Phyrexian Arena,
  Night's Whisper, Sign in Blood, Black Market Connections, Snuff Out) and feeds the lifegain payoffs: Sanguine Bond,
  Enduring Tenacity, Vito, Marauding Blight-Priest, Foolish Fate, Leech Collector.
- **She blocks and attacks.** A 4/5 deathtouch body holds off nearly any attacker. Bonders' Enclave counts her as a
  creature with power 4 or more.
- **She is the thing the table will answer.** Everything in the deck that makes the table draw is only good with a
  punisher out. Cast her first, protect her, then start the engines.

**Where she is.** She starts in the command zone, so no tutor can find her and she is never in your opening hand. Each
recast from the command zone costs {2} more per previous cast. When she dies you choose, as she hits the graveyard,
whether to put her back in the command zone. Leave her in the graveyard when you hold Yawgmoth's Will (it lets you cast
her from the graveyard for {2}{B}{B} with no tax) or have Witch of the Moors on the battlefield (at your end step, if you
gained life this turn, it returns a creature card from your graveyard to your hand, and she is then cast from hand for
{2}{B}{B}). With neither, move her to the command zone and pay the tax.

**When to cast her.** Turn 2 with Sol Ring on turn 1 and two Swamps (two lands make {B}{B}, Sol Ring makes {C}{C}), or
with Dark Ritual and two Swamps. Turn 3 with a rock and three lands (Arcane Signet, Mind Stone, Thought Vessel, Charcoal
Diamond), turn 4 off four lands, or with a turn-2 Jet Medallion ({1} off her {2}{B}{B}). A rock plus three lands is a
real edge, so the deck runs five rocks, Sol Ring among them. Cast her before any symmetric draw card. If the table is
holding up removal mana, cast her anyway: she makes them spend it, and Underworld Dreams, Fate Unraveler, Orcish
Bowmasters and the drain pieces don't need her.

### K'rrik, Son of Yawgmoth

**K'rrik, Son of Yawgmoth** ({4}{B/P}{B/P}{B/P}, 2/2, legendary, lifelink). *For each {B} in a cost, you may pay 2 life
rather than pay that mana. Whenever you cast a black spell, put a +1/+1 counter on K'rrik.* Each {B/P} in its own cost
is {B} or 2 life, so it casts for {4} and 6 life: a four-mana play whatever your lands make. It is the deck's life-to-mana
converter, and the deck has the life for it (Sheoldred, Gray Merchant, Exsanguinate, Bloodthirsty Conqueror, lifelink).

Once it is on the battlefield, every {B} in any cost can be paid with 2 life, in spells and in activated abilities.
Generic mana, {X} and colourless costs are not reduced.

| Card | Cost | With K'rrik out (all {B} paid with life) |
|---|---|---|
| Vampiric Tutor | {B} | 2 life, plus the 2 life it costs |
| Dark Ritual | {B} | 2 life, and it still makes {B}{B}{B} |
| Sign in Blood | {B}{B} | 4 life, plus the 2 life it costs |
| Demonic Tutor, Night's Whisper, Infernal Grasp, Feed the Swarm, Go for the Throat, Bitter Triumph | {1}{B} | {1} and 2 life (plus each card's own life cost) |
| Deadly Rollick, Ill-Gotten Inheritance | {3}{B} | {3} and 2 life (Deadly Rollick is free while you control a commander) |
| Yawgmoth's Will, Foolish Fate, Overkill, Painful Lesson, Toxic Deluge | {2}{B} | {2} and 2 life |
| Grim Tutor | {1}{B}{B} | {1} and 4 life, plus the 3 life it costs |
| Diabolic Tutor, Enduring Tenacity | {2}{B}{B} | {2} and 4 life |
| Sheoldred | {2}{B}{B} | {2} and 4 life |
| Underworld Dreams | {B}{B}{B} | 6 life, no mana |
| Exquisite Blood | {4}{B} | {4} and 2 life |
| Sanguine Bond, Gray Merchant, Polluted Bonds, Bloodgift Demon, Bloodthirsty Conqueror | {3}{B}{B} | {3} and 4 life |
| Peer into the Abyss | {4}{B}{B}{B} | {4} and 6 life |
| Exsanguinate, Torment of Hailfire | {X}{B}{B} | {X} and 4 life: X is 2 higher for the same mana |
| Damnable Pact | {X}{B}{B} | {X} and 4 life, plus the X life it costs if you aim it at yourself |
| Invoke Despair | {1}{B}{B}{B}{B} | {1} and 8 life |
| Castle Locthwain's draw | {1}{B}{B} | {1} and 4 life (plus the life it costs) |

Not reduced: Sol Ring, the other rocks, Howling Mine, Temple Bell, Memory Jar, Psychosis Crawler, The One Ring,
Swiftfoot Boots and the equip cost, Cabal Coffers' {2}. Snuff Out's {3}{B} is never paid when you pay its 4 life instead.

- **What it does for the plan.** A hand of Gray Merchant, Sanguine Bond and Bloodthirsty Conqueror that three lands can't cast
  becomes castable, a rock-less turn 4 can cast Sheoldred and a tutor in the same turn, and the kill turn needs fewer
  lands: Exquisite Blood plus Sanguine Bond is 7 mana and 6 life instead of 10 mana, and Exquisite Blood plus Enduring
  Tenacity is 6 mana and 6 life instead of 9 (every {B} paid with life).
- **Three more jobs.** It is a lifelink body, and every point it gains is a Sanguine Bond, Enduring Tenacity, Vito or
  Marauding Blight-Priest trigger. It counts 3 toward devotion for Gray Merchant (each Phyrexian {B/P} symbol is a black
  symbol). Every black spell you cast puts a +1/+1 counter on it, so it grows into a real attacker.
- **Dark Ritual on turn 2.** With two lands, Dark Ritual makes {B}{B}{B}, the second land adds {B}, and K'rrik costs
  {4} and 6 life. Sheoldred then costs {2} and 4 life on turn 3.
- **When paying life is safe.** Sheoldred is out (2 back per card you draw), K'rrik or another lifelinker is attacking,
  the spell you pay for gains the life back (Gray Merchant, Exsanguinate, Sheoldred), your life is high (about 25 or
  more before paying), and the table has no burn, no go-wide aggro and no Exquisite Blood plus Sanguine Bond or Vito
  of its own.
- **When it is not.** Your life is low, the table is red or a fast aggro deck, Edgar's loop pieces are out, you are
  already paying for Phyrexian Arena, The One Ring or Grim Tutor with no Sheoldred, or the spell can be countered (life
  is paid on casting and does not come back). Peer into the Abyss halves the *target's* life, not yours, so its 6 life
  is a plain cost, but check your total after it. Ambition's Cost also takes life, and so do Damnable Pact aimed at
  you with no Sheoldred out and Toxic Deluge, so don't stack them on a K'rrik turn.
- **Which {B} to pay.** Pay life only for the pips you can't pay with mana. Count your lands first, then use life to
  close the gap, not as a default.

### How the deck wins

Four lines, plus a replayed turn. Line 1 is the combo; 2 and 3 are one-turn kills; 4 is the slow grind that needs no
single card; the Yawgmoth's Will turn doubles whichever kill you already cast. The deck has no counterspells, so every
line is protected by timing and redundancy (see "Protecting Sheoldred and the win turn").

#### Line 1: a Blood half + a Bond half

Two kinds of card make the loop.

- **Blood halves** (an opponent loses life, you gain that much): **Exquisite Blood** ({4}{B}) and **Bloodthirsty
  Conqueror** ({3}{B}{B}, 5/5 flying deathtouch).
- **Bond halves** (you gain life, an opponent loses life): **Sanguine Bond** ({3}{B}{B}), **Enduring Tenacity**
  ({2}{B}{B}, 4/3 enchantment creature), **Vito, Thorn of the Dusk Rose** ({2}{B}, 1/3; target opponent loses that much)
  and **Marauding Blight-Priest** ({2}{B}, 3/2; each opponent loses 1 whenever you gain life).

1. Get one Blood half and one Bond half onto the battlefield: two Blood halves times four Bond halves is eight pairs.
   **A Blood half is required.** Two Bond halves together don't loop, and neither do two Blood halves: each only takes
   life from the opponent, and nothing gives you life back.
2. Any life gain or opponent life loss now starts the loop. The Blood half gains you N, the Bond half takes N from an
   opponent, the Blood half gains N again. It repeats until every opponent is dead (Bond, Tenacity and Vito pick a new
   target as they die; Blight-Priest hits every opponent).
3. Cheap starters: Sheoldred's trigger in anyone's draw step; **Polluted Bonds** ({3}{B}{B}: whenever a land an
   opponent controls enters, that player loses 2 life and you gain 2) on any opposing land drop, fetch land or ramp
   spell; **Psychosis Crawler** (each of your draws makes each opponent lose 1); Leech Collector's Bloodletting; Gray
   Merchant; Sephiroth's Intervention (gain 2); **Ill-Gotten Inheritance** (each of your upkeeps: 1 damage to each opponent, and you gain 1); a
   draw off Mind Stone with Sheoldred out.

Mana: Blood 5, Conqueror 5, Bond 5, Tenacity 4, Vito 3, Blight-Priest 3. The cheapest pair is a Blood half and Vito or
Blight-Priest (8 mana across two turns, or one turn with Cabal Coffers).

- **Order.** Play a cheap Bond half early (Vito or Blight-Priest on turn 3): with Sheoldred out every card you draw takes
  life from an opponent, and it is bait for creature removal that costs the table a card. Sanguine Bond is useful alone
  too, and as an enchantment most of the table's removal can't hit it. The Blood half goes last, as the kill piece, on a
  turn when the table is tapped low. Pick it by what the table holds: Exquisite Blood is an enchantment (it dodges
  creature removal and sweepers), Bloodthirsty Conqueror is a creature (it dodges counters aimed at noncreature spells
  and enchantment removal). Creature removal, sacrifice and edicts only delay Tenacity: its first death returns it as an
  enchantment that is not a creature, and it still drains. Enchantment removal, exile and bounce are what answer it (a
  bounced or blinked copy is a creature again).
- **One answer does not end it, but it ends the loop.** Destroy any piece while a trigger is on the stack and the loop
  stops after that trigger. The deck has two Blood halves and four Bond halves, so a second half can usually be cast or
  tutored the next turn.
- **Exsanguinate on top.** With a Bond half out, an Exsanguinate for X gains you 3X (one gain), and the Bond half
  turns that into 3X from one opponent.

#### Line 2: Peer into the Abyss

**Peer into the Abyss** ({4}{B}{B}{B}, sorcery): *Target player draws cards equal to half the number of cards in their
library and loses half their life. Round up each time.*

1. With Sheoldred out, cast it on one opponent. They draw about half their library, and each card is 2 life.
2. The maths: with N cards in the library and L life, they lose 2 for each of N/2 cards (about N) plus half of L. That is
   lethal when N + L/2 is at least L, so a library of half their life total or more. At 40 life, a library of 20
   or more. Mid-game libraries are 70 or more, so it kills any one opponent.
3. With Underworld Dreams alone it is 1 a card: lethal when the library is at least their life total. With Dreams and
   Fate Unraveler it is 2 a card, the same as Sheoldred.
4. With Orcish Bowmasters out, each of those cards (Peer is cast on your own turn) is also 1 damage to any target and an Orc
   Army counter.

Seven mana: Cabal Coffers, rocks and Dark Ritual get there. Demonic, Diabolic or Vampiric Tutor finds it. It is a
sorcery, so cast it when the table can't answer in response: a counterspell stops it, and Teferi's Protection (Aurelia,
Edgar) stops the life loss. Peer on one opponent leaves the other two untouched, so it is the first kill of three.

#### Line 3: Exsanguinate or Torment of Hailfire off Cabal Coffers

**Cabal Coffers** ({2}, {T}: add {B} for each Swamp you control). With 30 Swamps it is big mana by the mid-game.

| Lands in play (Coffers + Swamps) | Mana from lands | Exsanguinate X |
|---|---|---|
| 6 (5 Swamps) | 8 | 6 |
| 8 (7 Swamps) | 12 | 10 |
| 10 (9 Swamps) | 16 | 14 |

(Every other land is a Swamp, and Swamps pay the {2}; add rock mana on top. With **Urborg, Tomb of Yawgmoth** out every
land is a Swamp, Coffers and Urborg included, so every land you control counts.)

- **Exsanguinate** ({X}{B}{B}): *Each opponent loses X life. You gain life equal to the life lost this way.* X = 10 with
  three opponents is 30 life gained. Alone it is a big hit on the table; with Sanguine Bond it is 30 off one opponent;
  with a Blood half it also starts Line 1.
- **Torment of Hailfire** ({X}{B}{B}): *Repeat X times: each opponent loses 3 life unless they sacrifice a nonland
  permanent or discard a card.* A big X either kills or strips the table of permanents and cards.
- **Gray Merchant of Asphodel** ({3}{B}{B}): each opponent loses life equal to your devotion to black, and you gain it.
  Sheoldred (2), Underworld Dreams (3), K'rrik (3), Phyrexian Arena (2), Seizan (2), Enduring Tenacity (2), Polluted
  Bonds (2) and the Merchant (2) all count, so with Sheoldred and Underworld Dreams out it is at least 7
  from each, and 10 with K'rrik beside them.
- **K'rrik** pays the {X}{B}{B} of Exsanguinate and Torment of Hailfire with 4 life, so the same mana gives an X 2
  higher (8, 12 and 16 in the table above, with rocks on top).
- **Yawgmoth's Will** turns a spent Exsanguinate, Torment of Hailfire or Gray Merchant into a second one the same turn.

#### Line 4: the fair win, draw engines and drain

Sheoldred with Howling Mine, Seizan, Temple Bell, Underworld Dreams and Fate Unraveler drains each opponent every round.
Orcish Bowmasters turns each extra draw into a ping and an Orc Army counter (the first card an opponent draws in their
own draw step is the only one it ignores), and Psychosis Crawler turns each of your own draws into 1 life from each
opponent. Memory Jar with Sheoldred out is 14 from each opponent in one turn, and with Bowmasters out each of its seven draws is a ping. Bloodgift Demon, Ob Nixilis and the flying drain creatures keep the pressure on when the engines are
answered. Drain is the deck's main source of eliminations.

### Game plan

**Turns 1-3.** A rock and cheap plays; Sheoldred on turn 2 to 4.

- Turn 1: Sol Ring, Charcoal Diamond (it enters tapped, so play it when you have nothing to cast), a Swamp, Castle
  Locthwain or Urborg (Castle enters tapped unless you control a Swamp), or Dark Ritual only for a payoff you can cast.
- Turn 2: a rock (Arcane Signet, Mind Stone, Thought Vessel) or Jet Medallion, or a two-drop: Orcish Bowmasters (flash,
  so it can wait for an extra draw), Night's Whisper, Sign in Blood, Leech Collector, Vampiric Tutor at an opponent's end
  step.
- Turn 3: Sheoldred off a rock, or Underworld Dreams (a good turn-3 play on its own: every card an opponent draws costs
  them 1), Phyrexian Arena, Gix, Vito or Marauding Blight-Priest.

**Turns 4-6.** Sheoldred first if she isn't out. Then, in order: equip Swiftfoot Boots, a punisher beside her
(Underworld Dreams, Fate Unraveler, Orcish Bowmasters), then the draw engines (Howling Mine, Seizan, Temple Bell, Memory
Jar, the end-step lands), then the drain pieces (Sanguine Bond, Exquisite Blood, Enduring Tenacity, Bloodthirsty
Conqueror). K'rrik fits here once Sheoldred is out and your life is high: cast it for {4} and 6 life, and the rest of the
turn's black spells cost life instead of mana.

**Late.** Run the engines, draw with The One Ring, Black Market Connections and Damnable Pact aimed at yourself, and take
the kill that fits: Line 1 if both pieces are out, Peer into the Abyss if Sheoldred is safe, Exsanguinate off Cabal
Coffers for the table. With K'rrik out, every {B} in the kill pieces costs 2 life instead of a mana: spend the saved
mana on the second piece.

**Hold up or go.**

- **Go** when the table is tapped low or holding one removal spell at most, when the kill is two pieces and you have
  both, or when someone else is about to win.
- **Hold** the symmetric draw cards (Howling Mine, Seizan, Memory Jar, Temple Bell, Mikokoro, Geier Reach Sanitarium,
  Painful Lesson aimed at an opponent) until a punisher is out. Hold Memory Jar until she is safe; it gives every
  opponent seven cards for the turn.
- **Hold mana at the end of the opponent's turn** for Mikokoro, the Sanitarium or Temple Bell, Orcish Bowmasters or
  Rooftop Assassin, and instant removal. Snuff Out (4 life, no mana) is the answer you can hold on a tapped-out turn.

### The cards by role

#### Punish their draws

Sheoldred (2 life a card), **Underworld Dreams**, **Fate Unraveler** (1 damage a card each) and **Orcish Bowmasters**
(1 damage and an Orc Army counter for each draw except the first in their draw step) stack: with Sheoldred, Dreams and
Fate Unraveler out, every card an opponent draws costs them 4. Underworld Dreams, Fate Unraveler and Bowmasters work
without Sheoldred, so they keep the pressure on while she is dead or not yet cast. Fate Unraveler is also a 4-mana
creature. **Psychosis Crawler** ({5}, power and toughness equal to the cards in your hand) punishes your own draws
instead: each card you draw makes each opponent lose 1 life.

#### Make them draw

These turn the punishers on. Play the symmetric ones only once a punisher is out.

- **Howling Mine** and **Temple Bell**: each player draws an extra card in their draw step (Mine), or each player draws
  a card when you tap the Bell ({T}, any time you could activate it). Each opponent draws two a turn with the Mine (4
  life with Sheoldred) while you draw two too (4 life back).
- **Mikokoro, Center of the Sea** and **Geier Reach Sanitarium** (lands): each player draws, at will. Mikokoro: {2},
  {T}, each player draws a card. The Sanitarium: {2}, {T}, each player draws then discards. Use them at an opponent's end
  step so you keep your mana.
- **Seizan, Perverter of Truth**: in each player's upkeep that player loses 2 life and draws two cards. With Sheoldred
  out each opponent loses 6 per upkeep, and you take 2 and draw two (net +2 life).
- **Bloodgift Demon** ({3}{B}{B}, 5/4 flier): at the beginning of your upkeep, target player draws a card and loses 1
  life. Aimed at an opponent with Sheoldred out that is 3 life a turn; aimed at yourself it is a card for 1 life.
- **The wheel**: **Memory Jar** ({T}, sacrifice: each player exiles their hand and draws seven, then at the next end step
  discards and gets the old hand back) is 14 life from each opponent with Sheoldred out, and you gain 14. Cast it and
  use it in the same turn, in your second main phase.
- **Hand one opponent cards**: **Painful Lesson**, **Cost of Brilliance** and **Sign in Blood** each make the target
  player draw two and lose 2 life (Cost of Brilliance adds a +1/+1 counter on a creature), and **Vault Plunderer**
  (target player draws one and loses 1). With Sheoldred out these are 6 or 3 life against the chosen opponent.
  **Damnable Pact** is the same effect for X cards (see "Draw your own"): never aim any of them at an opponent when no
  punisher is out. **Peer into the Abyss** is the big one (Line 2).

#### Draw your own

Each card you draw is 2 life with Sheoldred out, so the deck is full of draw: **Night's Whisper** ({1}{B}: draw two,
lose 2), **Sign in Blood** ({B}{B}: aimed at yourself, draw two, lose 2), **Black Market Connections** (at the start of
your first main phase choose any of: a Treasure for 1 life, a card for 2 life, a 3/2 changeling for 3 life), **Phyrexian
Arena** (a card each upkeep, 1 life), **Ambition's Cost** (three cards, 3 life), **Syphon Mind** (each other player
discards; you draw a card for each), **Mind Stone** (cash in for a card), **Castle Locthwain** ({1}{B}{B}, {T}: draw a
card, then lose life equal to the cards in your hand, so use it with a small hand), **Bonders' Enclave** ({3}, {T}:
draw, with a creature of power 4 or more, which Sheoldred is) and **Gix, Yawgmoth Praetor** (when a creature deals combat
damage to an opponent, its controller may pay 1 life to draw; that includes opposing creatures, so with Sheoldred out an
attacker drawing off Gix costs them 2 more).

**The One Ring** ({4}, indestructible) draws more every turn: tap it to add a burden counter, then draw a card per
counter. You lose 1 life per counter each upkeep, but with Sheoldred out each of those draws is 2 life back, so it
pays for itself. Cast, it also gives you protection from everything until your next turn.

**Damnable Pact** ({X}{B}{B} sorcery): *target player draws X cards and loses X life.* Aimed at yourself with Sheoldred
out, each card gains you 2 and the spell costs 1, so you gain X life net and draw X cards: a big refill. The X life is lost as the spell resolves, before Sheoldred's gains, so keep X below your life total; if Sheoldred is answered in response you only pay the life, and an opposing Sheoldred, Underworld Dreams or Fate Unraveler makes each card of yours cost more. The cards are
draws, so Sheoldred fires. Cast it on your own turn with spare mana, not at the end of an opponent's turn. Without
Sheoldred it only costs you X life. Aimed at an opponent with Sheoldred out, each card costs them 3 (2 from Sheoldred, 1
from the spell), plus 1 a card for each Underworld Dreams and Fate Unraveler out, but it also hands them X cards, so
never aim it at an opponent when no punisher is out. With K'rrik out the {B}{B} can be paid with 4 life; X is not
reduced.

**Thought Vessel** gives you no maximum hand size, so a big hand from Memory Jar or the draw spells is never discarded
down to seven.

#### Gain life, use it

Sheoldred's lifegain turns on the payoffs: **Foolish Fate** (the target's controller loses 3 if you gained life this
turn), **Leech Collector // Bloodletting** (the first lifegain each turn makes Bloodletting available for {B}: each
opponent loses 2), **Witch of the Moors**, **Sanguine Bond**, **Enduring Tenacity**, **Vito** and **Marauding
Blight-Priest**. The life is also a resource: it pays for The One Ring's burden counters, Snuff Out, Infernal Grasp and
Toxic Deluge.

#### Drain creatures

**Bloodthirsty Conqueror** (5/5 flying deathtouch) gains you life equal to whatever an opponent loses, and is the second
Blood half. **Vito** and **Marauding Blight-Priest** are the cheap Bond halves: each point of life you gain, from any source,
is life an opponent loses. **Ob Nixilis, the Fallen** ({3}{B}{B}, 3/3) makes a player lose 3 life each time a land enters
under your control and grows by three +1/+1 counters: one land drop a turn is 3 to an opponent and a bigger threat.
**Gray Merchant** is drain on entering. **Ill-Gotten Inheritance** ({3}{B} enchantment) is slow drain: at the beginning of each of your upkeeps it deals 1 damage to each opponent and you gain 1 life, which also starts Line 1 when its pieces are out; for {5}{B} you can sacrifice it to deal 4 damage to one opponent and gain 4.

**Enduring Tenacity** ({2}{B}{B}, 4/3 enchantment creature): whenever you gain life, target opponent loses that much
life. When it dies as a creature it returns to the battlefield as an enchantment that is not a creature, and it still
drains. Creature removal, sacrifice and edicts only delay it; enchantment removal, exile and bounce answer it. With
Exquisite Blood it is a two-card infinite drain (Line 1).

**Witch of the Moors** ({3}{B}{B}, 4/4 deathtouch): at the beginning of your end step, if you gained life this turn,
each opponent sacrifices a creature and you return up to one creature card from your graveyard to your hand. With
Sheoldred out you gain 2 on every draw, so it is a repeatable edict and a card back each turn; the drain creatures
and Exquisite Blood turn it on without her. It is a 5-drop: cast it as soon as the mana allows, after Sheoldred.

**Polluted Bonds** ({3}{B}{B}) is the drain on the table's own mana: every land an opponent plays costs that player 2
life and gains you 2, with any Bond half out that gain is 2 more from an opponent, and with Exquisite Blood out their
life loss is 2 more for you. It starts Line 1 on its own once a Blood half and a Bond half are out.

#### Remove threats

Nine spot removal spells, two sweepers and Rooftop Assassin. **Cheap instants:** **Infernal Grasp** ({1}{B}: destroy
target creature, lose 2 life), **Go for the Throat** ({1}{B}: destroy target nonartifact creature), **Bitter Triumph**
({1}{B}: discard a card or pay 3 life; destroy target creature or planeswalker), **Foolish Fate** and **Overkill**
(-0/-9999), **Sephiroth's Intervention** (destroy, gain 2 life), **Deadly Rollick** ({3}{B} instant, exile target
creature, free while you control a commander, so free while Sheoldred is out) and **Snuff Out** ({3}{B}, or pay 4 life
instead if you control a Swamp: destroy target nonblack creature; it can't be used on a black creature, which many of the
table's commanders are). **Feed the Swarm** ({1}{B} sorcery: destroy target creature or enchantment an opponent controls,
and you lose life equal to its mana value) is the deck's only targeted answer to an enchantment such as Aluren, Intruder Alarm,
Smothering Tithe or Rhystic Study. **Invoke Despair** (edict for a creature, an enchantment and a planeswalker, with 2
life lost and a card for you each time they have none; needs {B}{B}{B}{B}). **Rooftop Assassin** (flash, flying,
lifelink) destroys a creature that was dealt damage this turn, and Orcish Bowmasters' ping can set that up.

**Sweepers.** **Toxic Deluge** ({2}{B}, sorcery): pay X life, all creatures get -X/-X until end of turn. It hits your own
creatures too: X = 4 kills every creature with toughness 4 or less and leaves Sheoldred (4/5) alive as a 0/1, and K'rrik,
Witch and Vito die. Toxic Deluge is the list's only sweeper: against a go-wide board of small creatures (Krenko's Goblins, Edgar's Vampire tokens, mana elves) it is the answer, or the drain races it. With K'rrik out, Foolish Fate, Overkill, Painful Lesson and Toxic Deluge cost {2} and 2 life, and
Sephiroth's Intervention {3} and 2 life.

#### Recursion

**Yawgmoth's Will** ({2}{B}, sorcery): until end of turn you may play lands and cast spells from your graveyard; cards
that would go to your graveyard are exiled. Replay the turn's draw spells and tutors, a spent Exsanguinate, Torment of
Hailfire or Gray Merchant, or Sheoldred herself if she was left in the graveyard. **Witch of the Moors** returns a
creature card to your hand at your end step if you gained life that turn. Graveyard hate (Bojuka Bog, Dauthi Voidwalker,
Skullsnatcher, Farewell) turns Yawgmoth's Will off.

#### Ramp

Five rocks: **Sol Ring**, **Arcane Signet** (it makes {B}), **Mind Stone** (it can cash in for a card), **Thought Vessel**
and **Charcoal Diamond** (it enters tapped and makes {B}); **Jet Medallion** (each black spell costs {1} less; it
cuts only the generic part of a cost), plus **Dark Ritual** (Sheoldred a turn early) and the Treasure from Black Market
Connections. **Cabal Coffers** is the late-game ramp, and **K'rrik, Son of Yawgmoth** turns life into mana: each {B}
in a cost can be paid with 2 life.

### Mulligans

The first mulligan is free (you see two 7-card hands). After that it is a London mulligan: draw 7, put one card on the
bottom for each mulligan after the free one. Sheoldred is in the command zone, so she is never in the hand.

| Hand | Keep when |
|---|---|
| 7 cards | 2-5 lands including a source of {B} (Swamp, Castle Locthwain or Urborg); a play on turns 2-4 (a rock, Dark Ritual, a two-drop, a punisher or Sheoldred's four mana by turn 4); and one reason to go: a tutor, a punisher (Underworld Dreams, Fate Unraveler, Orcish Bowmasters), a draw engine, a drain piece or cheap removal. A two-lander needs a rock or Dark Ritual and cheap plays. Five lands need two real spells. 0-1 lands and 6+ lands go back |
| 6 or 5 cards | 2-4 lands including a {B} source, and two plays that cost 3 or less |
| 4 cards | Keep |

- **No black source goes back.** A hand with only Bonders' Enclave, Mikokoro, the Sanitarium or Cabal Coffers (which
  needs Swamps) can't cast Sheoldred. With Urborg in the hand and in play, every land taps for {B}.
- **Count turn 3-4 mana.** Three lands plus a rock is Sheoldred on turn 3; four lands is turn 4. A hand with only
  6-7 mana spells (Peer into the Abyss, Invoke Despair) is a ship, whatever the lands. K'rrik counts as
  a 4-mana spell ({4} and 6 life), not a 7.
- **Bottom first:** lands past the fourth, Peer into the Abyss, Memory Jar. Keep rocks, Dark Ritual,
  tutors and the cheap punishers.
- **Symmetric draw cards** (Howling Mine, Memory Jar, Temple Bell) are not a reason to keep: they are dead until a
  punisher is out.

| # | Hand | Call | Why |
|---|---|---|---|
| 1 | Swamp, Swamp, Sol Ring, Underworld Dreams, Night's Whisper, Orcish Bowmasters, Phyrexian Arena | Keep | Swamp and Sol Ring on turn 1, Swamp on turn 2: two Swamps and Sol Ring are four mana, so Sheoldred lands from the command zone on turn 2. Dreams, Arena and Bowmasters follow |
| 2 | Swamp, Swamp, Swamp, Mikokoro, Center of the Sea, Demonic Tutor, Howling Mine, Underworld Dreams | Keep | Dreams on turn 3 (BBB), Sheoldred on turn 4, then Howling Mine is safe to play; the Tutor finds a drain piece |
| 3 | Swamp, Cabal Coffers, Geier Reach Sanitarium, Bonders' Enclave, Peer into the Abyss, Memory Jar, Exsanguinate | Ship | One Swamp and three lands that can't make {B} without help, no play before turn 5, and Memory Jar and Peer are dead without a punisher |
| 4 | Swamp, Swamp, Mind Stone, Dark Ritual, Orcish Bowmasters, Vito, Gray Merchant of Asphodel | Keep (close) | Two lands, but Mind Stone on turn 2 means turn 3 is two Swamps, Mind Stone and Dark Ritual: 5 mana for Sheoldred with no third land, and Bowmasters and Vito are cheap plays while she is out |
| 5 | Swamp, Swamp, Swamp, Swamp, Swamp, Mikokoro, Center of the Sea, Peer into the Abyss | Ship | Six lands go back, and the one spell costs 7 |
| 6 | Swamp, Swamp, Swamp, Castle Locthwain, Enduring Tenacity, Exquisite Blood, Grim Tutor | Keep | Sheoldred on turn 4, Tenacity on turn 5 (four mana, no fifth land needed), Blood on turn 6 once a fifth land arrives: the first card an opponent draws starts the loop. Grim Tutor finds Sanguine Bond or an answer |
| 7 | Swamp, Swamp, Swamp, Sol Ring, Vampiric Tutor, Painful Lesson, Fate Unraveler | Keep | Sol Ring and three Swamps are Sheoldred on turn 3 (or turn 2 with the first two Swamps), Fate Unraveler beside her, and Painful Lesson waits for a punisher. Vampiric Tutor at an opponent's end step puts the next piece on top, and that draw is 2 life with Sheoldred out |
| 8 | Swamp, Swamp, Ill-Gotten Inheritance, Polluted Bonds, Invoke Despair, Peer into the Abyss, Ambition's Cost | Ship | Two lands, no rock, and every spell costs 4 or more |

### What to tutor for

Sheoldred is in the command zone, so no tutor can find her: the tutors find everything around her. Every tutor except
Vampiric Tutor is a sorcery, and the found card goes to your hand, not through a draw, so Sheoldred gains you nothing from
it (Vampiric Tutor puts it on top of the library, so the draw that takes it is a draw, and it does fire her). Every
tutor can find any card in the deck, K'rrik included. Before you cast one, name the card you will cast next and check
that the mana for it is there this turn or the next.

#### Every card that finds a card

| Card | Cost | Card goes to | Can find | Speed | The catch |
|---|---|---|---|---|---|
| Demonic Tutor | {1}{B} | Hand | Any card | Sorcery | Game Changer. The cheapest clean tutor, so it is the one to chain into the found card in the same turn |
| Vampiric Tutor | {B} | Top of the library | Any card | Instant | Game Changer. Costs 2 life, and you have the card only at your next draw (that draw is 2 life with Sheoldred out). Cast it at an opponent's end step so the draw step takes the card |
| Diabolic Tutor | {2}{B}{B} | Hand | Any card | Sorcery | Four mana: on turn 4 it competes with Sheoldred for the mana. Best from turn 6 on, with the found card castable next turn |
| Grim Tutor | {1}{B}{B} | Hand | Any card | Sorcery | Costs 3 life and Sheoldred gives none back. Off at low life, without Sheoldred out, and against Exquisite Blood plus Sanguine Bond or Vito on the table |
| Profane Tutor | {1}{B} to suspend 2 | Hand, two of your upkeeps later | Any card | Sorcery, cast free when the last time counter comes off | The slowest tutor: the card arrives two turns after you suspend it, but it costs no life and only two mana. Use it on turns with spare mana and no urgent piece |

Other cards that find cards, none of them a choice: **Damnable Pact** (draws X cards), **Black Market Connections**
(a card each turn for 2 life) and the draw spells. They do not replace a tutor on a turn that needs one specific card.

#### Which card, by board state

Count mana before you pick: lands, rocks and Cabal Coffers (it needs {2} and then makes {B} for each Swamp, so Coffers
plus 5 Swamps is 8 mana from 6 lands). Cast the tutor first, then check what is left for the found card. With K'rrik
out, each {B} in the tutor and the target can be paid with 2 life (see "K'rrik, Son of Yawgmoth").

**Sheoldred not yet cast (turns 1-4)**

| Your mana and hand | Opponents | Tutor | Target | Why, and what to watch |
|---|---|---|---|---|
| Turn 2, two lands, no rock, nothing to do on turn 3 | Slow or empty boards | Demonic Tutor | Sol Ring | Turn 3: cast Sol Ring ({1}) and tap it for {C}{C}, with two lands left for {B}{B}: Sheoldred on turn 3 off three lands. Use it when the hand has no better play. With Sol Ring already drawn, Dark Ritual does the same job once |
| Turn 3 or 4, Sheoldred castable this turn, your hand has no protection | White or blue-black with open mana, or cheap removal shown | Demonic Tutor if there is mana left, else next turn | Swiftfoot Boots | Cast Sheoldred first, Boots and equip ({2} plus {1}) the turn after. Boots stop targeted removal only, not wipes, edicts or Council's Judgment |
| Turn 3, three Swamps, hand holds Howling Mine, Seizan, Temple Bell or Memory Jar and no punisher | Anything | Demonic Tutor | Underworld Dreams | {B}{B}{B} turns the symmetric cards on and works with Sheoldred dead or not yet cast. Sheoldred still comes first if she is castable |
| Turn 3, a creature wipe is likely (Damn, Toxic Deluge, Blasphemous Act, Farewell shown) | Wipe deck | Any tutor | Underworld Dreams, Sanguine Bond or Exquisite Blood | Enchantments survive a creature wipe. Hold creatures in hand and do not tutor for Vito, Blight-Priest, Bloodthirsty Conqueror, K'rrik or the Merchant into it |
| Turn 1-3, an opponent is set up to win with a creature engine (Chulane with Aluren, Krenko, Marwyn's elves, Yuriko) and you hold no answer | Engine on the table | Demonic Tutor, then hold the mana | Infernal Grasp, Foolish Fate or Overkill (Snuff Out against a nonblack creature) | Tutor on your turn, keep the mana, kill the engine creature on their turn or in response to the activation. Infernal Grasp is two mana, Foolish Fate and Overkill three, Snuff Out costs 4 life and no mana. Overkill beats regeneration and indestructible. Feed the Swarm answers an Aluren or Intruder Alarm itself, at sorcery speed |

**Sheoldred out, engines running (turns 4-6)**

| Your mana and hand | Opponents | Tutor | Target | Why, and what to watch |
|---|---|---|---|---|
| 5 or 6 mana, Sheoldred safe, no Line 1 piece out | Tapped low or open removal | Demonic Tutor | Sanguine Bond, or Vito for a cheaper start | Bond works alone (every card you draw takes 2 from an opponent) and is an enchantment, which creature removal misses; Vito and Marauding Blight-Priest are 3-mana Bond halves that fit the same turn as the tutor. The Blood half goes last |
| A Blood half out, no Bond half | Any | Demonic Tutor (6 mana for tutor plus Tenacity, 5 for Vito or Blight-Priest) | Vito or Marauding Blight-Priest, or Enduring Tenacity | Vito and the Priest are the cheap pieces (3 mana), Tenacity (4) is the one creature removal only delays. The loop starts the same turn |
| A Bond half out, no Blood half | Tapped out, or one removal spell at most | Any tutor | Exquisite Blood or Bloodthirsty Conqueror | Either is the loop piece (5 mana). Cast it only when they cannot respond: one answer to a piece ends the loop for now. Exquisite Blood against creature removal and sweepers, the Conqueror against counters for noncreature spells. Starters: a draw with Sheoldred out, K'rrik's lifelink, a land drop with Polluted Bonds out, any drain creature |
| Sheoldred out and an opponent draws a lot (Chulane, Korvold, Rhystic Study, Brainstorm) | Any | Any tutor | Underworld Dreams, then Fate Unraveler or Orcish Bowmasters | They lose 2 for each draw already, and each punisher adds 1: three punishers are 4 a card. Dreams is {B}{B}{B}. Bowmasters pings on each draw but the first in their draw step |
| Sheoldred out, table with removal mana up, no way to keep going if she dies | Open blue or white mana | Any tutor | Underworld Dreams, Fate Unraveler or Phyrexian Arena | A second source of damage and draws that does not depend on her |
| A go-wide board (Krenko, Edgar Markov, Aurelia, Marwyn, Chatterfang) | Many X/2 creatures | Any tutor, cast it next turn if the mana is short | Toxic Deluge | It is 3 mana and X life and kills your own creatures too: X = 4 leaves Sheoldred alive. It is the only sweeper in the list, so against a board it cannot answer, race with the drain |
| An opponent has Opposition Agent | Agent on the board | Do not tutor | Kill the Agent first (Foolish Fate, Overkill, Infernal Grasp) | It takes whatever you search for |
| Tergrid or Liliana of the Veil out | Discard or edicts | Demonic Tutor, cast the card the same turn | The card you can cast now | So the card is not discarded or taken in hand. No Sanitarium or Memory Jar |

**Sheoldred removed, or the table is ahead**

| Situation | Tutor | Target | Why, and what to watch |
|---|---|---|---|
| Sheoldred in the graveyard (you left her there) | Demonic Tutor | Yawgmoth's Will or Witch of the Moors | Yawgmoth's Will ({2}{B}) then {2}{B}{B} casts her with no tax, 7 mana in one turn. Witch ({3}{B}{B}) returns her to your hand at your end step if you gained life that turn, and she is cast from hand next turn for {2}{B}{B} |
| Sheoldred exiled, bounced or stolen, or put in the command zone | Any tutor | Underworld Dreams, Fate Unraveler or Orcish Bowmasters | Recast her from the command zone ({2} more per earlier cast). The punishers hold the damage meanwhile. If she was stolen, kill the thief's creature (Infernal Grasp, Overkill) |
| A wipe just resolved, Sheoldred was the only creature | Any tutor | Vito or Marauding Blight-Priest, or Bowmasters at their end step | Enchantments (Dreams, Arena, Bond, Blood, Polluted Bonds) are still out. Rebuild with cheap creatures first, then recast Sheoldred |
| An opponent has a combo that wins next turn | Demonic Tutor, then hold the mana | Snuff Out (free, nonblack creature), Infernal Grasp, Go for the Throat, Bitter Triumph, Foolish Fate, Overkill, Deadly Rollick (free with Sheoldred out) or Sephiroth's Intervention (all instants) | You tutor at sorcery speed (or Vampiric Tutor at instant speed, then draw it), so you need the answer in hand and the mana up at the right time. Rooftop Assassin only kills a creature that has been dealt damage |
| An opponent has graveyard hate out (Bojuka Bog, Dauthi Voidwalker, Skullsnatcher, Farewell) | Any tutor | The card itself, hard-cast | Yawgmoth's Will and a Sheoldred left in the graveyard are off |
| Life 15 or less, Sheoldred out | Demonic Tutor only | Gray Merchant, Exsanguinate or Rooftop Assassin | Grim Tutor, Vampiric Tutor and K'rrik's life payments are off. Merchant and Exsanguinate gain it back |
| Life 15 or less, Sheoldred not out | Demonic Tutor | Gray Merchant or Exsanguinate | You cannot gain from draws yet. Do not pay life for a tutor or for K'rrik |

**The kill**

| Your board and mana | Opponents | Tutor | Target | Why, and what to watch |
|---|---|---|---|---|
| Sheoldred or Underworld Dreams out, 7 mana (or 4 and 6 life with K'rrik), one opponent is the threat | That player's library at least half their life total, no Teferi's Protection or counterspell mana | Demonic or Diabolic Tutor | Peer into the Abyss | Lethal on one opponent (Line 2). A sorcery that leaves the other two untouched, so it is the first kill of three |
| Cabal Coffers plus 5 or more Swamps, table not yet at lethal | Tapped low | Demonic or Diabolic Tutor | Exsanguinate | X is mana minus 2 (6 at 6 lands, 10 at 8, 14 at 10), 2 higher with K'rrik. 3X life for you against three opponents. With a Bond half out it is a second drain, with a Blood half it starts the loop |
| The same Coffers mana, opponents low on cards and permanents (or you want to strip a combo piece) | Few cards in hand, bare boards | Demonic or Diabolic Tutor | Torment of Hailfire | Each repeat takes 3 life or a card or a permanent, their choice. Worse than Exsanguinate against full hands and boards, and it gains you nothing |
| Devotion (count it: Sheoldred 2, Dreams 3, K'rrik 3, Arena 2, Seizan 2, Tenacity 2, Polluted Bonds 2, Bond 2, Gix 2, plus the Merchant's 2) reaches the lowest opponent's life, or a Bond half is out | Any | Any tutor | Gray Merchant | Five mana and a creature, so it dodges counterspells that only hit noncreature spells. With a Bond or Blood half out it is a Line 1 starter |
| Sheoldred out and every opponent at 14 or less (21 with Underworld Dreams also out) | Libraries of 7 or more | Demonic Tutor if 7 mana | Memory Jar | Cast it and tap it the same turn: every opponent draws seven, 14 life each, and you gain 14. Never with no punisher out |

**K'rrik, Son of Yawgmoth: when to find it**

| Situation | Tutor | Target | Why, and what to watch |
|---|---|---|---|
| Turn 4-6, 5-6 lands or rocks, hand holds two or more black spells with three or more {B} pips between them (Gray Merchant, Sanguine Bond, Bloodthirsty Conqueror), life 25 or more, Sheoldred out or lifelink available | Demonic Tutor (2 mana, K'rrik 4 and 6 life) | K'rrik | Turns the hand into mana: the next turn's spells cost 2 life per {B}. Check your life for the whole turn first |
| Exquisite Blood and Sanguine Bond (or Enduring Tenacity) in hand, mana is the only thing missing, life 25 or more | Demonic Tutor | K'rrik | 7 mana and 6 life for Blood and Bond (6 mana and 6 life for Blood and Tenacity), every {B} paid with life. The kill is then a turn earlier. Do it only if the table cannot punish the life |
| Cabal Coffers is online (about 10 mana) | Any tutor | The kill (Exsanguinate, Peer, Gray Merchant), not K'rrik | You already have the mana, and K'rrik is a 7-mana-value creature that dies to removal while the life is still gone |
| Life under 20, or a red or aggro table, or Edgar's loop pieces out | Any tutor | Not K'rrik | Each {B} would cost 2 life you cannot afford to lose, and a drain that starts the table's loop is worse than the mana |
| A removal-heavy table with open mana | Any tutor | K'rrik only after Boots is on Sheoldred | Removal will meet K'rrik first. The life paid on casting is not refunded |

#### Priority order for the common cases

1. **Sheoldred cannot land on turn 4 (no rock, three lands).** A mana source that makes turn 3 or 4 work (Sol Ring, Dark Ritual, Arcane Signet, Mind Stone).
2. **An opponent wins next turn.** The instant answer (Snuff Out, Infernal Grasp, Foolish Fate, Overkill) with mana kept up, before anything else.
3. **Sheoldred is out, the table has removal.** Swiftfoot Boots, then a second punisher (Underworld Dreams, Fate Unraveler, Orcish Bowmasters).
4. **Sheoldred is out and safe.** A cheap Bond half or Sanguine Bond (they work alone), then the missing Line 1 piece (the Blood half last).
5. **Mana is the problem and life is high.** K'rrik, cast the same turn if the tutor leaves 4 mana.
6. **The kill is ready.** Peer into the Abyss, Exsanguinate or Gray Merchant for the table, whichever the opponents' life totals and your mana allow.
7. **Sheoldred is dead.** Yawgmoth's Will or Witch of the Moors if she is in the graveyard, otherwise a replacement punisher and a recast.

#### Tutor mistakes

- **Tutoring for a card you cannot cast soon.** A tutor that leaves no mana for the found card, with removal on the table,
  gives the table a turn to answer or stop you. Tutor with the mana left over.
- **Tutoring for a Blood half into untapped removal.** One answer ends the loop. Find a Bond half first, the Blood half last.
- **Grim Tutor or Vampiric Tutor at low life, without Sheoldred out, or against Edgar's Exquisite Blood pieces.** The
  life is not given back before the draw.
- **Profane Tutor when you need the card this turn.** It arrives two upkeeps later.
- **Tutoring under Tergrid or Liliana of the Veil** without casting the card the same turn, so it is discarded.
- **Tutoring while an opponent controls Opposition Agent.**
- **Paying K'rrik life for a spell that gets countered, or at low life.** The life does not come back.
- **Peer into the Abyss at a library that is small compared to the life total,** or into Teferi's Protection or open counters.
- **Tutoring for a creature piece (Vito, Blight-Priest, Conqueror, K'rrik, Merchant) into a wipe** you have seen coming.
  Hold it, or find an enchantment.

### Protecting Sheoldred and the win turn

The deck has no counterspells. Protection is timing, tokens, a few cards and a second engine.

| Threat to Sheoldred | Answer in the deck |
|---|---|
| Targeted removal (Swords to Plowshares, Path to Exile, Go for the Throat, Terminate) | **Swiftfoot Boots**: hexproof, equip {1}, sorcery speed, so equip it as soon as she lands. Boots don't stop wipes, edicts, or Council's Judgment (it is a vote, not a target) |
| Combat danger | Her deathtouch holds off an attacker; **Orcish Bowmasters** (flash) and **Rooftop Assassin** (flash) are blockers you can add on their turn, and cheap instant removal answers the attacker (Snuff Out is free) |
| Edicts (Sheoldred's Edict, Plaguecrafter, Fleshbag Marauder, Liliana of the Veil) | Keep tokens and cheap bodies out: the Orc Army token from Bowmasters, Vault Plunderer, Leech Collector, Marauding Blight-Priest. Sheoldred's Edict can choose nontoken creatures, so keep a cheap nontoken creature too |
| Wipes (Damn, Toxic Deluge, Blasphemous Act, Innocent Blood) | Don't overextend while she is your only engine. Hold creatures in hand. The enchantment engines (Dreams, Arena, Bond, Blood, Polluted Bonds) survive a creature wipe |
| Exile, bounce, Farewell, Cyclonic Rift | Recast her from the command zone. Keep Underworld Dreams, Fate Unraveler, Phyrexian Arena and the drain enchantments as engines that don't need her |
| Steal (New Blood) or turn into an Elk (Oko) | Kill the creature that took her (Infernal Grasp, Overkill, Foolish Fate), or kill the Elk and recast her from the command zone |
| She is dead | See "After a stopped attempt" |

**The One Ring** ({4}) gives you, not her, protection from everything until your next turn: no damage and no targeted
spells for you, a good turn to cast a draw engine. It does not save a creature from a wipe.

**For the kill turn:** the deck's lines are mostly enchantments and sorceries, so
- cast the kill when the blue and white players are tapped low (Yuriko, Chulane, Atraxa, Aurelia);
- make your last piece a creature against counterspell decks (Fierce Guardianship, Dovin's Veto and Swan Song can't
  hit Gray Merchant or Bloodthirsty Conqueror, which are creatures; Swan Song can still counter Enduring
  Tenacity, an enchantment creature spell, but Veto and Fierce can't);
- keep a second line: a Blood half plus Bond halves, Peer into the Abyss as the backup, Exsanguinate for the table.

### Interaction on opponents' turns

The deck's instant-speed answers. Everything else is a sorcery.

| Card | Cost, speed | What to use it on |
|---|---|---|
| Snuff Out | {3}{B}, or 4 life with a Swamp, instant | A nonblack creature, with no mana spent: the answer for a tapped-out turn. It cannot target a black creature |
| Infernal Grasp | {1}{B} instant | Any creature, for 2 life: the combo creature (Krenko, Yuriko, Korvold) or an attacker |
| Go for the Throat, Bitter Triumph | {1}{B} instant | A nonartifact creature; a creature or planeswalker (discard a card or pay 3 life) |
| Foolish Fate | {2}{B} instant | A creature that is attacking, or the combo creature. Extra 3 life loss if you gained life this turn: cast it after a draw |
| Overkill | {2}{B} instant | The biggest threat, or a creature with regeneration or indestructible (it shrinks, it doesn't destroy). Not a hexproof creature |
| Sephiroth's Intervention | {3}{B} instant | A creature; the 2 life gained also starts Line 1 if the pieces are out |
| Deadly Rollick | {3}{B} instant, free while you control a commander | Exile target creature: with Sheoldred out it costs no mana, so it is the answer you can hold on a tapped-out turn. Without her it is four mana |
| Orcish Bowmasters | {1}{B} flash | In response to an opponent's extra draw, or at the end of their turn: 1 damage to any target (an X/1 creature, a player) and an Orc Army for edict fodder |
| Rooftop Assassin | {3}{B} flash | After blocks and damage: it destroys an opposing creature that was dealt damage this turn and survived. Also a flying lifelink blocker |
| Vampiric Tutor, Dark Ritual | {B} instant | Vampiric Tutor at their end step so the draw step takes the card; Dark Ritual to pay for an instant above |
| Mikokoro, Geier Reach Sanitarium, Temple Bell, Castle Locthwain | {2}, {T} (Mikokoro, Sanitarium); {T} (Bell); {1}{B}{B}, {T} (Castle) | An opponent's end step: each player draws, and Sheoldred drains the opponents (the Castle draws only you) |

- **Hold {3}{B} (four mana) on the opponent's turn** when Rooftop Assassin or a removal spell would change a fight.
- **Don't fight what you can't answer.** The deck has no artifact removal and no counterspell. Feed the Swarm and Invoke
  Despair are the enchantment answers; against Aluren, Intruder Alarm and Smothering Tithe, kill the creatures that use
  them as well.

### Matchups (Tiers 3 and 4)

Your tiers are t3 (Atraxa, Aurelia, Korvold, Marwyn, Tergrid) and t4 (Chulane, Edgar Markov, Krenko, Prosper, Yuriko).
The list below is the cards and plans against each commander; it is not a simulated result.

**Types of deck that are bad for you:**
- **Decks that stop draws or punish the symmetric cards.** If a deck is built around card draw, or has a combo that
  wants cards, hold the Mine, Seizan, Mikokoro, the Sanitarium, Temple Bell and the wheels until a punisher is out, and
  kill that deck first.
- **Big creature decks.** Use Sheoldred's deathtouch and the removal; save Overkill for the biggest threat.
- **Lifegain decks.** The drain is slower against them; push Exsanguinate and the drain creatures for the last points.
- **Fast combat decks.** The deck's answers are cheap removal and one sweeper: keep two mana up for Infernal Grasp or
  hold Snuff Out, and keep Toxic Deluge for the board that is already out.

| Opponent | Their threat | Your plan | Cards that matter |
|---|---|---|---|
| Atraxa, Praetors' Voice (t3, WUBG infect and planeswalkers) | Poison: Blighted Agent (unblockable), Plague Stinger and Skithiryx (fliers), then a proliferate every turn. Ten poison kills at any life total, so your life gain doesn't help. Farewell exiles creatures, enchantments, artifacts and graveyards; Oko turns Sheoldred into an Elk; their Narset caps you at one draw a turn (the Mine and Seizan give you nothing extra; their draws still cost them) | Kill the poison creatures on sight and race. Orcish Bowmasters pings the 1/1 poison creatures, and Rooftop Assassin blocks fliers. With Sheoldred out, don't pay Rhystic Study: their draw costs them 2. Keep a creature in hand and a second engine for after Farewell (it exiles the graveyard too, so a Sheoldred left there and Yawgmoth's Will are gone) | Infernal Grasp, Go for the Throat, Foolish Fate, Overkill, Swiftfoot Boots (against Swords, Path, Assassin's Trophy), Underworld Dreams, Orcish Bowmasters |
| Aurelia, the Warleader (t3, Boros extra combats) | Hasty flying attackers (Aurelia, Hero of Bladehold, Goblin Rabblemaster, Legion Warboss) with Embercleave and extra combats; Mother of Runes and Giver of Runes protect them; Blasphemous Act. Smothering Tithe makes a Treasure for them for each card you draw unless you pay {2} | Kill Mother of Runes and Giver of Runes first. Block with Sheoldred (deathtouch). Toxic Deluge for X = 2 kills Mother, Giver, Rabblemaster, Warboss and their Goblin tokens, and X = 4 also kills Aurelia and Hero of Bladehold (both toughness 4) and leaves Sheoldred alive. With Tithe out, draw fewer extra cards | Toxic Deluge, Snuff Out (Aurelia is not black), Infernal Grasp, Foolish Fate; Boots don't stop Council's Judgment |
| Korvold, Fae-Cursed King (t3, Jund sacrifice) | Korvold draws and grows off every sacrifice; Blood Artist, Zulaport Cutthroat, Mayhem Devil, Goblin Bombardment, Impact Tremors; Chatterfang + Pitiless Plunderer; Terror of the Peaks; Toxic Deluge and cheap removal | Their deck draws a lot (Korvold, Skullclamp), so punishers hit hard: Underworld Dreams, Fate Unraveler, Orcish Bowmasters (each extra draw is a ping), then Sheoldred. Kill Korvold (he is black, so not with Snuff Out). Your life gain out-races their pings. Toxic Deluge clears the tokens | Underworld Dreams, Fate Unraveler, Bowmasters, Infernal Grasp, Foolish Fate, Overkill, Toxic Deluge, Boots |
| Marwyn, the Nurturer (t3, mono-green elves) | A board of mana elves, then Craterhoof Behemoth once Gaea's Cradle and the elves are out (Natural Order, Green Sun's Zenith, Chord of Calling, Finale of Devastation find it). Killing Craterhoof after it enters doesn't stop its trigger | Kill the mana elves early and sweep before the swing. They draw little, so the punishers are weaker here: lean on the creatures and Exsanguinate. | Orcish Bowmasters, Infernal Grasp, Foolish Fate, Snuff Out (Marwyn is green), Toxic Deluge before the swing, Sheoldred as a blocker (Craterhoof tramples over it) |
| Tergrid, God of Fright (t3, mono-black discard and edicts) | Tergrid takes every nontoken permanent you sacrifice or discard. Edicts (Sheoldred's Edict, Plaguecrafter, Fleshbag Marauder, Innocent Blood), Liliana of the Veil, Waste Not (their gain when you discard), Painful Quandary. Their deck has its own Sheoldred (your draws cost you 2), Whispering One, Gray Merchant, Phyrexian Obliterator. Dauthi Voidwalker and Bojuka Bog hate the graveyard | Never sacrifice or discard a nontoken permanent while Tergrid is out (Mind Stone's draw, Memory Jar, Geier Reach Sanitarium, Bitter Triumph's discard: pay the 3 life instead). Edicts take your choice of creature, so keep tokens and cheap bodies (Sheoldred's Edict can force a nontoken one). Play out your hand before Liliana. Kill their Sheoldred (Snuff Out can't: it is black). Keep your hand to 7 | Orc Army tokens from Orcish Bowmasters, Infernal Grasp, Go for the Throat, Foolish Fate, Overkill, Underworld Dreams |
| Chulane, Teller of Tales (t4, Bant value and Aluren) | Chulane draws and ramps off every creature spell. Aluren or Intruder Alarm with Shrieking Drake or Whitemane Lion loops creatures, draws the deck and wins with Thassa's Oracle, Laboratory Maniac or Craterhoof. Counterspell, Dovin's Veto, Swan Song, Fierce Guardianship; Cyclonic Rift; Drannith Magistrate (you can't cast Sheoldred from the command zone or the graveyard) | A punisher is your answer to the loop: with Sheoldred out each card they draw costs 2, so drawing the deck kills them; Orcish Bowmasters pings each draw. Cast creature spells into counterspell decks (Veto, Swan Song and Fierce can't hit creatures; Swan Song can still hit Enduring Tenacity). Kill Chulane, the bouncers and Drannith Magistrate. Feed the Swarm answers Aluren and Intruder Alarm | Sheoldred, Underworld Dreams, Fate Unraveler, Orcish Bowmasters, Snuff Out and Infernal Grasp on Chulane, Feed the Swarm, Overkill (Magistrate), creature finishers (Gray Merchant, Bloodthirsty Conqueror) |
| Edgar Markov (t4, Mardu Vampires) | Eminence makes a 1/1 Vampire for every Vampire spell cast, from the command zone. Wide first-strike attacks (Edgar, Drana). Their own Sanguine Bond, Vito and Exquisite Blood loops. Cheap removal (Swords, Path, Bolt, Terminate, Fatal Push, Anguished Unmaking), Damn, New Blood, Teferi's Protection | Sheoldred and the removal hold the board. Do not pay life (Phyrexian Arena, The One Ring, Damnable Pact aimed at yourself, Grim Tutor, K'rrik's 2 life per {B}) when their Exquisite Blood or Bloodthirsty Conqueror is out beside Sanguine Bond, Vito or Marauding Blight-Priest: your life loss starts their loop. Kill their Vito or Blight-Priest first. Toxic Deluge kills the 1/1 Vampire tokens. Snuff Out can't hit black Vampires | Toxic Deluge, Infernal Grasp, Foolish Fate, Overkill, Boots |
| Krenko, Mob Boss (t4, mono-red goblins) | Krenko doubles the Goblins each activation; Impact Tremors, Goblin Bombardment, Purphoros, Coat of Arms; haste from Lightning Greaves and Swiftfoot Boots. Thornbite Staff + an outlet is infinite. Blood Moon turns your nonbasic lands into Mountains | Kill Krenko before he untaps (Snuff Out is free, Infernal Grasp, Foolish Fate and Overkill are instants), and sweep the Goblin tokens with Toxic Deluge. Keep Sheoldred home as a blocker. Under Blood Moon only the 30 Swamps make {B} (Cabal Coffers, Urborg and Castle Locthwain don't) | Snuff Out, Infernal Grasp, Foolish Fate, Overkill, Toxic Deluge |
| Prosper, Tome-Bound (t4, Rakdos exile and Treasure) | Exiles cards to play them and makes a Treasure each time; Opposition Agent (steals your tutors); Bolas's Citadel + Sensei's Divining Top + Aetherflux Reservoir; Underworld Breach; Toxic Deluge, Blasphemous Act, Delayed Blast Fireball | They draw less than most, so the punishers are weaker: lean on the drain creatures and Exsanguinate. Kill Opposition Agent on sight and don't tutor under it. Kill Prosper and the Treasure drainers (Marionette Master). Dauthi Voidwalker and Bojuka Bog shut off Yawgmoth's Will | Infernal Grasp, Foolish Fate, Overkill, Gray Merchant, Exsanguinate |
| Yuriko, the Tiger's Shadow (t4, Dimir ninjas) | Cheap evasive attackers (Ornithopter, Faerie Seer, Spectral Sailor), then ninjutsu Yuriko or another Ninja: each Ninja hit reveals a card and you lose life equal to its mana value (Shadow of Mortality 15, Temporal Trespass 11). Counterspell, Force of Will, Fierce Guardianship, Flare of Denial, Misdirection; Cyclonic Rift; Rhystic Study | Keep blockers back: their evasive creatures are small, and Cover of Darkness gives fear, which black creatures like yours can block. Orcish Bowmasters kills a 1/1 flier such as Faerie Seer or Spectral Sailor. They draw a lot (Brainstorm, Rhystic Study, Top), so punishers hit hard. Cast creatures into the counters. Pay for Rhystic Study only on the spell that matters | Sheoldred and Rooftop Assassin as blockers, Underworld Dreams, Fate Unraveler, Orcish Bowmasters, Infernal Grasp, Foolish Fate |

### After a stopped attempt

| What happened | Rebuild |
|---|---|
| Sheoldred destroyed | If she is in the graveyard: Yawgmoth's Will, or Witch of the Moors to return her to hand. Otherwise recast her from the command zone for {2} more per previous cast |
| Sheoldred exiled or bounced, or a wipe | Recast her from the command zone (or from hand). Meanwhile Underworld Dreams, Fate Unraveler, Orcish Bowmasters, Phyrexian Arena and the drain pieces carry the damage; hold the symmetric draw cards |
| A Blood half, a Bond half or Enduring Tenacity answered | Tutor for the missing piece. Vito and Marauding Blight-Priest are the cheapest Bond halves (3 mana); Tenacity is the cheapest that creature removal only delays (4 mana; it returns as an enchantment, which still counts as a piece). Cast a Blood half last, once the table is tapped |
| Peer into the Abyss countered or fizzled | Exsanguinate, or Gray Merchant, as the table-wide finish. Peer is a sorcery: wait until the table is tapped out |
| Board wiped | Underworld Dreams, Arena, Bond, Blood and Polluted Bonds survive a creature wipe. Rebuild with Vito, Blight-Priest, Orcish Bowmasters, then recast Sheoldred |
| Graveyard hate (Bojuka Bog, Voidwalker, Farewell) | Switch to hard casting: tutors, Rooftop Assassin; Yawgmoth's Will and a Sheoldred in the graveyard are off |
| Counterspell on the key piece | Hold the second piece. Cast a creature next (the counters are mostly noncreature), or wait for a turn when they tap out |
| K'rrik answered | Costs already paid stay paid. Count your mana again before the next black spell: the {B} pips are back to mana. Recast it for {4} and 6 life if your life allows |
| Life too low to run the engines | Stop The One Ring, Phyrexian Arena, Damnable Pact aimed at you (unless Sheoldred is out) and the tutors that cost life. Draw with Sheoldred first, or gain with Exsanguinate or Gray Merchant |

- A failed attempt shows the table your plan. The next try needs a second line ready.

### Life management

| Source | Cost | What gains it back |
|---|---|---|
| Phyrexian Arena | 1 per upkeep | Its card (2 life with Sheoldred) |
| The One Ring | 1 per burden counter, each upkeep | Each draw is 2 life with Sheoldred |
| Seizan | 2 in your upkeep | The 2 cards (4 with Sheoldred) |
| Cost of Brilliance, Painful Lesson, Sign in Blood, Night's Whisper (aimed at you) | 2 | The 2 cards (4 with Sheoldred) |
| Ambition's Cost | 3 | The 3 cards (6 with Sheoldred) |
| Black Market Connections | 1 (Treasure), 2 (a card), 3 (a 3/2 body), each mode chosen | The card (2 with Sheoldred) |
| Bloodgift Demon (aimed at you) | 1 per upkeep | The card (2 with Sheoldred) |
| Castle Locthwain | The number of cards in your hand after the draw | The card (2 with Sheoldred) |
| Grim Tutor | 3 | Nothing: the card is put into your hand, not drawn |
| Vampiric Tutor | 2 | The draw that takes the card (2 with Sheoldred) |
| Damnable Pact (aimed at you) | X, one per card | The X cards (2X with Sheoldred, so X net) |
| Infernal Grasp, Bitter Triumph (no discard), Feed the Swarm, Snuff Out | 2, 3, the permanent's mana value, 4 | Nothing |
| Toxic Deluge | X | Nothing |
| K'rrik, Son of Yawgmoth | 6 to cast it, then 2 per {B} paid with life | Lifelink on K'rrik, Gray Merchant, Exsanguinate, Sheoldred's draws |
| Gix | 1 per card drawn off combat damage | The card (2 with Sheoldred) |

Sources of life: Sheoldred (2 per draw), Exsanguinate, Gray Merchant, Bloodthirsty Conqueror and Exquisite Blood (the
life the table loses), Polluted Bonds (2 per opposing land), Sephiroth's Intervention (2), Rooftop Assassin and K'rrik
(lifelink), Vito's {3}{B}{B} (your creatures gain lifelink).

**Rules of thumb.** Without Sheoldred, stop The One Ring, Damnable Pact aimed at you, Grim Tutor and Vampiric Tutor, and
keep your life well above what the table can swing in one round. Check Toxic Deluge's X before you cast it. Against
Edgar's loop pieces, do not pay life at all. With K'rrik out, add up the life for the whole turn before the first spell
(each {B} is 2) and leave yourself at least 15 unless the turn wins.

### Common mistakes

1. Playing Howling Mine, Seizan, Memory Jar, Temple Bell, Mikokoro or the Sanitarium before a punisher is out. They
   refuel the table.
2. Tapping out for the engines while Sheoldred is your only engine and the table has removal or a wipe. Boots first.
3. Treating two Bond halves, or two Blood halves, as a combo. The loop needs one of each.
4. Casting a Blood half into untapped removal. One answer breaks the loop; go when the table is tapped, and keep a second
   line.
5. Paying life (Arena, Damnable Pact aimed at you, Grim Tutor, The One Ring, Snuff Out) without Sheoldred, or against
   Edgar's Exquisite Blood pieces.
6. Tutoring while an opponent controls Opposition Agent.
7. Sacrificing or discarding a nontoken permanent while Tergrid is out (Mind Stone, Memory Jar, the Sanitarium),
   or ending the turn with 8 cards.
8. Putting Sheoldred in the command zone when you hold Yawgmoth's Will or have Witch of the Moors on the battlefield, which
   bring her back without the tax, and paying the tax when she is already in the graveyard.
9. Casting Toxic Deluge for an X that also kills Sheoldred (5 or more) or the creatures the plan needs.
10. Casting Snuff Out at a black creature: it can't target one.
11. Paying life with K'rrik at low life, against a burn or aggro table, or for a spell that gets countered. The life does
    not come back.

## Mana

36 lands: 30 Swamp, Cabal Coffers, Castle Locthwain, Urborg, Tomb of Yawgmoth, Bonders' Enclave, Mikokoro, Center of the
Sea and Geier Reach Sanitarium. Thirty-two lands make {B} outright (30 Swamps, Castle Locthwain and Urborg, which is
itself a Swamp). **Cabal Coffers** ({2}, {T}: add {B} for each Swamp you control) needs no help from other land
types: with 30 Swamps, six Swamps out make it net {B}{B}{B}{B}, more every turn, for Exsanguinate and the big turns.
**Urborg, Tomb of Yawgmoth** makes every land a Swamp, so Coffers counts all your lands and the three colourless lands
(Enclave, Mikokoro, the Sanitarium) tap for {B}; it does the same for the lands of every opponent, who can then tap them
for {B}. **Castle Locthwain** enters tapped unless you control a Swamp. Bonders' Enclave, Mikokoro and the Sanitarium make
{C} and draw cards.

Other mana: five rocks (Sol Ring, Arcane Signet, which makes {B}; Mind Stone; Thought Vessel; Charcoal Diamond, which
enters tapped and makes {B}), Jet Medallion ({1} off each black spell), Dark Ritual, the Treasure from Black Market
Connections, and K'rrik, which pays each {B} with 2 life. Tutors: Demonic Tutor, Diabolic Tutor, Grim Tutor, Vampiric
Tutor and Profane Tutor. Interaction: nine removal spells (Deadly Rollick and Snuff Out among them), Toxic Deluge,
Rooftop Assassin and Orcish Bowmasters; protection: Swiftfoot Boots and the enchantment engines.

## Bracket and Rule 0

Bracket 4: four Game Changers (The One Ring, Demonic Tutor, Orcish Bowmasters, Vampiric Tutor) and the combos named here.
There are no extra turns and no mass land destruction, but there are infinite combos: Exquisite Blood or Bloodthirsty
Conqueror with Sanguine Bond, Enduring Tenacity, Vito or Marauding Blight-Priest is a two-card infinite drain (eight
pairs), Polluted Bonds with one Blood half and one Bond half makes every opposing land drop start it, and Sheoldred + Peer
into the Abyss (or Underworld Dreams + Peer into the Abyss) is a near-instant kill on an opponent whose library is large
compared with their life. Name these before the game. Tell the table about the four Game Changers and the five tutors, tell
it that Howling Mine, Memory Jar, Temple Bell and the other draw cards are symmetric on purpose (they make everyone draw,
and Sheoldred, Underworld Dreams, Fate Unraveler and Orcish Bowmasters are what make that cost the opponents), and tell
it that Urborg, Tomb of Yawgmoth makes every land a Swamp, opponents' lands included.

## Decklist by type (100)

**Commander (1).** Sheoldred, the Apocalypse

**Lands (36).** Bonders' Enclave, Cabal Coffers, Castle Locthwain, Geier Reach Sanitarium, Mikokoro Center of the Sea, Urborg Tomb of Yawgmoth, 30 Swamp

**Creatures (17).** Bloodgift Demon, Bloodthirsty Conqueror, Enduring Tenacity, Fate Unraveler, Gix Yawgmoth Praetor, Gray Merchant of Asphodel, K'rrik Son of Yawgmoth, Leech Collector // Bloodletting, Marauding Blight-Priest, Ob Nixilis the Fallen, Orcish Bowmasters, Psychosis Crawler, Rooftop Assassin, Seizan Perverter of Truth, Vault Plunderer, Vito Thorn of the Dusk Rose, Witch of the Moors

**Enchantments (7).** Black Market Connections, Exquisite Blood, Ill-Gotten Inheritance, Phyrexian Arena, Polluted Bonds, Sanguine Bond, Underworld Dreams

**Artifacts (11).** Arcane Signet, Charcoal Diamond, Howling Mine, Jet Medallion, Memory Jar, Mind Stone, Sol Ring, Swiftfoot Boots, Temple Bell, The One Ring, Thought Vessel

**Sorceries (18).** Ambition's Cost, Cost of Brilliance, Damnable Pact, Demonic Tutor, Diabolic Tutor, Exsanguinate, Feed the Swarm, Grim Tutor, Invoke Despair, Night's Whisper, Painful Lesson, Peer into the Abyss, Profane Tutor, Sign in Blood, Syphon Mind, Torment of Hailfire, Toxic Deluge, Yawgmoth's Will

**Instants (10).** Bitter Triumph, Dark Ritual, Deadly Rollick, Foolish Fate, Go for the Throat, Infernal Grasp, Overkill, Sephiroth's Intervention, Snuff Out, Vampiric Tutor

## Import list (100)

```
1 Ambition's Cost
1 Arcane Signet
1 Bitter Triumph
1 Black Market Connections
1 Bloodgift Demon
1 Bloodthirsty Conqueror
1 Bonders' Enclave
1 Cabal Coffers
1 Castle Locthwain
1 Charcoal Diamond
1 Cost of Brilliance
1 Damnable Pact
1 Dark Ritual
1 Deadly Rollick
1 Demonic Tutor
1 Diabolic Tutor
1 Enduring Tenacity
1 Exquisite Blood
1 Exsanguinate
1 Fate Unraveler
1 Feed the Swarm
1 Foolish Fate
1 Geier Reach Sanitarium
1 Gix, Yawgmoth Praetor
1 Go for the Throat
1 Gray Merchant of Asphodel
1 Grim Tutor
1 Howling Mine
1 Ill-Gotten Inheritance
1 Infernal Grasp
1 Invoke Despair
1 Jet Medallion
1 K'rrik, Son of Yawgmoth
1 Leech Collector // Bloodletting
1 Marauding Blight-Priest
1 Memory Jar
1 Mikokoro, Center of the Sea
1 Mind Stone
1 Night's Whisper
1 Ob Nixilis, the Fallen
1 Orcish Bowmasters
1 Overkill
1 Painful Lesson
1 Peer into the Abyss
1 Phyrexian Arena
1 Polluted Bonds
1 Profane Tutor
1 Psychosis Crawler
1 Rooftop Assassin
1 Sanguine Bond
1 Seizan, Perverter of Truth
1 Sephiroth's Intervention
1 Sheoldred, the Apocalypse
1 Sign in Blood
1 Snuff Out
1 Sol Ring
1 Swiftfoot Boots
1 Syphon Mind
1 Temple Bell
1 The One Ring
1 Thought Vessel
1 Torment of Hailfire
1 Toxic Deluge
1 Underworld Dreams
1 Urborg, Tomb of Yawgmoth
1 Vampiric Tutor
1 Vault Plunderer
1 Vito, Thorn of the Dusk Rose
1 Witch of the Moors
1 Yawgmoth's Will
30 Swamp
```
