# Rograkh & Thrasios (Green Creature Combo) — Behaviour Spec (Tier 6)

Sources (all opened 2026-10-17; [AD] ranks first):

- **[EDH]** "Building RogThras (Rograkh and Thrasios) for cEDH", Harvey McGuinness, EDHREC, 2026-05-05. https://edhrec.com/articles/building-rogthras-rograkh-and-thrasios-for-cedh. Its sample list is nearly our list (same Cradle build). Read in full (curl).
- **[EDM]** "Competitive Temur: Rog and Thras, the BEST Pairing for cEDH", jegpeg, EDHMeta, 2026-07-11. https://edhmeta.com/competitive-temur-rog-and-thras-the-best-pairing-for-cedh/. Read through WebFetch, so it is a tool summary, not verbatim text.
- **[SB24]** "An Introduction to Rograkh's Bouncy House (Rog/Thrasios) in cEDH", Sam Black, Commander's Herald, 2024-04-18. https://commandersherald.com/an-introduction-to-rograkhs-bouncy-house-rog-thras-in-cedh. WebFetch summary (curl gets HTTP 406). It describes a different build (Cloudstone Curio / Dockside / Imperial Recruiter); only its general advice is used.
- **[SB23]** "Rograkh, Son of Rohgahh—Thrasios, Triton Hero cEDH Primer", Sam Black, Hipsters of the Coast, 2023-08-18. https://www.hipstersofthecoast.com/2023/08/rograkh-son-of-rohgahh-thrasios-triton-hero-cedh-primer/. Read in full. An older midrange build; used for the Thrasios, Eternal Witness + Snap + Cradle, and counterspell rationale.
- **[DB]** cedh-decklist-database.com entry "Rograkh Thrasios Cradlestorm+" (2026-02-16, 'competitive'): one paragraph, links only to a Moxfield list (HTTP 403, not readable).
- **[T16]** EDHTop16 decklist page, Zach Shepherd, Pittsburgh 2026 t/cEDH $10k-$25,000, 1st of 195, 2026-07-11 (the list we use). It gives a list only, no text.
- **[AD]** "Thras-and-Silent Rog", AnthonyD3288, Moxfield primer, https://moxfield.com/decks/BNWjUa_lA0mzxNpgpHDGSA/primer, changelog to 2026-10-12. The dedicated primer for this commander pair and the newest source. Moxfield blocks automated reads, so this spec uses the extract JD supplied (his printout, 52 pages): `primers/rograkh-thrasios-anthonyd.md` (page numbers there are the PDF's). The author's own list differs from ours, so every card is checked against our list below. Where [AD] and an older source disagree, [AD] wins.
- Oracle text checked against Scryfall (`/cards/collection`, one POST) for every card named below; two cards are marked where the text was not rechecked.

Spec written 2026-10-17.

Tags: **[primer: AnthonyD]** = [AD] says it. Other **[primer: X]** tags are older sources that [AD] does not contradict. **[inference]** = my reasoning from the Oracle text.

**Limits of the sources.** [AD] gives win lines, tutors, weaknesses and a few play rules, but no mulligan advice, no turn-by-turn plan and no timing rules for this list. Timing below is **[inference]** unless cited. Cards the primers use that are **not in our list**: Underworld Breach, Brain Freeze, Lion's Eye Diamond, Izzet Guildmage, This Town Ain't Big Enough, Displacer Kitten, Marwyn, Six, Flash Photography, Retraction Helix, Banishing Knack, Cloudstone Curio, Dockside Extortionist, Imperial Recruiter, Seedborn Muse, Kinnan, Orcish Bowmasters (re-checked against the import list, 0 hits each). So the Breach line, the Guildmage line, the "Exodia" line and every This Town line are **not in our list** (see Win lines).

## Game plan

The deck makes a huge amount of mana, then spends it with Thrasios to draw the library and finish. **[primer: AnthonyD]** Goal: make infinite mana, put it into Thrasios ({4}: scry 1, reveal, land onto the battlefield tapped or draw), and win; "Infinite mana = win." Thrasios sits in the command zone and works at instant speed, so with infinite mana it can answer any spell by digging for a counter or protection. **[primer: AnthonyD]** In our list every infinite line needs Gaea's Cradle (or two Badgermole Cubs), so the early job is to raise the Cradle count: cheap creatures and token makers early are "a main priority". **[primer: AnthonyD]** Finish is Finale of Devastation (X of 10 or more) or Faerie Mastermind with Green Sun's Zenith. **[primer: AnthonyD]**

