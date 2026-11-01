# Sheoldred, the Apocalypse — Mono-Black Drain

A mono-black deck that makes the whole table draw cards while Sheoldred, the Apocalypse punishes every one of them. It
wins by draining: three opponents' draw steps cost them 6 life a round with Sheoldred out, more with Underworld Dreams
and Fate Unraveler beside her, and Exsanguinate and Peer into the Abyss close the game. It
is weak when Sheoldred is exiled or keeps getting removed, and when the symmetric draw cards are played with no
punisher out, because they then just refuel the table.

| | |
|---|---|
| **Commander** | Sheoldred, the Apocalypse ({2}{B}{B}, 4/5 deathtouch, black) |
| **Bracket** | 4 (4 Game Changers: Ad Nauseam, The One Ring, Demonic Tutor, Bolas's Citadel) |
| **Simulator** | deck key `sheoldred`; results in `decklists/pool/pool-results.md`, history in `decklists/CHANGELOG.md` |

## Strategy

### The commander
**Sheoldred, the Apocalypse**: *Deathtouch. Whenever you draw a card, you gain 2 life. Whenever an opponent draws a
card, they lose 2 life.* Each of three opponents loses 2 life in their own draw step, so 6 life a round from the
draw steps alone, and every card you draw is 2 life back. She is a 4/5 deathtouch body for four mana, so she also
blocks and attacks well. She is the deck's one big engine: every draw spell, wheel and symmetric effect gets better
with her out. What breaks her is exile, bounce, or removal you can't answer. Keep a protection spell or a way to
bring her back (see "Protect her") before you start making the table draw.

### Game plan
- **Turns 1-3:** play a mana rock (Arcane Signet, Fellwar Stone, Mind Stone, Commander's Sphere, Potioner's Trove)
  and a cheap blocker or value creature (Burrog Banemaker, Fetid Imp, Undercity Dire Rat, Blood
  Artist, Phyrexian Rager, Vault Plunderer). **Underworld Dreams** ({B}{B}{B}) is a fine turn-3 play on its own: every
  card an opponent draws costs them 1.
- **Turns 4-6:** Sheoldred on turn 3 or 4. Once she is out, equip protection first, then start the draw engines.
- **Late game:** keep the engines running, drain with the creatures, and finish with Exsanguinate or
  Peer into the Abyss. If Sheoldred dies, recast her, or return her with Phyrexian Delver (see "Recursion").
  Demonic Tutor or Diabolic Tutor finds whichever of the two you need.

### Punish their draws
Sheoldred (2 life a card), **Underworld Dreams** and **Fate Unraveler** (1 damage a card each) stack: with all three
out, every card an opponent draws costs them 4. Underworld Dreams and Fate Unraveler work without Sheoldred, so they
keep the pressure on while she is dead or not yet cast. Fate Unraveler is also a 4-mana creature body.

### Make them draw
These are the cards that turn the punishers on. Play the symmetric ones only once a punisher is out.
- **Howling Mine**: each player draws an extra card in their draw step, so each opponent draws two a turn (4 life with
  Sheoldred) while you draw two too (4 life back).
- **Mikokoro, Center of the Sea** and **Geier Reach Sanitarium** (lands) and **Otherworld Atlas**: each player draws, at
  will. Mikokoro: {2}, {T}, each player draws a card. The Sanitarium: {2}, {T}, each player draws then discards. The
  Atlas: tap to add a charge counter, or tap to make each player draw a card per counter, so it gets bigger every turn
  you charge it. Use them at an opponent's end step so you keep your mana.
- **Seizan, Perverter of Truth**: in each player's upkeep that player loses 2 life and draws two cards. With Sheoldred
  out each opponent loses 6 per upkeep, and you take 2 and draw two (net +2 life).
- **The wheels**: **Memory Jar** ({T}, sacrifice: each player draws seven, then discards at the end step and gets their
  old hand back) is 14 life from each opponent with Sheoldred out, and you gain 14. **Dark Deal** has each player
  discard their hand and draw one fewer; a big opposing hand turns into a big loss.
- **Hand one opponent cards**: **Cost of Brilliance** (target player draws two, loses 2, and a +1/+1 counter on a
  creature) and **Vault Plunderer** (target player draws one, loses 1). With Sheoldred out these are 6 or 3 life
  against the chosen opponent. **Peer into the Abyss** makes one opponent
  draw half their library and lose half their life, which is lethal with Sheoldred out.

### Draw your own
Each card you draw is 2 life with Sheoldred out, so the deck is full of draw: **Phyrexian Arena** (a card each upkeep,
1 life), **Seymour Flux** (pay 1 life each upkeep for a card and a +1/+1 counter), **Ambition's Cost**, **Cast Away
Doubt** (draw two; 2 damage to each player, which Sheoldred's lifegain more than covers), **Syphon Mind** (each other
player discards; you draw a card for each), **Phyrexian Rager**, **Deadly Dispute** (sacrifice a creature or artifact, draw two, make a Treasure), **Carnage Altar** ({3}, sacrifice a
creature: draw a card), **Bonders' Enclave** (draw with {3} and a creature of power 4 or more, which Sheoldred is) and
**Gix, Yawgmoth Praetor** (when a creature deals combat damage to an opponent, its controller may pay 1 life to draw;
that includes opposing creatures, so with Sheoldred out an attacker drawing off Gix costs them 2 more).

**The One Ring** ({4}, indestructible) draws more every turn: tap it to add a burden counter, then draw a card per
counter. You lose 1 life per counter each upkeep, but with Sheoldred out each of those draws is 2 life back, so it
pays for itself. Cast, it also gives you protection from everything until your next turn. **Call of the Ring**
({1}{B}) has the Ring tempt you each upkeep, and each time you choose a Ring-bearer you may pay 2 life to draw a
card; Sheoldred gives the 2 back.

**Ad Nauseam** ({3}{B}{B} instant) turns spare life into a hand: reveal the top card, put it into your hand, lose life
equal to its mana value, repeat. Cast it at the end of the opponent's turn before yours, with your mana untapped,
and stop at a hand you can use next turn (about seven plus a card per three mana) or while your life is still well
above what the table could swing for. The cards are put into your hand, not drawn, so Sheoldred gains you nothing
from them. The deck's low curve (average mana value under 3) makes it cheap: about 2–3 life a card.

**Bolas's Citadel** ({3}{B}{B}{B}) lets you look at the top card of your library and play it, paying life equal to
its mana value instead of mana. The low curve keeps that cheap, and Sheoldred's lifegain keeps the life total up.
These cards are played, not drawn, so Sheoldred gains you nothing from them directly.

### Finishers
**Exsanguinate** (X life from each opponent, you gain it) is the plain kill. **Peer into the Abyss** is the draw-based
kill. **Necrosquito** ({3}{B}, flying) is a body that grows into a finisher: it enters with two oil counters (a 2/2) and
gets an oil counter, so +1/+1, whenever another creature or artifact you control goes to the graveyard. Carnage Altar,
Deadly Dispute, the sacrifice-cost removal, chump blocks and your own Treasures and rocks all feed it, and it flies over
the table's blockers. **Grave Titan** ({4}{B}{B}, 6/6 deathtouch) makes two 2/2 Zombies when it enters and each time it
attacks. The Zombies block for you and are the sacrifice fodder for Carnage Altar, Deadly Dispute and the
sacrifice-cost removal, and each one that dies drains with Blood Artist and Zulaport Cutthroat out.

### Protect her
- **Swiftfoot Boots** (hexproof and haste, equip {1}) and **Darksteel Plate** (indestructible, equip {2}): equip one
  as soon as Sheoldred lands. Boots stop targeted removal; the Plate stops destroy effects and damage, but not exile
  or -X/-X.
- **Battle-Rage Blessing** (deathtouch and indestructible): hold it up
  for a destroy effect or a bad block.
- **Perigee Beckoner**: when it enters, another creature of yours gets +2/+0 and returns if it dies this turn. Cast it
  on Sheoldred before she attacks or blocks into a risk; warp ({1}{B}) lets you reuse it later.
- **Recursion** (below) is the backup when protection isn't there, and **Demonic Tutor** ({1}{B}) or **Diabolic Tutor** ({2}{B}{B}) finds Sheoldred
  again if she was exiled or shuffled away.

### Remove threats
12 removal spells keep the board manageable: **Terror** (nonblack, nonartifact), **Last Gasp** (-3/-3), **Vanquish the
Weak** (power 3 or less), **Disembowel** (X equals the target's mana value), **Overkill** (-0/-9999), **Foolish Fate**,
**Sephiroth's Intervention** (destroy, gain 2 life), **Wander Off** (exile), **Invoke Despair** (edict for a creature,
an enchantment and a planeswalker, with a loss of 2 life and a card for you each time they have none; needs {B}{B}{B}{B}),
**Soul Enervation** (flash, -4/-4), and the sacrifice-cost cheap ones **Silence the Echo** and **Lethal Throwdown** (each cheap
with a creature to sacrifice; Silence the Echo can instead pay {3}). **Rooftop Assassin** (flash, flying, lifelink) destroys a creature that was dealt damage this turn, and
**Accursed Marauder** makes each player sacrifice a nontoken creature (it can sacrifice itself).

### Gain life, use it
Sheoldred's lifegain turns on the lifegain payoffs: **Foolish Fate** (the target's controller loses 3 if you gained
life this turn) and **Leech Collector // Bloodletting** (the first lifegain each turn prepares {B} Bloodletting: each
opponent loses 2). **Potioner's Trove** gains 2 after you cast an instant or sorcery. The life is also a resource:
it pays for The One Ring's burden counters, Call of the Ring, Bolas's Citadel and Ad Nauseam.

