# Kinnan, Bonder Prodigy — Behaviour Spec (Tier 6)

Sources (all opened 2026-10-17; oracle text checked on Scryfall, S10). Spec written 2026-10-17.

| Id | Page | Date shown |
| --- | --- | --- |
| S1 | Learn cEDH, "Kinnan" (Evan Pierce) — https://learncedh.com/decklists/kinnan | "Mar 11", year not shown |
| S2 | EDHREC, "Building Kinnan, Bonder Prodigy for cEDH" (Harvey McGuinness) — https://edhrec.com/articles/building-kinnan-bonder-prodigy-for-cedh | May 19, 2026 |
| S3 | Commander's Herald, "An Introduction To Kinnan In cEDH" (Drake Sasser) — https://commandersherald.com/an-introduction-to-kinnan-in-cedh/ | Dec 8, 2023 |
| S4 | cEDH Decklist Database, entry "Kinnan Infinite Mana" — https://cedh-decklist-database.com | 2023-12-15 |
| S5 | Forest Birds Go, "From zero to seven mana in two seconds flat: Kinnan EDH" — https://forestbirdsgo.wordpress.com/2020/05/17/from-zero-to-seven-mana-in-two-secondkinnan-edh/ | 2020-05-17 (casual, old) |
| S6 | Commander Spellbook and EDHREC combo pages, "Kinnan + Basalt Monolith" — https://commanderspellbook.com/combo/129-4131/ , https://edhrec.com/combos/simic/129-4131 | none |
| S7 | EDHTop16, Kinnan page and the SIEGE cEDH 10K decklist (our list) — https://edhtop16.com/commander/Kinnan%2C%20Bonder%20Prodigy | 2026-06-13 event |
| S8 | EDHREC, Kinnan cEDH page (4,742 decks) — https://edhrec.com/commanders/kinnan-bonder-prodigy/cedh | Oct 2026 |
| S9 | Moxgate, "How to Play Kinnan" (a Bracket 3 list, different cards; low weight) — https://www.moxgate.com/guides/kinnan-bonder-prodigy-commander/ | Updated Mar 2026 |
| S10 | Scryfall oracle text, every card named below | n/a |

Not readable: Moxfield and TappedOut primers (HTTP 403), MTGNexus (403), r/CompetitiveEDH (redirect, no text). No source is a
full written primer for this exact 100. The closest list is the one in S2. Our list differs from every primer: it has **no
Thassa's Oracle, Isochron Scepter, Walking Ballista, Seedborn Muse, Nezahal, Crop Rotation, Cephalid Coliseum or Mana Drain**.
Claims below that need one of those cards are marked as unavailable.

## Game plan

