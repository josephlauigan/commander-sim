# Rograkh & Thrasios (Green Creature Combo) — Behaviour Spec (Tier 6)

Sources (all opened 2026-10-17):

- **[EDH]** "Building RogThras (Rograkh and Thrasios) for cEDH", Harvey McGuinness, EDHREC, 2026-05-05. https://edhrec.com/articles/building-rogthras-rograkh-and-thrasios-for-cedh. Its sample list is nearly our list (same Cradle build). Read in full (curl).
- **[EDM]** "Competitive Temur: Rog and Thras, the BEST Pairing for cEDH", jegpeg, EDHMeta, 2026-07-11. https://edhmeta.com/competitive-temur-rog-and-thras-the-best-pairing-for-cedh/. Read through WebFetch, so it is a tool summary, not verbatim text.
- **[SB24]** "An Introduction to Rograkh's Bouncy House (Rog/Thrasios) in cEDH", Sam Black, Commander's Herald, 2024-04-18. https://commandersherald.com/an-introduction-to-rograkhs-bouncy-house-rog-thras-in-cedh. WebFetch summary (curl gets HTTP 406). It describes a different build (Cloudstone Curio / Dockside / Imperial Recruiter); only its general advice is used.
- **[SB23]** "Rograkh, Son of Rohgahh—Thrasios, Triton Hero cEDH Primer", Sam Black, Hipsters of the Coast, 2023-08-18. https://www.hipstersofthecoast.com/2023/08/rograkh-son-of-rohgahh-thrasios-triton-hero-cedh-primer/. Read in full. An older midrange build; used for the Thrasios, Eternal Witness + Snap + Cradle, and counterspell rationale.
- **[DB]** cedh-decklist-database.com entry "Rograkh Thrasios Cradlestorm+" (2026-02-16, 'competitive'): one paragraph, links only to a Moxfield list (HTTP 403, not readable).
- **[T16]** EDHTop16 decklist page, Zach Shepherd, Pittsburgh 2026 t/cEDH $10k-$25,000, 1st of 195, 2026-07-11 (the list we use). It gives a list only, no text.
- Oracle text checked against Scryfall (`/cards/collection`, 75 names in one call) for every card named below.

Spec written 2026-10-17.

**Limits of the sources.** No primer says anything specific about mulligans, turn-by-turn play or when to go, for *this* list. [SB24] gives general mulligan rules for a related build. Everything on timing below is tagged **[inference]** unless it cites a source. Our list does **not** contain Retraction Helix, Banishing Knack, Underworld Breach, Brain Freeze, Lion's Eye Diamond, Cloudstone Curio, Dockside Extortionist, Imperial Recruiter or Seedborn Muse, so several primer lines are not available (see Open questions).

## Game plan

The deck makes a huge amount of mana from Gaea's Cradle, creatures and untap effects, then converts it into a win. **[primer: EDH, EDM, SB24]** The main plan is infinite mana, and every named loop here needs Gaea's Cradle. With infinite mana, Thrasios draws the library, then the deck casts Finale of Devastation for X of 10 or more (hasty, huge team) or loops Faerie Mastermind with Green Sun's Zenith. **[primer: EDH, EDM]** It does not aim at a fixed turn; EDH says it is for players who "don't want to risk it all on closing the game by turn two". It plays a midrange-to-turbo game: Rograkh (cost {0}) comes down turn 1, free counters (Fierce Guardianship, Deflecting Swat, Force of Will) back it up, and it wins when a loop is assembled and protected. Stance: **hold up and race**: develop and keep free interaction, then go. **[primer: EDH, SB23, SB24; the stance is [inference]]**

## Win lines

All three lines need a **Gaea's Cradle** and several creatures. Commanders in play count as creatures. Net mana per loop is small (+1) but unbounded, and everything is a creature or land action, so it is not stopped by counterspells once running.

