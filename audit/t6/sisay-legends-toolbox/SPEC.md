# Sisay, Weatherlight Captain (Legends Toolbox) — Behaviour Spec (Tier 6)

Sources (all opened 2026-10-17; full links in the Sources list at the end):
[A] Archidekt primer "Sisay It Ain't So" (cEDH combo-midrange, shown as "12 days ago", no date); [J] "[cEDH] Odds and Ends
for Sisay" by 賽の河原, note.com, 2026-08-19; [H] "An Introduction To Sisay, Weatherlight Captain in cEDH" by Veiss Versa,
Commander's Herald, 2024-04-22; [G] "How To Play The Sisay ... cEDH Deck" by Johnny Garcia, TheGamer, 2025-03-12;
[D] "Legendary Oversaturation Brings Sisay ... to New Heights", Draftsim, 2026-08-20; [P] docs.mdbook.me "Sisay, Weatherlight
Captain" page, 2026-10-16; [E] EDHREC combo page "Marvin, Murderous Mimic + Ioreth of the Healing House"; [T] EDHTop16 list
page, Ian Flannery, The Cookout 2026, 2026-08-29, #2 of 238, 5-1-3 (the list in our file; the page has no write-up);
[DB] cedh-decklist-database.com Sisay entries (one-paragraph blurbs, 2023; no primer text). Oracle text: Scryfall,
checked 2026-10-17 for all 100 cards. Moxfield pages returned HTTP 403. Spec written 2026-10-17.

How well the sources fit our list: [A] is the closest match (it names Deadpool, Forbidden Orchard, Mount Doom, Agatha's Soul
Cauldron, Ioreth, Marvin, Selvala, Shang-Chi, Enduring Vitality, all in our list). [H] and [G] are older and name cards our
list lacks (Aminatou, Nicol Bolas, Oath of Teferi, Finale of Devastation, Dockside Extortionist, Faeburrow Elder, Thassa's
Oracle, Demonic Consultation). [J] says Dockside Extortionist was banned and recent builds favour mana creatures.

## Game plan
Sisay is a legends-matter toolbox that wins by infinite mana, not by a single spell. The deck ramps (Moxen, Sol Ring, Birds,
Hierarchs, Bloom Tender), lands Sisay, puts cheap legends onto the battlefield with her {W}{U}{B}{R}{G} ability (it gets stronger
with each coloured legend), and uses the untap engine it finds to loop mana, then finishes with Mount Doom or a Deadpool
trick. **[primer]** [A] aims for mana on turns 1-2, Sisay on turns 2-3 and a win threat around turns 3-4, but says it can go
longer; [H] calls it slower than most cEDH decks but able to set up lethal fast. Stance: **hold up and grind first, then go
off once protected**. [A] says to make opponents spend interaction first and to "protect the engine, not automatically the
commander". It has no one-shot combo [J]. Its tutored permanents arrive without being cast, so counterspells can't stop them
[A][D]. The deck also taxes and slows the table (Teferi, Lavinia, Esper Sentinel, Rhystic Study, Spider-Woman) while it sets up
[P][J].

Sisay's numbers **[primer, Oracle]**: she is 2/2 for {2}{W}. She gets +1/+1 for each colour among *other* legendary permanents
you control, so at most 7/7. Her ability fetches a legendary permanent with mana value *less than* her power, so power 2 finds
MV 0-1 only, power 3 finds MV 2, power 4 finds MV 3, power 5 finds MV 4. Each activation costs a full WUBRG (5 mana).

## Win lines
Mana is counted from an empty board with every piece cast from hand. Pieces fetched by Sisay cost her activation (5 coloured
mana each) instead of their cost, but need her power high enough (see above).