Kinnan is a two-mana commander that doubles every mana dork and rock, so the deck makes 7 or more mana on turn 3 and
spends it on a creature "flip" engine or an infinite-mana combo **[primer S2, S3, S5]**. It wants Kinnan on the
battlefield by turn 2 and a win by about turn 3–5 **[primer S1: "turn 2 Kinnan, turn 3 artifact ramp"; S2: "four-plus
mana on turn two"; turn 3–5 target is an inference from those]**. Its stance is **hold up, then win**: it keeps counters up, uses
Kinnan's ability at end of turn to dig, and only commits when it has the combo plus protection **[primer S2: "hold up
mana for interaction, then ... end step value play"; S3]**. It is not a lock deck and it does not race with damage.

## Win lines

Kinnan's ability needs coloured mana ({5}{G}{U}). The loops below make colourless mana unless stated. Cards in **bold** are
in our list. All four lines need **Kinnan** on the battlefield.

| Line | Pieces (in our list) | Mana (total, colours) | Steps | Where it can be answered (step, by what) |
| --- | --- | --- | --- | --- |
| **A. Basalt Monolith loop** (main line) **[primer S1, S2, S6]** | **Kinnan**, **Basalt Monolith**, an outlet: **Thrasios, Triton Hero** (draw) and/or **Mirage Mirror** (filter to {U}{G}); finish with **Finale of Devastation** | Kinnan {G}{U} (2) + Monolith {3} + Thrasios {G}{U} (2) = 7 total over the game, 5 on the win turn if Kinnan is already out. Finale needs {G}{G} from real sources plus X from loop mana. With Mirage Mirror instead of Thrasios: 3 + {2} per activation, colourless only | 1. Cast Monolith. 2. Tap it: CCC + Kinnan's C = 4. 3. Pay {3}: untap. Net +1 {C}; repeat. 4. Outlet: Thrasios {4} per card (scry 1, reveal; land goes to the battlefield, else draw), or Mirror alternating a dual land and Monolith for infinite {U}{G} **[primer S2]**. 5. Draw the deck, cast every creature, cast Finale with X of 10 or more: creatures get +X/+X and haste **[primer S2: "cast every creature, then cast an infinitely-large Finale"]**. 6. Attack | Kinnan: removal or bounce before step 1 (any), in response to the Monolith loop (the loop needs Kinnan's trigger). Monolith: a counterspell on the cast (step 1), artifact removal, Null Rod or Collector Ouphe (stops the tap for mana) **[primer S1 lists Ouphe and Null Rod]**. Thrasios: removal or counter on the cast. Finale: counter on the cast (step 5), a fog or flash blockers or a sweeper in combat (step 6) **[inference]** |
| **B. Hullbreaker Horror loop** **[primer S2, S3]** | **Kinnan**, **Hullbreaker Horror**, two zero- or low-cost rocks that are mana-positive together: S2 shows **Mox Amber** and **Lotus Petal**. Others S1 names with Kinnan: **Sol Ring**, **Grim Monolith**, **Mana Vault**, **Mox Opal** (needs three artifacts) paired with **Fellwar Stone**, **Arcane Signet** or **Talisman of Curiosity**. Outlet and finish as in line A | Kinnan 2 + Horror {5}{U}{U} (7) = 9 total. The rocks are free (Amber, Petal). Loop mana is {U}{G} when Amber is used (Amber needs a legendary creature, Kinnan is one), so Kinnan's ability works at once **[inference from Amber's oracle]** | 1. With Horror out, tap Amber (2 mana with Kinnan). 2. Cast Petal (0): Horror trigger returns Amber. 3. Cast Amber (0): Horror trigger returns Petal. 4. Tap Amber again. Each cycle nets mana. 5. Spend on Kinnan's ability to find Thrasios, then as in line A **[primer S2]** | Horror cannot be countered (flash, so cast in response or at end of turn) **[oracle]**. Answers: removal on Horror before the loop (the trigger on the stack still resolves), a counter on any loop spell (Amber or Petal cast; the loop stops if one is countered), Null Rod / Ouphe on the rocks, bounce on Kinnan **[inference]**. |
| **C. Horror board lock** **[primer S2 calls Horror "a win condition that's almost impossible to remove"; the lock details are inference]** | **Kinnan**, **Hullbreaker Horror**, infinite mana from A or B | As A or B | With infinite mana and free spells, every cast bounces an opposing permanent or spell, so the table cannot keep a board. Then draw the deck, Finale or attack | Same as B. Needs a spell to cast each time, so a once-per-turn spell limit or a "can't cast spells" effect stops it **[inference]** |
| **D. Finite Finale / big turn** **[primer S3 "big flips"; S9 for a finite Finale]** | **Kinnan**, many rocks and dorks, **Finale of Devastation**, **Dramatic Reversal** (untaps all nonland permanents, including Basalt Monolith), **Twincast** (copies Reversal), **Hidden Strings** (untaps Monolith) | Finale with X of 10 needs 12 mana ({G}{G} real). Board in test D makes about 37 **[inference: 22 first tap, +17 after Reversal, -2 for it]** | Tap everything, Reversal, tap again, cast Finale for X, attack with the fetched creature pumped +X/+X and hasty | Counter on Reversal or Finale, removal on the fetched creature |

