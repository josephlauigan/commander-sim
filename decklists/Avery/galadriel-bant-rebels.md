# Galadriel, Light of Valinor — Bant Rebels

2026-10-02
**Added 2026-10-02.** In the simulator as deck key `galadriel` (`python3 -m commander_sim --deck galadriel --pool t3`) and in practice mode. Every card is modeled.

## Strategy

Flood the battlefield with creatures, and get paid for each one. Galadriel turns every creature entering into mana, a bigger team or a card, and a chain of Mercadian Masques Rebels puts creatures onto the battlefield every turn without spending cards.

Galadriel, Light of Valinor ({2}{G}{W}{U}) reads: *Alliance — whenever another creature you control enters, choose one that hasn't been chosen this turn: add {G}{G}{G}; put a +1/+1 counter on each creature you control; or scry 2, then draw a card.*

**Three creatures a turn is the target.** Each mode can be chosen once per turn, so the first three creatures to enter each turn each pay out, and the fourth gets nothing. Order them:
- **Mana first** in your main phase when it lets you cast something else this turn. {G}{G}{G} from the first creature often pays for the second.
- **Counters** before combat with a wide board. With six creatures out, it's six counters.
- **Scry 2 and draw** any other time, and always at instant speed (a Rebel fetched at an opponent's end step).

**The Rebel chain.** A searcher pays mana and taps to put a Rebel permanent card straight from your library onto the battlefield. A fetched Rebel isn't cast, so counterspells can't stop it, and it still triggers Galadriel, Cathars' Crusade and every other enters payoff.

| Searcher | Its mana value | Activation | Finds a Rebel with mana value |
| --- | --- | --- | --- |
| Ramosian Sergeant | 1 | {3}, {T} | 2 or less |
| Ramosian Lieutenant | 2 | {4}, {T} | 3 or less |
| Ramosian Captain | 3 | {5}, {T} | 4 or less |
| Defiant Vanguard | 3 | {5}, {T} | 4 or less |
| Ramosian Commander | 4 | {6}, {T} | 5 or less |
| Lin Sivvi, Defiant Hero | 3 | {X}, {T} | X or less |

Each searcher finds a bigger one, so one early searcher becomes the whole chain: Sergeant finds Lieutenant, Lieutenant finds Lin Sivvi or Captain, Captain finds Commander. Ramosian Revivalist ({6}, {T}) returns a Rebel with mana value 5 or less from your graveyard, and Lin Sivvi's {3} puts a dead Rebel back on the bottom of your library to be found again. Mirror Entity is a changeling, so every searcher can find it.

**The eighteen Rebels and what to fetch.**
- **Searchers first** while you have fewer than two: they turn spare mana into a creature every turn.
- **Lawbringer and Lightbringer** exile a red or a black creature (tap and sacrifice). Fetch one when an opponent's best threat is that colour.
- **Ballista Squad** shoots an attacking or blocking creature for X. **Errant Doomsayers** and **Whipcorder** tap a creature: an opponent's attacker before their combat, or your opponent's blocker before yours.
- **Cho-Manno** takes no damage, **Defiant Vanguard** kills whatever it blocks, and **Knight of the Holy Nimbus** regenerates unless an opponent pays {2}.
- **Nightwind Glider** (protection from black) and **Thermal Glider** (protection from red) are evasive bodies. **Amrou Seekers** can only be blocked by white or artifact creatures. **Jhovall Queen** is a 4/7 vigilance wall (Lin Sivvi with X = 6).

**Maskwood Nexus makes everything a Rebel.** Your creatures, creature spells and creature cards are every creature type, so the searchers can find any creature in your library (Adeline, Mentor of the Meek, Reya Dawnbringer with Lin Sivvi at X = 9). Every type-matters card counts every creature too. Its {3}, {T} makes a 2/2 changeling Shapeshifter.

**More payoffs for creatures entering.**
- **Cathars' Crusade:** a +1/+1 counter on each creature you control every time a creature enters.
- **Panharmonicon:** every enters trigger of your permanents happens twice: Galadriel (two modes per creature), Cathars' Crusade (two counters each), Welcoming Vampire, Tocasia's Welcome, Mentor of the Meek, Kindred Discovery, Recruiter of the Guard and Farhaven Elf.
- **Card draw:** Welcoming Vampire (power 2 or less, once a turn), Tocasia's Welcome (mana value 3 or less, once a turn), Mentor of the Meek (pay {1} per small creature), Kindred Discovery (enters or attacks), Beast Whisperer and Vanquisher's Banner (creature spells).
- **Bodies:** Adeline makes a 1/1 for each opponent whenever you attack, Elspeth makes three Soldiers a turn, and Voice of Resurgence leaves an Elemental as big as your creature count.

**Name a type.** Kindred Discovery, Door of Destinies, Patchwork Banner, Vanquisher's Banner and Secluded Courtyard each name a creature type as they enter.
- **Human (19 creatures) or Rebel (18 plus Mirror Entity)** are the two to choose between. Rebels enter mostly by being fetched, which is what Kindred Discovery and the Banners' +1/+1 want. Door of Destinies and Vanquisher's Banner's draw count *spells cast*, and the Humans are cast more often (Adeline, Mentor, Recruiter, Mangara, Cartographer).
- **With Maskwood Nexus out, the choice doesn't matter:** every creature counts.
- **Secluded Courtyard's** coloured mana only casts creature spells of its type (or pays for those creatures' abilities), so name the type you cast the most.