### Drain creatures
**Blood Artist**, **Al Bhed Salvagers**, **Sneering Shadewriter** (drain 2 on entering) and **Screeching Soulbreaker**
(drain 1 each attack) keep the pressure on when Sheoldred is gone. **Ob Nixilis, the Fallen** ({3}{B}{B}, 3/3) makes a
player lose 3 life each time a land enters under your control and grows by three +1/+1 counters: one land drop a turn
is 3 to an opponent and a bigger threat. **Gray Merchant of Asphodel** ({3}{B}{B}) drains each opponent by your
devotion to black when it enters, and you gain what they lose: Sheoldred ({B}{B}), Underworld Dreams ({B}{B}{B}),
Phyrexian Arena ({B}{B}), Seizan ({B}{B}) and the Merchant itself ({B}{B}) all count, so with Sheoldred and
Underworld Dreams out it is at least 7 from each. **Zulaport Cutthroat** drains each opponent
1 (and gains you 1) whenever a creature you control dies; Carnage Altar, Deadly Dispute, Silence the Echo, Lethal
Throwdown, Accursed Marauder and chump blocks feed it.
**Fetid Imp** and **Burrog Banemaker** are deathtouch blockers that protect the life total. **Undercity Dire Rat** is a
cheap body that leaves a Treasure.

