# Y'shtola, Night's Blessed — Esper cEDH

The cEDH version of the Y'shtola deck (the Bracket 4 list stays as `yshtola`). Fast mana, the full tutor package and
free counterspells, built to win with Thassa's Oracle on turn 3 to 5, with Sanguine Bond + Exquisite Blood as the
backup that Y'shtola starts herself. The expensive cards are proxies. The deck is weak to a faster combo deck that goes
first while it has no counterspell up, to stax it can't remove (a Rule of Law effect makes the combo turn take two
turns), and to Blood Moon effects (only three basics).

| | |
|---|---|
| **Commander** | Y'shtola, Night's Blessed ({1}{W}{U}{B}, 2/4, vigilance; white-blue-black) |
| **Bracket** | 5 (cEDH; 23 Game Changers: Thassa's Oracle, Ad Nauseam, Demonic Tutor, Vampiric Tutor, Imperial Seal, Mystical Tutor, Enlightened Tutor, Intuition, Mana Vault, Chrome Mox, Mox Diamond, Grim Monolith, Rhystic Study, Necropotence, Smothering Tithe, Force of Will, Fierce Guardianship, Teferi's Protection, Grand Arbiter Augustin IV, Notion Thief, Orcish Bowmasters, Opposition Agent, Drannith Magistrate) |
| **Simulator** | deck key `yshtola-cedh`; Tier 6 (cEDH) only; history in `decklists/CHANGELOG.md` |

## Strategy

### The commander

Y'shtola, Night's Blessed ({1}{W}{U}{B}, 2/4, vigilance): *At the beginning of each end step, if a player lost 4 or
more life this turn, you draw a card. Whenever you cast a noncreature spell with mana value 3 or greater, Y'shtola
deals 2 damage to each opponent and you gain 2 life.*

In this list she is a card engine and a combo piece, not the plan:
- **Her draw** comes from your own life too: Necropotence, Ad Nauseam and the fetch lands and pain lands lose life, so
  a turn with Necropotence or Ad Nauseam draws an extra card at the end step.
- **Her trigger starts the drain combo.** With Sanguine Bond and Exquisite Blood out, any trigger (2 life gained, 2
  lost by each opponent) loops until every opponent is dead.
- **Her triggers in the list** (noncreature, mana value 3 or more): Necropotence, Rhystic Study, Grim Tutor,
  Intuition, Windfall, Timetwister, Teferi's Protection, Anguished Unmaking, Smothering Tithe, Ad Nauseam, Sanguine
  Bond, Exquisite Blood, and the free spells, whose mana value counts even when you pay nothing: Force of Will (5),
  Force of Negation and Fierce Guardianship (3), Deadly Rollick and Mindbreak Trap (4).

### How it wins

1. **Thassa's Oracle + Demonic Consultation or Tainted Pact.** Two cards, four or five mana. Tutors find whichever
   piece is missing.
2. **Sanguine Bond + Exquisite Blood**, started by Y'shtola's trigger or any life gain. Slower (5 + 5 mana), and it
   needs Y'shtola or another lifegain source, but it is a second, unrelated win that the same tutors find.
3. **Ad Nauseam or Necropotence** draws into either combo; **Notion Thief + Windfall or Timetwister** takes every
   card the table would draw, then Thassa's Oracle wins.

### Game plan

- **Turns 1-2: fast mana and a draw engine.** Sol Ring, Mana Vault, a Mox, Lotus Petal, rituals; Rhystic Study,
  Mystic Remora, Esper Sentinel or Necropotence. Hold up a free counterspell when you can.
- **Turns 2-4: tutor and protect.** Tutor for the missing combo piece. Cast Grand Abolisher, Silence or Orim's Chant
  first, or hold Force of Will, Fierce Guardianship or Pact of Negation for the combo turn.
- **Stop the table.** Free counterspells and Swan Song, Flusterstorm and Mindbreak Trap stop a faster combo; Opposition
  Agent, Drannith Magistrate and Grand Arbiter Augustin IV slow the tutors and command-zone decks.

## Key lines