**The Dramatic Reversal combo in the primers needs Isochron Scepter, which is not in our list** **[primer S4, S5, S9; list check]**. In our list Reversal
is a one-shot ramp spell. It is not a stand-alone win. The pool file's note "Basalt Monolith, Dramatic Reversal" is
therefore only half right. Lead to decide (see Open questions).

**Win conditions that need cards we do not have:** Thassa's Oracle finish (S5), Cephalid Coliseum decking loop (S1), Walking Ballista (S3, S9),
Tidespout Tyrant (S1). Not available. **Faerie Mastermind** ("{3}{U}: each player draws a card") is in our list. With infinite mana it also
decks us, so it is only a win after Endurance resets our graveyard and we keep cards in the library **[inference; S1 uses it with Coliseum]**. Do not
model it as a win line.

**Tutors that find each piece (all in our list):**

| Piece | Tutors and routes |
| --- | --- |
| Basalt Monolith (MV 3) | **Transmute Artifact** (sacrifice an artifact, pay the difference), **Trophy Mage** (artifact of MV 3), **Drift of Phantasms** transmute ({1}{U}{U}, discard: a card of MV 3, sorcery speed) **[S2 names Transmute Artifact and Trophy Mage as the artifact tutors; Drift is inference from oracle]**. Copies: **Copy Artifact**, **Mirrormade**, **Clever Impersonator**, **Imposter Mech** on a Monolith already on the battlefield **[inference]** |
| Hullbreaker Horror (MV 7) | **Chord of Calling** (X=7, convoke), **Nature's Rhythm** (X=7), **Finale of Devastation** (X=7), **Invasion of Ikoria** (X creature fetch, non-Human), Kinnan's ability (top five) **[primer S3: "green creature tutors do a fine job"; X values inference]**. Copies: **Mockingbird**, **Clever Impersonator**, **Flesh Duplicate** **[inference]** |
| Thrasios (MV 2) | Chord (X=2), Nature's Rhythm (X=2), Finale (X=2), Invasion of Ikoria, Kinnan's ability **[primer S2]** |
| Mox Amber / Lotus Petal / Sol Ring | Drawn (Rhystic Study, Remora, Sylvan Library, The One Ring, Borne Upon a Wind). No tutor for the rocks except Transmute Artifact **[inference]** |

## Mulligan

- **Keep:** two lands (or a land and a fast mana source) plus one mana creature or rock, and Kinnan or a way to reach
  him. "All an opening hand needs is two lands and a mana dork or rock" **[primer S5]**. A turn-2 Kinnan is the ideal **[primer S1]**.
- **Mulligan aggressively:** look for ramp (Sol Ring, Birds of Paradise) and card advantage **[primer S1]**. Ship: no
  mana source, all lands, or a hand of expensive creatures with no ramp **[inference]**.
- **Typical keep:** land, land or Mox, Birds / Llanowar / Elvish Mystic, Kinnan, one cheap counter (Fierce Guardianship, Force of Will with
  a blue card to pitch, Pact). Elvish Spirit Guide and Lotus Petal give a one-shot first mana for turn-1 Kinnan **[inference]**.
- Moxgate (Bracket 3 list) says keep with Kinnan, a one-mana creature and a land that fixes both colours **[primer S9, low weight]**.
- No source gives a hard number of lands. Use the keep above.

## Turn by turn