### Recursion
**Phyrexian Delver** returns a creature card from your graveyard (you lose life equal to its mana value): Sheoldred
if she died, Sneering Shadewriter for a second drain. It also triggers **Soul Enervation**, which drains 1 whenever creature
cards leave your graveyard. Keep it for Sheoldred.

### Ramp
Five rocks, Arcane Signet, Fellwar Stone, Mind Stone (it can cash in for a card), Commander's Sphere (also a card)
and Potioner's Trove, plus Undercity Dire Rat (a Treasure when it dies). Memory Jar, Otherworld Atlas, The One Ring
and Bolas's Citadel are the bigger artifacts you cast once the rocks are out.

### How it wins
- **The slow drain.** Sheoldred plus Howling Mine, Seizan and the end-step engines drains each of three opponents every round; with Underworld Dreams and Fate Unraveler out it is faster.
- **The burst.** Memory Jar with Sheoldred out is 14 life to each opponent at once. Peer into the Abyss on one
  opponent with Sheoldred out is lethal on its own. Exsanguinate for X is the table-wide finish.
- **Necrosquito** with a few sacrifices behind it is a large flier that attacks over the table's blockers.

### Weak spots and how to play around them
- **Exile, bounce and repeated removal on Sheoldred.** Play around it with Swiftfoot Boots and hold a protection spell;
  keep Phyrexian Delver, Demonic Tutor or a recast in reserve. Don't overextend into a wipe while she's your only
  engine. The One Ring's protection covers you for the turn it enters.
