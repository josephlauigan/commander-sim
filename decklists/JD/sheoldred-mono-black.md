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
**Sheoldred, the Apocalypse**: *Deathtouch. Whenever you draw a card, you gain 2 life. Whenever an opponent draws a
card, they lose 2 life.* Each of three opponents loses 2 life in their own draw step, so 6 life a round from the
draw steps alone, and every card you draw is 2 life back. She is a 4/5 deathtouch body for four mana, so she also
blocks and attacks well. She is the deck's one big engine: every draw spell, wheel and symmetric effect gets better
with her out. What breaks her is exile, bounce, or removal you can't answer. Keep a protection spell or a way to
bring her back (see "Protect her") before you start making the table draw.

### Game plan
- **Turns 1-3:** play a mana rock (Arcane Signet, Fellwar Stone, Mind Stone) or Dark Ritual into an early
  threat, and a cheap blocker or value creature (Viscera Seer, Undercity Dire Rat, Blood Artist, Vito, Thorn of the
  Dusk Rose, Vault Plunderer). **Underworld Dreams** ({B}{B}{B}) is a fine turn-3 play on its own: every
  card an opponent draws costs them 1.
- **Turns 4-6:** Sheoldred on turn 3 or 4. Once she is out, equip protection first, then start the draw engines.
- **Late game:** keep the engines running, drain with the creatures, and finish with Exsanguinate, Torment of
  Hailfire, Peer into the Abyss or the drain combo (see "The Coffers drain combo"). If Sheoldred dies, recast her, or
  return her with Phyrexian Delver (see "Recursion"). Demonic Tutor, Diabolic Tutor or Grim Tutor finds whichever you
  need.

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
1 life), **Ambition's Cost**, **Cast Away
Doubt** (draw two; 2 damage to each player, which Sheoldred's lifegain more than covers), **Syphon Mind** (each other
player discards; you draw a card for each), **Deadly Dispute** (sacrifice a creature or artifact, draw two, make a Treasure), **Skullclamp** (equip {1}: when
the creature dies, draw two), **Bonders' Enclave** (draw with {3} and a creature of power 4 or more, which Sheoldred is) and
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
from them. The cost stays moderate: the nonland cards average about 3.3 mana value and 36 of the 100 cards are lands (free), so a card costs about 2 life on average.

**Bolas's Citadel** ({3}{B}{B}{B}) lets you look at the top card of your library and play it, paying life equal to
its mana value instead of mana. The low curve keeps that cheap, and Sheoldred's lifegain keeps the life total up.
These cards are played, not drawn, so Sheoldred gains you nothing from them directly.

### Finishers
**Exsanguinate** (X life from each opponent, you gain it) is the plain kill. **Peer into the Abyss** is the draw-based
kill. **Torment of Hailfire** ({X}{B}{B}) is the second X spell: X times, each opponent loses 3 life
unless they sacrifice a nonland permanent or discard a card, so a big X off Cabal Coffers either kills or strips the
table.

### The Coffers drain combo
**Sanguine Bond** ({3}{B}{B}): whenever you gain life, target opponent loses that much. **Exquisite Blood** ({4}{B}):
whenever an opponent loses life, you gain that much. Together they are an infinite drain: any life gain or loss starts
the loop and it ends when every opponent is dead. **Vito, Thorn of the Dusk Rose** ({2}{B}, the same trigger as
Sanguine Bond on a creature) is a third piece, so any two of Bond, Blood and Vito win. Bond is good alone too: with
Sheoldred out every card you draw drains an opponent for 2, and an Exsanguinate for X gains you X per opponent, which
Bond turns into 3X damage to one opponent. **Cabal Coffers** with 31 Swamps pays for two pieces in one turn by the
mid-game. **Grim Tutor** ({1}{B}{B}, lose 3 life) is the third tutor beside Demonic and Diabolic Tutor. Added 11/04
(JD), not simulated. **Grave Titan** ({4}{B}{B}, 6/6 deathtouch) makes two 2/2 Zombies when it enters and each time it
attacks. The Zombies block for you and are the sacrifice fodder for Viscera Seer, Ashnod's Altar, Deadly Dispute and the
sacrifice-cost removal, and each one that dies drains with Blood Artist and Zulaport Cutthroat out.

### Protect her
- **Swiftfoot Boots** (hexproof and haste, equip {1}) and **Darksteel Plate** (indestructible, equip {2}): equip one
  as soon as Sheoldred lands. Boots stop targeted removal; the Plate stops destroy effects and damage, but not exile
  or -X/-X.