| Line | Pieces (in our list) | Mana (total, colours) | Steps | Where it can be answered |
|---|---|---|---|---|
| **A. Cradle + Breezecaller + Talon Gates** [primer: EDH, EDM] | Gaea's Cradle, Oboro Breezecaller ({1}{U}), Talon Gates of Madara (in hand), 6 creatures in play (Rograkh, Thrasios, Breezecaller count) | Loop needs no outside mana once started: tap Cradle for 6 {G}, spend 4 + 2, Gates gives {C}: net +1. Setup from an empty board **[inference]**: Breezecaller {1}{U} + Thrasios {G}{U} + about 3 more creature mana, so about 7 mana and both U and G | 1 Tap Cradle (6+ {G}). 2 Pay {4}: Talon Gates from hand to battlefield (ETB phase-out is optional). 3 Tap Gates ({C}). 4 Pay {2}, return Gates to hand: untap Cradle. 5 Repeat. Net +1. Filter colours with Gates' {1},{T} | Counter Breezecaller on cast (Force of Will etc. by the opponent). In the loop: kill or bounce Breezecaller or a creature so Cradle drops below 6 (net negative), or destroy Cradle in response to the untap (Boseiju/Wasteland-type effects). Stifle effects on the Breezecaller activation. Gates' {4} is an ability, so spell counters miss it |
| **B. Cradle + Eternal Witness + Snap** [primer: EDH, EDM, SB23] | Gaea's Cradle, Eternal Witness ({1}{G}{G}), Snap ({1}{U}), an Island or other land giving {U}; 5 other creatures counting commanders, or Cradle must tap for 5+ | Per cycle: cast Witness (3) + Snap (2) = 5 for {G}{G}{U} + 2. Cradle (5 {G}) + Island ({U}) = 6, then both untap: net +1. Setup **[inference]**: about 5 mana to start, plus the creatures | 1 Tap Cradle and Island. 2 Cast Witness (ETB: take anything, or Snap if it is in the graveyard). 3 Cast Snap on Witness: Witness to hand, untap Cradle and Island. 4 Tap them again, recast Witness (ETB returns Snap), recast Snap. Repeat. [EDH] says the Witness is not counted in the 5 creatures because it is bounced | Counter Witness or Snap on cast (any counter; Snap is an instant so Fierce Guardianship misses it but Force of Negation (noncreature) and Swan Song (instant) hit it). Exile Snap in the graveyard in response to Witness's ETB. Remove a creature so Cradle makes under 5 |
| **C. Mana to win** (finishers after A or B) [primer: EDH, EDM] | Thrasios ({4}: scry 1, reveal top, draw if nonland, land to battlefield tapped), Finale of Devastation, Faerie Mastermind ({3}{U}: each player draws), Green Sun's Zenith, Hexing Squelcher, Veil of Summer | Infinite colourless/green. Finale: {X}{G}{G} with X of 10 or more | 1 Pay {4} per Thrasios activation until the library is nearly gone (stop with at least 1 card: with an empty library the "otherwise draw" would hit an empty library **[inference]**). 2 Cast Hexing Squelcher (spells can't be countered) and Veil of Summer. 3 **Finale of Devastation** with X of 10 or more: creatures get +X/+X and haste, attack. **Or** Faerie Mastermind + GSZ: activate Mastermind to make everyone draw; cast GSZ each time so it shuffles back and you never deck first [EDM, EDH] | Respond to Thrasios activations with Louisoix's-type ability counters (opponent's). Wrath or fog in response to the attack. Counterspell on Finale (Hexing Squelcher/Veil prevent this). Removal on the attackers; opponent's instant-speed removal or a blocker with first strike |
| **D. Finite engine (not a win by itself)** [primer: EDM; rest [inference]] | Earthcraft + Wild Growth on a basic, Valley Floodcaller, Candelabra of Tawnos, Magus of the Candelabra, Formidable Speaker, Minamo, Deserted Temple, Training Grounds, Biomancer's Familiar | Large but finite mana | [EDM] says the Floodcaller + Earthcraft line "is not infinite" and wants two Floodcallers. Untap effects multiply Cradle: each of Candelabra, Magus, Speaker, Minamo (legendary), Deserted Temple can untap Cradle once. Training Grounds and Familiar cut Thrasios to {1} per activation [EDM], so a big finite pool draws many cards | Same as A |

