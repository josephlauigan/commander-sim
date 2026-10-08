# Y'shtola, Night's Blessed — Esper Drain

**Updated 2026-10-03.** Out: Clever Concealment, Disdainful Stroke, Momentary Blink, Rootborn Defenses, Take Up the Shield. In: Cast Away Doubt, Memory Trap, Static Net, Statute of Denial, Your Fate Ends Here. Five rarely cast cards (each cast in 5% or less of the games it was drawn, or mana value 2) out for spells of mana value 3 or more that trigger Y'shtola: Static Net (exile, 2 life, a Powerstone), Cast Away Doubt (draw two; with her trigger each opponent loses 4), Your Fate Ends Here (instant removal), Memory Trap (exile) and Statute of Denial (a counter that triggers her).
**Updated 2026-10-04.** Out: Arcane Sanctum, Dimir Guildgate, Irrigated Farmland, Scoured Barrens, Temple of Deceit, Temple of Enlightenment, Temple of Silence, Vivid Creek, Vivid Marsh, Vivid Meadow. In: 4 Island, 3 Plains, 3 Swamp. The ten lands that always entered tapped became basics. Tested: about +1.2 points on average (1,000 paired games per tier, adaptive AI), positive in four of five tiers; Y'shtola wants to land on turn 3 or 4 with a spell to follow, and no land now always enters tapped.
**Updated 2026-10-05.** Out: Enslave, Jester's Cap, Plea for Guidance. In: Bolas's Citadel, Consecrated Sphinx, Smothering Tithe. Enslave → Consecrated Sphinx, Plea for Guidance → Bolas's Citadel, Jester's Cap → Smothering Tithe. Three cards that sat in hand most games out for three Game Changers: the deck moves to Bracket 4 with five. Tested: +1.2 points on average, ahead in all five tiers (1,000 paired games per tier, adaptive AI); Smothering Tithe's Treasures ease the mana the deck is short of.
**Updated 2026-10-06.** Out: Vanquish the Horde. In: Mystical Tutor. Vanquish the Horde → Mystical Tutor (from Veyran): finds Exsanguinate or Debt to the Deathless at the end of an opponent's turn. Vanquish was her most redundant reset (Austere Command, Crux of Fate and Massacre Wurm stay). Six Game Changers (Bracket 4).
**Updated 2026-10-07.** Out: Statute of Denial. In: Polluted Bonds. Statute of Denial → Polluted Bonds: a five-mana enchantment that triggers her when cast, then drains an opponent 2 (and gains you 2, feeding Sanguine Bond and Marauding Blight-Priest) on every land they play. Statute was stuck in hand in about three quarters of the games it was drawn.

2026-10-03
**Added 2026-10-03.** In the simulator as deck key `yshtola` (`python3 -m commander_sim --deck yshtola --pool t3`) and in practice mode. Every card is modeled. It shares much of its Esper shell with the Zur deck (`zur-esper-auras.md`), but it is a separate deck with its own commander, plan and list; Zur the Enchanter is in the 99 here.

## Strategy

Drain the table a little with every spell, then all at once. Y'shtola turns your ordinary noncreature spells into damage to every opponent and life for you. Sanguine Bond and Marauding Blight-Priest turn that life into more damage. Exsanguinate and Debt to the Deathless finish the game.

Y'shtola, Night's Blessed ({1}{W}{U}{B}, 2/4, vigilance) reads: *at the beginning of each end step, if a player lost 4 or more life this turn, you draw a card. Whenever you cast a noncreature spell with mana value 3 or greater, Y'shtola deals 2 damage to each opponent and you gain 2 life.*

