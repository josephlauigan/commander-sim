# Kefka, Court Mage — Grixis Discard

A Grixis (blue-black-red) discard-and-draw deck. Kefka makes every player discard and draws you a card for each card
type in the pile; the deck is full of cards that want to be discarded, so the symmetry costs you little. Once Kefka
has flipped, every point of life the table loses on your turn is a card for you, and Psychosis Crawler or
Niv-Mizzet, the Firemind turns that into a two-card win. A second plan, Underworld Breach with a storm count, kills
from the graveyard. Since 10/31 the deck also carries Veyran's spell engine: pingers that hit every opponent whenever you
cast a spell (Guttersnipe, Kessig Flamebreather, Firebrand Archer), doubled by Veyran and Harmonic Prodigy, so after the
flip each cheap spell is a handful of cards. It is weak to graveyard hate, to fast pressure that kills you before the
flip, and to wipes that take Kefka with them.

| | |
|---|---|
| **Commander** | Kefka, Court Mage ({2}{U}{B}{R}, 4/5, Grixis) // Kefka, Ruler of Ruin (5/7 flying) |
| **Bracket** | 4 (3 Game Changers: Jeska's Will, Mystical Tutor, Underworld Breach; Bracket 4 for the Crawler/Niv two-card win, see Bracket and Rule 0) |
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
- **Turns 1 to 3.** Develop cheap looters and pingers (Rummaging Goblin, Blightspeaker, Harmonic Prodigy, Kessig
  Flamebreather, Firebrand Archer, Young Pyromancer), play Arcane Signet or Lotus Petal and a land every turn.
  Keep the cards that want to be discarded in hand rather than playing them out.
- **Turns 4 to 6.** Land Kefka (with Lightning Greaves for two triggers in one turn), keep attacking when it is
  safe, and use each trigger to bin madness, mayhem and graveyard-castable cards. Build toward {8}: Treasure,
  rocks, Dark Ritual and burst mana. Cast Psychosis Crawler early (Kefka's own draws and the looters ping the table with it); hold it only once Kefka has flipped and your library is not longer than the highest opponent life.
- **Late game.** Flip, then cast cheap spells with the pingers out, so every point of life lost draws you a card.
  Finish with the Crawler or a Niv-Mizzet loop, or with the Breach and storm line.

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
Rummaging Goblin, Frantic Search (draw two, discard two, untap three lands: free), Grab the Prize (2 damage to each opponent if the discard
isn't a land), Stadium Tidalmage, Alpharael, Rapturous Moment, Big Score and Seize the Spoils. They pick the right
card to bin before Kefka's trigger and fix the hand after it. Rapturous Moment, Big Score and Seize the Spoils also
make mana or Treasure toward the {8}. **Skullclamp** keeps the hand full between triggers.

### Kuja
**Kuja, Genome Sorcerer** ({2}{B}{R}, 3/4) makes a tapped 0/1 Wizard at each of your end steps. Each token deals 1
damage to each opponent whenever you cast a noncreature spell, so every discard outlet, ritual and Kefka spell burns
the table. With four Wizards (Kefka, both Niv-Mizzets, Veyran and Harmonic Prodigy count) he flips into Trance Kuja,
Fate Defied, a 4/6 that doubles Wizard damage. His tokens are Wizards, so Harmonic Prodigy doubles their pings too.
Cast him before a spell-heavy turn.

### The spell engine (from Veyran, 10/31)
- **Pingers:** **Guttersnipe** (2 damage to each opponent per instant or sorcery), **Kessig Flamebreather** and
  **Firebrand Archer** (1 to each opponent per noncreature spell). **Young Pyromancer** makes a 1/1 per instant or
  sorcery: blockers against the creature decks, and bodies for Skullclamp and Susurian Voidborn.
- **Doublers:** **Veyran, Voice of Duality** makes every trigger caused by casting or copying an instant or sorcery
  happen an extra time (the pingers, Young Pyromancer, Storm-Kiln Artist, Kuja's tokens). **Harmonic Prodigy** does the
  same for every Shaman and Wizard trigger: Guttersnipe, Kessig, Young Pyromancer, Kuja's tokens, both Niv-Mizzets and
  **Kefka himself**, enter and attack triggers and the flipped draw. They stack: with both out, Guttersnipe deals 6 to
  each opponent per instant or sorcery, and after the flip that is 18 cards.
- **Niv-Mizzet, Parun** ({U}{U}{U}{R}{R}{R}, can't be countered) draws a card whenever any player casts an instant or
  sorcery and deals 1 damage per card you draw: the same loop as the Firemind with the flipped Kefka.

### Pressure on their hands
**Intellect Devourer**
makes each opponent exile a card from hand and lets you play it, spending mana as though it were any colour.
Opponents low on cards discard their best one to Kefka.

### Ramp and the {8}
Treasure (Big Score, Seize the Spoils, Contract Killing, Skullport Merchant, and **Storm-Kiln Artist**: a Treasure per
instant or sorcery), Arcane Signet and Lotus Petal, **Birgi, God of Storytelling** ({R} whenever you cast a spell),
**Dark Ritual**, **Seething Song** (Blazing Firesinger's instant side), **Jeska's Will** (red mana for each card in an opponent's hand, or exile the top three
cards and play them; with a commander on the battlefield you may do both) and the burst cards get you there:
Rapturous Moment refunds five.

### Making them lose life
- **On every spell:** Guttersnipe, Kessig Flamebreather, Firebrand Archer and Kuja's tokens (see The spell engine).
- **Repeatable pings:** Blightspeaker, Susurian Voidborn (each creature or artifact of yours that dies drains one
  opponent, and Treasure you crack counts).
- **One-shots:** Corroding Dragonstorm, Grab the Prize, Lightning Bolt and Massacre Wurm (2 for each of their creatures
  that dies).
- **Price of Progress** ({1}{R}, instant) deals 2 damage to each player for each nonbasic land they control. Kefka runs
  20 basics, so it costs you little, and on your turn with Kefka flipped each opponent's loss draws you that many cards:
  three opponents with four nonbasics each is 24 damage and 24 cards. Not with Fiery Emancipation out unless it kills:
  tripled, it can draw you out of cards (see Key lines).
- **Combat:** a flying 5/7 that hits for 5 draws you 5.

### Tutors
**Diabolic Tutor**, **Profane Tutor** (suspended for {1}{B}, it finds any card two turns later), **Mystical Tutor** ({U},
instant: an instant or sorcery to the top) and **Solve the Equation** ({2}{U}: an instant or sorcery to your hand). Use
Diabolic and Profane for the Crawler, Niv-Mizzet or Underworld Breach; Mystical and Solve for Brain Freeze, Grapeshot,
Yawgmoth's Will or Jeska's Will, whichever the board needs.

### Removal, counters and wipes
**Lightning Bolt**, **Reality Shift** (they manifest the top
card), **Extinguish the Light**, **Deadly Embrace**
(draws a card for each creature that died this turn), **Contract Killing** (two Treasure) and **Chaos Warp**
(the owner shuffles the permanent away and may get a permanent from the top of the library, so use it on the
biggest threat). **Arcane Denial** counters a spell (its controller may draw up to two cards next upkeep, and you draw one) and **Counterspell** ({U}{U}) counters anything. **Massacre Wurm** is the sweeper: it kills the opponents' small creatures, and each one costs its controller 2 life.

### Protection
**Lightning Greaves** gives haste and shroud with a free equip. **Counterspell** and **Arcane Denial** answer removal
on the stack. **Mistrise Village** ({U}, {T}: your next spell this turn can't be countered) protects the storm turn.

### Graveyard and storm
**Underworld Breach**, **Brain Freeze**, **Dark Ritual**, **Lotus Petal**, **Grapeshot** and **Yawgmoth's Will**
(see Key lines). **Think Twice** has flashback, so a discarded copy is still two casts and two rounds of pinger
triggers. **Mystic Sanctuary** (enters untapped with three other Islands) puts an instant or sorcery from the graveyard
back on top: Grapeshot, Brain Freeze or Frantic Search.

### How it wins
**Psychosis Crawler + Kefka, Ruler of Ruin.** Each card you draw makes each opponent lose 1; each opponent's loss
draws you a card. In a four-player game every card drawn draws three more, so the loop runs until the table is dead
or your library is. It is mandatory: count your library against the highest life total before starting it, and play
the Crawler after the flip, not before, when your library is short. Before the flip the Crawler is still worth
casting early: Kefka's own draws and the looters ping the whole table with it.

**Niv-Mizzet, the Firemind + Kefka, Ruler of Ruin.** Each card you draw deals 1 damage to any target; damage to an
opponent on your turn draws you a card, which pings again. One ping hits one opponent, so the loop needs a library
larger than the opponents' life added together, but you choose every target: when the library is short, aim the
pings at creatures (or at yourself) and stop. Niv also taps to draw, which starts it. **Niv-Mizzet, Parun** runs the
same loop, started by any instant or sorcery.

**The spell turn after the flip.** With Guttersnipe (and Veyran or Harmonic Prodigy) out, each cheap instant or
sorcery is several points of life lost by each opponent and that many cards for you: chain Frantic Search, Think
Twice, Dark Ritual and the tutors into Grapeshot or Brain Freeze.

**Fiery Emancipation** ({3}{R}{R}{R}) triples the damage your sources deal. With Psychosis Crawler or Niv-Mizzet each card
you draw becomes 3 damage (to each opponent with the Crawler), so the flipped Kefka's loop needs a third of the
library, and Grapeshot copies and Lightning Bolt hit three times as hard. Cast it once the Crawler or Niv is out or in
hand, or after the flip; otherwise it waits (six mana is a whole turn).

**Price of Progress after the flip** is a burst of cards: every point each opponent loses draws you one. With Fiery
Emancipation out the damage triples, and so do the draws (often 60 or more), which can empty your library. Cast it
then only if it kills the table, or with the Crawler or Niv out so the draws finish the loop before you run out.

**Bribery** ({3}{U}{U}) puts the best creature from an opponent's library onto your battlefield: a bomb or a combo
creature on your side, and one fewer in their deck. Pick the opponent whose deck holds the biggest threat.

**Underworld Breach + storm.** See Key lines. It is a mill, not a two-card infinite.

### Weak spots and how to play around them
- **Graveyard hate** hurts Breach, Yawgmoth's Will and every cast-from-graveyard card. Without them, Kefka still
  draws and the Crawler loop still works.
- **Removal before the flip.** Kefka is a 4/5 that has to attack or survive a turn cycle. Hold Counterspell or
  Lightning Greaves mana, and remember he can be recast from the command zone.
- **Short library.** Never start the Crawler or Niv loop without counting your library against the highest life total.
- **Hand size.** Discard down to seven is the limit to watch after a flip turn.
- **Fast pressure and large creatures.** You win late. Spend removal on the threats that would kill you first, and
  use **Massacre Wurm**, Young Pyromancer's tokens or a counterspell to buy time.

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

- **Lands:** 36 (16 nonbasic + 20 basic)
- **Mana:** Arcane Signet, Lotus Petal, Birgi, Storm-Kiln Artist, Dark Ritual, Seething Song, Jeska's Will, Treasure makers and burst cards. Yawgmoth's Will replays Dark Ritual and Lotus Petal from the graveyard. Basics: 8 Swamp, 7 Mountain, 5 Island; Mistrise Village enters untapped with a Mountain.
- **Card draw:** Kefka, Frantic Search, Think Twice, Decorum Dissertation (a free draw-two each turn after the first cast), Skullclamp, Niv-Mizzet (taps to draw), Marvin, Murderous Mimic (a 2-mana 2/2 that has the activated abilities of your other creatures: it copies Niv-Mizzet's draw, Blightspeaker and Rummaging Goblin), the looters, and the cheap digging spell Expressive Iteration (look at three: one to hand, one to the bottom, one exiled and playable this turn). **Glimpse the Unthinkable** ({U}{B}: target player mills ten) fills your graveyard for Underworld Breach and Yawgmoth's Will, or points at an opponent to start the mill-out that Brain Freeze finishes. **Tutors:** Diabolic Tutor, Profane Tutor (suspended), Mystical Tutor, Solve the Equation. **Graveyard:** Underworld Breach, Yawgmoth's Will, Mystic Sanctuary.
- **Interaction:** Lightning Bolt, Reality Shift, Extinguish the Light, Deadly Embrace, Contract Killing, Chaos Warp; counters Arcane Denial and Counterspell; one-sided sweeper Massacre Wurm.
- **Protection:** Lightning Greaves (haste), the counterspells, and Mistrise Village (makes a spell uncounterable).

## Bracket and Rule 0

**Bracket 4,** with three Game Changers: **Jeska's Will**, **Mystical Tutor** and **Underworld Breach**. Three Game Changers alone would fit Bracket 3; the deck is Bracket 4 because of the two-card win. There is no two-card infinite combo, but Psychosis Crawler or Niv-Mizzet, the Firemind with the flipped Kefka is a two-card win: Kefka is always available from the command zone, so the deck only needs the Crawler or Niv, Kefka on the battlefield and {8} for the flip. Four tutors (Diabolic Tutor, Profane Tutor, Mystical Tutor and Solve the Equation). Niv-Mizzet, Parun is a second Niv two-card win with the flipped Kefka. The Breach + Brain Freeze line is a second way to win, but it is a mill, not a two-card infinite: it needs the graveyard to fill first.

Disclose before the game:
- **Jeska's Will** and **Underworld Breach** (the Game Changers). Breach recasts the graveyard, so mention it as a combo enabler.
- **The Crawler and Niv-Mizzet loops** with Kefka, Ruler of Ruin.
- **The Breach + Brain Freeze line**, which can mill an opponent out in one turn (Dark Ritual, Lotus Petal and Grapeshot feed the storm count).
- **Symmetric discard** every time Kefka enters or attacks.

Re-check the official list at https://commanderbrackets.com/faq before an event.

## Decklist by type (100)

**Commander (1).** Kefka, Court Mage // Kefka, Ruler of Ruin

**Creatures (26).** Alpharael Dreaming Acolyte, Birgi God of Storytelling, Blazing Firesinger // Seething Song, Blightspeaker, Eternal Student, Firebrand Archer, Garrison Excavator, Guttersnipe, Harmonic Prodigy, Intellect Devourer, Kessig Flamebreather, Kuja Genome Sorcerer // Trance Kuja Fate Defied, Marvin Murderous Mimic, Massacre Wurm, Niv-Mizzet Parun, Niv-Mizzet the Firemind, Psychosis Crawler, Rummaging Goblin, Scarlet Spider Kaine, Skullport Merchant, Stadium Tidalmage, Stinkweed Imp, Storm-Kiln Artist, Susurian Voidborn, Veyran Voice of Duality, Young Pyromancer

**Enchantments (4).** Corroding Dragonstorm, Demonic Embrace, Fiery Emancipation, Underworld Breach

**Artifacts (4).** Arcane Signet, Lightning Greaves, Lotus Petal, Skullclamp

**Sorceries (15).** Bribery, Contract Killing, Deadly Embrace, Decorum Dissertation, Diabolic Tutor, Expressive Iteration, Glimpse the Unthinkable, Grab the Prize, Grapeshot, Jeska's Will, Profane Tutor, Rapturous Moment, Seize the Spoils, Solve the Equation, Yawgmoth's Will

**Instants (14).** Arcane Denial, Big Score, Brain Freeze, Chaos Warp, Counterspell, Dark Ritual, Extinguish the Light, Frantic Search, Lightning Bolt, Mystical Tutor, Price of Progress, Reality Shift, Think Twice, Twinned Vision

**Lands (36).** Blood Crypt, Command Tower, Exotic Orchard, Gemstone Mine, Haunted Ridge, Mistrise Village, Mystic Sanctuary, Scorched Geyser, Shivan Reef, Spirebluff Canal, Steam Vents, Stormcarved Coast, Sulfur Falls, Sulfurous Springs, Temple of Epiphany, Underground River, 5 Island, 7 Mountain, 8 Swamp

## Import list (100)

```
1 Alpharael, Dreaming Acolyte
1 Arcane Denial
1 Arcane Signet
1 Big Score
1 Birgi, God of Storytelling
1 Blazing Firesinger // Seething Song
1 Blightspeaker
1 Blood Crypt
1 Brain Freeze
1 Bribery
1 Chaos Warp
1 Command Tower
1 Contract Killing
1 Corroding Dragonstorm
1 Counterspell
1 Dark Ritual
1 Deadly Embrace
1 Decorum Dissertation
1 Demonic Embrace
1 Diabolic Tutor
1 Eternal Student
1 Exotic Orchard
1 Expressive Iteration
1 Extinguish the Light
1 Fiery Emancipation
1 Firebrand Archer
1 Frantic Search
1 Garrison Excavator
1 Gemstone Mine
1 Glimpse the Unthinkable
1 Grab the Prize
1 Grapeshot
1 Guttersnipe
1 Harmonic Prodigy
1 Haunted Ridge
1 Intellect Devourer
1 Jeska's Will
1 Kefka, Court Mage // Kefka, Ruler of Ruin
1 Kessig Flamebreather
1 Kuja, Genome Sorcerer // Trance Kuja, Fate Defied
1 Lightning Bolt
1 Lightning Greaves
1 Lotus Petal
1 Marvin, Murderous Mimic
1 Massacre Wurm
1 Mistrise Village
1 Mystic Sanctuary
1 Mystical Tutor
1 Niv-Mizzet, Parun
1 Niv-Mizzet, the Firemind
1 Price of Progress
1 Profane Tutor
1 Psychosis Crawler
1 Rapturous Moment
1 Reality Shift
1 Rummaging Goblin
1 Scarlet Spider, Kaine
1 Scorched Geyser
1 Seize the Spoils
1 Shivan Reef
1 Skullclamp
1 Skullport Merchant
1 Solve the Equation
1 Spirebluff Canal
1 Stadium Tidalmage
1 Steam Vents
1 Stinkweed Imp
1 Storm-Kiln Artist
1 Stormcarved Coast
1 Sulfur Falls
1 Sulfurous Springs
1 Susurian Voidborn
1 Temple of Epiphany
1 Think Twice
1 Twinned Vision
1 Underground River
1 Underworld Breach
1 Veyran, Voice of Duality
1 Yawgmoth's Will
1 Young Pyromancer
5 Island
7 Mountain
8 Swamp
```