It has no fixed turn; EDH says it is not built to win on turn two. **[primer: EDH]** Rograkh ({0}) comes down turn 1 and two cheap commanders turn on Fierce Guardianship, Deflecting Swat and Flare of Duplication early. **[primer: AnthonyD]** AnthonyD calls the deck a top-3 deck as of April 2026. Stance: **hold up and race**: develop and keep free interaction, then go when the loop is protected. **[inference]**

## Win lines

Commanders in play count as creatures for Cradle. Loops net about +1 mana per cycle but are unbounded.

| Line | Pieces (in our list) | Mana (total, colours) | Steps | Where it can be answered (step, by what) |
|---|---|---|---|---|
| **A. Oboro Breezecaller + Gaea's Cradle + Talon Gates of Madara** [primer: AnthonyD] (his Plan A) | Gaea's Cradle, Oboro Breezecaller ({1}{U}), Talon Gates of Madara (on the battlefield or in hand), Cradle tapping for 6+. All in our list | Loop is mana-positive by 1 with Cradle at 6: Cradle 6 + Gates 1 = 7 in, {2} + {4} = 6 out [Oracle]. Setup from an empty board **[inference]**: Oboro {1}{U} (U needed), plus the creatures, plus {4} if Gates is still in hand: about 7 mana in all | 1 Tap Cradle and Gates. 2 {2}, return Gates (a land) to hand with Oboro: untap Cradle. 3 {4}: put Gates from hand onto the battlefield untapped (an ability, not a spell). 4 Tap both again. Each Gates entry phases out one target creature (up to one), so the loop phases out every opposing creature. Gates filters any colour for {1}. [AD] says it works at instant speed if Oboro is already out, casts **no spells**, and beats one-spell-per-turn stax and Rhystic Study. With an extra {2} the Oboro activation can be put on the stack again over interaction. At exactly 5 creatures the loop is mana-neutral **[inference]** but still phases the table out ("even five is enough") [primer: AnthonyD] | Oboro on cast (counterspell). Kill or bounce Oboro or a creature before Cradle falls under 6. Destroy Cradle in response to the untap. Cursed Totem shuts off Oboro. Rule of Law, Damping Sphere and similar do not stop it (no spells). Grafdigger's Cage does not stop it (Oboro already out) [primer: AnthonyD] |
| **B. Eternal Witness + Snap + Gaea's Cradle** [primer: AnthonyD, EDH, EDM, SB23] | Cradle tapping for 5+, Eternal Witness ({1}{G}{G}), Snap ({1}{U}), a land giving {U}. All in our list | Per cycle: Witness 3 + Snap 2 = 5 out; Cradle 5 + one {U} land = 6 in; Snap untaps the lands: +1. Setup about 5 mana plus the creatures **[inference]** | 1 Tap Cradle and the {U} land. 2 Cast Witness. 3 Snap Witness: untap up to two lands. 4 Witness returns Snap; repeat: infinite {G}, infinite ETBs. Witness is bounced, so it does not count toward Cradle **[primer: EDH]** | Counter Witness or Snap. Exile Snap in response to Witness's ETB. Remove a creature so Cradle makes under 5. Rule of Law-type effects break it (it casts spells) [primer: AnthonyD]. Cage and Runestone do not touch it [inference: Snap is an instant and Witness enters from the stack] |
| **C. Badgermole Cub + Mockingbird (any clone) with Oboro + Cradle** [primer: AnthonyD] | Badgermole Cub, Mockingbird (or Clever Impersonator, Phantasmal Image, Flesh Duplicate), Oboro Breezecaller, Gaea's Cradle, a land giving {U}. All in our list | Not counted in [AD]. Needs an exact count I have not verified | Cast Cub: earthbend 1 makes a {U} land a 0/0 haste creature-land. Mockingbird copies the creature-land (the copy is only a land). Oboro returns the Mockingbird-land to hand; recast it each loop. Cub adds {G} whenever a creature is tapped for mana. Not instant speed unless the clone has flash [primer: AnthonyD] | Counter Cub or the clone on cast. Kill the creature-land (it returns tapped when it dies or is exiled, per Oracle). Cursed Totem |
| **D. Badgermole Cub x2 (Cub + a clone) + two earthbent creature-lands** [primer: AnthonyD] | Badgermole Cub, a clone of it, two lands made creatures, Eternal Witness, Snap. All in our list | Runs the line B loop without Cradle. Exact mana not counted in [AD] | Cub earthbends one land; the copy earthbends another; creature-lands tapped for mana add {G} each from the two Cubs; run Witness + Snap | As line B, plus removing a Cub |
| **E. Faerie Mastermind + Green Sun's Zenith (the finisher)** [primer: AnthonyD] | Faerie Mastermind ({1}{U}), Green Sun's Zenith, Veil of Summer, infinite mana. All in our list | Infinite mana first. Mastermind's ability {3}{U} each loop | Stack Mastermind's "each player draws a card" with infinite mana until the opponents deck. Cast Veil of Summer first. Cast GSZ with X=0 (X=1 against Mental Misstep) so GSZ shuffles back into the library and you never run out of cards. No targeting, so it beats The One Ring protection. GSZ X=0 needs a green creature of MV 0: Dryad Arbor is one in our list **[inference]** | Counter Mastermind or Veil; Mastermind removal in response to activations (then recast it); Cage stops GSZ's put-onto-battlefield part [Oracle] |
| **F. Finale of Devastation (the other finisher)** [primer: AnthonyD, EDH] | Finale, Thrasios, creatures, infinite mana (or a very large pool). All in our list | {X}{G}{G} with X of 10 or more: 12+ mana | Cast all creatures first, then Finale in **main phase 1** (before combat). Creatures get +X/+X and haste; attack. Finale also searches the graveyard, so it gets around Opposition Agent and a hated library [primer: AnthonyD] | Counter Finale. Sweeper or removal in response. Blockers and fog. Cage/Runestone stop the creature from entering |
| **G. Springheart Nantuko + Vibrance damage loops** [primer: AnthonyD] | Springheart Nantuko (bestowed on Eternal Witness), Vibrance, Eternal Witness, plus the Oboro/Gates loop (line A). All in our list | Bestow {1}{G}; per landfall {1}{G}; evoke Vibrance {R}{R} (or {R/G}{R/G}); the {R}{R} needs red sources (Gates filters) | During line A each Gates entry is a landfall: pay {1}{G}, copy Witness, Witness returns Vibrance, evoke it for {R}{R}: 3 damage per loop to any target | Counter Nantuko or Vibrance; kill Witness; Cursed Totem |
| **H. Underworld Breach + Brain Freeze + Lion's Eye Diamond (+ Gamble, Shifting Woodland)** [primer: AnthonyD, "the BEST WAY to play the deck"] | **Not in our list.** Our list has Gamble and Shifting Woodland, but no Underworld Breach, no Brain Freeze and no Lion's Eye Diamond (0 hits in the import list) | n/a | n/a (do not use) | n/a |
| **I. Izzet Guildmage copying Vitalize / Snap / Dramatic Reversal** [primer: AnthonyD] | **Not in our list** (Izzet Guildmage absent). Snap and Dramatic Reversal are in our list but no copier | n/a | n/a | n/a |
| **J. "Exodia" and This Town Ain't Big Enough lines** [primer: AnthonyD] | **Not in our list** (This Town Ain't Big Enough absent: so Witness + This Town + Candelabra, Vibrance + Witness + This Town, and the Exodia chain are all out). Candelabra, Sowing Mycospawn, Valley Floodcaller, Springheart Nantuko, Veil are in our list | n/a | n/a | n/a |
| **K. Finite engine** (not a win by itself) [primer: AnthonyD, EDM] | Earthcraft + Wild Growth on a basic, Valley Floodcaller, Candelabra of Tawnos, Magus of the Candelabra, Formidable Speaker, Minamo, Deserted Temple, Training Grounds, Biomancer's Familiar | Large but finite | [AD] says Valley Floodcaller + Earthcraft + clones is big but finite, not infinite alone. Formidable Speaker ({1},{T}) untaps Cradle; each of Candelabra, Magus, Speaker, Minamo, Deserted Temple untaps Cradle once **[inference]** | As A |

