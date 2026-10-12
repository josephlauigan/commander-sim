# Kefka, Court Mage — Grixis Discard

**Updated 2026-10-11 (mana base).** JD's rule: basics over slow lands. Out: Temple of Malice, Boiling Rock Prison, Sinister Hideout, Temple of Deceit, Treno, Dark City, Spectacle Summit, Temple of Epiphany, Vivid Marsh, Path of Ancestry, Evolving Wilds, Fabled Passage (everything that enters tapped or fetches a tapped basic). In: 5 Swamp, 3 Mountain, 3 Island (now 15 Swamp, 9 Mountain, 6 Island). The six untapped nonbasics stay. Still 36 lands; the cut lands go back to the bulk list.
**Updated 2026-10-11 (trade with Veyran, moved as functional blocks).** In from Veyran: Deflecting Swat and Slip Out the Back (protection), Underworld Breach (graveyard), Niv-Mizzet, the Firemind (a second draw-to-damage kill with the flipped Kefka), Jin-Gitaxias, Progress Tyrant, and Steam Vents, Sulfur Falls, Spirebluff Canal and Stormcarved Coast (for 4 Mountains). Out: Mystic Confluence and Splatter Technique (to Veyran), Arcane Omens and Inner Fire (to bulk), Triskelion (to the binder). Paired look-ahead test, 1,000 games per tier: **+1.3 points** (28.1% -> 29.4%). Two Game Changers now (Rhystic Study, Underworld Breach).

2026-10-11
**Added 2026-10-11.** Built from JD's bulk and binder around Kefka, Court Mage, who leaves Sauron for the job (Sephiroth, Planet's Heir took his slot there). Fully modeled and measured on 2026-10-11: 26.6% over all tiers (see "Simulator status").

## Strategy

Make everyone discard, draw more than they do, then flip Kefka and turn every point of life the table loses on your turn into cards — until Psychosis Crawler turns those cards back into life loss and the loop ends the game.

Kefka, Court Mage ({2}{U}{B}{R}, 4/5) reads: *whenever Kefka enters or attacks, each player discards a card. Then you draw a card for each card type among cards discarded this way. {8}: each opponent sacrifices a permanent of their choice. Transform Kefka. Activate only as a sorcery.* The back face, **Kefka, Ruler of Ruin** (5/7, flying), reads: *whenever an opponent loses life during your turn, you draw that many cards.*