| Line | Pieces (in our list) | Mana (total, colours) | Steps | Where it can be answered (step, by what) |
| --- | --- | --- | --- | --- |
| **1. Mana loop, Bloom Tender + Derevi + Emiel** **[primer]** [A][G] | Bloom Tender (non-legendary: draw or tutor it), Derevi, Emiel, Sisay as the tutor. Finisher: Mount Doom. Kinnan adds +1 mana per Bloom Tender tap **[inference]** | Cast from hand: Bloom Tender 2 (1G) + Derevi 3 (GWU) + Emiel 4 (2WW) = 9, then 3 more per loop. Five colours among your permanents make Bloom Tender tap for WUBRG (5). Via Sisay: Sisay 3 (2W) + one 5-mana activation per piece | 1. Have Bloom Tender, Derevi, Emiel and five colours among permanents. 2. Tap Bloom Tender for 5. 3. Pay {3}: Emiel blinks Derevi. 4. Derevi's enter trigger untaps Bloom Tender. 5. Repeat: net +2 per loop (+3 with Kinnan). Pay the 5 for Sisay activations. Finish: see below | Before Sisay's activation: kill Sisay or the mana creature. In response to Emiel's ability: kill Derevi or Emiel (the loop dies if Derevi is exiled). On Derevi's trigger: Ertai-style counters or Stifle effects. Afterwards: graveyard hate, search hate, activated-ability hate (Cursed Totem) [A] |
| **2. Cradle loop, Derevi + Emiel + Gaea's Cradle + Agatha's Soul Cauldron** **[primer]** [A] | Derevi, Emiel, Cradle, Cauldron (Cradle and Cauldron are legendary, so Sisay fetches them). Needs 4+ creatures | Cast: Derevi 3 + Emiel 4 + Cauldron 2 = 9, plus creatures to feed Cradle. Cradle makes G per creature. Cauldron lets green mana pay creature abilities (Sisay's) as any colour. 3 per loop | Tap Cradle for 4+ G. Pay {3}: Emiel blinks Derevi. Derevi untaps Cradle. Repeat. Surplus G pays Sisay via Cauldron | Same as line 1, plus: kill creatures in response (Cradle shrinks), artifact removal on Cauldron, land removal on Cradle |
| **3. Untap loop, Marvin + Ioreth + a legendary mana source** **[primer]** [A][E][J] | Marvin, Ioreth, plus one other legendary creature that makes mana: Selvala, Shang-Chi, or any legend with Enduring Vitality [A]. **[inference]** Cryptolith Rite (same text as Vitality) and Relic of Legends (taps legends for mana) also work; Sisay is a legend too. Tyvar or Shang-Chi give haste to abilities | Cast: Marvin 2 + Ioreth 3 (2U) + Selvala 3 (1GG) = 8; with Shang-Chi instead: 2+3+2 = 7 and the combo works the turn they land (Shang-Chi lets abilities ignore summoning sickness). Shang-Chi's mana only pays creature abilities **[inference]**, so Mount Doom needs other mana | Marvin has Ioreth's ability. Ioreth (tap) untaps Marvin and the mana source; Marvin (tap, Ioreth's ability) untaps Ioreth and the mana source. The source taps for mana each time. Selvala needs {G} and makes X (greatest power, at least 2) so net +1 or more | Needs Marvin and Ioreth free of summoning sickness [E]. Answered by killing or bouncing Marvin or Ioreth with the loop's first activation on the stack, or a Cursed Totem effect. It uses no spells, so counterspells do nothing |
| **4. Finisher: Mount Doom** **[primer]** [A][G] | Mount Doom (legendary land; Sisay fetches it with power 1+) | {1}{B}{R} and tap per 1 damage to each opponent. Needs an untap per ping: Derevi (extra blink = 3) or Marvin/Ioreth | With infinite mana, each ping = 3 for Mount Doom + one extra untap (3 for another Emiel blink). 40 pings for a 40-life opponent | Kill the untap source (Derevi/Emiel or Marvin/Ioreth) before the next ping. Damage already dealt stays. Lifegain from opponents raises the number of pings. Mount Doom itself can be destroyed or bounced in response to an activation |
| **5. Finisher: Deadpool + Forbidden Orchard** **[primer]** [A] | Deadpool, Trading Card, Forbidden Orchard, Emiel, infinite mana | Deadpool 4 (2BR) | With infinite mana tap Orchard repeatedly: each tap gives an opponent a 1/1 Spirit. Emiel blinks Deadpool; as it enters it swaps text boxes with a Spirit. The Spirit now says "at the beginning of your upkeep, you lose 3 life". Repeat per Spirit. Opponents lose 3 life per Spirit at their upkeep [A] | Slow: it wins at an opponent's upkeep, so it gives them a turn. Kill the Spirits, or win before upkeep. **[inference]** a faster finisher is Mount Doom |
| **6. Last resort: Sisay beats** **[primer]** [H] | Sisay with 5 coloured legends is 7/7 | n/a | Attack | Chump blockers, any removal. [H] says this means the deck is near the bottom of its win options |

**Tutors for each piece.** Sisay finds legends: Derevi (MV3, power 4+), Emiel (MV4, power 5+), Marvin (MV2), Ioreth (MV3),
Selvala (MV3), Shang-Chi (MV2), Kinnan (MV2), Tyvar (MV3), Cradle, Cauldron (MV2), Mount Doom (MV0), Teferi (MV3/4), Deadpool
(MV4). Bloom Tender and Forbidden Orchard are not legendary: find them with Worldly Tutor, Eladamri's Call, Chord of Calling or
Green Sun's Zenith (Bloom Tender, Selvala, Kinnan, Shang-Chi, Derevi are green creatures) or Demonic/Vampiric Tutor (anything).
Enlightened Tutor finds artifacts and enchantments: Marvin (an artifact creature), Cauldron, Relic of Legends, Cryptolith Rite,
Mox Amber **[Oracle]**. Emiel and Ioreth are white and blue: only Worldly (any creature), Eladamri's Call, Chord of Calling and the
black tutors fetch them. [A] says "Sisay rarely needs to naturally draw its combos".

## Mulligan
No source gives a full mulligan rule. What the sources say: [H] "avoid slow hands; the first three turns need acceleration."
Bad hands in [H]: five lands and removal only; a hand whose only mana-producing plan is a tutor (Teferi and Bolas stuck in hand);
a hand with nothing to do before turn 3. Good hands in [H]: a mana rock plus a tutor-able creature like Selvala; Bloom Tender
plus a payoff; three mana on turn 2 for Sisay, then a mana creature.
- **Keep [inference from H]:** 2-3 lands or fast mana, a mana creature or rock, and either Sisay-castable colours (W for Sisay,
  fixing for WUBRG) or a tutor. Protection (a free counter or Silence) is a bonus, not a need.
- **Ship [inference from H]:** no mana sources; hands with only legends that cost 4+; hands with five lands and no play.
- **Typical keep:** two lands (one a fetch or Tundra-type), Birds or Noble Hierarch, Sol Ring or a Mox, one cheap legend (Lotho,
  Lavinia, Tataru Taru) or a tutor, and Force of Will with a blue card.
Colours are a real constraint: the deck needs all five (WUBRG) to activate Sisay [J].

## Turn by turn
**Turns 1-2 [primer: A, H].** Deploy mana first: a land, then a Mox, Sol Ring, Mana Vault or a dork (Birds, Hierarchs, Bloom
Tender). Cast cheap legends that also work as taxes or engines: Lavinia (turn 2 slows opponents [H]), Esper Sentinel, Lotho,
Tataru Taru. Enchantments that draw cards on opponents' spells (Rhystic Study, Mystic Remora, Smothering Tithe) come out when
mana allows. Sisay on turn 2-3.
**Turns 3-4 [primer: A].** Sisay in play, then first activation to fetch the cheapest legend that raises her power (a mono-coloured
2-drop like Lavinia, Kinnan or Shang-Chi). Each activation raises her power by one if a new colour appears, which opens the next
fetch. Teferi, Time Raveler comes down to lock opponents to sorcery speed. Fetch the missing engine piece (Marvin, Ioreth,
Derevi, Emiel). **[primer: A]** activate Sisay more than once per turn cycle, using a second activation at an opponent's end step.
**Later [primer: A, G].** If the engine isn't ready, keep making mana and fetch answers (Spider-Woman, Lavinia, Teferi) while
holding counters. Colossal Skyturtle (channel) and Cauldron-granted abilities give recursion [G, A].

## Interaction
**What it holds (from our list):** free counters Force of Will, Fierce Guardianship, Pact of Negation, Mental Misstep (MV1),
Swan Song (enchantment/instant/sorcery), An Offer You Can't Refuse (noncreature), Flusterstorm (instant/sorcery); Mindbreak Trap
(exile spells when an opponent cast 3+); Silence (opponents can't cast spells this turn); Deflecting Swat and Redirect Lightning
(redirect); Legolas's Quick Reflexes (split second); removal: Abrupt Decay (MV3 or less), Dismember, Cyclonic Rift, Touch the
Spirit Realm; Ertai Resurrected (flash, counters a spell, ability or trigger, or kills a creature or planeswalker); on the board:
Teferi, Time Raveler (opponents at sorcery speed), Lavinia (counters spells cast with no mana; caps noncreature spells to MV
≤ their lands), Deafening Silence, Esper Sentinel, Rhystic Study, Mystic Remora, Smothering Tithe, Spider-Woman (artifacts
and creatures opponents control enter tapped). **[Oracle]**
**Two cards need Sisay on the battlefield:** Fierce Guardianship and Deflecting Swat are free only "if you control a commander";
Sisay in the command zone does not count **[Oracle]**.
**Spends [inference]:** free counters on spells that win or lock (combo spells, Tutors the opponent uses to go off, stax pieces
aimed at the engine), saving Silence for the turn it combos. Mental Misstep on 1-mana fast mana. Ertai for a trigger or a
creature that threatens the engine. The sources do not give a spend table.
**Lets resolve [inference]:** opponents' ramp and non-threatening permanents, since the card-draw enchantments (Rhystic Study,
Remora) tax them anyway.
**Who to stop when two players threaten:** the primers do not say. **[inference]** Stop the player whose line is already
on the stack or who needs only one more step; spend a free counter on that, keep Silence for its own turn.

## When it goes for the win
**[primer: A]** It goes when the engine is assembled and it can pay for protection, not before: "avoid committing too early,
keep protection available, aim to activate Sisay more than once per turn, make opponents spend interaction first."
- **Open mana and protection [inference from A]:** wants at least one counter or Silence in hand for the key step, and enough
  spare mana to pay for the first Sisay activation plus a re-do if an answer arrives.
- **The three answer windows [A]:** before the activation (removal on Sisay or a mana engine); in response to the activation
  (removing the thing to be untapped); after the piece enters (removal, ability hate, search hate, graveyard hate). Each is a
  reason to wait if the table holds open mana.
- **Waits when:** a table player has untapped open mana and cards in hand (a flash removal or counter), a hate piece is out
  (Cursed Totem, Opposition Agent, Weathered Runestone [H]), or Sisay's power is too low to reach the next piece.
- **Goes when:** the loop starts with abilities only (lines 1-3 use no spells), the table is tapped out or Silence resolves,
  or the table's interaction has been spent on earlier threats. **[inference]** After Silence, only abilities remain for opponents.

## What it fears
| Fear | Source | Play around |
| --- | --- | --- |
| Multiple layers of stax (Opposition Agent, Cursed Totem, Weathered Runestone) | [H] | Default to card draw and removal [H]; Abrupt Decay hits MV3 or less; Cyclonic Rift; Touch the Spirit Realm |
| Removal on Sisay or on a mana engine | [A] | "Protect the engine, not the commander": hold counters/Silence; commander tax makes recasting costly |
| Activated-ability, search and graveyard hate | [A] | Pieces arrive without spells; keep Ertai and Swan Song for hate pieces |
| Mana-base strain (WUBRG) | [J] | The deck runs 28 lands, many duals, fetches, Mana Confluence, City of Brass, Exotic Orchard |
| Fragile utility creatures (Orcish Bowmasters) | [J] | [J] cut the Queen of Dale for this; ours keeps her **[inference: more fragile engine]** |
| Chump blockers vs the beat-down line | [G] | Use loops, not combat |
| Thassa's Oracle / Demonic Consultation from opponents | [J] | Gleaming Splendor is [J]'s still-being-tested answer; Spider-Woman does not stop it |

## Against Godo
How this deck stops Godo's lines (Oracle text checked):
- **Godo cast (creature spell, {5}{R}).** Force of Will and Pact of Negation can counter it. Fierce Guardianship, An Offer You
  Can't Refuse and Swan Song cannot (not creature spells), and Flusterstorm cannot (instant/sorcery only). Mental Misstep
  counters the 1-mana rocks that pay for Godo. Lavinia counters any spell cast with no mana spent (Godo's free spells, a Force of
  Will pitch cast against her) and Teferi, Time Raveler keeps Godo's deck from responding at instant speed.
- **Godo's enter trigger (fetches Helm of the Host).** Ertai Resurrected (flash, {2}{U}{B}) counters a triggered ability; its
  controller draws a card. Silence on Godo's turn before he is cast stops the whole turn.
- **Helm attached, then token copies each combat.** Cyclonic Rift, Touch the Spirit Realm or Teferi, Time Raveler's −3 can
  remove Helm or Godo before beginning of combat. Dismember (-5/-5) kills Godo even with Hammer of Nazahn's indestructible.
  Abrupt Decay cannot (Godo MV6, Helm MV4 are above 3). Spider-Woman makes every Godo token enter tapped, so the tokens can't attack
  that combat **[inference from Oracle]**.
- **Dualcaster Mage + copy spell.** Force of Will/Pact on Dualcaster (creature) or on the spell; Flusterstorm on the copies
  (unless the controller pays 1); Swan Song on a sorcery copy spell; Ertai on Dualcaster's trigger or on the creature; Deflecting
  Swat or Redirect Lightning to change the target of the copy.
- **Our timing [inference]:** Godo goes off on turn 2-3; this deck usually has not yet landed Sisay by then, so only the
  non-commander free counters (Force of Will, Pact, Mental Misstep, Mindbreak Trap if Godo's deck casts 3 spells) are live.
  **For the Godo spec:** Godo should expect FoW/Pact on the commander, and Ertai on the trigger. It beats this deck by
  winning before Sisay and her engine are down, by baiting free counters with rituals first, and by using Hexing Squelcher or
  Cavern of Souls (naming Human) to make Godo uncounterable.

## Decision scenarios (become AI tests)
1. **Ramp or hold FoW?** Turn 1, hand: Birds, Sol Ring, Force of Will, one blue card, a land. Opponent casts Sol Ring then a
   second turn-1 mana spell. Action: play Birds-line, keep FoW for a combo piece, do not Misstep a 2-mana spell. Reason: Misstep
   only hits MV1. Tag **[inference]**.
2. **Sisay on turn 2.** Hand: Sisay, two lands plus a Mox. Open table. Action: cast Sisay with the Mox (3 mana), no counter
   backup needed. Reason: [A] puts Sisay on turns 2-3. Tag **[primer: A]**.
3. **First activation target.** Sisay in play (power 2), WUBRG available. Action: fetch Ragavan (MV1, red, raises power to 3), then use the next activation at power 3 for an MV2 piece (Kinnan, Lavinia, Marvin, Shang-Chi). Reason: power 2 only reaches MV 0-1 (Oracle). Tag **[inference from Oracle]**.
4. **Mana loop available.** Bloom Tender, Derevi and Emiel on board, five colours present, opponents have 2 open Islands.
   Action: wait one turn unless Silence is in hand; otherwise cast Silence and go. Reason: the three answer windows [A]. Tag
   **[primer: A]**.
5. **Protect the engine.** Sisay is targeted by removal with a loop piece on board. Action: counter the removal, not save the
   commander if a Rift/Decay only hits Sisay and the engine is complete. Reason: "protect the engine, not the commander." Tag
   **[primer: A]**.
6. **Stax in play.** Opponent has Cursed Totem out. Action: remove Totem first (Cyclonic Rift, or Abrupt Decay if its MV is 3 or less), then start creature-ability loops. Reason: [H] lists Totem as a hard lock. Tag **[inference from H]**.
7. **Marvin line, summoning sick.** Marvin and Ioreth were cast this turn, Selvala in play. Action: wait a turn unless Shang-Chi
   or Tyvar is available. Reason: the loop needs no summoning sickness [E]. Tag **[primer: E]**.
8. **Two threats at the table.** Player A has 4 mana open and is one step from a combo; player B shows a stax piece. Action:
   counter A's combo spell, handle B's piece with Decay/Rift on its own turn. Reason: stop the imminent win first. Tag
   **[inference]** (the primers don't say).
9. **Godo cast on turn 3 by an opponent.** Sisay not in play, FoW with a blue card in hand. Action: counter Godo with Force of
   Will; Fierce Guardianship is dead here. Reason: Fierce can't counter creatures. Tag **[Oracle]**.
10. **Godo trigger on the stack.** Ertai in hand, 4 mana open. Action: flash Ertai, counter the enter trigger. Reason:
    stops Helm, the opponent draws a card. Tag **[inference from Oracle]**.
11. **Mount Doom finish.** Infinite mana, Derevi and Emiel present, opponents on 40, no Silence. Action: ping with Mount Doom
    untapped by blinks, fetch Deadpool as backup only if Mount Doom is missing. Reason: Mount Doom is faster than the Spirit plan
    (which waits for upkeep). Tag **[primer: A]**.

## Combo tests (become scripted tests)
1. **Bloom Tender / Derevi / Emiel.** Board: Sisay, Bloom Tender, Derevi, Emiel, Kinnan (five colours total among permanents);
   all untapped; Mount Doom in hand or on board. Opponents: three players at 40, no interaction. Expected: loop gains net mana;
   Mount Doom pings until all opponents are at 0.
2. **Cradle loop.** Board: Derevi, Emiel, Agatha's Soul Cauldron, Gaea's Cradle, four more creatures, Sisay. Expected: Cradle
   untaps via Derevi each loop; infinite G, converted for Sisay's activations; the engine fetches Mount Doom and wins.
3. **Marvin + Ioreth + Selvala.** Board: Marvin, Ioreth (no summoning sickness), Selvala, Mount Doom. Expected: infinite mana;
   Mount Doom (untapped by the Marvin/Ioreth loop) defeats all opponents.
   Variant: Marvin, Ioreth and Shang-Chi cast the same turn win the same turn (Shang-Chi's haste for abilities).
4. **Deadpool + Forbidden Orchard.** Board: infinite mana, Deadpool, Emiel, Forbidden Orchard. Expected: Spirits created,
   Deadpool blinked per Spirit; opponents lose 3 life per Spirit at their next upkeep.
5. **Sisay beats.** Board: Sisay with five colours of legends (7/7), no blockers. Expected: 7 damage per attack.

## Open questions
- The sources give no mulligan rule and no pod-politics rule. The "stop whom" choice is our inference.
- None of [A], [H], [G] says what the Cookout list actually won with; the [T] page has no notes. Lines 1-3 and 4-5 come from [A], the
  closest primer, but [A]'s card list was not checked against the Cookout list card by card (names match for the cards cited).
- Line 3 summoning-sickness and mana-type details (Shang-Chi mana restriction, Cryptolith Rite and Relic of Legends as sources)
  are our reading of the Oracle text, not the primer.
- [J] cuts Spider-Woman (can't stop Thassa's Oracle) and the Queen of Dale; both are in our list.
- How the "Kinnan" 96% inclusion rate in [EDHREC] is used is not explained in any source; Draftsim says Lotho works even at Sisay power 2.
- Whether the AI should treat Sisay's WUBRG cost as a hard colour check (it is, per the Oracle).
- Age: [H] (2024) and [G] (2025) pre-date Dockside Extortionist's ban [J]; their Nicol Bolas, Finale of Devastation and Oracle lines
  need cards that are not in our list and are not used here.

## Sources
- Archidekt, "Sisay It Ain't So" primer: https://archidekt.com/decks/24667571
- 賽の河原, "[cEDH] Odds and Ends for Sisay, Weatherlight Captain", 2026-08-19: https://note.com/unazuki_misstep/n/n6a2689bb81a6
- Veiss Versa, "An Introduction To Sisay, Weatherlight Captain in cEDH", 2024-04-22: https://commandersherald.com/an-introduction-to-sisay-in-cedh/
- Johnny Garcia, TheGamer, 2025-03-12: https://www.thegamer.com/magic-the-gathering-sisay-weatherlight-captain-cedh-deck-guide/
- Draftsim, 2026-08-20: https://draftsim.com/mtg-sisay-new-tech/
- docs.mdbook.me, 2026-10-16: https://docs.mdbook.me/mtg/cedh/cedh-sisay
- EDHREC combo: https://edhrec.com/combos/mono-blue/1432-5963 ; EDHREC cEDH page: https://edhrec.com/commanders/sisay-weatherlight-captain/cedh
- EDHTop16 list: https://edhtop16.com/tournament/the-cookout-2026/decklist/GHPNb0E7DnWMVIsJIhYSXre58rB3
- cEDH Decklist Database: https://cedh-decklist-database.com
- Scryfall (Oracle text): https://scryfall.com