**Tutors that find each piece** (Scryfall text checked).

| Piece | Found by |
|---|---|
| Gaea's Cradle | Crop Rotation ([primer: EDH, EDM], best), Expedition Map, Sowing Mycospawn (cast trigger puts a land onto the battlefield), Vibrance (GG: land to hand), Gamble |
| Talon Gates of Madara | Same land tutors [EDH, EDM] |
| Oboro Breezecaller (MV 2) | Chord of Calling (X=2), Nature's Rhythm (X=2), Finale of Devastation (X=2), Invasion of Ikoria (non-Human, X=2), Formidable Speaker (ETB, discard a card) |
| Eternal Witness (MV 3, green, Human) | Green Sun's Zenith (X=3), Chord (X=3), Nature's Rhythm (X=3), Finale (X=3), Formidable Speaker. **Not** Invasion of Ikoria (Witness is Human) |
| Snap (MV 2) | Spellseeker (instant or sorcery, MV 2 or less), Gamble, Eternal Witness from the graveyard, Noxious Revival (puts it on top of the library) |
| Finale of Devastation | Gamble; Spellseeker no (MV 2 or less only) |
| Faerie Mastermind / Green Sun's Zenith | Chord (X=2), Nature's Rhythm, Finale, Speaker for Mastermind; Gamble for GSZ |

The tutor roles are **[inference]** from the Oracle text. [SB24] and [EDM] confirm that Crop Rotation and the creature tutors are the deck's way to find Cradle and combo creatures.

## Mulligan

- **Keep:** at least two lands and a piece of acceleration; keep hands with Crop Rotation or another way to get Gaea's Cradle; look for mana and card-flow engines, not one specific combo. **[primer: SB24, general rules for a related build]**
- **Ship:** no acceleration; two or more purely reactive spells unless they can be used proactively or the hand has a clear win. **[primer: SB24]**
- **Typical keep** **[inference]**: 2-3 lands (one a fetch or dual), Rograkh in the command zone (free), one of Sol Ring, Mox Amber, Springleaf Drum, Birds or Elvish Spirit Guide, plus a free counter (Fierce Guardianship, Force of Will, Deflecting Swat) or Crop Rotation.

## Turn by turn

No source gives a turn-by-turn plan for this list. Everything here is **[inference]** except what is cited.

- **Turns 1-2:** Cast Rograkh for {0} (turn 1) **[primer: EDH, "guaranteed to be usable on turn one"]**. This turns on Mox Amber (needs a legendary creature), Fierce Guardianship and Deflecting Swat (free with a commander), and Springleaf Drum. Play fast mana (Sol Ring, Mana Vault, Moxen, Lotus Petal) and a mana creature (Birds, Badgermole Cub, Gene Pollinator, The Cabbage Merchant). Cast Rhystic Study or Mystic Remora if the table is not blue-heavy. Hold mana for a free counter.
- **Turns 3-4:** Cast Thrasios (it is a creature for Cradle), play Gaea's Cradle (find it with Crop Rotation) and add creatures: Chatterstorm for tokens, Formidable Speaker (discard a card, tutor a creature), clones. Go for line A or B when six (A) or five (B) creatures plus the pieces are in play and the opponent shows little. Use Candelabra of Tawnos to untap Cradle once per turn cycle **[primer: SB24, EDH]**.
- **Later:** If no loop, Thrasios becomes a card engine: Training Grounds and Biomancer's Familiar cut its cost, so activate every turn; keep drawing toward Crop Rotation, Witness, Snap, Breezecaller **[primer: SB23, EDH, EDM]**.

## Interaction

