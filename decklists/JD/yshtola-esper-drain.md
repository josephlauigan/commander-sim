# Y'shtola, Night's Blessed — Esper Drain

Drain the table a little with every spell, then all at once. Y'shtola turns your ordinary noncreature spells into damage to every opponent and life for you, Sanguine Bond and Marauding Blight-Priest turn that life into more damage, and Exsanguinate and Debt to the Deathless finish the game. The deck wins slowly, so it is weak to fast creature pressure and to fast combo decks that win before the drain gets there.

| | |
|---|---|
| **Commander** | Y'shtola, Night's Blessed ({1}{W}{U}{B}, 2/4, vigilance; white-blue-black) |
| **Bracket** | 4 (7 Game Changers: Necropotence, Notion Thief, Consecrated Sphinx, Bolas's Citadel, Smothering Tithe, Mystical Tutor, Grand Arbiter Augustin IV) |
| **Simulator** | deck key `yshtola`; results in `decklists/pool/pool-results.md`, history in `decklists/CHANGELOG.md` |

## Strategy

### The commander

Y'shtola, Night's Blessed ({1}{W}{U}{B}, 2/4, vigilance): *At the beginning of each end step, if a player lost 4 or more life this turn, you draw a card. Whenever you cast a noncreature spell with mana value 3 or greater, Y'shtola deals 2 damage to each opponent and you gain 2 life.*

**Thirty-four spells trigger her.** Every noncreature spell in the list with mana value 3 or more counts: rocks (Chromatic Lantern, Coalition Relic, Champion's Helm), lock Auras, tutors, draw spells, removal and wipes. X counts too, so Exsanguinate and Debt to the Deathless with X = 1 or more trigger her. Each trigger is 6 damage across a four-player table and 2 life for you. Creatures and spells of mana value 2 or less don't trigger her.

**Her draw needs 4 life lost by one player in one turn.** One trigger is only 2 to each opponent, so a draw needs a second source in the same turn:
- **Two triggers**, such as a removal spell on an opponent's turn plus your own main-phase spell, or two spells in one turn. Cheap instants with mana value 3 (Anguished Unmaking, Generous Gift, Your Fate Ends Here) on an opponent's turn are the easiest second trigger. Cast Away Doubt does it alone: its 2 damage plus her 2 is 4 to each opponent.
- **One trigger plus a drain**, from Sanguine Bond, Blight-Priest, Ill-Gotten Inheritance or Urborg Syphon-Mage.
- **Your own life.** Any player counts, you included. Read the Bones and Night's Whisper cost 2 each, Anguished Unmaking 3, and Necropotence as much as you like.

She checks at *every* end step, so instant-speed triggers on opponents' turns draw too. She is a 2/4, so she dies to a lot of removal: protect her (see below), and keep casting spells of mana value 3 or more every turn while she is out.

### Game plan

**Turns 1 to 3: mana.** Turn 2 Sol Ring, Arcane Signet, Talisman of Progress or Thought Vessel; Chromatic Lantern or Coalition Relic on three (each is a rock *and* a trigger once she is out). Play Y'shtola on turn 3 or 4 with a spell to follow. Keep a cheap lock Aura or Propaganda for a hand that has to deal with attackers.

**Turns 4 to 6: the engine.** Every turn cast a spell of mana value 3 or more. Prefer trigger spells first, while the opponents' life is high and blockers don't matter. Look for Sanguine Bond first (the tutors find it: Diabolic Tutor, Dark Petition, Idyllic Tutor). Lock or remove the biggest creature and put Propaganda or Windborn Muse out so the table can't attack you.

**Late game.** The first trigger usually lands by turn 6 to 8. Count before you cast an X spell: Exsanguinate or Debt to the Deathless plus Y'shtola's trigger plus the Blight-Priest's lifegain pings plus Bond often kills two players in one turn. If the table is not close, keep drawing and taxing.

### Life gain is the engine

- **Sanguine Bond:** whenever you gain life, target opponent loses that much. Each Y'shtola trigger becomes 2 to everyone plus 2 more to one opponent, and an X drain's lifegain becomes a second, larger drain.
- **Marauding Blight-Priest:** whenever you gain life, each opponent loses 1. That's one more to everyone per trigger, and per drain.
- **Ill-Gotten Inheritance:** 1 to each opponent and 1 life every upkeep (which sets off the Priest and Bond). {5}{B}, sacrifice: 4 damage to an opponent and 4 life.
- **Urborg Syphon-Mage:** {2}{B}, {T}, discard a card: each other player loses 2 and you gain the total. Discard spare lands.
- **Static Net:** exiles an opponent's nonland permanent and gains you 2 life, a lifegain event of its own on top of her trigger.
- **Polluted Bonds:** whenever a land an opponent controls enters, they lose 2 and you gain 2. Each opponent's land drop is a drain plus a lifegain event for the Priest and Bond, and casting it triggers her too.
- **Cast Away Doubt:** draw two; it deals 2 damage to each player, so with her trigger each opponent loses 4 (you take 2 and gain 2).

### The finishers

Exsanguinate (each opponent loses X) and Debt to the Deathless (each opponent loses 2X) gain you everything they drain. With Sanguine Bond out, that gain hits one opponent again. Debt for X = 5 at a table of three opponents at 25 life:
- each opponent loses 10, you gain 30;
- Bond: one opponent loses 30 more;
- Y'shtola: 2 more to each opponent;
- Blight-Priest: 1 to each opponent for each lifegain event.

The second opponent dies too if they're at 12. Debt to the Deathless is the best closer in the deck: save mana for it. Mystical Tutor ({U}) can put Exsanguinate or Debt to the Deathless on top at the end of an opponent's turn so you draw it for your own.

### Draw and engines

**Curiosity belongs on Y'shtola.** *Whenever enchanted creature deals damage to an opponent, you may draw a card.* Her trigger damages each opponent separately, so Curiosity on her draws three cards per trigger at a full table. It costs {U}, and Zur can fetch it.

**Mana, then cards.** The deck usually holds more spells than it can cast, so **Smothering Tithe** comes first: each opponent who draws without {2} to spare gives you a Treasure, and casting it triggers Y'shtola. **Bolas's Citadel** plays lands and spells off the top of the library for life, which the drains keep refilling (cast it at 25 life or more), and it triggers her too. **Consecrated Sphinx** draws two whenever an opponent draws; it's a creature, so it doesn't trigger her, and it's best cast when the hand is running low. **Necropotence** skips your draw step and turns life into cards (cast it with a healthy life total; the drains refill it). **Mystic Remora** draws a card when an opponent casts a noncreature spell unless they pay {4}: pay its cumulative upkeep for a turn or two at most, then let it go. **Esper Sentinel** draws a card when an opponent casts their first noncreature spell each turn, unless they pay {X} (its power). **Notion Thief** (flash) turns each extra card an opponent draws into a card for you, and goes well with **Deep Analysis**: target an opponent with it while Notion Thief is out and you draw the cards; its flashback ({1}{U}, 3 life) is a second Y'shtola trigger. The other draw spells are Night's Whisper, Read the Bones and Tezzeret's Gambit.

**Take their cards.** Gonti, Lord of Luxury (the best of the top four of an opponent's library), Thief of Sanity (each time it connects, the best of the top three; the other two go to their graveyard) and Hostage Taker (exiles their best creature or artifact: cast it before the Taker leaves the battlefield, or it goes back to its owner) let you cast an opponent's card with mana of any type. Bribery takes a creature outright. A stolen noncreature spell with mana value 3 or more triggers Y'shtola like your own.

**Discount.** Grand Arbiter Augustin IV makes your white and blue spells cost {1} less each (a white-and-blue spell {2} less) and the opponents' spells {1} more. **Venat, Heart of Hydaelyn** draws a card the first time each turn you cast a legendary spell (Y'shtola, Gonti, Augustin, Zur, Dakkon, The Eternal Wanderer); its {7} ability exiles a nonland permanent and flips it to Hydaelyn, the Mothercrystal, which gives another creature a +1/+1 counter and indestructible each turn (and draws a card if it is legendary, such as Y'shtola).

### Keep the table off you

Y'shtola is a 2/4 and the deck wins slowly. The table's creature decks are the main danger, so deal with attackers early.
- **Pillowfort:** Propaganda and Windborn Muse tax every attacker {2}. Cast them early against creature decks.
- **Locks:** Arrest, Prison Sentence, Luminous Bonds, Bound in Silence and Encrust shut down threats (and each triggers Y'shtola as you cast it).
- **Removal:** Path to Exile, Fatal Push, Go for the Throat, Generous Gift, Anguished Unmaking, Your Fate Ends Here, Skyclave Apparition, Prayer of Binding, Memory Trap, Static Net, The Eternal Wanderer and Dakkon, Shadow Slayer (-3: exile target creature).
- **Wipes:** Austere Command. Massacre Wurm is a one-sided sweeper of small creatures that drains 2 for each one that dies.
- **Counter:** Dovin's Veto stops a removal spell aimed at Y'shtola.

**Dakkon, Shadow Slayer** ({W}{U}{B}) is a noncreature spell with mana value 3, so casting it triggers Y'shtola. He enters with loyalty equal to the number of lands you control, surveils 2 with his +1 and exiles a creature with his -3, and his -6 can put an artifact from your hand or graveyard onto the battlefield.

### Protect Y'shtola

- **Equipment:** Lightning Greaves (shroud) and Champion's Helm (+2/+2, hexproof on a legend).
- **Restoration Angel:** flash, and blinks Y'shtola out of an exile or steal effect.
- **Counterspell:** Dovin's Veto stops the removal spell itself.
- **Thought Vessel:** no maximum hand size, so Necropotence, Consecrated Sphinx and Notion Thief never draw into a discard; also a two-mana rock (it doesn't trigger Y'shtola: mana value 2).

### How it wins

The damage comes mostly from Y'shtola's and Ill-Gotten Inheritance's triggers, then from drains (Sanguine Bond, Blight-Priest, the X spells, Syphon-Mage), and only a little from combat. Most games end with a drain: a big Debt to the Deathless or Exsanguinate with Bond and the Priest out, after the triggers have set up the kill. Combat damage and Syphon-Mage finish the rest.

### Weak spots and how to play around them

- **Creature decks (big attackers).** The deck loses almost always to combat. Land Propaganda or Windborn Muse early, lock or remove the biggest threat, and keep Austere Command or Massacre Wurm for the wide boards.
- **Fast combo decks.** The drain is too slow to race a deck that wins on turn 4 to 6; hold Dovin's Veto and Memory Trap / Static Net for the key piece, and Notion Thief or Mystic Remora to slow their draws.
- **Removal on Y'shtola.** Keep Restoration Angel, Lightning Greaves, Champion's Helm or Dovin's Veto up while she is the engine.
- **Cheap spells.** Spells of mana value 1 or 2 (Fatal Push, Path to Exile, Dovin's Veto) are good cards but don't trigger her: don't cast them if they cost a trigger.
- **Slow starts.** Expect to reach full power around turns 6 to 8; mulligan hands with no mana or no spell of mana value 3.

## Key lines

**The opening.** Turn 2 Sol Ring, Arcane Signet or Talisman of Progress, Y'shtola on turn 3 or 4. Then every turn cast a spell of mana value 3 or more. A Chromatic Lantern or Coalition Relic after her is a rock *and* a trigger.

**Bond first.** Sanguine Bond is the first thing the tutors find (Diabolic Tutor, Dark Petition, Idyllic Tutor). It costs 5 and Zur can't fetch it. Once it's out, every trigger drains an extra 2 and every X spell drains twice.

**Sequencing a turn.**
- **Trigger spells first,** while opponents' life is high and blockers don't matter.
- **The drain second,** once triggers have set up the kill.
- **Two triggers in one turn draw a card** at the end step.

**The kill.** Count before you cast an X spell: each opponent loses X (or 2X), plus 2 from Y'shtola, plus the Blight-Priest's 1 per lifegain. With Bond, the whole gain goes to one opponent on top. Dark Petition with two instants or sorceries in your graveyard adds {B}{B}{B}, which often pays for that turn's X.

**Zur in the 99.** Zur the Enchanter ({1}{W}{U}{B}, 1/4 flier) searches for an enchantment with mana value 3 or less whenever it attacks:
- **Necropotence** early with a healthy life total;
- **Propaganda** when the table attacks you;
- **Curiosity** for Y'shtola;
- **a lock Aura** on the biggest threat. A fetched Aura isn't cast and doesn't target, so it goes around hexproof and ward.

Zur can find Arrest, Bound in Silence, Curiosity, Encrust, Luminous Bonds, Memory Trap, Mystic Remora, Necropotence, Prison Sentence and Propaganda. (Zur's enchantment is put onto the battlefield, not cast, so it doesn't trigger Y'shtola.)

**Notion Thief + Deep Analysis.** Target an opponent with Deep Analysis while Notion Thief is out and you draw the cards; its flashback ({1}{U}, 3 life) is a second Y'shtola trigger.

## Mana and consistency

The list is exactly 100 cards.

37 lands (13 nonbasic, 8 Plains, 7 Island, 9 Swamp) plus five rocks: Sol Ring, Arcane Signet, Talisman of Progress, Chromatic Lantern, Coalition Relic (Thought Vessel is a sixth, colourless).

| Measure | Count | Note |
| --- | --- | --- |
| White lands | 15 | 8 Plains plus Adarkar Wastes, Caves of Koilos, Command Tower, Glacial Fortress, Isolated Chapel, Prairie Stream, Shattered Sanctum |
| Blue lands | 12 | 7 Island plus Adarkar Wastes, Command Tower, Glacial Fortress, Prairie Stream, Underground River |
| Black lands | 14 | 9 Swamp plus Caves of Koilos, Command Tower, Isolated Chapel, Shattered Sanctum, Underground River |
| Any colour, with a condition | 2 | Exotic Orchard (an opponent's land), Spire of Industry (1 life, with an artifact) |
| No mana the turn they land | 3 | Evolving Wilds, Terramorphic Expanse, Fabled Passage (each finds any basic, now one of 24) |
| Always enter tapped | 0 | no land always enters tapped |
| Sometimes enter tapped | 4 | Glacial Fortress, Isolated Chapel (need a basic type), Prairie Stream (two basics), Shattered Sanctum (two other lands); with 24 basics they rarely do |

Black is the main colour (Necropotence's {B}{B}{B}, Massacre Wurm's {B}{B}{B}, the X drains' {B}{B}), so Swamps outnumber the other basics. Blue has the fewest sources. Most blue spells cost a single {U}, and Bribery's {U}{U} is the exception.

**Curve.** One-drops 7, two-drops 8, three-drops 23, four-drops 16 (counting Y'shtola), five 4, six 5. Average mana value about 3.3, with Exsanguinate and Debt counted at X = 0. It is a slow curve by design: Y'shtola pays you for mana value 3 and up.

**Card types.** 15 creatures (plus Y'shtola), 16 enchantments, 9 artifacts, 8 instants, 12 sorceries, 2 planeswalkers.

**Finding cards.**
- **Engines:** Necropotence, Consecrated Sphinx, Bolas's Citadel, Mystic Remora, Esper Sentinel, Notion Thief, Curiosity on Y'shtola, and her own end-step draw.
- **Mana:** Smothering Tithe's Treasures, on top of the rocks.
- **Draw spells:** Night's Whisper, Read the Bones, Deep Analysis, Tezzeret's Gambit, Cast Away Doubt.
- **Cards taken from opponents:** Gonti, Thief of Sanity, Hostage Taker.
- **Tutors:** Diabolic Tutor and Dark Petition (any card), Idyllic Tutor (an enchantment), Mystical Tutor (an instant or sorcery, to the top), and Zur's attacks.

**Interaction.**
- **Counterspell (one):** Dovin's Veto.
- **Spot removal spells (six):** Path to Exile, Fatal Push, Go for the Throat, Generous Gift, Anguished Unmaking, Your Fate Ends Here.
- **Removal on permanents:** Skyclave Apparition, Hostage Taker, Prayer of Binding, Static Net, Memory Trap, The Eternal Wanderer, Dakkon, Shadow Slayer, Massacre Wurm.
- **Removal Auras (five):** Arrest, Prison Sentence, Luminous Bonds, Bound in Silence, Encrust.
- **Theft:** Bribery.
- **Board wipe (one):** Austere Command.

**Life is a resource.** Necropotence, Read the Bones, Night's Whisper, Anguished Unmaking, Tezzeret's Gambit, Deep Analysis's flashback, Caves of Koilos, Adarkar Wastes, Underground River, Spire of Industry and Talisman of Progress all cost life. The deck gains it back steadily: 2 per Y'shtola trigger, plus every drain.

## Bracket and Rule 0

**Bracket 4,** with seven Game Changers: **Necropotence**, **Notion Thief**, **Consecrated Sphinx**, **Bolas's Citadel**, **Smothering Tithe**, **Mystical Tutor** and **Grand Arbiter Augustin IV**. There are no infinite combos, no mass land destruction and no extra turns, so it's a Bracket 4 deck by its Game Changers, not by combos.

Disclose before the game:

- **Necropotence**, **Notion Thief**, **Consecrated Sphinx**, **Bolas's Citadel**, **Smothering Tithe**, **Mystical Tutor** and **Grand Arbiter Augustin IV** (the seven Game Changers).
- **Three tutors** (Diabolic Tutor, Dark Petition, Idyllic Tutor) plus Mystical Tutor, and Zur's enchantment search.
- **The win is a drain:** Y'shtola pings the table on every big spell, and Exsanguinate or Debt to the Deathless with Sanguine Bond can take out two players in one turn.
- **Theft:** Bribery, Hostage Taker, Gonti and Thief of Sanity use opponents' cards.

Re-check the official list at https://commanderbrackets.com/faq before an event.

## Decklist by type (100)

**Commander (1).** Y'shtola, Night's Blessed

**Creatures (15).** Consecrated Sphinx, Esper Sentinel, Gonti, Lord of Luxury, Grand Arbiter Augustin IV, Hostage Taker, Marauding Blight-Priest, Massacre Wurm, Notion Thief, Restoration Angel, Skyclave Apparition, Thief of Sanity, Urborg Syphon-Mage, Venat, Heart of Hydaelyn // Hydaelyn, the Mothercrystal, Windborn Muse, Zur the Enchanter

**Planeswalkers (2).** Dakkon, Shadow Slayer, The Eternal Wanderer

**Enchantments (16).** Arrest, Bound in Silence, Curiosity, Encrust, Ill-Gotten Inheritance, Luminous Bonds, Memory Trap, Mystic Remora, Necropotence, Polluted Bonds, Prayer of Binding, Prison Sentence, Propaganda, Sanguine Bond, Smothering Tithe, Static Net

**Artifacts (9).** Arcane Signet, Bolas's Citadel, Champion's Helm, Chromatic Lantern, Coalition Relic, Lightning Greaves, Sol Ring, Talisman of Progress, Thought Vessel

**Instants (8).** Anguished Unmaking, Dovin's Veto, Fatal Push, Generous Gift, Go for the Throat, Mystical Tutor, Path to Exile, Your Fate Ends Here

**Sorceries (12).** Austere Command, Bribery, Cast Away Doubt, Dark Petition, Debt to the Deathless, Deep Analysis, Diabolic Tutor, Exsanguinate, Idyllic Tutor, Night's Whisper, Read the Bones, Tezzeret's Gambit

**Lands (37).** Adarkar Wastes, Caves of Koilos, Command Tower, Evolving Wilds, Exotic Orchard, Fabled Passage, Glacial Fortress, Isolated Chapel, Prairie Stream, Shattered Sanctum, Spire of Industry, Terramorphic Expanse, Underground River, 8 Plains, 7 Island, 9 Swamp

## Import list (100)

```
1 Adarkar Wastes
1 Anguished Unmaking
1 Arcane Signet
1 Arrest
1 Austere Command
1 Bolas's Citadel
1 Bound in Silence
1 Bribery
1 Cast Away Doubt
1 Caves of Koilos
1 Champion's Helm
1 Chromatic Lantern
1 Coalition Relic
1 Command Tower
1 Consecrated Sphinx
1 Curiosity
1 Dakkon, Shadow Slayer
1 Dark Petition
1 Debt to the Deathless
1 Deep Analysis
1 Diabolic Tutor
1 Dovin's Veto
1 Encrust
1 Esper Sentinel
1 Evolving Wilds
1 Exotic Orchard
1 Exsanguinate
1 Fabled Passage
1 Fatal Push
1 Generous Gift
1 Glacial Fortress
1 Go for the Throat
1 Gonti, Lord of Luxury
1 Grand Arbiter Augustin IV
1 Hostage Taker
1 Idyllic Tutor
1 Ill-Gotten Inheritance
1 Isolated Chapel
1 Lightning Greaves
1 Luminous Bonds
1 Marauding Blight-Priest
1 Massacre Wurm
1 Memory Trap
1 Mystic Remora
1 Mystical Tutor
1 Necropotence
1 Night's Whisper
1 Notion Thief
1 Path to Exile
1 Polluted Bonds
1 Prairie Stream
1 Prayer of Binding
1 Prison Sentence
1 Propaganda
1 Read the Bones
1 Restoration Angel
1 Sanguine Bond
1 Shattered Sanctum
1 Skyclave Apparition
1 Smothering Tithe
1 Sol Ring
1 Spire of Industry
1 Static Net
1 Talisman of Progress
1 Terramorphic Expanse
1 Tezzeret's Gambit
1 The Eternal Wanderer
1 Thief of Sanity
1 Thought Vessel
1 Underground River
1 Urborg Syphon-Mage
1 Venat, Heart of Hydaelyn // Hydaelyn, the Mothercrystal
1 Windborn Muse
1 Y'shtola, Night's Blessed
1 Your Fate Ends Here
1 Zur the Enchanter
7 Island
8 Plains
9 Swamp
```