- **Perigee Beckoner**: when it enters, another creature of yours gets +2/+0 and returns if it dies this turn. Cast it
  on Sheoldred before she attacks or blocks into a risk; warp ({1}{B}) lets you reuse it later.
- **Recursion** (below) is the backup when protection isn't there, and **Demonic Tutor** ({1}{B}), **Diabolic Tutor** ({2}{B}{B}) or **Grim Tutor** ({1}{B}{B}) finds Sheoldred
  again if she was exiled or shuffled away.

### Remove threats
Seven removal spells and Grave Pact keep the board manageable: **Vanquish the Weak** (power 3 or less), **Overkill** (-0/-9999), **Foolish Fate**,
**Sephiroth's Intervention** (destroy, gain 2 life), **Wander Off** (exile), **Invoke Despair** (edict for a creature,
an enchantment and a planeswalker, with a loss of 2 life and a card for you each time they have none; needs {B}{B}{B}{B}),
and the sacrifice-cost cheap one **Silence the Echo** (cheap with a creature to sacrifice, or pay {3}). **Rooftop Assassin** (flash, flying, lifelink) destroys a creature that was dealt damage this turn, and
**Accursed Marauder** makes each player sacrifice a nontoken creature (it can sacrifice itself). **Grave Pact** ({1}{B}{B}{B}) makes each opponent sacrifice a creature whenever one of
yours dies, so every sacrifice for value is also removal.

### Gain life, use it
Sheoldred's lifegain turns on the lifegain payoffs: **Foolish Fate** (the target's controller loses 3 if you gained
life this turn) and **Leech Collector // Bloodletting** (the first lifegain each turn prepares {B} Bloodletting: each
opponent loses 2). The life is also a resource:
it pays for The One Ring's burden counters, Call of the Ring, Bolas's Citadel and Ad Nauseam.

### Drain creatures
**Blood Artist**, **Al Bhed Salvagers** and **Screeching Soulbreaker**
(drain 1 each attack) keep the pressure on when Sheoldred is gone. **Ob Nixilis, the Fallen** ({3}{B}{B}, 3/3) makes a
player lose 3 life each time a land enters under your control and grows by three +1/+1 counters: one land drop a turn
is 3 to an opponent and a bigger threat. **Gray Merchant of Asphodel** ({3}{B}{B}) drains each opponent by your
devotion to black when it enters, and you gain what they lose: Sheoldred ({B}{B}), Underworld Dreams ({B}{B}{B}),
Phyrexian Arena ({B}{B}), Seizan ({B}{B}) and the Merchant itself ({B}{B}) all count, so with Sheoldred and
Underworld Dreams out it is at least 7 from each. **Zulaport Cutthroat** drains each opponent
1 (and gains you 1) whenever a creature you control dies; the sacrifice outlets, Deadly Dispute, Silence the Echo,
Accursed Marauder and chump blocks feed it. Every point these drain creatures gain you is also a Sanguine Bond or Vito
trigger. **Undercity Dire Rat** is a cheap body that leaves a Treasure.

### The aristocrats package: a second way to win
**Mikaeus, the Unhallowed** ({3}{B}{B}{B}) gives your other non-Human creatures +1/+1 and undying. With **Triskelion**
({6}, enters with three +1/+1 counters; remove one: 1 damage to any target) it is an infinite-damage combo: remove
every counter, aim enough pings at Triskelion itself to kill it (with Mikaeus it is a 2/2 once the counters are gone, so
two pings), and the rest at an opponent. It dies with no counters, so undying returns it, and it comes back with its
three counters plus the undying one. Demonic Tutor, Diabolic Tutor and Grim Tutor find
the missing piece. **Yawgmoth, Thran Physician** (pay 1 life, sacrifice another creature: a -1/-1 counter on up to one
creature and draw a card) is a sacrifice outlet that draws; a -1/-1 counter cancels the +1/+1 counter undying leaves,
so Mikaeus's creatures can die and return again and again. **Viscera Seer** (sacrifice a creature: scry 1) and
**Ashnod's Altar** (sacrifice a creature: {C}{C}) are the free outlets, and **Phyrexian Tower** is a land that
sacrifices a creature for {B}{B}. Every death drains with Blood Artist, Zulaport Cutthroat and Al Bhed Salvagers, and
**Sephiroth, Fabled SOLDIER** drains 1 more for each other creature that dies; after four in one turn he transforms and
every creature death drains for the rest of the game. With Grave Pact out, each loop also costs every opponent a
creature.