**Holds** (all in our list): free or cheap counters — Fierce Guardianship (free with a commander; noncreature spells), Force of Will, Force of Negation (free on opponents' turns; noncreature), Misdirection, Mindbreak Trap (free after the opponent casts 3 spells), Flare of Denial (sacrifice a nontoken blue creature), Pact of Negation, Swan Song, Flusterstorm, Mental Misstep, Commandeer (steal a noncreature spell, exile two blue cards), Louisoix's Sacrifice (counters an activated or triggered ability or a noncreature spell; sacrifice a legendary creature or pay {2}). Redirect and copy: Deflecting Swat (free with a commander), Flare of Duplication (sacrifice a nontoken red creature; Rograkh and Hexing Squelcher are red). Protection: Veil of Summer, Hexing Squelcher (its spells can't be countered), Allosaurus Shepherd (green spells can't be countered). Bounce and removal: Chain of Vapor, Snap, Otawara (channel), Boseiju (channel, destroys artifact, enchantment or nonbasic land).

**Use.** **[primer: SB23, SB24; the allocation is [inference]]**: The deck wants counters that protect its own plays and that win the stack on the opponent's turn. [SB24] says it does not want to trade two cards for one and does not want to "police the table". So it spends free counters on (1) a spell that would stop its own loop or attack (a sweeper, a graveyard hate piece, a game-winning combo), (2) a combo piece of an opponent who is about to win. Cheap pings (Mental Misstep, Flusterstorm) are used on cheap rocks and fast mana. Rhystic Study and Mystic Remora are left to tax, not protected. It lets resolve: a spell that does not threaten its board or its own combo and a game that is not yet at a win.

**Two threats.** **[inference]** Stop the opponent who can win next turn or this turn first; with equal threats stop the one it cannot beat in a later turn (the faster deck). If it has a line available itself, it prefers to use the counter to protect its own win rather than stop a threat it can beat in a race.

## When it goes for the win

**[inference]**, from [SB24] ("empty your hand", redundancy) and [EDH].

| Go now when | Wait when |
|---|---|
| The pieces are in play or in hand and the loop starts with enough creatures (6 for A; 5 for B plus Island) | Fewer than the needed creatures, or Cradle missing |
| It holds at least one free counter or Veil of Summer / Hexing Squelcher in play for the key step | The opponent has 2+ untapped blue mana or is holding up a free counter and the deck has no protection |
| The loop is already running (A and B are board actions and cannot be countered once started) | A sweeper or removal is likely and the creatures are exposed |
| An opponent is about to win and the loop wins first | The table has a Null Rod-like piece only for non-creature lines (not relevant to A and B) |

[SB24] says the deck accepts that combos get disrupted and relies on redundancy (three lines here) instead of Defense Grid-style protection.

## What it fears

- **Creature removal and sweepers** (Orcish Bowmasters, Toxic Deluge, Massacre Girl-type effects): every line needs creatures; [SB23] says "a removal spell can stop any of them" and notes sweeper exposure. Plays around it by keeping lines redundant, holding Veil / Hexing Squelcher, and not overcommitting before a loop.
- **Land removal / Cradle hate** (Wasteland, Strip Mine, Boseiju-style effects): both A and B need Cradle. Plays around it with Crop Rotation as a second source and other mana lines. **[inference]**
- **Graveyard hate** (Rest in Peace, Bojuka Bog): kills Witness returning Snap (B). **[inference]**
- **Stax on creatures or mana** (Null Rod does not hit this; Torpor Orb, Hushbringer, Collector Ouphe hits Springleaf Drum, Mox Amber, Mana Vault). **[inference]** [SB23] says Thrasios plays well against slow stax as long as it can still activate.
- **Fast combo decks.** Weak card advantage and a poor fit for one-for-one trading mean it cannot interact forever **[primer: SB24]**.
- **Hard to pilot / many close decisions** **[primer: SB24]**.

## Against Godo

**Godo + Helm of the Host (from the ETB).** Answers in our list, by step:

| Step | Answer |
|---|---|
| Godo ({5}{R}) on the stack | Force of Will, Pact of Negation, Flare of Denial, Mindbreak Trap (counter creature spells). Fierce Guardianship, Force of Negation and Swan Song cannot (not noncreature/instant/sorcery). Misdirection can't (Godo has no target) |
| Godo's ETB trigger | **Louisoix's Sacrifice** counters the trigger (so no Helm). Deflecting Swat can redirect the trigger but Godo's search has no target to change |
| Godo on the battlefield, Helm fetched | Bounce Godo in response to the Helm trigger: Chain of Vapor, Snap, Otawara channel. Helm then has no creature **[inference: depends on the rules for Helm with no equipped creature]** |
| Helm of the Host | Boseiju, Who Endures (channel, costs less with Rograkh/Thrasios out), destroy it in response to equip |
| Equip ({5} / Hammer / Squire) | Louisoix's Sacrifice counters the equip ability |

**Dualcaster Mage + a copy spell.** Counter Dualcaster Mage on cast (Force of Will, Pact, Flare of Denial, Mindbreak Trap). Counter the copy spell (Fierce Guardianship, Force of Negation, Swan Song, Commandeer). Louisoix's Sacrifice counters Dualcaster's ETB trigger. Deflecting Swat or Misdirection change a single target. Dualcaster is a combo of a creature and a spell, so it needs two answers only if the first fails. **[inference]**

**Godo against this deck (for the Godo spec).** **[inference]** The deck has about 10 ways to stop Godo's creature or trigger at 0-2 mana. Godo's best plays: cast on a turn the opponent is tapped out or has spent its free counters; bait with a cheap noncreature spell; hold Pyroblast or Red Elemental Blast (counter a blue spell, destroy a blue permanent) to beat Force of Will, Pact of Negation, Flare of Denial, Misdirection and to kill the blue pieces (Thrasios, Breezecaller, Mastermind are blue). Godo should win the race: the deck's loops need Cradle plus 5-6 creatures and so usually land on turn 3 or later, while Godo aims for turn 2-3. Play around Louisoix's Sacrifice (keep a second Helm carrier: Jaxis, Cursed Mirror) and Boseiju on Helm.

## Decision scenarios (become AI tests)

| # | Position | Right action | Reason | Tag |
|---|---|---|---|---|
| 1 | Turn 1 on the play. Hand: Mox Amber, Birds, Forest, Crop Rotation, Fierce Guardianship. Rograkh in the command zone | Cast Rograkh ({0}), then Mox Amber, then land and Birds | Rograkh turns on Mox Amber and free Guardianship; a turn 1 start gives the fast mana line | [primer: EDH, SB24] |
| 2 | Turn 4. Board: Rograkh, Thrasios, Birds, Dryad Arbor, Cradle (6 creatures needed: has 4 + Breezecaller in hand), 5 mana open, opponent has 2 Islands untapped and a Force of Will showing in hand count | Hold Breezecaller; cast a creature (Chatterstorm for tokens) to bait first | Needs six creatures and protection; the opponent has counter mana | [inference] |
| 3 | Own board: Cradle, Breezecaller, Talon Gates in hand, 6 creatures, opponent has no untapped blue mana, cards in hand: Veil of Summer | Start line A: tap Cradle, pay {4} to put Gates in, tap Gates, {2} bounce Gates to untap Cradle, repeat, then Thrasios | The loop is board actions; no open counters | [primer: EDH, EDM] |
| 4 | Own turn, Witness and Snap in hand, Cradle with 5 other creatures, Island. Opponent has Force of Will and 3 open blue mana | Wait, or cast a bait spell first; with Veil of Summer or Hexing Squelcher in play, go | Witness or Snap may be countered. Veil / Squelcher stop counters | [primer: EDH; the wait is [inference]] |
| 5 | Opponent casts a sweeper (Toxic Deluge) with the deck holding Fierce Guardianship | Counter it with Fierce Guardianship (free with a commander) | A sweeper kills all lines | [inference] |
| 6 | Opponent has Cyclonic Rift and 7 mana on their turn; the deck has a loop planned for next turn | Counter Rift (Force of Negation free, or Fierce Guardianship) if it targets the key piece; else let it resolve | Spend free counters to protect key pieces; avoid trading 2-for-1 | [primer: SB24; inference] |
| 7 | Two opponents threaten: A has an Oracle-style combo ready next turn, B has a slow engine. Deck has one free counter and no combo yet | Counter A's combo piece | Stop the threat that wins first | [inference] |
| 8 | Opponent casts Godo (6 mana). The deck has Louisoix's Sacrifice, Force of Will, 2 other cards | Let Godo resolve, then counter the ETB trigger with Louisoix's Sacrifice (or Force of Will on Godo itself if the trigger is not answerable) | The ETB trigger is Godo's only way to fetch Helm; countering the trigger costs {U} + {2} or a legendary creature | [inference] |
| 9 | Infinite mana reached, library 40 cards, Thrasios, Training Grounds | Activate Thrasios until 1-2 cards remain, then cast Hexing Squelcher and Veil of Summer, then Finale with X of 10+ | Draw the deck, then protect the win | [primer: EDH, EDM; stop point [inference]] |
| 10 | Mulligan: 7 cards, 1 land, Sol Ring, Mox Amber, Rograkh in command zone, 3 spells | Keep | Fast mana and a free commander replace lands; a 1-lander without fast mana is a ship | [primer: SB24 (keep rule); inference] |
| 11 | Mulligan: 7 cards, 2 lands, Force of Will, Pact of Negation, Fierce Guardianship, Mental Misstep, 1 other spell | Ship | Four reactive spells and no acceleration | [primer: SB24] |

## Combo tests (become scripted tests)

| Line | Starting board and hand | Expected result |
|---|---|---|
| A | Board: Rograkh, Thrasios, Birds, Dryad Arbor, Oboro Breezecaller, Gaea's Cradle, Island (6 creatures). Hand: Talon Gates of Madara. Mana open: none | Tap Cradle for 6; Gates in ({4}), tap Gates (+1 {C}), Breezecaller ({2}, bounce Gates) untaps Cradle. Repeat 5 times: each loop leaves +1 mana. Floating mana grows without bound |
| B | Board: Rograkh, Thrasios, Birds, Dryad Arbor, a fifth creature, Gaea's Cradle (taps for 5), Island. Hand: Eternal Witness, Snap. Graveyard empty | First cycle: Witness, Snap on Witness; second: Witness returns Snap; mana +1 per cycle. After 3 cycles the mana floating is at least 3 above start |
| C | Infinite mana, Thrasios and Training Grounds, library of N cards, Finale and Hexing Squelcher in hand/library | Activate Thrasios until library = 1; draw Finale; cast Squelcher; Finale X=10+; attackers get haste and +X/+X. Opponent with no answer loses the same turn |
| D (finite) | Board: Cradle (6 creatures), Candelabra, Magus of the Candelabra, Minamo, 5 mana open | Cradle taps 3 times (with Candelabra X=1 and Magus X=1): about 18 mana for 2 mana. Not infinite |

## Open questions

- No source describes this exact list's mulligan or timing; every timing rule above is an inference. The lead should decide whether to accept inferences for the AI plan or keep it minimal.
- The primer lines Valley Floodcaller + rock + Retraction Helix / Banishing Knack and Brain Freeze / Underworld Breach are **not** in our list. Floodcaller has no partner here. [EDM] still lists Brain Freeze and Underworld Breach.
- Main primer win lines for related builds (Cloudstone Curio + Dockside, Kinnan + Monolith) need cards absent from our list. They are not used.
- Is Rograkh a valid Flare of Duplication / Flare of Denial sacrifice (red nontoken creature)? Yes by Oracle text; no source says the pilot does it.
- Whether the Breezecaller loop also works with Talon Gates' ETB used to phase out an opposing blocker for the attack: [EDH] says the ETB is optional; no source says it is used for the finish.
- Whether Helm of the Host's trigger does anything if the equipped creature is bounced in response — rules ruling not checked.
- Thrasios on an empty library: the exact lose condition was not checked against the Comprehensive Rules.
- Dramatic Reversal, Louisoix's Sacrifice, Noxious Revival, Hidden Strings, Flare of Duplication, Mockingbird and the other clones have no stated role in any source.