**Thirty-six spells trigger her.** Every noncreature spell in the list with mana value 3 or more counts: rocks (Chromatic Lantern, Coalition Relic, Champion's Helm), lock Auras, tutors, draw spells, removal and wipes. X counts too, so Exsanguinate with X = 1 or more triggers her, and so does Secure the Wastes with X = 2 or more. Each trigger is 6 damage across a four-player table and 2 life for you. Creatures and spells of mana value 2 or less don't trigger her.

**Her draw needs 4 life lost by one player in one turn.** One trigger is only 2 to each opponent, so a draw needs a second source in the same turn:
- **Two triggers**, such as a removal spell on an opponent's turn plus your own main-phase spell, or two spells in one turn.
- **One trigger plus a drain**, from Sanguine Bond, Blight-Priest, Ill-Gotten Inheritance or Urborg Syphon-Mage.
- **Your own life.** Any player counts, you included. Read the Bones and Night's Whisper cost 2 each, Anguished Unmaking 3, and Necropotence as much as you like.

She checks at *every* end step, so instant-speed triggers on opponents' turns draw too.

**Life gain is the engine.**
- **Sanguine Bond:** whenever you gain life, target opponent loses that much. Each Y'shtola trigger becomes 2 to everyone plus 2 more to one opponent, and an X drain's lifegain becomes a second, larger drain.
- **Marauding Blight-Priest:** whenever you gain life, each opponent loses 1. That's one more to everyone per trigger, and per drain.
- **Ill-Gotten Inheritance:** 1 to each opponent and 1 life every upkeep (which sets off the Priest and Bond). {5}{B}, sacrifice: 4 damage to an opponent and 4 life.
- **Urborg Syphon-Mage:** {2}{B}, {T}, discard a card: each other player loses 2 and you gain the total. Discard spare lands.
- **Static Net:** exiles an opponent's nonland permanent and gains you 2 life, a lifegain event of its own on top of her trigger.
- **Polluted Bonds** (10-07): whenever a land an opponent controls enters, they lose 2 and you gain 2. Each opponent's land drop is a drain plus a lifegain event for the Priest and Bond, and casting it triggers her too.

**The finishers.** Exsanguinate (each opponent loses X) and Debt to the Deathless (each opponent loses 2X) gain you everything they drain. With Sanguine Bond out, that gain hits one opponent again. Debt for X = 5 at a table of three opponents at 25 life:
- each opponent loses 10, you gain 30;
- Bond: one opponent loses 30 more;
- Y'shtola: 2 more to each opponent;
- Blight-Priest: 1 to each opponent for each lifegain event.

The second opponent dies too if they're at 12.

**Curiosity belongs on Y'shtola.** "Whenever enchanted creature deals damage to an opponent, you may draw a card." Her trigger damages each opponent separately, so Curiosity on her draws three cards per trigger at a full table. It costs {U}, and Zur can fetch it.

**Take their cards.** Gonti, Hostage Taker and Thief of Sanity exile an opponent's card that you may cast with mana of any type. Bribery takes a creature outright.

**Mana, then cards (the 10-05 Game Changers).** The deck usually holds more spells than it can cast, so **Smothering Tithe** comes first: each opponent who draws without {2} to spare gives you a Treasure, and casting it triggers Y'shtola. **Bolas's Citadel** plays lands and spells off the top of the library for life, which the drains keep refilling (cast it at 25 life or more), and it triggers her too. **Consecrated Sphinx** draws two whenever an opponent draws; it's a creature, so it doesn't trigger her, and it's best cast when the hand is running low.
- **Gonti:** the best of the top four of an opponent's library.
- **Thief of Sanity:** each time it connects, the best of the top three; the other two go to their graveyard.
- **Hostage Taker:** exiles their best creature or artifact. Cast it before the Taker leaves the battlefield, or it goes back to its owner.

A stolen noncreature spell with mana value 3 or more triggers Y'shtola like your own.

**Keep the table off you.** Y'shtola is a 2/4, and the deck wins slowly.
- **Pillowfort:** Propaganda and Windborn Muse tax every attacker {2}.
- **Locks:** Arrest, Prison Sentence, Luminous Bonds, Bound in Silence and Encrust shut down threats (and each triggers Y'shtola as you cast it).
- **Removal:** Path to Exile, Fatal Push, Go for the Throat, Generous Gift, Anguished Unmaking, Skyclave Apparition, Prayer of Binding and The Eternal Wanderer.
- **Wipes:** Austere Command and Crux of Fate. Massacre Wurm is a one-sided sweeper of small creatures that drains 2 for each one that dies.

**Protect Y'shtola.**
- **Equipment:** Lightning Greaves (shroud) and Champion's Helm (+2/+2, hexproof on a legend).
- **Restoration Angel:** flash, and blinks Y'shtola out of an exile or steal effect.
- **Counterspell:** Dovin's Veto stops the removal spell itself.
- The protection instants (Take Up the Shield, Rootborn Defenses, Clever Concealment, Momentary Blink) left on 10-03: they were rarely cast, and removal does more against the creature decks that beat this one.
- **Bastion Protector:** commanders get +2/+2 and indestructible.

## Key lines

**The opening.** Turn 2 Sol Ring, Arcane Signet or Talisman of Progress, Y'shtola on turn 3 or 4. Then every turn cast a spell of mana value 3 or more. A Chromatic Lantern or Coalition Relic after her is a rock *and* a trigger.

**Bond first.** Sanguine Bond is the first thing the tutors find (Diabolic Tutor, Dark Petition, Idyllic Tutor). It costs 5 and Zur can't fetch it. Once it's out, every trigger drains an extra 2 and every X spell drains twice.

**Sequencing a turn.**
- **Trigger spells first,** while opponents' life is high and blockers don't matter.
- **The drain second,** once triggers have set up the kill.
- **Two triggers in one turn draw a card** at the end step.

**The kill.** Count before you cast an X spell: each opponent loses X (or 2X), plus 2 from Y'shtola, plus the Blight-Priest's 1 per lifegain. With Bond, the whole gain goes to one opponent on top. Dark Petition with two instants or sorceries in your graveyard adds {B}{B}{B}, which often pays for that turn's X.

**Zur in the 99.** Zur the Enchanter ({1}{W}{U}{B}, 1/4 flier) still searches for an enchantment with mana value 3 or less whenever it attacks:
- **Necropotence** early with a healthy life total;
- **Propaganda** when the table attacks you;
- **Curiosity** for Y'shtola;
- **a lock Aura** on the biggest threat. A fetched Aura isn't cast and doesn't target, so it goes around hexproof and ward.

Zur can find Arrest, Bound in Silence, Curiosity, Encrust, Luminous Bonds, Mystic Remora, Necropotence, Prison Sentence and Propaganda.

**Notion Thief + Deep Analysis.** Target an opponent with Deep Analysis while Notion Thief is out and you draw the cards; its flashback ({1}{U}, 3 life) is a second Y'shtola trigger.

## Consistency

The list is exactly 100 cards.

37 lands (11 nonbasic, 8 Plains, 8 Island, 10 Swamp) plus five rocks: Sol Ring, Arcane Signet, Talisman of Progress, Chromatic Lantern, Coalition Relic.

| Measure | Count | Note |
| --- | --- | --- |
| White lands | 14 | 8 Plains plus Caves of Koilos, Command Tower, Glacial Fortress, Isolated Chapel, Prairie Stream, Shattered Sanctum |
| Blue lands | 11 | 8 Island plus Command Tower, Glacial Fortress, Prairie Stream |
| Black lands | 14 | 10 Swamp plus Caves of Koilos, Command Tower, Isolated Chapel, Shattered Sanctum |
| Any colour, with a condition | 2 | Exotic Orchard (an opponent's land), Spire of Industry (1 life, with an artifact) |
| No mana the turn they land | 3 | Evolving Wilds, Terramorphic Expanse, Fabled Passage (each finds any basic, now one of 26) |
| Always enter tapped | 0 | the ten that did (Arcane Sanctum, Dimir Guildgate, Irrigated Farmland, Scoured Barrens, three Temples, three Vivid lands) became basics on 10-04 |
| Sometimes enter tapped | 4 | Glacial Fortress, Isolated Chapel (need a basic type), Prairie Stream (two basics), Shattered Sanctum (two other lands); with 26 basics they rarely do |

Black is the main colour (Necropotence's {B}{B}{B}, Massacre Wurm's {B}{B}{B}, the X drains' {B}{B}), so Swamps outnumber the other basics. Blue has the fewest sources. Most blue spells cost a single {U}, and Bribery's {U}{U} is the exception.

**Curve.** One-drops 8, two-drops 7, three-drops 22, four-drops 15, five 4, six 6. Average mana value about 3.3, with Exsanguinate and Debt counted at X = 0. It is a slower curve than the Zur deck's, by design: Y'shtola pays you for mana value 3 and up.

**Card types.** 14 creatures, 15 enchantments, 8 artifacts, 9 instants, 15 sorceries, 1 planeswalker.

**Finding cards.**
- **Engines:** Necropotence, Consecrated Sphinx, Bolas's Citadel, Mystic Remora, Esper Sentinel, Notion Thief, Curiosity on Y'shtola, and her own end-step draw.
- **Mana:** Smothering Tithe's Treasures, on top of the rocks.
- **Draw spells:** Night's Whisper, Read the Bones, Deep Analysis, Tezzeret's Gambit, Cast Away Doubt.
- **Cards taken from opponents:** Gonti, Thief of Sanity, Hostage Taker.
- **Tutors:** Diabolic Tutor and Dark Petition (any card), Idyllic Tutor (an enchantment), and Zur's attacks.

**Interaction.**
- **Counterspell (one):** Dovin's Veto.
- **Spot removal spells (six):** Path to Exile, Fatal Push, Go for the Throat, Generous Gift, Anguished Unmaking, Your Fate Ends Here.
- **Removal on permanents:** Skyclave Apparition, Hostage Taker, Prayer of Binding, Static Net, Memory Trap, The Eternal Wanderer, Massacre Wurm.
- **Removal Auras (five):** Arrest, Prison Sentence, Luminous Bonds, Bound in Silence, Encrust.
- **Theft:** Bribery.
- **Board wipes (two):** Austere Command, Crux of Fate.
- **Mystical Tutor** (10-06, from Veyran): at an opponent's end step, put Exsanguinate or Debt to the Deathless on top for your turn.

## What the simulator found

Measured on 2026-10-03 with the look-ahead AI against the loose profile (opponents counter and remove freely: the worst case), 120 games per tier, the same settings as the other recent decks in `decklists/pool/pool-results.md`. An even share is 25%.

| Tier | Y'shtola | Zur (same settings) | Galadriel (same settings) |
| --- | --- | --- | --- |
| T1 High B2 / Low B3 | **31.7%** (24-40) | 24.2% | 15.0% |
| T2 Mid B3 | **50.8%** (42-60) | 15.0% | 11.7% |
| T3 High B3 | **37.5%** (29-46) | 16.7% | 8.3% |
| T4 Low B4 | **35.8%** (28-45) | 10.0% | 12.5% |
| T5 High B4 | 14.2% (9-22) | 16.7% | 8.3% |

**Above an even share through Low Bracket 4, below it against High Bracket 4.** Tier 2 is the standout, at about half of all games, level with Sephiroth. Against Tier 5 the fast combo decks (Kinnan, Urza, Yawgmoth) win before the drain gets there.

**How it wins** (a 400-game breakdown against Tier 3 with the faster heuristic AI, which wins 39% there):
- **The damage it deals:** 46 to opponents per game. 47% comes from Y'shtola's and Ill-Gotten Inheritance's triggers, 31% from drains (Sanguine Bond, Blight-Priest, the X spells, Syphon-Mage) and only 21% from combat.
- **The kills:** 55% by drain, 23% by triggers, 21% by combat.
- **When the engine comes online:** Y'shtola's first trigger lands by turn 6 in 43% of games and by turn 8 in 65% of games. She triggers at some point in 86% of games.
- **The finisher:** Debt to the Deathless is the card most tied to winning. The deck wins 75% of the games it's cast in.
- **How it loses:** almost always to combat (Aurelia above all), not to combo. Propaganda and Windborn Muse are worth casting early against creature decks.

**Cards that sat in hand.** Rootborn Defenses, Clever Concealment, Take Up the Shield and Momentary Blink were cast in 5% or less of the games they were drawn. They only come out in answer to removal or a wipe, and the AI rarely needed them. They were replaced on 10-03 (with Disdainful Stroke) by Static Net, Cast Away Doubt, Your Fate Ends Here, Memory Trap and Statute of Denial; these results are from the list before that change.

## Bracket and Rule 0

**Bracket 4,** with six Game Changers: **Necropotence**, **Notion Thief**, **Consecrated Sphinx**, **Bolas's Citadel**, **Smothering Tithe** (the last three added 10-05) and **Mystical Tutor** (10-06). There are still no two-card infinite combos (Sanguine Bond isn't paired with Exquisite Blood), no mass land destruction and no extra turns, so it's a Bracket 4 deck by its Game Changers, not by combos.

Disclose before the game:

- **Necropotence**, **Notion Thief**, **Consecrated Sphinx**, **Bolas's Citadel**, **Smothering Tithe** and **Mystical Tutor** (the six Game Changers).
- **Three tutors** (Diabolic Tutor, Dark Petition, Idyllic Tutor), plus Zur's enchantment search.
- **The win is a drain:** Y'shtola pings the table on every big spell, and Exsanguinate or Debt to the Deathless with Sanguine Bond can take out two players in one turn.
- **Theft:** Bribery, Hostage Taker, Gonti and Thief of Sanity use opponents' cards.

Re-check the official list at https://commanderbrackets.com/faq before an event.

## Decklist by type (100)

**Commander (1).** Y'shtola, Night's Blessed

**Creatures (14).** Bastion Protector, Consecrated Sphinx, Esper Sentinel, Gonti, Lord of Luxury, Hostage Taker, Marauding Blight-Priest, Massacre Wurm, Notion Thief, Restoration Angel, Skyclave Apparition, Thief of Sanity, Urborg Syphon-Mage, Windborn Muse, Zur the Enchanter

**Planeswalkers (1).** The Eternal Wanderer

**Enchantments (16).** Arrest, Bound in Silence, Curiosity, Encrust, Ill-Gotten Inheritance, Luminous Bonds, Memory Trap, Mystic Remora, Necropotence, Polluted Bonds, Prayer of Binding, Prison Sentence, Propaganda, Sanguine Bond, Smothering Tithe, Static Net

**Artifacts (8).** Arcane Signet, Bolas's Citadel, Champion's Helm, Chromatic Lantern, Coalition Relic, Lightning Greaves, Sol Ring, Talisman of Progress

**Instants (9).** Anguished Unmaking, Dovin's Veto, Fatal Push, Generous Gift, Go for the Throat, Mystical Tutor, Path to Exile, Secure the Wastes, Your Fate Ends Here

**Sorceries (14).** Austere Command, Bribery, Cast Away Doubt, Crux of Fate, Dark Petition, Debt to the Deathless, Deep Analysis, Diabolic Tutor, Exsanguinate, Idyllic Tutor, Night's Whisper, Read the Bones, Tezzeret's Gambit, Triplicate Spirits

**Lands (37).** Caves of Koilos, Command Tower, Evolving Wilds, Exotic Orchard, Fabled Passage, Glacial Fortress, Isolated Chapel, Prairie Stream, Shattered Sanctum, Spire of Industry, Terramorphic Expanse, 8 Plains, 8 Island, 10 Swamp

## Import list (100)

```
1 Anguished Unmaking
1 Arcane Signet
1 Arrest
1 Austere Command
1 Bastion Protector
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
1 Crux of Fate
1 Curiosity
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
1 Secure the Wastes
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
1 Triplicate Spirits
1 Urborg Syphon-Mage
1 Windborn Muse
1 Y'shtola, Night's Blessed
1 Your Fate Ends Here
1 Zur the Enchanter
8 Island
8 Plains
10 Swamp
```

## Flags and tuning levers

**Tapped lands were the biggest cost, and are gone.** The ten lands that always entered tapped became basics on 10-04 (about +1.2 points in testing). Four duals still sometimes enter tapped (Glacial Fortress, Isolated Chapel, Prairie Stream, Shattered Sanctum); with 26 basics they rarely do. To add fixing back without tapped lands: Drowned Catacomb, Godless Shrine, Watery Grave, Hallowed Fountain.

**Two triggers a turn is the target.** Her draw needs 4 life lost by one player, which one trigger alone never does. Cheap instants with mana value 3 (Anguished Unmaking, Generous Gift, Your Fate Ends Here) on an opponent's turn are the easiest second trigger. Cast Away Doubt does it alone: its 2 damage plus her 2 is 4 to each opponent.

**Weakest slots for this commander.**
- **Fatal Push, Path to Exile, Dovin's Veto:** good cards, but mana value 1 or 2 doesn't trigger Y'shtola.
- **Secure the Wastes:** triggers only with X = 2 or more.
- **Bastion Protector and Restoration Angel:** creatures, so no trigger.

Replacements worth testing:
- **Removal with mana value 3:** Mortify (instant) or Vindicate (sorcery).
- **More lifegain payoffs:** Vito, Thorn of the Dusk Rose; Exquisite Blood. Exquisite Blood with Sanguine Bond is an infinite combo and moves the deck toward Bracket 4.

**Life is still a resource.** Necropotence, Read the Bones, Night's Whisper, Anguished Unmaking, Tezzeret's Gambit, Deep Analysis's flashback, Caves of Koilos, Spire of Industry and Talisman of Progress all cost life. Unlike the Zur deck, this one gains it back steadily: 2 per Y'shtola trigger, plus every drain.

**Mystic Remora's upkeep grows.** Pay it for a turn or two at most, then let it go.