### Recursion
**Phyrexian Delver** returns a creature card from your graveyard to the battlefield (you lose life equal to its mana
value): recover Sheoldred, or return Gray Merchant for another drain. **Raise Dead** ({B}) returns a creature to your hand,
not the battlefield; use it to recover Mikaeus, Triskelion, Yawgmoth, Vito or Gray Merchant, then pay to cast it again.
Moving Sheoldred to the command zone instead makes her unavailable to these graveyard effects.

### Ramp
Three rocks, Arcane Signet, Fellwar Stone and Mind Stone (it can cash in for a card), plus **Dark Ritual** (Sheoldred a
turn early), Ashnod's Altar and Undercity Dire Rat (a Treasure when it dies). Memory Jar, Otherworld Atlas, The One Ring
and Bolas's Citadel are the bigger artifacts you cast once the rocks are out.

### How it wins
- **The slow drain.** Sheoldred plus Howling Mine, Seizan and the end-step engines drains each of three opponents every round; with Underworld Dreams and Fate Unraveler out it is faster.
- **The burst.** Memory Jar with Sheoldred out is 14 life to each opponent at once. Peer into the Abyss on one
  opponent with Sheoldred out is lethal on its own. Exsanguinate for X is the table-wide finish.
- **Mikaeus + Triskelion**, or a Yawgmoth loop with Blood Artist, Zulaport Cutthroat and Sephiroth out.
- **Exquisite Blood + Sanguine Bond or Vito**: an infinite drain from any life gain or loss.
- **Cabal Coffers mana** into a big Exsanguinate or Torment of Hailfire.

### Weak spots and how to play around them
- **Exile, bounce and repeated removal on Sheoldred.** Play around it with Swiftfoot Boots and hold a protection spell;
  keep Phyrexian Delver, Demonic Tutor or a recast in reserve. Don't overextend into a wipe while she's your only
  engine. The One Ring's protection covers you for the turn it enters.
- **Paying life without Sheoldred.** The One Ring's upkeep loss, Call of the Ring and Bolas's Citadel all cost life;
  without her lifegain, stop tapping the Ring and play fewer cards off the Citadel.
- **Opponents who stop draws or punish the symmetric cards.** If a deck is built around card draw, or has a combo that
  wants cards, hold the symmetric draw cards (the Mine, the Atlas, Mikokoro, the Sanitarium, Seizan, the wheels) until
  a punisher is out and kill that deck first.
- **Big creature decks.** Use Sheoldred's deathtouch, Grave Pact and the removal; save Overkill
  and Wander Off for the biggest threat.
- **Lifegain decks.** The drain is slower against them; push Exsanguinate and the drain creatures for the last points.

## Key lines

- **Turns 1-3:** a rock and a cheap blocker; Sheoldred on turn 3 or 4. Underworld Dreams is a fine turn-3 play on its
  own.
- **With Sheoldred out:** equip Swiftfoot Boots or Darksteel Plate first, then the draw engines: Howling Mine, Seizan,
  Mikokoro and Otherworld Atlas at each opponent's end step, Memory Jar the turn she's safe. Hold a protection instant
  for removal aimed at her. The One Ring and Bolas's Citadel turn her lifegain into cards.
- **Without her:** removal, Underworld Dreams, Fate Unraveler and the drain creatures (Blood Artist, Al Bhed Salvagers,
  Screeching Soulbreaker, Gray Merchant, Zulaport Cutthroat) keep the pressure on until she
  comes back; hold the symmetric draw cards.
- **The kill:** Peer into the Abyss on an opponent with Sheoldred out, Exsanguinate or Torment of Hailfire off Cabal
  Coffers for the table, or Exquisite Blood with Sanguine Bond or Vito. Demonic, Diabolic or Grim Tutor finds Peer into
  the Abyss once she is safe, or the missing half of Mikaeus + Triskelion or of the drain combo.
- **Mulligan:** keep hands with two or three lands, a rock and a creature or a draw spell; send back hands with no
  black source.

## Mana and consistency

36 lands: 31 Swamp, Cabal Coffers, Phyrexian Tower, and three colorless lands that draw cards:
Bonders' Enclave, Mikokoro, Center of the Sea and Geier Reach Sanitarium. That is 31 black sources, plus Phyrexian
Tower with a creature to sacrifice. **Cabal Coffers** ({2}, {T}: add {B} for each Swamp you control) took Command Tower's slot (JD, 2026-11-04): with 31 Swamps it needs no Urborg, and six Swamps out make it net {B}{B}{B}{B}, more every turn, for Exsanguinate and the big turns. Ramp: three mana rocks (Arcane Signet, Fellwar Stone, Mind Stone), Dark Ritual,
Ashnod's Altar and Undercity Dire Rat. Tutors: Demonic
Tutor, Diabolic Tutor and Grim Tutor.
Interaction: seven removal spells and Grave Pact, 2 protection cards (Swiftfoot Boots, Darksteel Plate) and
Perigee Beckoner.