- **Paying life without Sheoldred.** The One Ring's upkeep loss, Call of the Ring and Bolas's Citadel all cost life;
  without her lifegain, stop tapping the Ring and play fewer cards off the Citadel.
- **Opponents who stop draws or punish the symmetric cards.** If a deck is built around card draw, or has a combo that
  wants cards, hold the symmetric draw cards (the Mine, the Atlas, Mikokoro, the Sanitarium, Seizan, the wheels) until
  a punisher is out and kill that deck first.
- **Big creature decks.** Use the cheap deathtouch blockers (Fetid Imp, Burrog Banemaker) and the removal; save Overkill
  and Wander Off for the biggest threat.
- **Lifegain decks.** The drain is slower against them; push Exsanguinate and the drain creatures for the last points.

## Key lines

- **Turns 1-3:** a rock and a cheap blocker; Sheoldred on turn 3 or 4. Underworld Dreams is a fine turn-3 play on its
  own.
- **With Sheoldred out:** equip Swiftfoot Boots or Darksteel Plate first, then the draw engines: Howling Mine, Seizan,
  Mikokoro and Otherworld Atlas at each opponent's end step, Memory Jar the turn she's safe. Hold a protection instant
  for removal aimed at her. The One Ring and Bolas's Citadel turn her lifegain into cards.
- **Without her:** removal, Underworld Dreams, Fate Unraveler and the drain creatures (Blood Artist, Al Bhed Salvagers,
  Sneering Shadewriter, Screeching Soulbreaker, Gray Merchant, Zulaport Cutthroat) keep the pressure on until she
  comes back; hold the symmetric draw cards.
- **The kill:** Peer into the Abyss on an opponent with Sheoldred out, or Exsanguinate for the table. Demonic Tutor or
  Diabolic Tutor finds Peer into the Abyss once she is safe.
- **Mulligan:** keep hands with two or three lands, a rock and a creature or a draw spell; send back hands with no
  black source.

## Mana and consistency

36 lands: 32 Swamp, Command Tower (makes black here), and three colorless lands that draw cards: Bonders' Enclave,
Mikokoro, Center of the Sea and Geier Reach Sanitarium. That is 33 black sources. Ramp: five mana artifacts (Arcane
Signet, Fellwar Stone, Mind Stone, Commander's Sphere, Potioner's Trove) plus Undercity Dire Rat. Tutors: Demonic
Tutor and Diabolic Tutor.
Interaction: 12 removal spells, 3 protection cards (Swiftfoot Boots, Darksteel Plate, Battle-Rage
Blessing) and Perigee Beckoner.

## Bracket and Rule 0