Other [AD] lines not in our list: Displacer Kitten + Witness + Snap + Lotus Petal (also cut by the author on 2026-10-07), Marwyn + Six + Lion's Eye Diamond.

**Tutors that find each piece** (Scryfall text checked; [AD] confirms the tutor families).

| Piece | Found by |
|---|---|
| Gaea's Cradle | Crop Rotation, Expedition Map, Sowing Mycospawn (cast trigger finds a land untapped even if countered), Vibrance ({G}{G}: land to hand), Gamble [primer: AnthonyD, EDH, EDM] |
| Talon Gates of Madara | Crop Rotation, Vibrance, Mycospawn, Gamble [primer: AnthonyD] |
| Oboro Breezecaller (MV 2) | Chord of Calling (X=2), Nature's Rhythm (X=2), Finale (X=2), Invasion of Ikoria (non-Human creature only: finds Oboro, not Witness or Spellseeker), Formidable Speaker (ETB, discard a card). Green Sun's Zenith cannot (Oboro is blue) [primer: AnthonyD; the GSZ note is inference] |
| Eternal Witness (MV 3, green Human) | Green Sun's Zenith (X=3), Chord (X=3), Nature's Rhythm (X=3, also by harmonize), Finale (X=3), Formidable Speaker. **Not** Invasion of Ikoria [primer: AnthonyD] |
| Snap (MV 2) | **Spellseeker** (instant or sorcery MV 2 or less): [AD] calls Spellseeker "a one card win... provided we have the mana", chaining to Snap, Green Sun's Zenith, Nature's Rhythm, Crop Rotation. Also Gamble, Witness from the graveyard, Noxious Revival [primer: AnthonyD] |
| Finale of Devastation | Gamble. **Spellseeker can find it**: in the library Finale has MV 2 ({X}{G}{G}, X=0), so it is a sorcery of MV 2 **[inference from Oracle; this fixes the earlier spec]** |
| Faerie Mastermind / Green Sun's Zenith | Chord (X=2), Nature's Rhythm, Finale, Speaker for Mastermind; Gamble and Spellseeker (MV 1) for GSZ [primer: AnthonyD; inference] |
| Green Sun's Zenith targets | Eternal Witness (combo), Enduring Vitality (mana), Biomancer's Familiar (cost reducer) [primer: AnthonyD] |
| Land targets of Crop Rotation | Gaea's Cradle (ritual with 2+ creatures), Talon Gates (line A), Emergence Zone (flash for a turn: win on another player's turn or over an attempt), Minamo, Shifting Woodland [primer: AnthonyD] |