- **Thassa's Oracle wins when its enter trigger resolves with your library empty.** Cast Thassa's Oracle, then with
  its trigger on the stack cast Demonic Consultation (naming a card that isn't in the deck) or Tainted Pact to exile
  the library. Either piece can also go first; the library only has to be empty when the trigger resolves.
- **Protect the combo turn:** Silence, Orim's Chant or Grand Abolisher before the pieces, or two free counterspells.
- **Ad Nauseam at instant speed** at the end of the opponent before you, then untap with the hand.

## Mana and consistency

The list is exactly 100 cards: 29 lands and 12 other mana sources (Sol Ring, Mana Vault, Chrome Mox, Mox Diamond, Lotus
Petal, Grim Monolith, Arcane Signet, three Talismans, Dark Ritual, Cabal Ritual).

| Measure | Count | Note |
| --- | --- | --- |
| Dual lands | 19 | original duals, shock lands, Battlebond lands, fast lands, pain lands, check lands (3 each), Silent Clearing |
| Fetch lands | 6 | each finds an original dual, a shock land or a basic: nine targets |
| Basics | 3 | one Island, Plains, Swamp: Path to Exile, Blood Moon, fetch targets |
| Any colour | 1 | Command Tower |

## Bracket and Rule 0

Bracket 5: 23 Game Changers, two-card wins from the tutors, fast mana. Play it at a cEDH table; say so before the game.
Thassa's Oracle + Demonic Consultation / Tainted Pact and Sanguine Bond + Exquisite Blood are two-card wins.

## Decklist by type (100)
**Commander (1).** Y'shtola, Night's Blessed

**Creatures (13).** Brazen Borrower, Dark Confidant, Drannith Magistrate, Esper Sentinel, Ethersworn Canonist, Faerie Mastermind, Grand Abolisher, Grand Arbiter Augustin IV, Notion Thief, Opposition Agent, Orcish Bowmasters, Ranger-Captain of Eos, Thassa's Oracle

**Enchantments (6).** Exquisite Blood, Mystic Remora, Necropotence, Rhystic Study, Sanguine Bond, Smothering Tithe

**Artifacts (10).** Arcane Signet, Chrome Mox, Grim Monolith, Lotus Petal, Mana Vault, Mox Diamond, Sol Ring, Talisman of Dominance, Talisman of Hierarchy, Talisman of Progress

**Sorceries (8).** Demonic Tutor, Gitaxian Probe, Grim Tutor, Imperial Seal, Ponder, Preordain, Timetwister, Windfall

**Instants (33).** Ad Nauseam, An Offer You Can't Refuse, Anguished Unmaking, Brainstorm, Cabal Ritual, Counterspell, Dark Ritual, Deadly Rollick, Demonic Consultation, Dovin's Veto, Enlightened Tutor, Fatal Push, Fierce Guardianship, Flusterstorm, Force of Negation, Force of Will, Intuition, Lim-Dûl's Vault, Mana Drain, Mental Misstep, Mindbreak Trap, Mystical Tutor, Orim's Chant, Pact of Negation, Path to Exile, Silence, Snuff Out, Spell Pierce, Swan Song, Swords to Plowshares, Tainted Pact, Teferi's Protection, Vampiric Tutor

**Lands (29).** Adarkar Wastes, Caves of Koilos, Command Tower, Concealed Courtyard, Darkslick Shores, Drowned Catacomb, Flooded Strand, Glacial Fortress, Godless Shrine, Hallowed Fountain, Isolated Chapel, Marsh Flats, Misty Rainforest, Morphic Pool, Polluted Delta, Scrubland, Sea of Clouds, Seachrome Coast, Silent Clearing, Tundra, Underground River, Underground Sea, Vault of Champions, Verdant Catacombs, Watery Grave, Windswept Heath, 1 Island, 1 Plains, 1 Swamp

(written by `cardgen deck`)

## Import list (100)

```
1 Ad Nauseam
1 Adarkar Wastes
1 An Offer You Can't Refuse
1 Anguished Unmaking
1 Arcane Signet
1 Brainstorm
1 Brazen Borrower
1 Cabal Ritual
1 Caves of Koilos
1 Chrome Mox
1 Command Tower
1 Concealed Courtyard
1 Counterspell
1 Dark Confidant
1 Dark Ritual
1 Darkslick Shores
1 Deadly Rollick
1 Demonic Consultation
1 Demonic Tutor
1 Dovin's Veto
1 Drannith Magistrate
1 Drowned Catacomb
1 Enlightened Tutor
1 Esper Sentinel
1 Ethersworn Canonist
1 Exquisite Blood
1 Faerie Mastermind
1 Fatal Push
1 Fierce Guardianship
1 Flooded Strand
1 Flusterstorm
1 Force of Negation
1 Force of Will
1 Gitaxian Probe
1 Glacial Fortress
1 Godless Shrine
1 Grand Abolisher
1 Grand Arbiter Augustin IV
1 Grim Monolith
1 Grim Tutor
1 Hallowed Fountain
1 Imperial Seal
1 Intuition
1 Isolated Chapel
1 Lim-Dûl's Vault
1 Lotus Petal
1 Mana Drain
1 Mana Vault
1 Marsh Flats
1 Mental Misstep
1 Mindbreak Trap
1 Misty Rainforest
1 Morphic Pool
1 Mox Diamond
1 Mystic Remora
1 Mystical Tutor
1 Necropotence
1 Notion Thief
1 Opposition Agent
1 Orcish Bowmasters
1 Orim's Chant
1 Pact of Negation
1 Path to Exile
1 Polluted Delta
1 Ponder
1 Preordain
1 Ranger-Captain of Eos
1 Rhystic Study
1 Sanguine Bond
1 Scrubland
1 Sea of Clouds
1 Seachrome Coast
1 Silence
1 Silent Clearing
1 Smothering Tithe
1 Snuff Out
1 Sol Ring
1 Spell Pierce
1 Swan Song
1 Swords to Plowshares
1 Tainted Pact
1 Talisman of Dominance
1 Talisman of Hierarchy
1 Talisman of Progress
1 Teferi's Protection
1 Thassa's Oracle
1 Timetwister
1 Tundra
1 Underground River
1 Underground Sea
1 Vampiric Tutor
1 Vault of Champions
1 Verdant Catacombs
1 Watery Grave
1 Windfall
1 Windswept Heath
1 Y'shtola, Night's Blessed
1 Island
1 Plains
1 Swamp
```
