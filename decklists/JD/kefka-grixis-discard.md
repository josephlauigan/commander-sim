# Kefka, Court Mage — Grixis Discard

A Grixis (blue-black-red) discard-and-draw deck. Kefka makes every player discard and draws you a card for each card
type in the pile; the deck is full of cards that want to be discarded, so the symmetry costs you little. Once Kefka
has flipped, every point of life the table loses on your turn is a card for you, and Psychosis Crawler or
Niv-Mizzet, the Firemind turns that into a two-card win. A second plan, Underworld Breach with a storm count, kills
from the graveyard. It is weak to graveyard hate, to fast pressure that kills you before the flip, and to wipes that
take Kefka with them.

| | |
|---|---|
| **Commander** | Kefka, Court Mage ({2}{U}{B}{R}, 4/5, Grixis) // Kefka, Ruler of Ruin (5/7 flying) |
| **Bracket** | 4 (2 Game Changers: Jeska's Will, Underworld Breach; Bracket 4 for the Crawler/Niv two-card win, see Bracket and Rule 0) |
| **Simulator** | deck key `kefka`; results in `decklists/pool/pool-results.md`, history in `decklists/CHANGELOG.md` |

## Strategy

### The commander
**Kefka, Court Mage** ({2}{U}{B}{R}, 4/5) reads: *whenever Kefka enters or attacks, each player discards a card.
Then you draw a card for each card type among cards discarded this way. {8}: each opponent sacrifices a permanent of
their choice. Transform Kefka. Activate only as a sorcery.* The back face, **Kefka, Ruler of Ruin** (5/7, flying),
reads: *whenever an opponent loses life during your turn, you draw that many cards.*

Every player loses a card, and you draw one for each card type in the pile: an artifact creature counts twice, a
land and an instant three different types. Two to four cards for your one is normal. Kefka triggers on entering and
on every attack, so he is a draw engine even before he flips. The flip costs {8}, is sorcery speed and makes every
opponent sacrifice a permanent. After it, Kefka turns the whole table's life loss into cards. He is a 4/5 that must
survive to attack, so protect him (see Protection), and note that he is always available from the command zone if he
is removed.

### Game plan
- **Turns 1 to 3.** Develop cheap looters and discard fodder (Rummaging Goblin, Sawblade Scamp, Rank Rat, Blightspeaker),
  play mana rocks (Arcane Signet, Sisay's Ring, Lotus Petal) and a land every turn.
  Keep the cards that want to be discarded in hand rather than playing them out.
- **Turns 4 to 6.** Land Kefka (with Lightning Greaves for two triggers in one turn), keep attacking when it is
  safe, and use each trigger to bin madness, mayhem and graveyard-castable cards. Build toward {8}: Treasure,
  rocks, Dark Ritual and burst mana. Cast Psychosis Crawler early (Kefka's own draws and the looters ping the table with it); hold it only once Kefka has flipped and your library is not longer than the highest opponent life.
- **Late game.** Flip, then cast pingers and one-shots in the same main phase so every point of life lost draws you
  a card. Finish with the Crawler or Niv-Mizzet loop, or with the Breach and storm line.

### Cards that want to be discarded
- **Scarlet Spider, Kaine** (mayhem: cast from the graveyard for {B/R} the turn you discard it), **Demonic Embrace**
  (cast from the graveyard by paying 3 life and discarding another card), **Twinned Vision** (flashback draws two)
  and **Eternal Student** (exile from the graveyard for two flying Inklings).
- **Stinkweed Imp** ({2}{B}, a 1/2 flier that destroys any creature it deals combat damage to) has dredge 5: once it
  is in the graveyard, any draw, Kefka's included, can mill five cards instead and return it to your hand. Discard it
  again for Kefka; the milled cards feed Underworld Breach and Yawgmoth's Will. Dredge needs five cards left in the
  library.
- **Garrison Excavator** makes a 2/2 Spirit every time cards leave your graveyard, so each of those becomes a body too.
- **Underworld Breach** turns the discard pile into a second hand. Every nonland card in your graveyard gains escape:
  cast it for its mana cost plus exiling three other cards. Kefka's discards and the looters fill the graveyard, so
  on the flip turn Breach recasts Lightning Bolt, Grab the Prize and the other one-shots, and every point of life
  lost is still a card. Each escape is two Garrison Excavator Spirits: one for the three exiled cards, one for the
  escaped card leaving. Breach is sacrificed at your end step, so cast it on the turn you use it.
- **Decorum Dissertation** ({3}{B}{B} sorcery, Paradigm): you draw two and lose 2 life, then the spell is exiled
  and you cast a free copy at the start of each of your first main phases, so it refills the hand for every Kefka
  trigger with no mana spent.

### Outlets and looting
Rummaging Goblin, Thrill of Possibility, Grab the Prize (2 damage to each opponent if the discard
isn't a land), Stadium Tidalmage, Alpharael, Rapturous Moment, Big Score and Seize the Spoils. They pick the right
card to bin before Kefka's trigger and fix the hand after it. Rapturous Moment, Big Score and Seize the Spoils also
make mana or Treasure toward the {8}. **Skullclamp** keeps the hand full between triggers.

### Kuja
**Kuja, Genome Sorcerer** ({2}{B}{R}, 3/4) makes a tapped 0/1 Wizard at each of your end steps. Each token deals 1
damage to each opponent whenever you cast a noncreature spell, so every discard outlet, ritual and Kefka spell burns
the table. With four Wizards (Kefka and Niv-Mizzet count) he flips into Trance Kuja, Fate Defied, a 4/6 that doubles
Wizard damage. Cast him before a spell-heavy turn.

### Pressure on their hands
**Rank Rat** and **Cackling Fiend** each make every opponent discard a card when they enter. **Intellect Devourer**
makes each opponent exile a card from hand and lets you play it, spending mana as though it were any colour.
Opponents low on cards discard their best one to Kefka.

### Ramp and the {8}
Treasure (Big Score, Seize the Spoils, Contract Killing, Skullport Merchant), the rocks (Arcane Signet, Sisay's Ring,
Lotus Petal), **Dark Ritual**, **Jeska's Will** (red mana for each card in an opponent's hand, or exile the top three
cards and play them; with a commander on the battlefield you may do both) and the burst cards get you there:
Rapturous Moment refunds five.

### Making them lose life
- **Repeatable pings:** Sawblade Scamp, Viashino Fangtail, Blightspeaker, Screeching Soulbreaker, Susurian Voidborn
  (each creature or artifact of yours that dies drains one opponent, and Treasure you crack counts).
- **One-shots:** Shimmercreep (each opponent loses 3 with three colours on the battlefield), Sneering Shadewriter,
  Corroding Dragonstorm, Grab the Prize, Lightning Bolt and Massacre Wurm (2 for each of their creatures that dies).
- **Combat:** a flying 5/7 that hits for 5 draws you 5.

### Tutors
**Diabolic Tutor**, **Profane Tutor** (suspended for {1}{B}, it finds any card two turns later) and **Grim Servant** (a card
with mana value up to your devotion to black; you lose 3 life). They are the only tutors, so they are slow; use them to find the Crawler or Niv-Mizzet, Underworld Breach or Yawgmoth's
Will, whichever the board needs.

### Removal, counters and wipes
**Lightning Bolt**, **Pongify** (they get a 3/3 Ape), **Reality Shift** (they manifest the top
card), **Extinguish the Light**, **Deadly Embrace**
(draws a card for each creature that died this turn), **Contract Killing** (two Treasure) and **Chaos Warp**
(the owner shuffles the permanent away and may get a permanent from the top of the library, so use it on the
biggest threat). **Arcane Denial** counters a spell (its controller may draw up to two cards next upkeep, and you draw one) and **Mana Leak** ({1}{U}) counters a spell unless its controller pays {3}, which is usually a hard counter once the table is tapped out. **Massacre Wurm** is the sweeper: it kills the opponents' small creatures, and each one costs its controller 2 life.

### Protection
**Lightning Greaves** gives haste and shroud with a free equip. **Deflecting Swat** is free while you control a
commander and gives a spell or ability new targets, so it redirects removal or a counter. **Slip Out the Back**
({U}) puts a +1/+1 counter on a creature and phases it out.

### Graveyard and storm
**Underworld Breach**, **Brain Freeze**, **Dark Ritual**, **Lotus Petal**, **Grapeshot** and **Yawgmoth's Will**
(see Key lines).

### How it wins
**Psychosis Crawler + Kefka, Ruler of Ruin.** Each card you draw makes each opponent lose 1; each opponent's loss
draws you a card. In a four-player game every card drawn draws three more, so the loop runs until the table is dead
or your library is. It is mandatory: count your library against the highest life total before starting it, and play
the Crawler after the flip, not before, when your library is short. Before the flip the Crawler is still worth
casting early: Kefka's own draws and the looters ping the whole table with it.

**Niv-Mizzet, the Firemind + Kefka, Ruler of Ruin.** Each card you draw deals 1 damage to any target; damage to an
opponent on your turn draws you a card, which pings again. One ping hits one opponent, so the loop needs a library
larger than the opponents' life added together, but you choose every target: when the library is short, aim the
pings at creatures (or at yourself) and stop. Niv also taps to draw, which starts it.

**Fiery Emancipation** ({3}{R}{R}{R}) triples the damage your sources deal. With Psychosis Crawler or Niv-Mizzet each card
you draw becomes 3 damage (to each opponent with the Crawler), so the flipped Kefka's loop needs a third of the
library, and Grapeshot copies and Lightning Bolt hit three times as hard. Cast it once the Crawler or Niv is out or in
hand, or after the flip; otherwise it waits (six mana is a whole turn).

**Bribery** ({3}{U}{U}) puts the best creature from an opponent's library onto your battlefield: a bomb or a combo
creature on your side, and one fewer in their deck. Pick the opponent whose deck holds the biggest threat.

**Underworld Breach + storm.** See Key lines. It is a mill, not a two-card infinite.

**Jin-Gitaxias, Progress Tyrant** copies the first artifact, instant or sorcery you cast each turn (a second
Diabolic Tutor, Night's Whisper or Lightning Bolt) and counters the first one each opponent casts. **Night's
Whisper** is a plain two-card draw.

### Weak spots and how to play around them
- **Graveyard hate** hurts Breach, Yawgmoth's Will and every cast-from-graveyard card. Without them, Kefka still
  draws and the Crawler loop still works.
- **Removal before the flip.** Kefka is a 4/5 that has to attack or survive a turn cycle. Hold Deflecting Swat,
  Slip Out the Back or Lightning Greaves mana, and remember he can be recast from the command zone.
- **Short library.** Never start the Crawler or Niv loop without counting your library against the highest life total.
- **Hand size.** Discard down to seven is the limit to watch after a flip turn.
- **Fast pressure and large creatures.** You win late. Spend removal on the threats that would kill you first, and
  use **Massacre Wurm** or Arcane Denial and Jin-Gitaxias to buy time.

## Key lines

**Kefka with haste.** Lightning Greaves on the turn he lands means two triggers in one turn: the enter trigger and the attack trigger, two discards for the table and two rounds of draws for you.

**Discard into value.** With Kefka's trigger on the stack, keep Scarlet Spider or Demonic Embrace in hand to discard: Scarlet Spider for {B/R} from the graveyard this turn, and Demonic Embrace any time later.

**The flip turn.** Flip at sorcery speed, then play pingers and one-shots in the same main phase: every point of life the table loses before your turn ends is a card. Massacre Wurm after the flip is often 6 to 12 cards.

**Underworld Breach + storm.** Breach (cast it for {1}{R}) also works as a second kill, with **Brain Freeze**, **Dark Ritual**, **Lotus Petal** and **Grapeshot**. Cast Brain Freeze on yourself to fill the graveyard (storm: one copy for each spell cast earlier in the turn, three cards milled per copy), then escape Dark Ritual and Lotus Petal again and again for mana and storm count (each escape exiles three other cards), escape Brain Freeze and point the copies at the opponents, who lose when they next draw from an empty library. Grapeshot is the same storm count as damage: one ping per copy to any target. Breach is sacrificed at your end step, so start only when you can finish, ideally into tapped-out opponents.

**Yawgmoth's Will: a second storm turn.** With Kefka's discards and the looters filling the graveyard, Will (cast it for {2}{B}) lets you cast cards from the graveyard this turn, so it replays **Dark Ritual**, **Lotus Petal**, **Brain Freeze** and **Grapeshot** from the pile. Diabolic Tutor finds Will, Breach or the missing piece, and Profane Tutor does the same two turns after it is suspended. Cards that would go to the graveyard this turn are exiled instead, so Will is a one-time engine.

**Decorum Dissertation goes down early.** It costs five once; after that Paradigm casts a free copy at the start of each of your first main phases, two cards a turn for 2 life. Cast it on a quiet turn before the flip, so the extra cards feed Kefka's discards and the graveyard for Breach and Yawgmoth's Will. Watch your life total: each copy costs 2.

**Skullclamp** and **Decorum Dissertation** keep the hand full between Kefka triggers.

**Mulligan.** Keep hands with lands, a rock or a cheap looter and a piece of interaction or card flow. A hand of only expensive spells and no way to discard them is slow; Kefka and the looters turn spare cards into new ones.

## Mana and consistency

- **Lands:** 36 (12 nonbasic + 24 basic)
- **Mana:** 3 rocks (Arcane Signet, Sisay's Ring, Lotus Petal), Dark Ritual, Jeska's Will, Treasure makers and burst cards.  Yawgmoth's Will replays Dark Ritual and Lotus Petal from the graveyard.
- **Card draw:** Kefka, Night's Whisper, Decorum Dissertation (a free draw-two each turn after the first cast), Skullclamp, Niv-Mizzet (taps to draw), Marvin, Murderous Mimic (a 2-mana 2/2 that has the activated abilities of your other creatures: it copies Niv-Mizzet's draw, Fangtail's ping, Blightspeaker and Rummaging Goblin), the looters, and the cheap digging spells Expressive Iteration (look at three: one to hand, one to the bottom, one exiled and playable this turn) and Sleight of Hand (look at two, one to hand). **Tutors:** Diabolic Tutor, Profane Tutor (suspended), Grim Servant (devotion to black). **Graveyard:** Underworld Breach, Yawgmoth's Will.
- **Interaction:** Lightning Bolt, Pongify, Reality Shift, Extinguish the Light, Deadly Embrace, Contract Killing, Chaos Warp; counters Arcane Denial and Mana Leak, plus Jin-Gitaxias countering each opponent's first artifact, instant or sorcery; one-sided sweeper Massacre Wurm.
- **Protection:** Lightning Greaves (haste), Deflecting Swat (free while you control a commander: redirects a removal spell or counter) and Slip Out the Back ({U}: phases a creature out).

## Bracket and Rule 0

**Bracket 4,** with two Game Changers: **Jeska's Will** and **Underworld Breach**. Two Game Changers alone would fit Bracket 3; the deck is Bracket 4 because of the two-card win. There is no two-card infinite combo, but Psychosis Crawler or Niv-Mizzet, the Firemind with the flipped Kefka is a two-card win: Kefka is always available from the command zone, so the deck only needs the Crawler or Niv, Kefka on the battlefield and {8} for the flip. Three tutors (Diabolic Tutor, Profane Tutor and Grim Servant). The Breach + Brain Freeze line is a second way to win, but it is a mill, not a two-card infinite: it needs the graveyard to fill first.

Disclose before the game:
- **Jeska's Will** and **Underworld Breach** (the Game Changers). Breach recasts the graveyard, so mention it as a combo enabler.
- **The Crawler and Niv-Mizzet loops** with Kefka, Ruler of Ruin.
- **The Breach + Brain Freeze line**, which can mill an opponent out in one turn (Dark Ritual, Lotus Petal and Grapeshot feed the storm count).
- **Jin-Gitaxias, Progress Tyrant**, which counters the first artifact, instant or sorcery each opponent casts each turn.
- **Symmetric discard** every time Kefka enters or attacks.

Re-check the official list at https://commanderbrackets.com/faq before an event.

## Decklist by type (100)

**Commander (1).** Kefka, Court Mage // Kefka, Ruler of Ruin

**Creatures (25).** Alpharael Dreaming Acolyte, Blightspeaker, Cackling Fiend, Eternal Student, Garrison Excavator, Grim Servant, Intellect Devourer, Jin-Gitaxias Progress Tyrant, Kuja Genome Sorcerer // Trance Kuja Fate Defied, Marvin Murderous Mimic, Massacre Wurm, Niv-Mizzet the Firemind, Psychosis Crawler, Rank Rat, Rummaging Goblin, Sawblade Scamp, Scarlet Spider Kaine, Screeching Soulbreaker, Shimmercreep, Skullport Merchant, Sneering Shadewriter, Stadium Tidalmage, Stinkweed Imp, Susurian Voidborn, Viashino Fangtail

**Enchantments (4).** Corroding Dragonstorm, Demonic Embrace, Fiery Emancipation, Underworld Breach

**Artifacts (5).** Arcane Signet, Lightning Greaves, Lotus Petal, Sisay's Ring, Skullclamp

**Sorceries (15).** Bribery, Contract Killing, Deadly Embrace, Decorum Dissertation, Diabolic Tutor, Expressive Iteration, Grab the Prize, Grapeshot, Jeska's Will, Night's Whisper, Profane Tutor, Rapturous Moment, Seize the Spoils, Sleight of Hand, Yawgmoth's Will

**Instants (14).** Arcane Denial, Big Score, Brain Freeze, Chaos Warp, Dark Ritual, Deflecting Swat, Extinguish the Light, Lightning Bolt, Mana Leak, Pongify, Reality Shift, Slip Out the Back, Thrill of Possibility, Twinned Vision

**Lands (36).** Blood Crypt, Command Tower, Exotic Orchard, Gemstone Mine, Haunted Ridge, Shivan Reef, Spirebluff Canal, Steam Vents, Stormcarved Coast, Sulfur Falls, Sulfurous Springs, Underground River, 6 Island, 5 Mountain, 13 Swamp

## Import list (100)

```
1 Alpharael, Dreaming Acolyte
1 Arcane Denial
1 Arcane Signet
1 Big Score
1 Blightspeaker
1 Blood Crypt
1 Brain Freeze
1 Bribery
1 Cackling Fiend
1 Chaos Warp
1 Command Tower
1 Contract Killing
1 Corroding Dragonstorm
1 Dark Ritual
1 Deadly Embrace
1 Decorum Dissertation
1 Deflecting Swat
1 Demonic Embrace
1 Diabolic Tutor
1 Eternal Student
1 Exotic Orchard
1 Expressive Iteration
1 Extinguish the Light
1 Fiery Emancipation
1 Garrison Excavator
1 Gemstone Mine
1 Grab the Prize
1 Grapeshot
1 Grim Servant
1 Haunted Ridge
1 Intellect Devourer
1 Jeska's Will
1 Jin-Gitaxias, Progress Tyrant
1 Kefka, Court Mage // Kefka, Ruler of Ruin
1 Kuja, Genome Sorcerer // Trance Kuja, Fate Defied
1 Lightning Bolt
1 Lightning Greaves
1 Lotus Petal
1 Mana Leak
1 Marvin, Murderous Mimic
1 Massacre Wurm
1 Night's Whisper
1 Niv-Mizzet, the Firemind
1 Pongify
1 Profane Tutor
1 Psychosis Crawler
1 Rank Rat
1 Rapturous Moment
1 Reality Shift
1 Rummaging Goblin
1 Sawblade Scamp
1 Scarlet Spider, Kaine
1 Screeching Soulbreaker
1 Seize the Spoils
1 Shimmercreep
1 Shivan Reef
1 Sisay's Ring
1 Skullclamp
1 Skullport Merchant
1 Sleight of Hand
1 Slip Out the Back
1 Sneering Shadewriter
1 Spirebluff Canal
1 Stadium Tidalmage
1 Steam Vents
1 Stinkweed Imp
1 Stormcarved Coast
1 Sulfur Falls
1 Sulfurous Springs
1 Susurian Voidborn
1 Thrill of Possibility
1 Twinned Vision
1 Underground River
1 Underworld Breach
1 Viashino Fangtail
1 Yawgmoth's Will
6 Island
5 Mountain
13 Swamp
```