**Interaction.**
- **Spot removal:** Swords to Plowshares, Path to Exile, Generous Gift, Beast Within, Return to Dust (two artifacts or enchantments in your main phase), plus Lawbringer, Lightbringer and Ballista Squad on the battlefield.
- **Theft:** Abduction (an opponent's creature, which goes home when it dies) and Bribery (the best creature in an opponent's library).
- **Board control:** Austere Command (choose creatures with mana value 4 or more and artifacts, and your small creatures live), Farewell, Cyclonic Rift, River's Rebuke, Elspeth's −3 (power 4 or more) and Crackdown (big nonwhite creatures stop untapping; your own green creatures too once Crusade grows them).
- **Protection:** Unbreakable Formation, Make a Stand and Rootborn Defenses make your team indestructible against a destroy wipe. Eerie Interlude exiles your creatures and returns them at the next end step, which beats exile and -X/-X wipes too, and re-triggers every enters payoff. Counterspell for the rest.

## Key lines

**The opening.** A one-mana elf or Sol Ring on turn 1, a searcher or a ramp spell on turn 2, then a three-drop payoff (Welcoming Vampire, Tocasia's Welcome, Mentor of the Meek, Adeline) on turn 3. Galadriel lands on turn 4 with a mana creature, so every later creature pays out.

**A turn with Galadriel out.** Cast a creature and choose {G}{G}{G}, which pays for a second creature. Choose counters for that one before combat. Then attack, and fetch a Rebel with a searcher at the end of an opponent's turn for the scry-and-draw mode.

**Panharmonicon with Cathars' Crusade and Galadriel.** Each creature entering puts two counters on everything and picks two of Galadriel's modes. With a searcher chain online, the team grows by two or three counters per creature every turn.

**Close with Adeline and counters.** Adeline's 1/1s enter attacking, so they trigger Galadriel's counters mode and Cathars' Crusade mid-combat. Rogue's Passage and Elspeth's emblem (+2/+2 and flying) push the last points through. Mirror Entity turns a wide board into X/X attackers with leftover mana.

**Wipe on your terms.** Austere Command with "artifacts" and "creatures with mana value 4 or more" leaves your 1-, 2- and 3-drop creatures standing. Hold Unbreakable Formation or Eerie Interlude for an opponent's wipe.

## What the simulator found

Against the opponent pools with the look-ahead AI (loose profile, 120 games per tier) the deck won 15.0% against Tier 1, 11.7% against Tier 2, 8.3% against Tier 3, 12.5% against Tier 4 and 8.3% against Tier 5; an even share is 25%. The deck's result depends on finding a searcher. In games where one appears, the AI puts about three Rebels onto the battlefield per game, and one game in nine reaches eight or more. With six searchers in 99 cards (plus Recruiter of the Guard, which finds the Sergeant, Lieutenant or Captain), about 40% of games see none. Galadriel's first Alliance trigger comes on turn 6 in a typical game; the AI chooses counters most often, then mana, then the draw.

## Consistency

The list is exactly 100 cards.

29 lands (15 nonbasic, 7 Plains, 4 Island, 3 Forest) plus a lot of ramp:
- **Mana creatures:** Llanowar Elves, Fyndhorn Elves, Elvish Mystic, Elvish Archdruid ({G} for each Elf).
- **Rocks:** Sol Ring, Arcane Signet, Chromatic Lantern, Patchwork Banner, Springleaf Drum (tap a creature for any colour).
- **Lands:** Cultivate, Farseek, Shared Roots, Sakura-Tribe Elder, Farhaven Elf, Planar Genesis.

| Measure | Count | Note |
| --- | --- | --- |
| White lands | 17 | 7 Plains plus Command Tower, Exotic Orchard, Glacial Fortress, Grand Coliseum, Prairie Stream, Seaside Citadel, Secluded Courtyard, Sunpetal Grove, Temple Garden, Temple of Plenty |
| Blue lands | 14 | 4 Island plus Command Tower, Exotic Orchard, Glacial Fortress, Grand Coliseum, Hinterland Harbor, Prairie Stream, Seaside Citadel, Secluded Courtyard, Temple of Mystery, Yavimaya Coast |
| Green lands | 14 | 3 Forest plus Command Tower, Exotic Orchard, Grand Coliseum, Hinterland Harbor, Seaside Citadel, Secluded Courtyard, Sunpetal Grove, Temple Garden, Temple of Mystery, Temple of Plenty, Yavimaya Coast |
| Any colour, with a condition | 3 | Exotic Orchard (an opponent's land), Grand Coliseum (1 damage), Secluded Courtyard (creature spells of its type) |
| Always enter tapped | 4 | Grand Coliseum, Seaside Citadel, Temple of Mystery, Temple of Plenty |
| Sometimes enter tapped | 5 | Glacial Fortress, Hinterland Harbor, Sunpetal Grove (need a basic type), Prairie Stream (two basics), Temple Garden (2 life to enter untapped) |
| No mana the turn it lands | 1 | Fabled Passage (finds any basic) |

**The mana base is light.** 29 lands is few for a deck whose commander costs five, so the ramp spells and mana creatures are load-bearing: keep a hand with three lands, or two lands and a mana creature. Farseek finds a Plains or Island card, including Prairie Stream and Temple Garden.

**Curve.** One-drops 8, two-drops 13, three-drops 27, four-drops 11, five 5, six 5, and Reya Dawnbringer at nine. Average mana value 3.2 for the 70 other spells.

**Card types.** 34 creatures, 1 planeswalker, 5 enchantments, 9 artifacts, 12 instants, 9 sorceries.

**Creature types.** 18 Rebels (plus Mirror Entity, a changeling), 19 Humans, 7 Elves (Galadriel included).

**Finding cards.**
- **Engines:** Welcoming Vampire, Tocasia's Welcome, Mentor of the Meek, Kindred Discovery, Beast Whisperer, Vanquisher's Banner, Mangara (when an opponent attacks you with two or more, or casts a second spell in a turn), and Galadriel's draw mode.
- **Spells:** Shamanic Revelation (a card per creature, and 4 life per creature with power 4 or more), Planar Genesis.
- **Tutors:** the Rebel searchers, Recruiter of the Guard (a creature with toughness 2 or less), Bribery.

## Bracket and Rule 0

**Bracket 3,** with two Game Changers: **Cyclonic Rift** and **Farewell**. There are no infinite combos, no mass land destruction and no extra turns.

Disclose before the game:

- **Cyclonic Rift** and **Farewell** (the two Game Changers).
- **The Rebel searchers are repeatable tutors** that put creatures onto the battlefield every turn.
- **Bribery** takes the best creature out of an opponent's library, and **Abduction** steals one from the battlefield.
- **Crackdown** stops big nonwhite creatures from untapping.
- **Panharmonicon** doubles enters triggers, including Galadriel's.

Re-check the official list at https://commanderbrackets.com/faq before an event.

## Decklist by type (100)

**Commander (1).** Galadriel, Light of Valinor

**Creatures (34).** Adeline, Resplendent Cathar, Amrou Seekers, Ballista Squad, Beast Whisperer, Cartographer, Cho-Manno, Revolutionary, Defiant Vanguard, Elvish Archdruid, Elvish Mystic, Errant Doomsayers, Farhaven Elf, Fyndhorn Elves, Jhovall Queen, Knight of the Holy Nimbus, Lawbringer, Lightbringer, Lin Sivvi, Defiant Hero, Llanowar Elves, Mangara, the Diplomat, Mentor of the Meek, Mirror Entity, Nightwind Glider, Ramosian Captain, Ramosian Commander, Ramosian Lieutenant, Ramosian Revivalist, Ramosian Sergeant, Recruiter of the Guard, Reya Dawnbringer, Sakura-Tribe Elder, Thermal Glider, Voice of Resurgence, Welcoming Vampire, Whipcorder

**Planeswalkers (1).** Elspeth, Sun's Champion

**Enchantments (5).** Abduction, Cathars' Crusade, Crackdown, Kindred Discovery, Tocasia's Welcome

**Artifacts (9).** Arcane Signet, Chromatic Lantern, Door of Destinies, Maskwood Nexus, Panharmonicon, Patchwork Banner, Sol Ring, Springleaf Drum, Vanquisher's Banner

**Instants (12).** Beast Within, Counterspell, Cyclonic Rift, Eerie Interlude, Generous Gift, Make a Stand, Path to Exile, Planar Genesis, Return to Dust, Rootborn Defenses, Swords to Plowshares, Unbreakable Formation

**Sorceries (9).** Austere Command, Bribery, Cultivate, Farewell, Farseek, Flicker, River's Rebuke, Shamanic Revelation, Shared Roots

**Lands (29).** Command Tower, Exotic Orchard, Fabled Passage, Glacial Fortress, Grand Coliseum, Hinterland Harbor, Prairie Stream, Rogue's Passage, Seaside Citadel, Secluded Courtyard, Sunpetal Grove, Temple Garden, Temple of Mystery, Temple of Plenty, Yavimaya Coast, 7 Plains, 4 Island, 3 Forest

## Import list (100)

```
1 Abduction
1 Adeline, Resplendent Cathar
1 Amrou Seekers
1 Arcane Signet
1 Austere Command
1 Ballista Squad
1 Beast Whisperer
1 Beast Within
1 Bribery
1 Cartographer
1 Cathars' Crusade
1 Cho-Manno, Revolutionary
1 Chromatic Lantern
1 Command Tower
1 Counterspell
1 Crackdown
1 Cultivate
1 Cyclonic Rift
1 Defiant Vanguard
1 Door of Destinies
1 Eerie Interlude
1 Elspeth, Sun's Champion
1 Elvish Archdruid
1 Elvish Mystic
1 Errant Doomsayers
1 Exotic Orchard
1 Fabled Passage
1 Farewell
1 Farhaven Elf
1 Farseek
1 Flicker
3 Forest
1 Fyndhorn Elves
1 Galadriel, Light of Valinor
1 Generous Gift
1 Glacial Fortress
1 Grand Coliseum
1 Hinterland Harbor
4 Island
1 Jhovall Queen
1 Kindred Discovery
1 Knight of the Holy Nimbus
1 Lawbringer
1 Lightbringer
1 Lin Sivvi, Defiant Hero
1 Llanowar Elves
1 Make a Stand
1 Mangara, the Diplomat
1 Maskwood Nexus
1 Mentor of the Meek
1 Mirror Entity
1 Nightwind Glider
1 Panharmonicon
1 Patchwork Banner
1 Path to Exile
7 Plains
1 Planar Genesis
1 Prairie Stream
1 Ramosian Captain
1 Ramosian Commander
1 Ramosian Lieutenant
1 Ramosian Revivalist
1 Ramosian Sergeant
1 Recruiter of the Guard
1 Return to Dust
1 Reya Dawnbringer
1 River's Rebuke
1 Rogue's Passage
1 Rootborn Defenses
1 Sakura-Tribe Elder
1 Seaside Citadel
1 Secluded Courtyard
1 Shamanic Revelation
1 Shared Roots
1 Sol Ring
1 Springleaf Drum
1 Sunpetal Grove
1 Swords to Plowshares
1 Temple Garden
1 Temple of Mystery
1 Temple of Plenty
1 Thermal Glider
1 Tocasia's Welcome
1 Unbreakable Formation
1 Vanquisher's Banner
1 Voice of Resurgence
1 Welcoming Vampire
1 Whipcorder
1 Yavimaya Coast
```