**Turns 1–2.** Play the cheapest mana first: land, Mox/Petal, one-drop dork (Birds, Llanowar, Elvish Mystic, Fyndhorn Elves,
Delighted Halfling) or Sol Ring; then Kinnan **[primer S1, S2, S5]**. "A turn-one Kinnan is a pretty easy feat in cEDH" **[S2]**. Kinnan costs
two, so he is hard to punish: opponents must kill him twice before the deck pays a pricier commander tax **[primer S3]**. Rocks played after Kinnan
make double mana, so Kinnan goes first and the rocks follow **[inference: Kinnan's static ability]**. Keep one counter up when it costs nothing
(Pact, Fierce Guardianship once Kinnan is out, Force of Will) **[primer S3]**. Rhystic Study or Mystic Remora is a good turn-2 play with open mana **[primer S2]**.

**Turns 3–4.** With 7 or more mana, either flip with Kinnan at the end of the opponent's turn (Consecrated Sphinx, Hullbreaker Horror,
Thrasios, Wan Shi Tong) or cast Basalt Monolith / Horror if it can resolve **[primer S3: the activated ability is used "to dodge counterspells and
to ignore Rhystic Study effects"]**. Gene Pollinator, Springleaf Drum and Enduring Vitality turn creatures into extra Kinnan triggers
(Enduring Vitality gives every creature "{T}: Add one mana of any colour") **[inference from oracle]**. Find an outlet (Thrasios) before going for Monolith **[primer S3, S2]**.

**Later.** Go for line A or B as soon as protection allows. If the combo is not in reach, keep flipping at end of turn and draw with Sphinx,
The One Ring, Study and Remora; use Chain of Vapor, Boseiju and Otawara on stax pieces **[primer S1: Chain of Vapor and similar answers remove stax pieces]**.

## Interaction

**What it holds (our list):**

| Type | Cards |
| --- | --- |
| Free counters | Force of Will, Force of Negation (opponent's turn only), Fierce Guardianship (needs Kinnan on the battlefield), Misdirection, Mental Misstep (MV 1 only), Disrupting Shoal (pitch a blue card with MV X), Commandeer (exile two blue cards), Mindbreak Trap (after three opposing spells), Pact of Negation (pay {3}{U}{U} next upkeep) |
| One-mana counters | Swan Song (enchantment, instant, sorcery only; gives a 2/2 flier), An Offer You Can't Refuse (noncreature, gives two Treasures), Flusterstorm (instant/sorcery), Dispel (instant only) |
| Bounce | Snapback (free with a blue card), Chain of Vapor, Sink into Stupor, Otawara (channel), Colossal Skyturtle (channel), Hullbreaker Horror (bounces on every spell it casts) |
| Other removal | Boseiju (artifact, enchantment, nonbasic land; costs {G} with Kinnan out), Volatile Stormdrake (exchange for a creature; it dies unless 4 energy covers its MV), Dizzy Spell (-3/-0) |
| Engines that tax | Rhystic Study, Mystic Remora, Sylvan Library |

**What it spends each on [inference from oracle text unless noted]:**
- Creature spells: Force of Will, Pact, Misdirection (single target only), Mental Misstep (MV 1), Disrupting Shoal. Fierce, Force of Negation, Offer, Commandeer cannot hit creatures.
- Noncreature artifacts and enchantments (stax, rocks): Fierce Guardianship, Force of Negation, Offer. Swan Song hits enchantments but not artifacts.
- Instants and sorceries (rituals, tutors, wheels): Flusterstorm, Dispel, Swan Song, Mindbreak Trap, Pact.
- A permanent already on the battlefield: Chain of Vapor, Sink into Stupor, Snapback (creatures), Otawara, Boseiju.

**When it lets something resolve.** It lets a spell resolve when it cannot win the game and the spell does not stop its own plan **[inference]**. It
expects to spend free counters on combos and stax, not on value spells **[primer S3: "interaction for every kind of win and plenty of free interaction
to get involved in stopping win attempts after resolving a value engine"]**. Rhystic Study and Remora collect from spells it lets pass **[primer S2]**.

**Who to stop when two players threaten to win [inference; no source covers this].** Stop the player who wins first in turn order and stack
position. A second free counter handles the next one. If it cannot stop both, it goes for its own win with whatever protection it has left. It does
not use its only counter on a value spell when the table has a combo deck in play.

## When it goes for the win

All conditions are **[inference]** unless tagged; the primers say only to protect the combo with counters **[S3, S9]** and "play around stax" **[S1]**.

| Go now | Wait |
| --- | --- |
| Kinnan on the battlefield and Basalt Monolith / Horror castable this turn, with one counter (Fierce, Force of Will, Pact) held for each opposing blue mana that is open | No counter held and an opponent has two or more blue mana open (cast Kinnan or a lesser spell first as bait) |
| An outlet (Thrasios) is on the battlefield or in hand with {G}{U} spare | Collector Ouphe, Null Rod, Stony Silence or a similar stax piece is on the table and not yet removed **[primer S1]** |
| Both the loop and Finale ({G}{G} real sources) are reachable in the same turn | Opponent shows a fog, a sweeper or a lot of flash blockers (Finale is a sorcery, the attack is the last step) |
| After a resolved Monolith loop, draw and keep at least one card in the library (Thrasios reveals lands onto the battlefield and can deck us) | Pact is the only counter and the next upkeep cannot pay {3}{U}{U} |

It uses Hullbreaker Horror instead of Monolith when the table is holding artifact hate or a counter for artifacts: Horror cannot be countered **[oracle]**.

## What it fears

| Threat | Why | How it plays around it |
| --- | --- | --- |
| Null Rod, Collector Ouphe, Stony Silence, Drannith Magistrate **[primer S1]** | Turns off Moxen, rocks and Monolith | Boseiju, Chain of Vapor, Otawara, Sink into Stupor on the piece; counter it on the cast with Fierce Guardianship or Force of Negation **[primer S1 names Chain of Vapor as an answer]** |
| Artifact sweepers and creature sweepers | Dies with its dorks and rocks **[primer S1: "reliance on nonland mana sources"]** | Holds creatures back, keeps a counter up, keeps Kinnan in the command zone as a rebuy (commander tax) **[inference]** |
| Removal on Kinnan before the loop | No Kinnan, no loop | Cast a mana creature first so the board is not all Kinnan; recast for 2 (+2 per death) **[primer S3]** |
| Stifle effects on Horror's trigger | Stops the loop trigger | Loop with Monolith (no trigger) instead |
| Graveyard hate | Hits Coliseum loops, which we do not run **[primer S1; list check]** | No effect on our lines |
| Fast combo decks that win before turn 4 | Its own win comes on turn 3-5 | Free counters and Rhystic Study; goes first when it can **[inference]** |

## Against Godo

Godo (mono-red, {5}{R}) fetches Helm of the Host when he enters, and Helm makes a hasty token Godo at each combat; the backup is Dualcaster Mage plus a
copy spell **[from decklists/JD/godo-cedh-equipment.md]**. Kinnan's answers, by step **[all inference from oracle text]**:

| Godo step | Answer in Kinnan's list | Notes |
| --- | --- | --- |
| Cast Godo (a creature spell, {5}{R}) | Force of Will, Pact of Negation, Disrupting Shoal (X=6: pitch Consecrated Sphinx), Hullbreaker Horror trigger (return the spell) | Fierce Guardianship, Force of Negation, Offer and Commandeer cannot target a creature spell. Mental Misstep only hits MV 1 spells (Sol Ring, Mana Vault; not the MV 0 Moxen) |
| Godo's enter trigger | Nothing stops it. It fetches Helm | Counters on abilities are not in the list |
| Helm enters (Hammer of Nazahn attaches it free) | Chain of Vapor or Snapback on Godo with the attach trigger on the stack; Boseiju ({G} with Kinnan) on Helm | Best window. Without Godo on the battlefield, Helm has nothing to copy |
| Equip {5} on the stack | Snapback or Chain on Godo | The ability fizzles if Godo leaves |
| Beginning of combat (Helm trigger) | Bounce Godo with Sink into Stupor or Chain; Horror bounce on each spell cast | Tokens are copies; bounced tokens cease to exist |
| Dualcaster Mage line: copy spell (Twinflame etc.) | Fierce Guardianship, Force of Negation, Dispel (instant), Swan Song (sorcery), Flusterstorm, Offer, Mindbreak Trap | Counter the copy spell before Dualcaster is flashed in |
| Dualcaster Mage (creature spell, MV 3) | Force of Will, Pact, Disrupting Shoal (X=3) | Chain / Snapback after it enters |

Kinnan beats Godo on speed only if it has a counter and the table has no earlier win: Kinnan's win turn (about turn 3–5) is no faster than Godo's (turn 2–3)
**[inference]**, so Kinnan must hold a creature-capable counter (Force of Will, Pact) on turns 2–3. **For the Godo spec:** Godo should play
around Force of Will and Pact on Godo, bounce on Godo with the trigger on the stack, and Hullbreaker Horror (uncounterable) bouncing his tokens.
Most of Kinnan.s counters (Fierce Guardianship, Force of Negation, Offer) cannot hit Godo, so Godo should expect only Force of Will, Pact, bounce and Horror.

## Decision scenarios (become AI tests)

| # | Position | Right action | Reason | Tag |
| --- | --- | --- | --- | --- |
| 1 | Turn 2, you hold Island, Forest, Birds of Paradise (on the battlefield), Kinnan, Force of Will + a blue card. Opponents show two untapped blue mana | Cast Kinnan, keep Force of Will up for the next spell | Kinnan is cheap and survives one removal; the engine matters more than the counter-bait | [primer S3] |
| 2 | Turn 3, Kinnan out, 8 mana, hand: Basalt Monolith, Thrasios, Fierce Guardianship, no creature counter. Opponent has three open blue mana and probably Force of Will | Do not cast Monolith yet; flip Kinnan at the end of their turn, or cast Thrasios first as bait | Monolith is a noncreature spell: Fierce Guardianship protects it only once; a bait spell makes them spend the counter | [inference] |
| 3 | Opponent casts Null Rod while you hold Fierce Guardianship, Force of Negation, 3 mana | Counter it with Fierce Guardianship (free) | The stax hits your rocks and Monolith; Fierce is free and Kinnan is out | [primer S1: stax is the main weakness] |
| 4 | Opponent resolves Collector Ouphe; you have Boseiju in hand, Kinnan out, {G} open | Channel Boseiju ({G} with Kinnan) on the Ouphe | The channel costs one mana with a legendary creature | [inference from oracle] |
| 5 | Your end step, 8 mana open, no combo ready, Rhystic Study in play, opponents tapped low | Activate Kinnan ({5}{G}{U}); take Hullbreaker Horror if rocks are on the battlefield, otherwise Sphinx / Thrasios | Counters cannot hit the ability; top five at end step is free value | [primer S3] |
| 6 | Kinnan out, Hullbreaker Horror on the battlefield, Mox Amber and Lotus Petal in hand, opponents have no stax and 2 blue mana open | Start the loop only while holding a free counter or one spare zero-cost spell. Cast Lotus Petal first, then Amber. If they counter a loop spell, cast the spare spell: Horror returns their counter to hand | A counter on a loop spell ends the loop. Horror bounces a spell on every cast, so a spare cast answers it | [primer S2 for the loop; the backup is inference] |
| 7 | Two opponents threaten wins: Player A needs one more spell to win, Player B has a Rhystic-style engine | Counter Player A's winning spell with the best free counter; ignore B | The one closest to winning is stopped first | [inference] |
| 8 | Basalt Monolith resolved, Kinnan out, Thrasios in play, 4 colourless spare, library 40 cards | Run the loop, activate Thrasios until about 5 cards remain, keep one card in library, then cast creatures and Finale for X | Thrasios puts lands onto the battlefield and draws; Finale also searches the graveyard | [primer S2; library count inference] |
| 9 | You cast Pact of Negation on a combo piece; you have 6 lands next turn | Fine: cast it | Next upkeep {3}{U}{U} is 5 mana | [inference] |
| 10 | Opponent casts Godo (six mana, the Helm trigger on the stack after Godo enters); you hold Snapback, Chain of Vapor, 3 mana | Wait until Helm is attached (or in response to the attach or equip ability) and bounce Godo | Helm has nothing to copy; bounce is cheaper than a counter | [inference] |
| 11 | Mulligan: Island, Forest, Birds, Elvish Mystic, Kinnan, Force of Will, Consecrated Sphinx | Keep | Two lands, two dorks, Kinnan, a free counter | [primer S5] |
| 12 | Mulligan: 5 lands, Sylvan Library, Rhystic Study, Consecrated Sphinx | Mulligan | No acceleration and no Kinnan | [primer S1] |

## Combo tests (become scripted tests)

| Line | Starting board and hand | Expected result |
| --- | --- | --- |
| A. Basalt Monolith + Thrasios | Battlefield: Kinnan, Thrasios, Island, Forest, Breeding Pool, Sol Ring, Birds of Paradise (all untapped). Hand: Basalt Monolith, Finale of Devastation. Library has creatures and lands. Opponents: no interaction | Cast Monolith (3). Tap: 4 colourless; pay 3; repeat. Net +1 per loop. Thrasios drawn to about 5 cards left. Finale with X of 10 or more: creatures +X/+X and haste. Attack with the fetched creature and any flier. Result: all three opponents dead |
| B. Hullbreaker Horror + Mox Amber + Lotus Petal | Battlefield: Kinnan, Hullbreaker Horror, Mox Amber, Thrasios, three lands. Hand: Lotus Petal, Finale of Devastation | Cast Petal: Horror returns Amber. Cast Amber: Horror returns Petal. Each loop: Amber taps for 2 (Kinnan). Net positive in {U}{G}. Use Kinnan or Thrasios; Finale; win as in A |
| C. Horror lock (no combat) | As B, but opponents control three permanents each, two cheap spells in hand | Each spell cast bounces an opposing permanent; after enough casts every opposing nonland permanent is in hand. The test checks that no opposing permanent remains |
| D. Finite big turn | Battlefield: Kinnan, Birds, Llanowar Elves, Elvish Mystic, Sol Ring, Grim Monolith, Mana Vault, 5 lands. Hand: Dramatic Reversal, Finale of Devastation | Tap everything, cast Reversal, tap again. Expected: about 37 mana (22 on the first tap, 17 more after Reversal, 2 spent on it), enough for Finale with X of 10 or more. Test checks the mana count and that the AI casts Reversal after tapping, not before |
| Negative | A with an opposing Null Rod on the battlefield | Monolith cannot tap for mana; the AI does not cast it as a win attempt and instead removes Null Rod (Chain, Boseiju, Otawara) or waits |

## Open questions

1. **Dramatic Reversal.** The pool file says Kinnan wins "through Basalt Monolith, Dramatic Reversal". Every primer's Reversal combo uses Isochron Scepter, which our list does not
   have. Lead decides: model Reversal as ramp only (this spec), or treat the primer note as a mistake.
2. **Emrakul, the Promised End.** It is in the tournament list but in none of the primers. Its cast trigger gives control of the opponent's next turn plus an extra
   turn. No source says how the pilot uses it. This spec treats it as an unplayed 13-drop; a Kinnan flip does not trigger the cast ability.
3. **Real win condition without Thassa's Oracle.** S2 says the win is Thrasios plus a huge Finale and an attack; no source says what the pilot did when an opponent
   held a fog or flying blockers. The deck has no stated back-up finisher beyond a Horror lock.
4. **Thrasios is the only listed card-draw outlet in our list** (plus Mirage Mirror filter). If Thrasios is removed, line A has no outlet beyond Kinnan's own flip, which
   needs coloured mana (Mirage Mirror gives it). The primers assume more outlets (Walking Ballista, Staff of Domination).
5. **Time to win.** Sources say "turn 2 Kinnan" and "four-plus mana on turn two", not when the combo is expected. The turn 3–5 target is inferred.
6. **Interaction priority.** No source covers which counter goes on which spell or how to pick between two threats. Scenarios 2, 7 and 10 are inferred.
7. **Pact of Negation.** The deck should only cast it when it can pay or wins; no source says so.
