# Kraum & Tymna (Blue Farm) — Behaviour Spec (Tier 6)

Sources (all opened 2026-10-17):

- **[LC]** Learn cEDH, "Tymna Kraum" by Evan Pierce, dated "Dec 2" (no year shown): https://learncedh.com/decklists/tymna-kraum
- **[EH]** EDHREC, "Building Blue Farm for cEDH With Kraum and Tymna", Harvey McGuinness, 2026-01-27: https://edhrec.com/articles/building-blue-farm-for-cedh-with-kraum-and-tymna
- **[DS]** Draftsim, "Blue Farm Competitive Commander Deck Guide", David Royale, 2025-07-18 (decklist: Jase Sanders' 1st place at "Shine Bright Like a Lion's Eye", June 2025): https://draftsim.com/blue-farm-edh-deck/
- **[DB]** cEDH Decklist Database, entry "Kraum Tymna Breach" (data-updated 2023-03-20): https://cedh-decklist-database.com
- **[Scry]** Scryfall Oracle text for every card named below (fetched 2026-10-17).
- Our list: `decklists/pool/staging-t6-cedh/kraum-tymna-blue-breach-midrange.md` (Breach the Bay 2, #2 of 429, 2026-08-22).
- Godo: `decklists/JD/godo-cedh-equipment.md` on `origin/godo-cedh`.

No r/CompetitiveEDH thread could be read (Reddit blocked the fetch; a search found none on this topic). Moxfield primers
("Blue Farm" by Tremnek, "Good Soup" by FreedomWaffle, "KT Razorcake") are linked from [LC] but return 403, so were not read.
Spec written 2026-10-17.

Tags: **[primer]** = a source above says it. **[inference]** = my reasoning from the Oracle text or the list. Where the
sources are silent I say so (see Open questions).

## Game plan

Blue Farm is a four-colour (WUBR) midrange-turbo deck. It lands a card-draw engine early (Rhystic Study, Mystic Remora,
Esper Sentinel, Tymna, Kraum) and holds up a deep stack of free and cheap counters, then picks a window and goes off in
one turn **[primer: LC, EH, DS]**. It usually aims to close between turns 3 and 5 **[primer: LC]**. Its stance is "hold
up, then race": it plays the long game with counters and engines, and turns turbo when it has 7+ mana (a hand that can
make that on turn 1–2 is a turbo keep) or when the table is tapped down **[primer: LC]**. It is often the table's
archenemy **[primer: LC]**. Nearly every game ends with a Thassa's Oracle trigger on an empty library **[primer: EH]**.

## Win lines

| Line | Pieces (in our list) | Mana (total, colours) | Steps | Where it can be answered |
| --- | --- | --- | --- | --- |
| 1. **Oracle + Consultation / Pact** (main) [primer: LC, EH, DS] | Thassa's Oracle + Demonic Consultation (or Tainted Pact) | Consultation line: 3 total, {U}{U}{B}. Pact line: 4 total, {U}{U}{1}{B}. | 1. Cast Oracle. 2. With its ETB trigger on the stack, cast Consultation naming a card not in the library (or Pact). 3. Library is exiled. 4. Trigger resolves: X (devotion) >= 0 cards left, win. | Step 1: counter Oracle (a creature spell, so only Force of Will, Pact of Negation and similar hit it). Step 2: counter Consultation/Pact (that leaves a full library, safe, unless Oracle was countered after: see below). Step 4: with Kraum or Esper Sentinel out, an opponent's spell can make it draw from an empty library; kill/bounce is not enough to stop the trigger [primer: LC]. |
| 2. **Underworld Breach loop** [primer: LC, EH, DS] | Underworld Breach, Lion's Eye Diamond, Brain Freeze; finish with Thassa's Oracle from the graveyard, or Brain Freeze on opponents | From empty: Breach 2 (needs {R}+{1}); LED gives {U}{U}{U}; escape Brain Freeze {1}{U}. So 2 mana from lands/rocks, 1 red among them, then the loop is mana-positive [inference]. | 1. Cast Breach. 2. Cast LED, sacrifice it for UUU (discards hand). 3. Escape Brain Freeze on self (storm copies; [LC/EH] say at least 6 cards milled). 4. Escape LED, crack again. 5. Repeat until the library is empty. 6. Escape Oracle (UU + exile 3), trigger wins. Alternative: aim the Brain Freeze copies at opponents once storm is high [primer: EH]. | Step 1: counter Breach. Step 2/4: counter LED when it is cast (escape casts are spells). Any step: counter the Brain Freeze cast (a counter on the original stops its copies). Graveyard hate in response. Breach is sacrificed at the end step, so a long stall wastes it [Scry]. |
| 3. **Necropotence + extra turn / flash** [primer: LC] | Necropotence + Final Fortune or Borne Upon a Wind or Valley Floodcaller; then line 1 or 2 | Necropotence {B}{B}{B} (Dark Ritual + one {B} source does it) [inference]; plus Borne {1}{U} or Final Fortune {R}{R} or Floodcaller {2}{U} held up. | 1. Cast Necropotence. 2. Pay life, exile many cards face down. 3. At the end step they enter the hand [Scry]. 4. Cast the enabler at instant speed (Borne lets everything be cast at flash speed) and win in the same end step or the extra turn. [LC] says 7–15 cards when mana is short, 30+ when an enabler is already paid for. | Counter Necropotence (3 black pips are hard to redo). Counter the enabler. Final Fortune loses the game at the end of the extra turn [Scry], so it only goes when the extra turn wins. |
| 4. **Ad Nauseam** [primer: LC, DS, DB] | Ad Nauseam; finds the pieces of line 1 or 2 | {3}{B}{B} = 5 total, {B}{B} + 3 any; Dark Ritual makes it cheap [inference]. | 1. Cast Ad Nauseam (instant). 2. Reveal until it has fast mana, the Oracle/Consultation pair or the Breach pieces. 3. Cast those with the mana found. Life: 40 start, Tymna also costs life [LC: "life total pressure"]. | Counter Ad Nauseam. Life total is the limiter, not an answer; a burn/drain opponent punishes it. |
| 5. **Mnemonic Betrayal** (backup) [primer: LC, DS] | Mnemonic Betrayal; opponents' graveyards hold a win | 3 total, {1}{U}{B}. | Exile opponents' graveyards, cast what is in them with any mana type. | Counter Betrayal. Needs a good graveyard; the sources do not say how often it wins. |
| 6. **Combat** (fallback) [primer: LC] | Kraum (4/4 flying haste), Orcish Bowmasters, small creatures with Tymna | Kraum 5 ({3}{U}{R}), Tymna 3 ({1}{W}{B}). | Attack with evasive creatures, draw off Tymna, grind. | Normal removal, blockers. |

Also in the list and named by sources as extras: Flash Photography (a copy of Oracle) [inference, not in any source],
Exhibition Tidecaller (mills three on each instant or sorcery) [inference, not in any source]. Our list has no Dockside
Extortionist, Wheel of Fortune, Gifts Ungiven, Cyclonic Rift, Swords to Plowshares, Pyroblast, Birgi or Archivist of
Oghma, which [DS]'s list (June 2025) has: our June-to-August 2026 list is slightly different. It adds Necropotence,
Intuition, Final Fortune, Culling the Weak, Voice of Victory, Commandeer, Snapback, Into the Flood Maw, Flash Photography
and Gleaming Splendor.

**Tutors that find each piece (in our list).**

- Oracle: Demonic Tutor, Vampiric Tutor, Imperial Seal, Gamble, Wishclaw Talisman, Beseech the Mirror, Intuition,
  Diabolic Intent (sacrifice a creature) [primer for the tutors in general: LC, DS; the target mapping is inference].
  Ranger-Captain of Eos fetches only mana value 1 or less, so not Oracle (MV 2) [Scry].
- Consultation / Pact / Brain Freeze / Ad Nauseam / Final Fortune / Borne Upon a Wind: Mystical Tutor (instant or sorcery),
  plus the any-card tutors above.
- Breach, LED, Necropotence, Rhystic Study: Enlightened Tutor (artifact or enchantment), plus the any-card tutors.
- Intuition puts the two cards the opponent does not choose into the graveyard, which feeds Breach [Scry; use is inference].

## Mulligan

- **Keep** hands with fast mana, early interaction, or a draw engine **[primer: LC]**. Specifically: a turn-1/2 engine
  (Rhystic Study, Mystic Remora, Esper Sentinel), or a "turbo hand" that can make 7+ mana for an early Ad Nauseam or
  Breach try, or interaction plus fast mana (Force of Will, Silence, Mox Diamond, Chrome Mox) **[primer: LC]**.
- "Mulligan aggressively for hands that enable turn 1 or 2 development" **[primer: LC]**.
- **Ship** hands with no mana development, no engine and no interaction; all-land hands; hands whose only plays are
  uncastable in our colours (e.g. only Breach and Brain Freeze with no red and no LED) **[inference]**.
- A typical keep: 2–3 lands (at least one fixer for U and B), a Mox or Sol Ring or Lotus Petal, one engine or tutor, and
  one counter (a free one such as Force of Will, Fierce Guardianship or Mental Misstep counts) **[inference from LC]**.
- The sources give no counts for lands or a pitch-count rule for Force of Will. See Open questions.

## Turn by turn

The sources describe phases, not exact turns **[primer: LC, DS]**. The ordering below is mostly inference.

- **Turns 1–2.** Fast mana first (Sol Ring, Moxen, Lotus Petal, Mana Vault; Gemstone Caverns as a turn-0 land on the
  draw) [primer: DS]. Then a cheap engine: Esper Sentinel, Rhystic Study, Mystic Remora, or Tymna/Kraum [primer: LC, DS].
  Early creatures that connect for Tymna: Bowmasters, Sentinel, Ragavan [primer: LC]. Keep a free counter in hand.
  Do not tap out for non-engines.
- **Turns 3–4.** Smothering Tithe, the second engine, a commander if it is cheap enough to cast and still hold a counter
  [inference]. Start tutoring for the win pieces with Demonic Tutor/Mystical Tutor while holding Flusterstorm and
  Fierce Guardianship [primer: DS]. A turbo hand can try Ad Nauseam or Breach on turn 2–3 [primer: LC "turbo hands"].
- **Later.** Pick the window (see below). Until then keep drawing off Rhystic/Remora/Kraum/Tymna, hold counters, and
  answer the opposing threat that is closest to winning [primer: DS "patience and timing"].

## Interaction

**What it holds** (all in our list):

| Kind | Cards | Cost |
| --- | --- | --- |
| Free counters (any spell) | Force of Will (exile a blue card + 1 life; or 3UU), Pact of Negation (pay 3UU next upkeep or lose) | pitch / deferred |
| Free counters (noncreature) | Fierce Guardianship (needs a commander on the battlefield), Force of Negation (only on someone else's turn), An Offer You Can't Refuse (U, gives two Treasures) | free or 1 |
| Cheap counters | Mental Misstep (MV 1 only), Flusterstorm (instant/sorcery, storm), Swan Song (enchantment/instant/sorcery, gives a 2/2 flyer), Dispel (instant), Mindbreak Trap (free after three opposing spells in a turn), Red Elemental Blast (blue spell/permanent) | U / R |
| Redirect/steal | Misdirection, Commandeer, Deflecting Swat (free with a commander) | mostly free |
| Bounce | Chain of Vapor, Into the Flood Maw, Snapback | U or free |
| Turn off the table | Silence, Orim's Chant (kicked: no attacks), Grand Abolisher, Voice of Victory, Ranger-Captain of Eos (sac: no noncreature spells) | W / WW / sac |
| Draw taxes | Rhystic Study, Mystic Remora, Esper Sentinel, Smothering Tithe, Gleaming Splendor, Faerie Mastermind | static |

Counter count [Scry]: Force of Will and Pact are the only counters in the list that hit creature spells (Mental Misstep
hits a creature only if it costs 1; Mindbreak Trap hits anything; Misdirection and Commandeer need a target they fit).
[inference]

**What it spends each on [primer: DS, EH, LC unless marked].**

- Free counters protect its own combo; the deck is built to "protect key pieces with counterspells or Silence effects" [LC].
- Silence/Orim's Chant/Grand Abolisher/Voice of Victory are for the turn it goes off, not for defence [LC, EH, DS].
- Cheap counters (Flusterstorm, Swan Song, Dispel, An Offer, Mental Misstep, REB) are spent on opposing fast mana, tutors,
  engines and combo pieces [DS lists them as one-mana answers to critical spells; the targeting priority is inference].
- Red Elemental Blast is for blue spells and permanents [DS], so it matters against another blue deck. [Scry]
- Bounce (Chain, Flood Maw, Snapback) answers a threatening creature or permanent without a counter [DS: Chain bounces
  "pesky permanents"]. Note Chain of Vapor lets the target's controller sacrifice a land to copy it [Scry].

**When it lets something resolve.** The sources do not give a rule. Reasonable rule **[inference]**: let a spell resolve
if it does not advance a win or hurt the engine (a ramp spell it can tax with Rhystic/Remora, a creature that is not a
threat); counter spells that tutor for a combo, fast mana when the opponent is the turn's likely winner, and anything
that stops its own win. Keep at least one free counter in hand once a win line is assembled.

**Who to stop when two players threaten to win.** The sources are silent. [inference]: stop the player closest to
winning this turn (a combo with its second piece already on the stack beats a developing board), then the player with the
most untapped mana, then the highest-threat deck. Do not spend the last free counter on a non-lethal spell if its own
win is one turn away.

## When it goes for the win

[primer: LC] "Look for windows where opponents are tapped out or lack resources to respond." [primer: LC, DS] "Use
redundancy ... adapt your line"; Silence effects and counters protect the attempt. [primer: LC] Hands that make 7+ mana
early are turbo hands and may go on turn 2–3.

Conditions as an AI rule **[inference from the above]**:

| Go | Wait |
| --- | --- |
| Opponents tapped out, or the deck holds Silence/Orim's Chant/Abolisher/Voice of Victory (cast first) | Two or more opponents have untapped blue mana and the deck holds no protection |
| At least one free counter in hand besides the combo pieces (Force of Will, Pact, Fierce Guardianship) | The combo uses the last counter and a creature spell is needed (Oracle is a creature: only Force of Will / Pact / a Silence effect protects it) |
| Library not already below the point of no return (Consultation/Pact) | A Kraum or Esper Sentinel on its own side with an opponent that has a second spell ready (empty-library draw) [LC] |
| Ad Nauseam: life >= ~20 and mana >= 5, with the finishing piece likely in the top cards | Life low (Tymna payment, Misstep, Force of Will) |
| Necropotence: an enabler (Borne/Final Fortune/Floodcaller) can be paid in the same turn | Final Fortune where the extra turn does not win |

## What it fears

| Fear | Source | How it plays around it |
| --- | --- | --- |
| Creature counters on Oracle (Force of Will, Pact) | [LC] If you cast Consultation first and Oracle is countered you lose next turn with no library. | Cast Oracle first; keep Consultation for the trigger. Cast the tutor first only if the table lacks creature counters [LC]. |
| Kraum or Esper Sentinel on its own board while it combos | [LC] opponents can cast spells to force an empty-library draw. | Remove or do not deploy before comboing, or leave cards in library with Tainted Pact [LC]. |
| Stax and graveyard hate (Rest in Peace etc.) | [LC] resilient to stax thanks to draw engines; graveyard hate follows from Breach [inference]. | Win with Oracle + Consultation instead of Breach. |
| Free/cheap counters from other blue decks, and Pyroblast/REB | [EH] it is the counter-heavy deck itself; REB/Pyroblast hit its blue cards [inference] | Silence effects, redundancy. |
| Life pressure | [LC cons] Tymna and Ad Nauseam | Pay little life, sequence Ad Nauseam late. |
| Being the archenemy | [LC] often the pod's target | Keep the board small; hold counters. |
| Blood Moon / Magus of the Moon | 25 nonbasic lands and 0 basics in our list [inference from the list]. The sources do not mention it. | Counter it (Force of Will, Swan Song, REB cannot: red). |
| Bounce/removal on engines | [inference] | Counters, Silence effects. |

## Against Godo

Godo's win is a chain of triggers (Godo's ETB fetches Helm of the Host, Helm makes a hasty token copy at the beginning of
combat, each token's first attack adds a combat), and the backup is Dualcaster Mage + a copy spell **[Godo notes]**.
Oracle text checked [Scry]. Everything below is **[inference]** unless noted (no source covers this matchup).

| Godo step | Answer in this deck | Notes |
| --- | --- | --- |
| Cast Godo ({5}{R}, a creature spell) | **Force of Will**, **Pact of Negation**. Mindbreak Trap if three or more spells were cast. | Fierce Guardianship, Force of Negation, Swan Song, Dispel, An Offer, Flusterstorm cannot counter a creature spell. This is the cleanest stop. |
| Godo ETB trigger fetches Helm | Nothing counters the trigger. Deflecting Swat can redirect an ability, but the trigger has no target. | Helm is put onto the battlefield, not cast, so noncreature counters miss it. |
| Attach Helm (equip {5}, Hammer, Squire or Magnetic Theft) | Equip is an ability: no counter. Chain of Vapor / Into the Flood Maw bounce Helm or Godo in response to equip. | Bouncing Godo with equip on the stack fizzles it. |
| Beginning of combat: Helm trigger | Bounce Godo (Chain of Vapor, Into the Flood Maw, Snapback) with the trigger on the stack: Helm then has no equipped creature to copy [Godo notes: "Helm then has nothing to copy"]. Godo returns to hand and can be recast, so counter the recast. | Silence/Orim's Chant do nothing: the loop uses no spells. |
| Token attacks, extra combat | Chain/Flood Maw/Snapback on tokens; Orim's Chant **kicked** stops attacks for the turn | Orim's Chant kicked is cast on Godo's turn before combat and needs {W}{W}; the Godo player may respond in kind [inference]. |
| Dualcaster Mage ({1}{R}{R}, flash) + copy spell | Counter Dualcaster (Force of Will, Pact); counter the copy spell (Swan Song, Dispel if instant, Force of Negation, An Offer, Fierce Guardianship, Flusterstorm, Mental Misstep if MV 1); countering the original copy spell stops the chain | Many answers; the deck is strongest here. |

Also relevant: Godo's list runs Red Elemental Blast and Pyroblast (hit this deck's blue cards), Blood Moon / Magus of the
Moon (this deck has 25 nonbasic lands), and Trinisphere [Godo notes]. The deck should keep Force of Will for Godo itself.
Silence effects do not stop the Helm loop, so this deck must hold a creature counter in the turn it expects Godo (six
mana on the table, Helm or Hammer fetched earlier) **[inference]**.

## Decision scenarios (become AI tests)

1. **Position:** T2, 2 lands + Mox Diamond in hand, Rhystic Study and Esper Sentinel in hand, Force of Will, opponents untapped. **Action:** play Esper Sentinel first (W), keep Rhystic for when mana is free. **Why:** the cheap engine develops without exposing Study to a Force of Will you cannot answer. [inference from LC "turn 1/2 engines"]. Tag: [inference].
2. **Position:** Opponent taps out for a 5-mana ramp creature; you hold Rhystic Study, Fierce Guardianship, Force of Will. **Action:** resolve Rhystic (they cannot answer); keep the counters. **Why:** engine first when they are tapped; counters are for combo [LC "windows"]. Tag: [inference, from LC "windows"].
3. **Position:** Opponent casts a tutor (Demonic Tutor equivalent) with 1 blue mana open on your side; you hold Mental Misstep, Flusterstorm, Fierce Guardianship. **Action:** Mental Misstep only if the tutor is MV 1; else Flusterstorm. **Why:** cheapest sufficient counter, save Fierce. Tag: [inference].
4. **Position:** You have 8 mana, Oracle + Consultation in hand, opponents have 2 untapped Islands, you hold Force of Will and Silence. **Action:** cast Silence first, then Oracle, then Consultation in response to the trigger. **Why:** Silence stops all responses this turn [Scry]; the line wins. Tag: [primer: LC protect with Silence, DS].
5. **Position:** Same, but the opponent shows Force of Will-style blue deck and you have only one pitch counter. **Action:** wait a turn or build to two protections. **Why:** if Oracle is countered after Consultation you lose; the order of Oracle then Consultation avoids this but Oracle can still be countered [LC]. Tag: [primer: LC].
6. **Position:** Kraum is on your battlefield, library about to be emptied, opponents have untapped mana and cards. **Action:** do not combo; first remove Kraum or use Tainted Pact to leave cards. **Why:** any opponent second spell lets you draw from an empty library [LC]. Tag: [primer: LC].
7. **Position:** Breach in play, LED in hand, Brain Freeze in graveyard with 4+ other cards, a red source already used for Breach. **Action:** run the loop; when storm >= opponent library size / 3, target opponents instead of self, otherwise empty yourself and escape Oracle. **Why:** both end the game [EH]. Tag: [primer: EH, DS].
8. **Position:** Necropotence resolved, 10 life left, Borne Upon a Wind in hand with {1}{U} up, opponents untapped. **Action:** exile a few more cards only if the enabler is castable at end step; otherwise stop. **Why:** [LC] 1-turn line needs the enabler paid; going lower with no enabler loses. Tag: [primer: LC].
9. **Position:** Two players threaten wins: one has a combo piece on the stack with no backup, another has 7 untapped mana and a tutor chain. **Action:** counter the tutor chain's key spell with the cheap counter; leave the slower engine. **Why:** stop the nearest win [inference]. Tag: [inference].
10. **Position:** You have Final Fortune and a hand that wins in the extra turn but only 1 counter. **Action:** hold Final Fortune; no extra turn unless the win is guaranteed. **Why:** you lose the game at that end step [Scry]. Tag: [primer: LC, Scry].
11. **Position:** Opponent has Godo with Helm out and a token about to attack, you hold Chain of Vapor, Silence. **Action:** bounce Helm or Godo before combat; Silence does nothing. **Why:** loop uses no spells. Tag: [inference].
12. **Position:** Opponent casts Godo with six mana; you hold Force of Will (blue card to pitch) and Fierce Guardianship (commander on battlefield). **Action:** Force of Will on Godo. **Why:** Fierce cannot counter a creature [Scry]. Tag: [inference].

## Combo tests (become scripted tests)

1. **Oracle + Consultation.** Board: 3 untapped lands (U, U, B available), empty graveyard, 99-card library not containing Oracle. Hand: Thassa's Oracle, Demonic Consultation. Expected: cast Oracle, respond to the trigger with Consultation naming a card not in the library, the library is exiled, the trigger resolves, the player wins. Variant: Tainted Pact with {U}{U}{1}{B}, which exiles the whole library when no duplicates remain [EH].
2. **Oracle + Pact, Oracle countered.** Same, opponent Force of Wills Oracle. Expected: if Consultation is already cast, the player has no library and loses at their next draw; the AI must not cast Consultation first without protection [LC].
3. **Breach loop.** Board: Underworld Breach cast from hand with 2 lands (R + 1 other), hand LED + Brain Freeze + Oracle, graveyard 4+ cards, library ~60. Expected: LED sacrificed for UUU, Brain Freeze escaped on self (mills 3 x copies, [EH] at least 6), LED and Brain Freeze loop until the library is empty, Oracle escapes and wins.
4. **Necropotence + Borne.** Board: Necropotence in play, 3 lands with {1}{U} untapped after, 20 life, hand empty. Expected: pay 12 life, end step hand refills, cast Borne Upon a Wind, then cast the combo pieces at instant speed and win via line 1 [LC].
5. **Ad Nauseam.** Board: 5+ mana with BB, 40 life, library arranged so the top cards reach Oracle + Consultation. Expected: cast Ad Nauseam, stop before life reaches 0, cast the revealed pair with remaining mana, win. Failure case: stopping too late loses [LC].
6. **Mnemonic Betrayal.** Board: opponent's graveyard holds a game-ending spell, 3 mana {1}{U}{B}. Expected: Betrayal exiles the graveyards, the player casts the spell with any mana type. [primer: LC, DS]; the sources do not say a specific target.
7. **Combat.** Board: Kraum + Tymna, opponent 20 life, no blockers. Expected: Kraum attacks for 4 flying, Tymna triggers draw one card (pay 1 life) in postcombat main.

## Open questions

- No primer gave a land count or a Force of Will keep rule; the Moxfield primers could not be read (403).
- [LC] has no year; it is likely 2024 and may predate our list's Necropotence, Final Fortune and Voice of Victory (the Necropotence section is in [LC], so it does cover Necro).
- None of the sources gives a rule for choosing who to counter, or when to let a spell resolve. The spec above is inference.
- How often the Breach line vs the Oracle line is used is not stated. [EH] says "nearly every game" ends with an Oracle trigger.
- Whether the AI should model Esper Sentinel/Kraum empty-library risk needs a decision (the engine needs the "draw from empty library = lose" check).
- Fierce Guardianship needs a commander on the battlefield: the sources do not say when the deck casts a commander to enable it; [inference] when the commander costs less than the counter it unlocks.
- Mana-base notes: no source covers Blood Moon, but 25 nonbasic lands and no basics makes it a real weakness against Godo's list.