Bracket 4, on the Game Changer count: four Game Changers (Ad Nauseam, The One Ring, Demonic Tutor, Bolas's Citadel).
There are no two-card combos, no extra turns and no mass land destruction. Tell the table about the four Game
Changers, and tell it that Howling Mine, Memory Jar, Dark Deal and the other draw cards are symmetric on purpose: they make everyone
draw, and Sheoldred, Underworld Dreams and Fate Unraveler are what make that cost the opponents.

## Decklist by type (100)

**Commander (1).** Sheoldred, the Apocalypse

**Lands (36).** Bonders' Enclave, Command Tower, Geier Reach Sanitarium, Mikokoro Center of the Sea, 32 Swamp

**Creatures (23).** Accursed Marauder, Al Bhed Salvagers, Blood Artist, Burrog Banemaker, Fate Unraveler, Fetid Imp, Gix Yawgmoth Praetor, Grave Titan, Gray Merchant of Asphodel, Leech Collector // Bloodletting, Necrosquito, Ob Nixilis the Fallen, Perigee Beckoner, Phyrexian Delver, Phyrexian Rager, Rooftop Assassin, Screeching Soulbreaker, Seizan Perverter of Truth, Seymour Flux, Sneering Shadewriter, Undercity Dire Rat, Vault Plunderer, Zulaport Cutthroat

**Enchantments (4).** Call of the Ring, Phyrexian Arena, Soul Enervation, Underworld Dreams

**Artifacts (13).** Arcane Signet, Bolas's Citadel, Carnage Altar, Commander's Sphere, Darksteel Plate, Fellwar Stone, Howling Mine, Memory Jar, Mind Stone, Otherworld Atlas, Potioner's Trove, Swiftfoot Boots, The One Ring

**Sorceries (12).** Ambition's Cost, Cast Away Doubt, Cost of Brilliance, Dark Deal, Demonic Tutor, Diabolic Tutor, Exsanguinate, Invoke Despair, Lethal Throwdown, Peer into the Abyss, Silence the Echo, Syphon Mind

**Instants (11).** Ad Nauseam, Battle-Rage Blessing, Deadly Dispute, Disembowel, Foolish Fate, Last Gasp, Overkill, Sephiroth's Intervention, Terror, Vanquish the Weak, Wander Off

## Import list (100)

```
1 Accursed Marauder
1 Ad Nauseam
1 Al Bhed Salvagers
1 Ambition's Cost
1 Arcane Signet
1 Battle-Rage Blessing
1 Blood Artist
1 Bolas's Citadel
1 Bonders' Enclave
1 Burrog Banemaker
1 Call of the Ring
1 Carnage Altar
1 Cast Away Doubt
1 Command Tower
1 Commander's Sphere
1 Cost of Brilliance
1 Dark Deal
1 Darksteel Plate
1 Deadly Dispute
1 Demonic Tutor
1 Diabolic Tutor
1 Disembowel
1 Exsanguinate
1 Fate Unraveler
1 Fellwar Stone
1 Fetid Imp
1 Foolish Fate
1 Geier Reach Sanitarium
1 Gix, Yawgmoth Praetor
1 Grave Titan
1 Gray Merchant of Asphodel
1 Howling Mine
1 Invoke Despair
1 Last Gasp
1 Leech Collector // Bloodletting
1 Lethal Throwdown
1 Memory Jar
1 Mikokoro, Center of the Sea
1 Mind Stone
1 Necrosquito
1 Ob Nixilis, the Fallen
1 Otherworld Atlas
1 Overkill
1 Peer into the Abyss
1 Perigee Beckoner
1 Phyrexian Arena
1 Phyrexian Delver
1 Phyrexian Rager
1 Potioner's Trove
1 Rooftop Assassin
1 Screeching Soulbreaker
1 Seizan, Perverter of Truth
1 Sephiroth's Intervention
1 Seymour Flux
1 Sheoldred, the Apocalypse
1 Silence the Echo
1 Sneering Shadewriter
1 Soul Enervation
1 Swiftfoot Boots
1 Syphon Mind
1 Terror
1 The One Ring
1 Undercity Dire Rat
1 Underworld Dreams
1 Vanquish the Weak
1 Vault Plunderer
1 Wander Off
1 Zulaport Cutthroat
32 Swamp
```