[AD] also lists Nature's Rhythm as a "one card win condition": two creatures from one card, e.g. X=3 Spellseeker -> Snap, harmonize for Witness; or X=2 Oboro, then harmonize X=3 Spellseeker -> Crop Rotation -> Talon Gates. All in our list. [primer: AnthonyD]

**Recursion for Oboro** (if it was killed): Eternal Witness, Noxious Revival, and Shifting Woodland (Delirium {2}{G}{G}: becomes a copy of a permanent card in the graveyard until end of turn; needs four card types there) [primer: AnthonyD; costs from Oracle].

## Mulligan

- **Keep:** at least two lands and a piece of acceleration; keep hands with Crop Rotation or another way to get Gaea's Cradle; look for mana and card-flow engines, not one specific combo. **[primer: SB24, general rules for a related build]**
- **Ship:** no acceleration; two or more purely reactive spells unless they can be used proactively or the hand has a clear win. **[primer: SB24]**
- **Typical keep** **[inference]**: 2-3 lands (one a fetch or dual), Rograkh in the command zone (free), one of Sol Ring, Mox Amber, Springleaf Drum, Birds or Elvish Spirit Guide, plus a free counter (Fierce Guardianship, Force of Will, Deflecting Swat) or Crop Rotation.

- [AD] gives no mulligan advice. A hand with no way to reach Cradle or no creature to raise the count is slow, but the keep rules above are unchanged. **[inference]**

## Turn by turn

[AD] gives no turn plan. Cited points are marked; the rest is **[inference]**.

- **Turns 1-2:** Cast Rograkh for {0} (turn 1) **[primer: EDH]**. Two cheap commanders turn on Fierce Guardianship, Deflecting Swat, Flare of Duplication and Mox Amber early **[primer: AnthonyD]**. Play fast mana (Sol Ring, Mana Vault, Moxen, Lotus Petal) and mana creatures (Birds, Badgermole Cub, Gene Pollinator, The Cabbage Merchant). Springleaf Drum is early ramp, and an early Cradle counts **[primer: AnthonyD]**. Get cheap creatures and token makers out as soon as possible (Springheart Nantuko, Chatterstorm) to raise the Cradle count **[primer: AnthonyD]**. Hold mana for a free counter.
- **Turns 3-4:** Cast Thrasios (a creature for Cradle). Play Cradle (find it with Crop Rotation). Add creatures: Formidable Speaker (discard a card, tutor a creature; also untaps Cradle), clones. Do **not** cast Oboro on a turn the deck is not going for the win: opponents know it is a threat and will remove it **[primer: AnthonyD]**. Hold Oboro (or tutor it at instant speed with Chord of Calling) until a win is possible. Candelabra of Tawnos untaps Cradle **[primer: SB24, EDH]**.
- **Later:** If no loop, Thrasios is a card engine; Training Grounds and Biomancer's Familiar cut its cost to {1}-{2}, so activate every turn. Keep drawing toward Crop Rotation, Witness, Snap, Oboro **[primer: SB23, EDH, EDM, AnthonyD]**. Spellseeker -> Snap/Gamble/GSZ is a one-card win when the mana is there **[primer: AnthonyD]**.

## Interaction