**Front face: a symmetric discard that you win.** Every player loses a card, and you draw one for each card type in the pile: an artifact creature counts twice, a land and an instant three different types. Two to four cards for your one is normal. The deck makes the discard cost you nothing:
- **Cards that want to be discarded.** Ichor Slick (madness: a cheap −3/−3), Scarlet Spider, Kaine (mayhem: cast from the graveyard for {B/R} the turn you discard it), Demonic Embrace (cast from the graveyard by paying 3 life and discarding another card), Twinned Vision (flashback draws two), Eternal Student (exile from the graveyard for two flying Inklings), Postmortem Professor (returns from the graveyard). Garrison Excavator makes a 2/2 Spirit every time cards leave your graveyard, so each of those becomes a body too.
- **Underworld Breach turns the discard pile into a second hand.** Every nonland card in your graveyard gains escape: cast it for its mana cost plus exiling three other cards. Kefka's discards and the looters fill the graveyard, so on the flip turn Breach recasts Lightning Bolt, Grab the Prize and the other one-shots, and every point of life lost is still a card. Each escape is two Garrison Excavator Spirits: one for the three exiled cards, one for the escaped card leaving. Breach is sacrificed at your end step, so cast it on the turn you use it.
- **Outlets and looting.** Rummaging Goblin, Hallway Heckler, Thrill of Possibility, Grab the Prize (2 to each opponent if the discard isn't a land), Stadium Tidalmage, Alpharael, Rapturous Moment, Big Score and Seize the Spoils. They pick the right card to bin before Kefka's trigger and fix the hand after it.
- **Pressure on their hands.** Rank Rat, Cackling Fiend, Duress, Dark Inquiry and Intellect Devourer (each opponent exiles a card from hand, and you may play those cards). Opponents low on cards discard their best one to Kefka.

**The flip costs {8}.** Treasure (Big Score, Seize the Spoils, Contract Killing, Skullport Merchant), the rocks (Arcane Signet, Thought Vessel, Sisay's Ring, Potioner's Trove) and the burst cards get there: Blazing Firesinger's prepared spell makes {R}{R}{R}{R}{R}, and Rapturous Moment refunds five. The flip itself is an edict on every opponent.

**Back face: life loss on your turn is card draw.** Each opponent who loses life during your turn draws you that many cards:
- **Repeatable pings:** Sawblade Scamp, Viashino Fangtail, Blightspeaker, Screeching Soulbreaker and Postmortem Professor (on attack), Susurian Voidborn (each Treasure you crack drains one opponent).
- **One-shots:** Shimmercreep (each opponent loses 3 with three colours on the battlefield), Sneering Shadewriter, Corroding Dragonstorm, Grab the Prize, Decorum Dissertation (aimed at an opponent: they draw two, but they lose 2 and you draw 2), Kaervek's Purge, Lightning Bolt, Disintegrate, and Massacre Wurm (2 for each of their creatures that dies).
- **Combat:** a flying 5/7 that hits for 5 draws you 5.

**The kill: Psychosis Crawler + Kefka, Ruler of Ruin.** Each card you draw makes each opponent lose 1; each opponent's loss draws you a card. In a four-player game every card drawn draws three more, so the loop runs until the table is dead or your library is. It is mandatory: count your library against the highest life total before starting it, and play the Crawler after the flip, not before, when your library is short. Before the flip the Crawler is still worth casting early: Kefka's own draws and the looters ping the whole table with it.

**The second kill: Niv-Mizzet, the Firemind + Kefka, Ruler of Ruin.** Each card you draw deals 1 damage to any target; damage to an opponent on your turn draws you a card, which pings again. One ping hits one opponent, so the loop needs a library larger than the opponents' life added together, but you choose every target: when the library is short, aim the pings at creatures (or at yourself) and stop. Niv also taps to draw, which starts it.

**Jin-Gitaxias, Progress Tyrant** copies the first artifact, instant or sorcery you cast each turn (a second Duress, Night's Whisper or Lightning Bolt) and counters the first one each opponent casts.

## Key lines

**Kefka with haste.** Lightning Greaves or Swiftfoot Boots on the turn he lands means two triggers in one turn: the enter trigger and the attack trigger, two discards for the table and two rounds of draws for you.

**Discard into value.** With Kefka's trigger on the stack, keep Ichor Slick, Scarlet Spider or Demonic Embrace in hand to discard: Ichor Slick is cast for its madness cost right away, Scarlet Spider for {B/R} from the graveyard this turn, and Demonic Embrace any time later.

**The flip turn.** Flip at sorcery speed, then play pingers and one-shots in the same main phase: every point of life the table loses before your turn ends is a card. Massacre Wurm after the flip is often 6 to 12 cards.

**Rhystic Study and Skullclamp** keep the hand full between Kefka triggers; Thought Vessel removes the hand-size limit the deck runs into after a flip turn.

## Consistency

- **Lands:** 36: ten nonbasics (Blood Crypt, Haunted Ridge, Underground River, Command Tower, Exotic Orchard, Gemstone Mine, and from Veyran Steam Vents, Sulfur Falls, Spirebluff Canal, Stormcarved Coast) and 26 basics (15 Swamp, 5 Mountain, 6 Island). Counting Arcane Signet and Potioner's Trove, that's about 23 black sources, 16 red and 16 blue (blue was 12 before the trade, which held back the double-blue cards). Steam Vents is untapped (2 life); the other three new duals are conditional: Sulfur Falls needs an Island or Mountain out, Spirebluff Canal enters tapped once you have three other lands, Stormcarved Coast enters tapped on turns one and two.
- **Mana:** 11 rocks, Treasure makers and burst cards.
- **Card draw:** Kefka, Rhystic Study, Night's Whisper, Stargaze, Skullclamp, Decorum Dissertation, Niv-Mizzet (taps to draw), the looters. **Tutors:** Diabolic Tutor, Grim Servant (devotion to black). **Graveyard:** Underworld Breach.
- **Interaction:** Lightning Bolt, Pongify, Reality Shift, Chaos Warp, Feed the Swarm, Extinguish the Light, Deadly Embrace, Kaervek's Purge, Disintegrate, Contract Killing; counter Arcane Denial, plus Jin-Gitaxias countering each opponent's first artifact, instant or sorcery; wipe Crux of Fate.
- **Protection:** Lightning Greaves and Swiftfoot Boots (both give haste), Deflecting Swat (free while you control a commander: redirects a removal spell or counter) and Slip Out the Back ({U}: phases a creature out).

## Bracket and Rule 0

**Bracket 3,** with two Game Changers: **Rhystic Study** and **Underworld Breach**. No two-card infinite combo, but Psychosis Crawler or Niv-Mizzet, the Firemind with the flipped Kefka is a two-card win: Kefka is always available from the command zone, so the deck only needs the Crawler or Niv, Kefka on the battlefield and {8} for the flip. Two tutors (Diabolic Tutor, Grim Servant).

Disclose before the game:
- **Rhystic Study** and **Underworld Breach** (the Game Changers). Breach recasts the graveyard, so mention it as a combo enabler.
- **The Crawler and Niv-Mizzet loops** with Kefka, Ruler of Ruin.
- **Jin-Gitaxias, Progress Tyrant**, which counters the first artifact, instant or sorcery each opponent casts each turn.
- **Symmetric discard** every time Kefka enters or attacks, plus targeted discard (Duress, Dark Inquiry).

Re-check the official list at https://commanderbrackets.com/faq before an event.

## Simulator status

Deck key `kefka` (`commander-sim --deck kefka --pool t3`). **Fully modeled** since 2026-10-11 (every card Full in `cardgen audit --mine`; card code and AI plan in `rust/crates/sim-core/src/impls/kefka.rs`, tests in `tests/cards_kefka.rs`). The AI discards madness, mayhem and graveyard-castable cards to Kefka's trigger first, flips as soon as {8} is spare, and starts the Psychosis Crawler loop only when the library outlasts the highest opponent's life total (each card drawn costs every opponent 1 life).

**Measured 2026-10-11** (look-ahead AI, loose profile, 960 games per tier, the same code as the other eight decks; `decklists/pool/pool-results.md` section 0d): T1 25.4%, T2 35.3%, T3 23.3%, T4 28.4%, T5 20.8%; **26.6% over all tiers**, just above an even share, in Sauron's group.

**AI fix, 2026-10-11** (`rust/agents/report-kefka-ai-fix.md`): the AI held Psychosis Crawler too long (cast in about 22% of the games it was drawn, now about 65%); it now casts Underworld Breach, aims Niv-Mizzet's pings so a short library is never drawn out, and gives Garrison Excavator its Spirits on Breach escapes. Fast AI on a fresh seed: 20.9% -> 26.8%. Look-ahead after the fix, before the trade: T1 26.6%, T2 38.1%, T3 24.4%, T4 29.4%, T5 21.9% (**28.1%**).

**The Veyran trade, 2026-10-11** (`audit/kefka-veyran/2026-10-11-trade/`, look-ahead, 1,000 paired games per tier): T1 27.7%, T2 39.8%, T3 27.8%, T4 29.3%, T5 22.4%: **29.4%**, +1.3 points, no tier worse. Deflecting Swat and Slip Out the Back alone were +0.6 of it.

Known simplifications: madness casts the card from the graveyard rather than from exile (so Garrison Excavator can see it leave); Garrison Excavator doesn't see an opponent exiling your graveyard.

## Where the cards come from

The commander, **Kefka, Court Mage**, comes out of Sauron. The 26 basics (15 Swamp, 5 Mountain, 6 Island) are your own. Nine cards come from Veyran in the 2026-10-11 trade (marked "Veyran" below). Every other card came off JD's lists on 2026-10-11, one copy each (bulk first where a card was on both lists): 7 from the binder, 57 from bulk. They're grouped by card color below so you can pull them a color at a time. Set codes are the printings the bulk list showed; where it names more than one, any of them.

### Black (27)

| Card | From | Set |
|---|---|---|
| Blightspeaker | Bulk | tsr |
| Cackling Fiend | Bulk | dmr |
| Contract Killing | Bulk | xln |
| Corroding Dragonstorm | Bulk | tdm |
| Crux of Fate | Bulk | fic |
| Dark Inquiry | Bulk | rix |
| Deadly Embrace | Bulk | fin |
| Decorum Dissertation | Bulk | sos |
| Demonic Embrace | Bulk | gn3 |
| Diabolic Tutor | Bulk | m11 |
| Duress | Binder | — |
| Eternal Student | Bulk | sos |
| Extinguish the Light | Bulk | dmu |
| Feed the Swarm | Bulk | soa |
| Grim Servant | Bulk | mh3 |
| Ichor Slick | Bulk | tsr |
| Intellect Devourer | Bulk | clb |
| Massacre Wurm | Bulk | m21 |
| Night's Whisper | Bulk | dmc |
| Postmortem Professor | Bulk | sos |
| Rank Rat | Bulk | fra |
| Screeching Soulbreaker | Bulk | fra |
| Shimmercreep | Bulk | ecl |
| Skullport Merchant | Bulk | list |
| Sneering Shadewriter | Bulk | sos |
| Stargaze | Bulk | soa |
| Susurian Voidborn | Bulk | eoe |

### Red (15)

| Card | From | Set |
|---|---|---|
| Big Score | Bulk | snc |
| Blazing Firesinger | Bulk | sos |
| Chaos Warp | Bulk | moc |
| Deflecting Swat | Veyran | — |
| Disintegrate | Binder | — |
| Garrison Excavator | Bulk | sos |
| Grab the Prize | Bulk | dsk |
| Hallway Heckler | Bulk | fra |
| Lightning Bolt | Binder | — |
| Rummaging Goblin | Bulk | cmr, m15, m13 |
| Sawblade Scamp | Bulk | one |
| Seize the Spoils | Bulk | sos |
| Thrill of Possibility | Bulk | dmu |
| Underworld Breach | Veyran | — |
| Viashino Fangtail | Bulk | rav |

### Blue (6)

| Card | From | Set |
|---|---|---|
| Arcane Denial | Bulk | msc |
| Jin-Gitaxias, Progress Tyrant | Veyran | — |
| Pongify | Bulk | 2xm |
| Reality Shift | Bulk | moc |
| Rhystic Study | Binder | — |
| Slip Out the Back | Veyran | — |

### Multicolor (7)

| Card | Colors | From | Set |
|---|---|---|---|
| Alpharael, Dreaming Acolyte | U/B | Bulk | eoe |
| Kaervek's Purge | B/R | Binder | — |
| Niv-Mizzet, the Firemind | U/R | Veyran | — |
| Rapturous Moment | U/R | Bulk | sos |
| Scarlet Spider, Kaine | B/R | Bulk | spm |
| Stadium Tidalmage | U/R | Bulk | sos |
| Twinned Vision | U/R | Bulk | fra |

### Colorless artifacts (8)

| Card | From | Set |
|---|---|---|
| Arcane Signet | Bulk | fic, msc |
| Lightning Greaves | Bulk | fic |
| Potioner's Trove | Bulk | sos |
| Psychosis Crawler | Bulk | c15 |
| Sisay's Ring | Binder | — |
| Skullclamp | Bulk | fic |
| Swiftfoot Boots | Binder | — |
| Thought Vessel | Bulk | msc |

### Lands (10)

| Card | From | Set |
|---|---|---|
| Blood Crypt | Bulk | rtr |
| Command Tower | Bulk | khc, fic |
| Exotic Orchard | Bulk | fic |
| Gemstone Mine | Bulk | tsb |
| Haunted Ridge | Bulk | fra |
| Spirebluff Canal | Veyran | — |
| Steam Vents | Veyran | — |
| Stormcarved Coast | Veyran | — |
| Sulfur Falls | Veyran | — |
| Underground River | Bulk | fic |

## Decklist by type (100)

**Commander (1).** Kefka, Court Mage // Kefka, Ruler of Ruin

**Creatures (25).** Alpharael Dreaming Acolyte, Blazing Firesinger, Blightspeaker, Cackling Fiend, Eternal Student, Garrison Excavator, Grim Servant, Hallway Heckler, Intellect Devourer, Jin-Gitaxias Progress Tyrant, Massacre Wurm, Niv-Mizzet the Firemind, Postmortem Professor, Psychosis Crawler, Rank Rat, Rummaging Goblin, Sawblade Scamp, Scarlet Spider Kaine, Screeching Soulbreaker, Shimmercreep, Skullport Merchant, Sneering Shadewriter, Stadium Tidalmage, Susurian Voidborn, Viashino Fangtail

**Enchantments (4).** Corroding Dragonstorm, Demonic Embrace, Rhystic Study, Underworld Breach

**Artifacts (7).** Arcane Signet, Lightning Greaves, Potioner's Trove, Sisay's Ring, Skullclamp, Swiftfoot Boots, Thought Vessel

**Sorceries (16).** Contract Killing, Crux of Fate, Dark Inquiry, Deadly Embrace, Decorum Dissertation, Diabolic Tutor, Disintegrate, Duress, Feed the Swarm, Grab the Prize, Ichor Slick, Kaervek's Purge, Night's Whisper, Rapturous Moment, Seize the Spoils, Stargaze

**Instants (11).** Arcane Denial, Big Score, Chaos Warp, Deflecting Swat, Extinguish the Light, Lightning Bolt, Pongify, Reality Shift, Slip Out the Back, Thrill of Possibility, Twinned Vision

**Lands (36).** Blood Crypt, Command Tower, Exotic Orchard, Gemstone Mine, Haunted Ridge, Spirebluff Canal, Steam Vents, Stormcarved Coast, Sulfur Falls, Underground River, 6 Island, 5 Mountain, 15 Swamp

## Import list (100)

```
1 Alpharael, Dreaming Acolyte
1 Arcane Denial
1 Arcane Signet
1 Big Score
1 Blazing Firesinger
1 Blightspeaker
1 Blood Crypt
1 Cackling Fiend
1 Chaos Warp
1 Command Tower
1 Contract Killing
1 Corroding Dragonstorm
1 Crux of Fate
1 Dark Inquiry
1 Deadly Embrace
1 Decorum Dissertation
1 Deflecting Swat
1 Demonic Embrace
1 Diabolic Tutor
1 Disintegrate
1 Duress
1 Eternal Student
1 Exotic Orchard
1 Extinguish the Light
1 Feed the Swarm
1 Garrison Excavator
1 Gemstone Mine
1 Grab the Prize
1 Grim Servant
1 Hallway Heckler
1 Haunted Ridge
1 Ichor Slick
1 Intellect Devourer
1 Jin-Gitaxias, Progress Tyrant
1 Kaervek's Purge
1 Kefka, Court Mage // Kefka, Ruler of Ruin
1 Lightning Bolt
1 Lightning Greaves
1 Massacre Wurm
1 Night's Whisper
1 Niv-Mizzet, the Firemind
1 Pongify
1 Postmortem Professor
1 Potioner's Trove
1 Psychosis Crawler
1 Rank Rat
1 Rapturous Moment
1 Reality Shift
1 Rhystic Study
1 Rummaging Goblin
1 Sawblade Scamp
1 Scarlet Spider, Kaine
1 Screeching Soulbreaker
1 Seize the Spoils
1 Shimmercreep
1 Sisay's Ring
1 Skullclamp
1 Skullport Merchant
1 Slip Out the Back
1 Sneering Shadewriter
1 Spirebluff Canal
1 Stadium Tidalmage
1 Stargaze
1 Steam Vents
1 Stormcarved Coast
1 Sulfur Falls
1 Susurian Voidborn
1 Swiftfoot Boots
1 Thought Vessel
1 Thrill of Possibility
1 Twinned Vision
1 Underground River
1 Underworld Breach
1 Viashino Fangtail
6 Island
5 Mountain
15 Swamp
```