## Bracket and Rule 0

Bracket 4: four Game Changers (Ad Nauseam, The One Ring, Demonic Tutor, Bolas's Citadel) and the combos named here.
There are no extra turns and no mass land destruction, but there are infinite combos: Mikaeus + Triskelion is a
two-card infinite-damage combo (Commander Spellbook lists it), Mikaeus + Yawgmoth + Gray Merchant of Asphodel is a
three-card loop (it needs a second nontoken non-Human creature with toughness 2 or more), Exquisite Blood + Sanguine
Bond (or + Vito, Thorn of the Dusk Rose) is a two-card infinite drain, and Sheoldred + Peer into the
Abyss (or Underworld Dreams + Peer into the Abyss) is a near-instant kill on an opponent whose library is large
compared with their life. Name these before the game. Tell the table about the four Game
Changers, and tell it that Howling Mine, Memory Jar, Dark Deal and the other draw cards are symmetric on purpose: they make everyone
draw, and Sheoldred, Underworld Dreams and Fate Unraveler are what make that cost the opponents.

## Decklist by type (100)

**Commander (1).** Sheoldred, the Apocalypse

**Lands (36).** Bonders' Enclave, Cabal Coffers, Geier Reach Sanitarium, Mikokoro Center of the Sea, Phyrexian Tower, 31 Swamp

**Creatures (23).** Accursed Marauder, Al Bhed Salvagers, Blood Artist, Fate Unraveler, Gix Yawgmoth Praetor, Grave Titan, Gray Merchant of Asphodel, Leech Collector // Bloodletting, Mikaeus the Unhallowed, Ob Nixilis the Fallen, Perigee Beckoner, Phyrexian Delver, Rooftop Assassin, Screeching Soulbreaker, Seizan Perverter of Truth, Sephiroth Fabled SOLDIER // Sephiroth One-Winged Angel, Triskelion, Undercity Dire Rat, Vault Plunderer, Viscera Seer, Vito Thorn of the Dusk Rose, Yawgmoth Thran Physician, Zulaport Cutthroat

**Enchantments (6).** Call of the Ring, Exquisite Blood, Grave Pact, Phyrexian Arena, Sanguine Bond, Underworld Dreams

**Artifacts (12).** Arcane Signet, Ashnod's Altar, Bolas's Citadel, Darksteel Plate, Fellwar Stone, Howling Mine, Memory Jar, Mind Stone, Otherworld Atlas, Skullclamp, Swiftfoot Boots, The One Ring

**Sorceries (14).** Ambition's Cost, Cast Away Doubt, Cost of Brilliance, Dark Deal, Demonic Tutor, Diabolic Tutor, Exsanguinate, Grim Tutor, Invoke Despair, Peer into the Abyss, Raise Dead, Silence the Echo, Syphon Mind, Torment of Hailfire

**Instants (8).** Ad Nauseam, Dark Ritual, Deadly Dispute, Foolish Fate, Overkill, Sephiroth's Intervention, Vanquish the Weak, Wander Off

## Import list (100)

```
1 Accursed Marauder
1 Ad Nauseam
1 Al Bhed Salvagers
1 Ambition's Cost
1 Arcane Signet
1 Ashnod's Altar
1 Blood Artist
1 Bolas's Citadel
1 Bonders' Enclave
1 Cabal Coffers
1 Call of the Ring
1 Cast Away Doubt
1 Cost of Brilliance
1 Dark Deal
1 Dark Ritual
1 Darksteel Plate
1 Deadly Dispute
1 Demonic Tutor
1 Diabolic Tutor
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
1 Leech Collector // Bloodletting
1 Memory Jar
1 Mikaeus, the Unhallowed
1 Mikokoro, Center of the Sea
1 Mind Stone
1 Ob Nixilis, the Fallen
1 Otherworld Atlas
1 Overkill
1 Peer into the Abyss
1 Perigee Beckoner
1 Phyrexian Arena
1 Phyrexian Delver
1 Phyrexian Tower
1 Raise Dead
1 Rooftop Assassin
1 Sanguine Bond
1 Screeching Soulbreaker
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
1 Vanquish the Weak
1 Vault Plunderer
1 Viscera Seer
1 Vito, Thorn of the Dusk Rose
1 Wander Off
1 Yawgmoth, Thran Physician
1 Zulaport Cutthroat
31 Swamp
```