**Holds** (all in our list): free or cheap counters: Fierce Guardianship (free with a commander; noncreature spells), Force of Will, Force of Negation (free on opponents' turns; noncreature), Misdirection, Mindbreak Trap, Flare of Denial, Pact of Negation, Swan Song, Flusterstorm, Mental Misstep, Commandeer, Louisoix's Sacrifice (counters an ability, trigger or noncreature spell; sacrifice a legendary creature or pay {2}). Redirect and copy: Deflecting Swat, Flare of Duplication (Rograkh and Hexing Squelcher are red nontoken creatures). Protection: Veil of Summer, Hexing Squelcher, Allosaurus Shepherd. Bounce and removal: Chain of Vapor, Snap, Otawara (channel), Boseiju (channel). Talon Gates' ETB also phases out a creature (a one-off answer to a blocker or to a Godo-style creature). **[inference]**

**Use.** The deck wants counters that protect its own plays and win the stack on the opponent's turn. It does not want to trade two cards for one or to "police the table". **[primer: SB23, SB24; the allocation is inference]** Spend free counters on (1) a spell that would stop its own loop or attack (sweeper, Cursed Totem, Cage/Runestone, Rule of Law effects), (2) a combo piece of an opponent about to win. Mental Misstep and Flusterstorm go on cheap rocks and fast mana. Rhystic Study and Mystic Remora tax and are left unprotected. **[inference]**

**Thrasios buries interaction.** Once there is infinite mana, respond to any spell, ability or trigger with Thrasios activations to dig for the answer. Example from [AD]: Orcish Bowmasters flashed in: activate in response to each trigger until Veil of Summer is found; if Veil is countered, activate until a counterspell is found; then assemble any line. **[primer: AnthonyD]** Before the loop is infinite this does not apply; the deck uses its held counters instead. **[inference]**

**Two threats.** **[inference]** Stop the opponent who can win next turn or this turn first; with equal threats stop the faster one. If it has a line itself, it prefers protecting its own win to stopping a threat it can beat in a race.

## When it goes for the win

| Go now when | Wait when |
|---|---|
| The pieces are in play or in hand and the count is met: Cradle at 6 for line A, at 5 plus a {U} land for line B **[primer: AnthonyD]** | Fewer creatures than needed, or Cradle missing |
| It holds a free counter, or Veil / Hexing Squelcher is available for the key step **[inference]** | The opponent has 2+ untapped blue mana or a free counter showing and the deck has no protection (spell-based lines B, E, F) **[inference]** |
| Line A is already running: it is abilities only, instant speed and not stopped by one-spell-per-turn effects **[primer: AnthonyD]** | A black player has {2}{B} open (Opposition Agent) and the line needs a library search **[primer: AnthonyD]** |
| An opponent is about to win and the loop wins first **[inference]** | Cursed Totem or a Rule of Law effect is on the table and the line it stops is the only one in hand **[primer: AnthonyD]** |
| Oboro is finally cast on a turn that is also the winning turn **[primer: AnthonyD]** | Oboro would be cast only to develop the board **[primer: AnthonyD]** |

[SB24] says the deck accepts that combos get disrupted and relies on redundancy rather than Defense Grid-style protection. [AD] agrees in practice: several lines plus Thrasios digging replace one protected line. **[primer: SB24; AnthonyD]**

## What it fears

[AD] has a weaknesses list; each item is the author's, with the deck's response.

- **Cursed Totem.** Shuts off Thrasios **and** Oboro (activated abilities of creatures). Counter it on the stack, remove it (Boseiju channel hits artifacts), or play around it. **[primer: AnthonyD]**
- **Grafdigger's Cage / Weathered Runestone.** Stop onto-battlefield tutors (GSZ, Finale, Chord, Nature's Rhythm, Invasion) and graveyard casts (harmonize); the outlet (Thrasios) still works. Cage stops creature cards entering from library or graveyard; Runestone stops nonland permanent cards, so land tutors (Crop Rotation, Vibrance, Mycospawn) still work against Runestone **[inference from Oracle]**. Line A (Oboro already out) still works. **[primer: AnthonyD; the Runestone split is inference]**
- **Opposition Agent** (needs {2}{B}, flash). Shuts off tutoring: respect a black player with {2}{B} open. Finale from the graveyard still works. Fetchland searches are also hit **[inference from Oracle]**. **[primer: AnthonyD]**
- **Rule of Law / Damping Sphere / Deafening Silence / High Noon / Archon of Emeria.** Ruin spell-based loops (B, C, E, F); activated-ability line A still works. **[primer: AnthonyD]**
- **Board wipes** (Toxic Deluge, Cyclonic Rift, Firestorm) and **Orcish Bowmasters** ("can effectively be a board wipe for us" in [AD]; the reason is [inference]: its pings kill small creatures and punish extra draws). Counter wipes if possible; keep Veil and hexproof ready. **[primer: AnthonyD]**
- **Silence effects** (Silence, Ranger-Captain of Eos, Angel's Grace). Stop the win on that turn. [AD] describes a "secret priority" cleanup trick to win through them: **out of scope for the sim** (see Open questions). **[primer: AnthonyD]**
- **Cradle hate** (Wasteland, Strip Mine, Boseiju). Both A and B need Cradle; Crop Rotation is a second source. **[inference]**
- **Graveyard hate** (Rest in Peace, Bojuka Bog). Hurts Finale from the graveyard, Shifting Woodland, Noxious Revival and harmonize. **[inference]**
- **Stax on mana** (Torpor Orb, Hushbringer, Collector Ouphe hits Springleaf Drum, Mox Amber, Mana Vault). [SB23] says Thrasios plays well against slow stax as long as it can still activate. **[inference; SB23]**
- **Seedborn Muse** is cut in [AD] because it brings unwanted attention (3-v-1s). The sim should not expect this deck to play it. **[primer: AnthonyD]**
- **Fast combo decks.** Weak card advantage; cannot interact forever. **[primer: SB24]**

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


**New from [AD] (tags as shown).**

- Once infinite mana is running, Thrasios digs in response to Godo's ETB trigger or the Helm trigger until Louisoix's Sacrifice or a counterspell is found. **[primer: AnthonyD]** Before that, only the held cards above work. **[inference]**
- Talon Gates' ETB phases out a target creature. The {4} ability is instant speed, so Gates can phase out Godo (or the equipped creature) before the Helm trigger resolves at combat; Louisoix's Sacrifice and Gates together give two answers to one equip. **[inference]**
- Line A also phases out every opposing creature, which stops Godo's attack and its token copy plan for the turn. **[primer: AnthonyD; Godo link is inference]**
- For the Godo spec: this deck can win without casting spells (line A) and answers at instant speed once Thrasios is online, so Godo should not expect it to tap out. Phase-out and Thrasios digging are the two answers to plan around. **[inference]**

## Decision scenarios (become AI tests)

| # | Position | Right action | Reason | Tag |
|---|---|---|---|---|
| 1 | Turn 1 on the play. Hand: Mox Amber, Birds, Forest, Crop Rotation, Fierce Guardianship. Rograkh in the command zone | Cast Rograkh ({0}), then Mox Amber, then land and Birds | Rograkh turns on Mox Amber and free Guardianship; a turn 1 start gives the fast mana line | [primer: AnthonyD, EDH, SB24] |
| 2 | Turn 4. Board: Rograkh, Thrasios, Birds, Dryad Arbor, Cradle (6 creatures needed: has 4 + Breezecaller in hand), 5 mana open, opponent has 2 Islands untapped and a Force of Will showing in hand count | Hold Breezecaller; cast a creature (Chatterstorm for tokens) to bait first | Needs six creatures and protection; the opponent has counter mana | [primer: AnthonyD (do not cast Oboro early); bait is inference] |
| 3 | Own board: Cradle, Breezecaller, Talon Gates in hand, 6 creatures, opponent has no untapped blue mana, cards in hand: Veil of Summer | Start line A: tap Cradle, pay {4} to put Gates in, tap Gates, {2} bounce Gates to untap Cradle, repeat, then Thrasios | The loop is board actions; no open counters | [primer: AnthonyD, EDH, EDM] |
| 4 | Own turn, Witness and Snap in hand, Cradle with 5 other creatures, Island. Opponent has Force of Will and 3 open blue mana | Wait, or cast a bait spell first; with Veil of Summer or Hexing Squelcher in play, go | Witness or Snap may be countered. Veil / Squelcher stop counters | [primer: EDH; the wait is [inference]] |
| 5 | Opponent casts a sweeper (Toxic Deluge) with the deck holding Fierce Guardianship | Counter it with Fierce Guardianship (free with a commander) | A sweeper kills all lines | [inference] |
| 6 | Opponent has Cyclonic Rift and 7 mana on their turn; the deck has a loop planned for next turn | Counter Rift (Force of Negation free, or Fierce Guardianship) if it targets the key piece; else let it resolve | Spend free counters to protect key pieces; avoid trading 2-for-1 | [primer: SB24; inference] |
| 7 | Two opponents threaten: A has an Oracle-style combo ready next turn, B has a slow engine. Deck has one free counter and no combo yet | Counter A's combo piece | Stop the threat that wins first | [inference] |
| 8 | Opponent casts Godo (6 mana). The deck has Louisoix's Sacrifice, Force of Will, 2 other cards | Let Godo resolve, then counter the ETB trigger with Louisoix's Sacrifice (or Force of Will on Godo itself if the trigger is not answerable) | The ETB trigger is Godo's only way to fetch Helm; countering the trigger costs {U} + {2} or a legendary creature | [inference] |
| 9 | Infinite mana reached, library 40 cards, Thrasios, Training Grounds | Activate Thrasios until 1-2 cards remain, then cast Hexing Squelcher and Veil of Summer, then Finale with X of 10+ | Draw the deck, then protect the win | [primer: AnthonyD, EDH, EDM; stop point is inference] |
| 10 | Mulligan: 7 cards, 1 land, Sol Ring, Mox Amber, Rograkh in command zone, 3 spells | Keep | Fast mana and a free commander replace lands; a 1-lander without fast mana is a ship | [primer: SB24 (keep rule); inference] |
| 11 | Mulligan: 7 cards, 2 lands, Force of Will, Pact of Negation, Fierce Guardianship, Mental Misstep, 1 other spell | Ship | Four reactive spells and no acceleration | [primer: SB24] |
| 12 | Own turn, Oboro in hand, Cradle with 4 creatures, Gates in hand, no loop possible this turn. Opponent has a black deck with open mana | Do not cast Oboro. Cast cheap creatures or token makers instead | Opponents know Oboro is a threat and will kill it; cast it only on a winning turn | [primer: AnthonyD] |
| 13 | Opponent casts Cursed Totem. The deck has Force of Will, Oboro and Thrasios on the board | Counter it (Force of Will) | It shuts off Thrasios and Oboro. Counter on the stack, remove it, or play around it | [primer: AnthonyD] |
| 14 | A black player has {2}{B} untapped (Opposition Agent). The deck holds Crop Rotation and Gamble; Finale is in the graveyard | Do not tutor or crack a fetch with the opponent's mana up; cast Finale from the graveyard if the win is on, else wait | Agent controls searches and exiles found cards. Finale from the graveyard avoids it | [primer: AnthonyD] |
| 15 | Rule of Law is in play. Board: Cradle (6 creatures), Oboro, Talon Gates on the battlefield | Run line A (abilities only); do not cast Witness, Snap or Finale | Rule of Law stops the second spell; Plan A casts no spells | [primer: AnthonyD] |
| 16 | Grafdigger's Cage is in play. Hand: GSZ, Finale, Nature's Rhythm. Board: Oboro, Cradle (6 creatures), Gates | Run line A; do not use tutors that put creatures onto the battlefield | Cage stops GSZ, Finale and Nature's Rhythm entering; Oboro is already out | [primer: AnthonyD] |
| 17 | Infinite mana. Opponent flashes in Orcish Bowmasters and its trigger is on the stack. Library has Veil of Summer and Force of Will | Activate Thrasios in response, repeating for each trigger, until Veil of Summer (then counterspell if Veil is countered) | Thrasios at instant speed buries interaction | [primer: AnthonyD] |
| 18 | Infinite mana, Mastermind and GSZ in hand with Veil, opponent has The One Ring out | Cast Veil first, then Mastermind's draw ability in a loop with GSZ X=0 | No targeting, so The One Ring does not stop it; GSZ shuffles back so the deck never decks | [primer: AnthonyD] |
| 19 | Spellseeker in hand, 12 mana available, Cradle in play; Witness in library, no Snap in hand | Spellseeker for Snap, Snap on Spellseeker (untap two lands), recast it for GSZ, GSZ X=3 for Witness, then line B | Spellseeker is a one-card win with the mana (3 + 2 + 3 + 4 = 12) | [primer: AnthonyD; mana sum is inference] |
| 20 | Board: Cradle with exactly 5 creatures, Oboro, Gates in hand. Opponent has two big attackers | Run line A once or twice to phase out their creatures; line A is mana-neutral, so finish with Thrasios using the mana you hold | Even 5 creatures phases out the table | [primer: AnthonyD; the neutral count is inference] |
| 21 | Oboro just died. Eternal Witness in hand, Noxious Revival in hand, Shifting Woodland on the battlefield, 4 card types in the graveyard | Recur Oboro (Witness is first; Woodland copy is a backup at {2}{G}{G}) only if the next turn is a win turn | Recursion is Witness, Woodland, Noxious Revival | [primer: AnthonyD] |
| 22 | Hand: Gamble, Crop Rotation, Veil of Summer. The deck has no Underworld Breach, Brain Freeze or Lion's Eye Diamond | Do not plan the Breach line. Gamble for Gaea's Cradle, Witness, Spellseeker or Finale as the position needs | Our list has none of the Breach pieces | [primer: AnthonyD; list check] |

## Combo tests (become scripted tests)

| Line | Starting board and hand | Expected result |
|---|---|---|
| A | Board: Rograkh, Thrasios, Birds, Dryad Arbor, Oboro Breezecaller, Gaea's Cradle, Island (6 creatures). Hand: Talon Gates of Madara. Mana open: none | Tap Cradle for 6; Gates in ({4}), tap Gates (+1 {C}), Breezecaller ({2}, bounce Gates) untaps Cradle. Repeat 5 times: each loop leaves +1 mana. Floating mana grows without bound |
| B | Board: Rograkh, Thrasios, Birds, Dryad Arbor, a fifth creature, Gaea's Cradle (taps for 5), Island. Hand: Eternal Witness, Snap. Graveyard empty | First cycle: Witness, Snap on Witness; second: Witness returns Snap; mana +1 per cycle. After 3 cycles the mana floating is at least 3 above start |
| F (Finale) | Infinite mana, Thrasios and Training Grounds, library of N cards, Finale and Hexing Squelcher in hand/library | Activate Thrasios until library = 1; draw Finale; cast Squelcher; Finale X=10+; attackers get haste and +X/+X. Opponent with no answer loses the same turn |
| E (Mastermind + GSZ) | Infinite mana, Thrasios drew the library to 4 cards, Mastermind and Veil in play, GSZ in hand, 3 opponents with 30-card libraries | Cast Veil; activate Mastermind's draw ability repeatedly; cast GSZ X=0 when a card is needed (Dryad Arbor is the target); opponents deck out, the deck's library never empties [primer: AnthonyD] |
| G (Vibrance loop) | Line A running, Springheart Nantuko bestowed on Eternal Witness, Vibrance in the graveyard, red source available | Each Gates entry copies Witness ({1}{G}); the copy returns Vibrance; evoke for {R}{R}: 3 damage; 3 damage per loop [primer: AnthonyD] |
| C/D (Cub) | Board: Badgermole Cub, Mockingbird copy of Cub, Oboro, Cradle, a {U} land | Not scripted until mana per loop is counted (see Open questions) [primer: AnthonyD] |
| K (finite) | Board: Cradle (6 creatures), Candelabra, Magus of the Candelabra, Minamo, 5 mana open | Cradle taps 3 times (with Candelabra X=1 and Magus X=1): about 18 mana for 2 mana. Not infinite |
| Not testable | Lines H, I, J | Not in our list | n/a |

## Open questions

- **Secret priority.** [AD] describes a cleanup-step trick (discard Rograkh to hand size and move it to the command zone, which gives priority after a Silence effect wears off; or Springheart Nantuko bestowed on an animated Shifting Woodland) to win through Silence, Ranger-Captain of Eos and Angel's Grace, on the deck's own turn only. This needs rules-level priority handling in the cleanup step. **Out of scope for the sim; no scenario is built on it.** The sim should treat a Silence effect as stopping the win that turn. [primer: AnthonyD]
- Lines H (Breach + Brain Freeze + LED), I (Izzet Guildmage) and J (This Town / Exodia) are not in our list. [EDM] still lists Brain Freeze and Underworld Breach for another build; [AD] calls the Breach line the best way to play the deck. The lead should decide whether the sim's Rograkh list stays as it is (the Pittsburgh list) or adds these. Not decided here.
- Mana per loop for lines C and D (Badgermole Cub) is not counted in [AD]; I did not verify it.
- GSZ in line E: [AD] says it shuffles itself back so there is always a card. How the deck draws GSZ again each cycle is not described. With Thrasios the deck can dig for it; the exact loop is not checked.
- Veil of Summer and Archon of Emeria were not rechecked against Scryfall in the last call; [AD]'s wording is used. Invasion of Ikoria // Zilortha is a double-named card and the collection call did not find it by that name; [AD]'s "non-Human creature only" is used as written.
- Whether the Breezecaller loop with Gates' phase-out is used to finish: [AD] says it phases out the table's creatures; no source says a finish uses it.
- Is Rograkh a valid Flare of Duplication / Flare of Denial sacrifice (red nontoken creature)? Yes by Oracle; no source says the pilot does it.
- Helm of the Host's trigger if the equipped creature is bounced or phased out in response: rules not checked.
- Thrasios on an empty library: the exact lose condition was not checked.
- Dramatic Reversal, Louisoix's Sacrifice, Hidden Strings, Flare of Duplication, Mockingbird and the other clones have no stated role in [AD] beyond Mockingbird in line C.
- No primer gives mulligan or timing rules for this list; those remain inferences. The lead should decide whether to accept them for the AI plan or keep it minimal.
