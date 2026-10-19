# Kinnan, Bonder Prodigy — Behaviour Spec (Tier 6)

Sources (all opened 2026-10-17; oracle text checked on Scryfall, S10). Spec written 2026-10-17, updated the same day with the NBC primer (P).

**Tags.** **[primer: NBC]** = the dedicated primer P. It outranks every other source: where P and an older source disagree, P wins and
the older claim is dropped or marked. **[primer S#]** = another source. **[inference]** = my reasoning from the cards.

| Id | Page | Date shown |
| --- | --- | --- |
| **P** | **FreedomWaffle, "[cEDH] Kinnan NBC (No Bad Cards)" primer, Moxfield (JD's 39-page PDF, /home/joseph/projects/mtg_sims/primers/kinnan-nbc.txt) — https://www.moxfield.com/decks/Mi4yPy0jFUGPx6pI5x3OVw/primer** | "Primer updated 1/14/2026" |
| S1 | Learn cEDH, "Kinnan" (Evan Pierce) — https://learncedh.com/decklists/kinnan | "Mar 11", year not shown |
| S2 | EDHREC, "Building Kinnan, Bonder Prodigy for cEDH" (Harvey McGuinness) — https://edhrec.com/articles/building-kinnan-bonder-prodigy-for-cedh | May 19, 2026 |
| S3 | Commander's Herald, "An Introduction To Kinnan In cEDH" (Drake Sasser) — https://commandersherald.com/an-introduction-to-kinnan-in-cedh/ | Dec 8, 2023 |
| S4 | cEDH Decklist Database, entry "Kinnan Infinite Mana" — https://cedh-decklist-database.com | 2023-12-15 |
| S5 | Forest Birds Go, "From zero to seven mana in two seconds flat: Kinnan EDH" — https://forestbirdsgo.wordpress.com/2020/05/17/from-zero-to-seven-mana-in-two-secondkinnan-edh/ | 2020-05-17 (casual, old) |
| S6 | Commander Spellbook and EDHREC combo pages, "Kinnan + Basalt Monolith" — https://commanderspellbook.com/combo/129-4131/ , https://edhrec.com/combos/simic/129-4131 | none |
| S7 | EDHTop16, Kinnan page and the SIEGE cEDH 10K decklist (our list) — https://edhtop16.com/commander/Kinnan%2C%20Bonder%20Prodigy | 2026-06-13 event |
| S8 | EDHREC, Kinnan cEDH page (4,742 decks) — https://edhrec.com/commanders/kinnan-bonder-prodigy/cedh | Oct 2026 |
| S9 | Moxgate, "How to Play Kinnan" (a Bracket 3 list, different cards; low weight) — https://www.moxgate.com/guides/kinnan-bonder-prodigy-commander/ | Updated Mar 2026 |
| S10 | Scryfall oracle text, every card named below (one /cards/collection call, 2026-10-17) | n/a |

**What P is and is not.** P is the author's own list, not our 100. It has a combo section, a win-condition section, a mulligan guide with seven
worked hands, and a card-choice section. Its FAQ answers ("How do we win?", "Why Hidden Strings?", "Why Dizzy Spell?") are collapsed in the PDF
and did not extract, so they are not used. P has **no** section on turn-by-turn play, interaction priorities, who to stop, what the deck
fears, or Godo. The mulligan guide says "UPDATE IN PROGRESS". Those sections stay mostly [inference] and the gaps are listed in Open questions.
Moxfield and TappedOut pages other than P, MTGNexus and r/CompetitiveEDH are not readable by tools.

**P's list versus ours (SIEGE 10K winner).** Cards P's lines use that are **not in our list**: Cephalid Coliseum, Smothering Tithe, Green Sun's
Zenith, Phyrexian Metamorph, Nyxbloom Ancient, Nezahal, Seedborn Muse, Veil of Summer, Pongify, Simic Signet, Tezzeret the Seeker, Treasure Vault,
Bribery, Thassa's Oracle and the other creature finishers P lists (Orcish Bowmasters, Mayhem Devil, Niv-Mizzet Parun, Twinshot Sniper, Ashling,
Glint-Horn Buccaneer). Also absent from ours: Isochron Scepter, Walking Ballista, Crop Rotation, Mana Drain. Lines that need them are marked
"not in our list". Checked by grep on `decklists/pool/staging-t6-cedh/kinnan-bonder-mana-combo.md`.

## Game plan

Kinnan is a two-mana commander that doubles every mana creature and rock. The deck makes a lot of mana early and spends it on card draw
(Rhystic Study, Mystic Remora, The One Ring, Consecrated Sphinx), on a Kinnan "flip" for a big creature, or on one of three infinite
combos **[primer: NBC: "midrange list that produces tons of mana and goes infinite to win the game"; "produces tons of mana"]**.
P's author is clear about pace: "While the deck is capable of executing fast combos, it thrives on patience and timing. We want to draw cards
until the ideal window presents itself" **[primer: NBC, Strategy]**. Kinnan's ability is the backup plan, used to "cheat high-value permanents
into play" **[primer: NBC]**. The deck still expects fast games: P's hand notes say "the game might be over" by turn 3 in a fast pod
**[primer: NBC, Hand 5]**. Stance: **hold up and draw, then win with protection**; not a lock deck, not a damage race **[primer: NBC; S2, S3]**.
Our list leans the same way: it has the P draw package (Study, Remora, three enchantment/artifact clones, One Ring) and fewer big flips.
Target win turn: 3–5 **[inference from P's "game might be over by turn 3" and "thrives on patience"]**.

## Win lines

Kinnan's ability needs coloured mana ({5}{G}{U}). The loops make colourless mana unless stated. Cards in **bold** are in our list.
Lines A, B and E are the ones P names as the deck's win paths **[primer: NBC, "Combos At A Glance", "Win Conditions At A Glance"]**.

| Line | Pieces (in our list) | Mana (total, colours) | Steps | Where it can be answered (step, by what) |
| --- | --- | --- | --- | --- |
| **A. Kinnan + Basalt Monolith** (main line) **[primer: NBC]** | **Kinnan**, **Basalt Monolith**, an outlet (see "Outlets" below). Finish with **Finale of Devastation** (line D1) or **Faerie Mastermind** decking (line E) | Kinnan {G}{U} (2) + Monolith {3}, or Monolith put in by Transmute Artifact. Outlet: Thrasios {G}{U} to cast, then {4} per draw. Finale needs {G}{G} from real sources plus X from loop mana | 1. Cast Monolith (or put it in with Transmute Artifact). 2. Tap it: {C}{C}{C} + Kinnan's {C} = 4. 3. Pay {3}: untap. 4. Repeat: net +1 {C} each loop = infinite colourless **[primer: NBC, steps 1–5]**. 5. Outlet, then win (D1 or E) | Kinnan: removal or bounce before step 1, or in response to the loop (the loop needs Kinnan's trigger). Monolith: a counter on the cast (step 1); Transmute Artifact puts it in without a cast, so only a counter on Transmute itself works **[primer: NBC "or the ability to tutor it into play"]**. Artifact removal on Monolith. Stax that stops mana abilities (Null Rod, Collector Ouphe) **[inference; S1]** |
| **B. Hullbreaker Horror + two rocks** **[primer: NBC]** | **Kinnan**, **Hullbreaker Horror**, two rocks that gain mana when cast between each other. With Kinnan: any of **Mox Opal**, **Mox Amber**, **Sol Ring**, **Grim Monolith**, **Mana Vault** with each other, or with **Basalt Monolith**, **Chrome Mox**, **Lotus Petal**; or Sol Ring / Grim Monolith / Mox Amber / Mox Opal / Mana Vault with **Fellwar Stone**, **Arcane Signet**, **Talisman of Curiosity** (P also lists Simic Signet: not in our list). Outlet as in A | Horror {5}{U}{U} (7) + Kinnan 2 = 9 total; the rocks cost {0} to {2} | P's example (Kinnan, Horror, Sol Ring in play, Mox Amber in hand): 1. Cast Amber; Horror's trigger returns Sol Ring. 2. Tap Amber for {U} or {G} (doubled by Kinnan). 3. Cast Sol Ring; the trigger returns Amber. 4. Tap Sol Ring for {C}{C}, doubled to 4. 5. Repeat = infinite mana **[primer: NBC]**. Same shape with Lotus Petal and Amber (Petal is cast, not sacrificed) **[primer: NBC lists Petal as a partner; the exact order is inference]** | Horror cannot be countered (flash, so it can be cast at end of turn) **[oracle]**. Removal on Horror before the loop (a trigger already on the stack still resolves). A counter on any loop spell ends the loop. Null Rod / Ouphe on the rocks. Bounce on Kinnan **[inference]**. Mox Opal needs three artifacts (metalcraft), which the bouncing breaks, so prefer Amber, Sol Ring, Vault **[inference from oracle]** |
| **C. Horror table lock** | **Kinnan**, **Hullbreaker Horror**, infinite mana from A or B | As A or B | With free spells, every cast bounces an opposing spell or permanent. P's setup uses it to "bounce all non-land permanents with Hullbreaker Horror and loop **Hidden Strings** to tap down all their lands" before the final win **[primer: NBC, Optional Setup]**. As a win by itself it is [inference] | Same as B. A "can't cast spells" effect stops it |
| **D1. Finale of Devastation** (the combat finish) **[primer: NBC]** | **Finale of Devastation**, enough creatures in play | {X}{G}{G}, X of 10 or more: creatures get +X/+X and haste. Needs Finale cast on your turn in a phase before attackers are declared, and **creatures in play equal to the opposing blockers + 3** **[primer: NBC]** | 1. Cast Finale, spending the mana on X. 2. Move to combat. 3. Swing so that at least one creature reaches each opponent **[primer: NBC]**. After infinite mana, first put every non-Human creature in with Kinnan (needs {U}{G}, see Outlets) and draw with Thrasios | Counter on Finale (step 1); fog, a sweeper or flash blockers (step 3) **[inference]** |
| **D2. Finite big turn** (no loop) | **Kinnan**, many rocks and dorks, **Finale of Devastation**, **Dramatic Reversal**, **Hidden Strings** | Finale with X of 10 needs 12 mana ({G}{G} real). A board like test D makes about 37 **[inference: 22 first tap, +17 after Reversal, -2 for it]** | Tap everything, Dramatic Reversal (untaps all your nonland permanents), tap again, cast Finale | Counter on Reversal or Finale; removal on the fetched creature |
| **E. Faerie Mastermind decking** **[primer: NBC, "Win Conditions": Mastermind + Endurance + Chain of Vapor or Otawara]** | **Faerie Mastermind**, **Endurance**, **Chain of Vapor** and/or **Otawara** (or Horror's trigger) to bounce Endurance, and infinite mana that includes {U} for each Mastermind activation (see note) | Mastermind {1}{U}; activation {3}{U} each; Endurance {1}{G}{G} each recast | 1. Win line A or B first (P: "we will assume we have access to all the cards in our deck with another infinite combo"). 2. Cast Endurance to put more cards in your library than your opponents' libraries hold (fill your graveyard first by casting spells). 3. Activate Mastermind: each player draws. 4. When your library runs low, recast Endurance (bounce it with Chain / Otawara / a Horror trigger, then cast it again) so your graveyard goes back to the bottom of your library. 5. Opponents draw from empty libraries. If an opponent responds to an activation, activate again on top **[primer: NBC]** | Counter on Endurance (the loop stalls); Mastermind removal (P: "if exiled you can take one of your opponents'" with a clone **[primer: NBC]**); Endurance's trigger is the only way to refill (Green Sun's Zenith, P's second refill, is not in our list) |
| **F. Mastermind + Smothering Tithe** | **Faerie Mastermind**, **Kinnan**, a Smothering Tithe **an opponent controls**, copied with **Clever Impersonator**, **Copy Enchantment** or **Mirrormade** **[primer: NBC]**. Smothering Tithe itself is **not in our list** | Start with 2 Treasures (Kinnan doubles them): Mastermind {3}{U} | Use 2 Treasures for Mastermind; resolve your Tithe triggers (3 Treasures); repeat, netting one Treasure per loop; then finish with E **[primer: NBC]**. P: start it on the turn of the player to your left so your Tithe triggers resolve first, and when opponents are low on mana | Removal on Mastermind is answered by activating again; counters on the clone spell; the opponent paying {2} for each Tithe trigger **[inference]**. Only possible when an opponent has played Tithe |
| **Not in our list** | Nyxbloom Ancient + Grim/Basalt Monolith; Cephalid Coliseum loops (three options, all with Endurance + Thrasios); Bribery; Thassa's Oracle, Orcish Bowmasters, Mayhem Devil, Niv-Mizzet, Twinshot Sniper, Ashling, Glint-Horn Buccaneer | | | Do not model |

**Outlets (what turns infinite mana into a win)** **[primer: NBC, Infinite Mana Outlets]**:

| Outlet | Needs | Notes |
| --- | --- | --- |
| **Thrasios, Triton Hero** | colourless {4} per use | Scry 1, reveal; land goes in tapped, else draw. "Draw your deck". Cannot draw lands (it plays them), so **Wan Shi Tong** is P's pick when lands must reach the hand |
| **Kinnan's ability** | infinite {U}{G} (not colourless) | "Activate Kinnan infinite times to put all non-Human creatures into play", then Thrasios to draw, then a win condition |
| **Mirage Mirror** | Basalt Monolith in play, infinite colourless | Turns the colourless loop into infinite {U}{G}: Mirror copies a dual land (tap for {U}/{G}), then copies Monolith (untap it), over and over, with the activations stacked **[primer: NBC, Mirage Mirror steps 1–10]**. Each copy costs {2} (oracle), paid from the infinite pool **[inference]** |
| Creature tutors | infinite mana | **Finale**, **Invasion of Ikoria**, **Nature's Rhythm**, **Chord of Calling** find Thrasios, then draw the deck. Green Sun's Zenith is not in our list |
| Artifact tutors | infinite mana | **Trophy Mage** (fetches an artifact with MV 3) finds Mirage Mirror for the {U}{G} loop. **Transmute Artifact** also finds it (sacrifice an artifact, pay the difference) **[primer: NBC]**. Tezzeret, Treasure Vault, Crop Rotation, Fabricate, Moonsilver Key: not in our list or cut |

**Note on Mastermind and colour.** Mastermind's activation needs {U} each time. Line A gives colourless only, so E after A needs Mirage Mirror (infinite
{U}{G}). After line B the rocks make {U}{G} directly (Amber). **[inference from oracle; P shows the loop but not this detail]**

**Tutors that find each piece (all in our list):**

| Piece | Tutors and routes |
| --- | --- |
| Basalt Monolith (MV 3) | **Transmute Artifact** (puts it onto the battlefield, no cast) and **Trophy Mage** (to hand) **[primer: NBC, tutor lists]**. **Drift of Phantasms** transmute, {1}{U}{U} sorcery speed **[inference from oracle]**. Copies: **Copy Artifact**, **Mirrormade**, **Clever Impersonator**, **Mirage Mirror** on a Monolith already in play **[primer: NBC lists these as copy effects; use on Monolith is inference]** |
| Hullbreaker Horror (MV 7) | **Chord of Calling** (X=7, convoke), **Nature's Rhythm** (X=7), **Finale** (X=7), **Invasion of Ikoria** (non-Human only), Kinnan's ability (top five) **[primer: NBC: "Into Play Tutors" and Kinnan ability]**. Copies: **Mockingbird**, **Clever Impersonator**, **Flesh Duplicate** **[primer: NBC copy list; targets inference]** |
| Thrasios (MV 2) | Chord (X=2), Nature's Rhythm, Finale, Invasion of Ikoria, Kinnan's ability **[primer: NBC]** |
| Faerie Mastermind (MV 2), Endurance (MV 3) | The same creature tutors. Mastermind also needs {U} per activation |
| Rocks (Amber, Petal, Sol Ring, Vaults) | Drawn. Only Transmute Artifact reaches a rock **[inference]**. Mox Diamond discards a land to enter |

## Mulligan

P's rule: look for three things in an opening hand, in this order **[primer: NBC, Mulligan Guide]**:

1. **Draw engines** (Rhystic Study, Mystic Remora, The One Ring and the clones).
2. **An early win attempt** (Basalt Monolith with a way to cast it; "our opponents might not be ready for").
3. **Mana.**

Reason: "If a mulligan has tons of mana, we can activate Kinnan to find a gameplan. If we have a draw engine, we'll likely draw into mana
development." Seat and pod matter: P judges each hand by turn order. The seven hands in P, with the verdict and why **[primer: NBC]**:

| Hand | Verdict | Why |
| --- | --- | --- |
| Sol Ring, Talisman of Curiosity, Mystic Remora (+ lands) | Keep, any seat | Turn 1 Sol Ring, Talisman, Remora: draws from turn 1 and pays Remora's upkeep |
| Cephalid Coliseum, Mystic Remora, Valley Floodcaller, Commandeer, Gemstone Caverns, Boseiju, The One Ring (going 4th) | Keep | Remora is the card to see. Pitch The One Ring to Gemstone Caverns to guarantee land drops, hold Commandeer up and take a Rhystic Study, Smothering Tithe or payoff |
| Worldly Tutor, Mox Diamond, Commandeer, Ancient Tomb, Rhystic Study, Delighted Halfling, Swan Song | Mulligan | One land: the downside of missing the first land drop is huge. It would be a keep at four cards (Diamond, Tomb, Study, Halfling) |
| Forbidden Orchard, Lotus Petal, Mystic Remora, Mindbreak Trap, Tezzeret, Arcane Signet, Astral Dragon (going 2nd) | Keep | Turn 1 Signet into Remora. Mindbreak Trap stops an early win attempt |
| Spellskite, Exotic Orchard, Command Tower, The One Ring, Fyndhorn Elves, Delighted Halfling, Valley Floodcaller (going last) | Mulligan | Playable, but turn 3 The One Ring going last is too slow; keep it at five or six cards |
| Simic Signet, Breeding Pool, The One Ring, Nyxbloom Ancient, Dramatic Reversal, Cephalid Coliseum, Lotus Petal | Mulligan | Lots of mana but only 4 on turn 3, tapping out for The One Ring |
| Bloom Tender, Seedborn Muse, Sink into Stupor, Clever Impersonator, Flooded Strand, Elvish Spirit Guide, Springleaf Drum (going first) | Keep | Turn 1 Bloom Tender (pitch Spirit Guide); turn 2 Kinnan and Springleaf Drum for 4 mana, Clever Impersonator on a Study / Remora or a mana rock |

Rules drawn from P:

- **Keep** a hand with one real draw engine and enough lands/mana to cast it by turn 2; keep a hand with fast mana plus Kinnan.
- **Ship** a hand with one land and no free mana, or a hand whose only plan is a turn-3 tap-out when you are last in the pod. Compare against what four cards would be.
- **A typical keep:** two lands, a mana rock or dork, Kinnan or a draw engine, one cheap counter. Elvish Spirit Guide, Lotus Petal and Mox Diamond
  make the first mana. The P hands show the shape: turn 1 rock into Remora; turn 1 dork, turn 2 Kinnan plus a rock.
- Older sources said "two lands and a dork or rock" **[primer S5]** and "mulligan for ramp and card advantage" **[primer S1]**. P agrees in outline and adds the engine-first order, so P's order is used.
- No source gives a land count.

## Turn by turn

P has no turn-by-turn section. It shows three opening lines inside the mulligan hands **[primer: NBC]**, and the rest comes from older sources and inference.

**Turns 1–2.**
- Best opening: turn 1 mana rock (Sol Ring, Arcane Signet, Talisman), then Mystic Remora, so it draws from the first opposing spell **[primer: NBC, Hands 1 and 4]**.
- Dork line: turn 1 dork (Fyndhorn Elves, Bloom Tender with Spirit Guide), turn 2 Kinnan plus Springleaf Drum or Delighted Halfling **[primer: NBC, Hands 5 and 7]**.
- Turn 2 with Kinnan and 4 mana: Clever Impersonator on an opposing or own Rhystic Study / Mystic Remora, else on a mana rock to set up bigger turns **[primer: NBC, Hand 7]**.
- Kinnan comes before rocks, so each rock taps for double **[inference: Kinnan's static ability]**. Kinnan costs two, so removal on him hurts little (the tax is 2 per death) **[primer S3]**.
- Keep one free counter up when it costs nothing **[primer S3]**.

**Turns 3–4.**
- Play the draw engines: The One Ring on turn 3 only if the pod is slow or you are first (P calls going last "not ideal") **[primer: NBC, Hand 5]**.
- With 7+ mana and no combo yet: activate Kinnan at the end of the opponent's turn to put in Consecrated Sphinx, Hullbreaker Horror, Thrasios, Faerie Mastermind or Wan Shi Tong. The ability cannot be countered by a spell counter and ignores Rhystic Study taxes
  **[primer S3; P: "cheat high-value permanents into play" as the backup plan]**.
- Gene Pollinator, Springleaf Drum and Enduring Vitality add Kinnan triggers: Enduring Vitality makes every creature tap for mana **[inference from oracle]**.
- Get an outlet (Thrasios) in play or in hand before going for Monolith **[primer S3, S2; P outlets]**.

**Later.** Keep drawing until a window opens: "We want to draw cards until the ideal window presents itself" **[primer: NBC]**. Go for line A or B once the protection test (see "When it goes for the win") passes. If it never does, keep flipping at end of turn and drawing with Sphinx, One Ring, Study and Remora.

## Interaction

P's own interaction suite (its list, not ours): Otawara, Chain of Vapor, Into the Flood Maw, Sink into Stupor, Force of Vigor; "on the stack": Pact of Negation, Dispel, Flusterstorm, Mental Misstep, Miscast, Swan Song, Fierce Guardianship, Force of Negation, Sink into Stupor, Mindbreak Trap, Force of Will **[primer: NBC, Card Choices]**.
P does not say which spell gets which counter. Our list differs: ours adds An Offer You Can't Refuse, Misdirection, Disrupting Shoal, Commandeer, Snapback, Dizzy Spell, Colossal Skyturtle, Volatile Stormdrake and Boseiju, and lacks Miscast, Into the Flood Maw, Force of Vigor.

**What it holds (our list):**

| Type | Cards |
| --- | --- |
| Free counters | Force of Will, Force of Negation (opponent's turn only), Fierce Guardianship (needs Kinnan on the battlefield), Misdirection, Mental Misstep (MV 1 only), Disrupting Shoal (pitch a blue card with MV X), Commandeer (exile two blue cards), Mindbreak Trap (after three opposing spells), Pact of Negation (pay {3}{U}{U} next upkeep) |
| One-mana counters | Swan Song (enchantment, instant, sorcery only; gives a 2/2 flier), An Offer You Can't Refuse (noncreature; gives two Treasures), Flusterstorm (instant/sorcery), Dispel (instant only) |
| Bounce | Snapback (free with a blue card), Chain of Vapor, Sink into Stupor, Otawara (channel; {1} cheaper with Kinnan), Colossal Skyturtle (channel), Hullbreaker Horror (bounces on every spell it casts) |
| Other removal | Boseiju (artifact, enchantment, nonbasic land; costs {G} with Kinnan out), Volatile Stormdrake, Dizzy Spell (-3/-0) |
| Engines that tax | Rhystic Study, Mystic Remora, Sylvan Library; the clones |

**What it spends each on:**
- Commandeer does not counter: it **takes** a noncreature spell ("gain control of target noncreature spell"). P's use is to take an opposing Rhystic Study, Smothering Tithe or payoff **[primer: NBC, Hand 2; oracle]**. It is the answer to an opposing engine or combo piece, and it cannot hit creatures.
- Mindbreak Trap: stops an early win attempt (three spells in a turn) **[primer: NBC, Hand 4]**.
- Creature spells: Force of Will, Pact, Misdirection (single target only), Mental Misstep (MV 1), Disrupting Shoal **[inference from oracle]**. Fierce, Force of Negation, Offer and Commandeer cannot hit creatures.
- Noncreature artifacts and enchantments (stax, rocks): Fierce Guardianship, Force of Negation, Offer. Swan Song hits enchantments but not artifacts **[inference]**.
- Instants and sorceries: Flusterstorm, Dispel, Swan Song, Mindbreak Trap, Pact **[inference]**.
- A permanent already in play: Chain of Vapor, Sink into Stupor, Snapback (creatures), Otawara, Boseiju **[inference]**.

**What the combos do with counters (P).** The deck's win loops are interaction-proof once running: "you can respond to removal spells with more activations, never letting the removal spells resolve" (Mastermind), and the Endurance/Horror bounce reuses counterspells **[primer: NBC]**. Before the final refill P says to "make your opponents fight over other important cards like Hullbreaker Horror, High Fae Trickster, Valley Floodcaller, Crop Rotation, Chord of Calling" so their counters are spent on payoffs before Endurance, and that it is "easy to win this battle if you have the mana advantage" **[primer: NBC]**. Check that you hold most of your counterspells before starting a loop **[primer: NBC]**.

**When it lets something resolve.** It lets a spell resolve when it cannot win the game and the spell does not stop its own plan **[inference]**. It spends free counters on combos and stax, not value spells
**[primer S3: "interaction for every kind of win and plenty of free interaction to get involved in stopping win attempts after resolving a value engine"]**. Rhystic Study and Remora collect from spells it lets pass **[primer S2]**.

**Who to stop when two players threaten to win [inference; P and the other sources do not cover this].** Stop the player who wins first in turn order and stack position. A second free counter handles the next one. If it cannot stop both, it goes for its own win with the protection it has left. It does not use its only counter on a value spell when a combo deck is in play. P's one hint: Mindbreak Trap and Commandeer are held for "an early win attempt" and for engines/payoffs **[primer: NBC, Hands 2 and 4]**.

## When it goes for the win

P's conditions, then the rest.

| Go now | Wait |
| --- | --- |
| The Faerie Mastermind loop: opponents have little mana open, so "their interaction will be limited by mana and you will be able to interact with every spell in your deck"; start at instant speed on the turn of the player to your left **[primer: NBC]** | Opponents have a lot of mana open for interaction **[primer: NBC]** |
| You hold most of your counterspells before starting a loop **[primer: NBC]** | You hold few counters (bait first: Horror, Trickster, Floodcaller, Chord, then the key spell) **[primer: NBC]** |
| Kinnan in play and Basalt Monolith castable or Transmute Artifact in hand, with a counter held for each opposing blue mana that is open **[inference]** | No counter held and an opponent has two or more blue mana open (cast Kinnan or a lesser spell first as bait) **[inference]** |
| An outlet (Thrasios) in play or in hand with {G}{U} spare **[primer: NBC outlets]** | Collector Ouphe, Null Rod, Stony Silence on the table and not yet removed **[primer S1]** (P rates the deck "resilient to stax" so this is weaker than S1 says) |
| For Finale: creatures in play at least the opposing blockers + 3 **[primer: NBC]** | Opponent shows a fog, a sweeper or many flash blockers (Finale is a sorcery, the attack is last) **[inference]** |
| A resolved loop and at least one card kept in the library (Thrasios and Mastermind deck us too) **[inference; P's Endurance step]** | Pact is the only counter and next upkeep cannot pay {3}{U}{U} **[inference]** |

No window yet: **draw cards and wait** (P) rather than force it. Use Hullbreaker Horror instead of Monolith when the table holds artifact hate or a counter for artifacts: Horror cannot be countered **[oracle]**.

## What it fears

P's pros and cons for the list: **cons** are "lack of good tutors", "weak to removal", "runs mediocre cards to enable the gameplan"; **pros** include "resilient to stax and good in most metas" **[primer: NBC, Overview]**. P gives no longer list, so the table below mixes P with the older sources.

| Threat | Why | How it plays around it |
| --- | --- | --- |
| Removal on Kinnan or the mana creatures (P: "weak to removal") **[primer: NBC]** | No Kinnan, no loop A; fewer mana | Draw engines carry on without Kinnan; recast for 2 (+2 per death) **[primer S3]**. Cast a creature first so the board is not all Kinnan **[inference]** |
| Poor consistency: few tutors **[primer: NBC]** | The combo is not always there | The draw engines (Study, Remora, clones, One Ring) dig for it; play patiently **[primer: NBC]** |
| Sweepers (creatures, artifacts) | Dies with its dorks and rocks **[primer S1: "reliance on nonland mana sources"]** | Hold creatures back, keep a counter up, Kinnan comes back from the command zone **[inference]** |
| Stifle effects on Horror's trigger; "can't cast" effects | Stops the loop trigger | Loop with Monolith (no trigger) instead **[inference]** |
| Fast combo decks that win before turn 4 | Its win comes on turn 3–5 | Mindbreak Trap, free counters, Rhystic Study; goes first when it can **[primer: NBC, Hand 4 for Trap; rest inference]** |
| Null Rod, Collector Ouphe, Stony Silence, Drannith Magistrate | Turns off Moxen, rocks and Monolith **[primer S1]** | Boseiju, Chain, Otawara, Sink into Stupor on the piece; counter it on the cast with Fierce Guardianship or Force of Negation **[primer S1]**. P calls the deck resilient to stax: Kinnan's flip and the draw engines still work **[primer: NBC]** |

## Against Godo

P has no content on Godo or any creature combo. Everything below is [inference] from the oracle text, except where it matches P's use of Chain of Vapor,
Otawara and Horror as the deck's bounce and its counterspell suite **[primer: NBC, Card Choices]**.

Godo (mono-red, {5}{R}) fetches Helm of the Host when he enters, and Helm makes a hasty token Godo at each combat; the backup is Dualcaster Mage plus a
copy spell **[from decklists/JD/godo-cedh-equipment.md]**. Kinnan's answers, by step **[all inference from oracle text]**:

| Godo step | Answer in Kinnan's list | Notes |
| --- | --- | --- |
| Cast Godo (a creature spell, {5}{R}) | Force of Will, Pact of Negation, Disrupting Shoal (X=6: pitch Consecrated Sphinx), Hullbreaker Horror trigger (return the spell) | Fierce Guardianship, Force of Negation, Offer and Commandeer cannot target a creature spell. Mental Misstep only hits MV 1 spells (Sol Ring, Mana Vault; not the MV 0 Moxen) |
| Godo's enter trigger | Nothing stops it. It fetches Helm | Counters on abilities are not in the list |
| Helm enters (it attaches free via Hammer of Nazahn) | Chain of Vapor or Snapback on Godo with the attach trigger on the stack; Boseiju ({G} with Kinnan) on Helm | Best window. Without Godo on the battlefield, Helm has nothing to copy. Helm is an artifact, so Commandeer cannot take it once cast (it is not a spell by then); Fierce Guardianship / Force of Negation can counter the Helm spell if it is cast |
| Equip {5} on the stack | Snapback or Chain on Godo | The ability fizzles if Godo leaves |
| Beginning of combat (Helm trigger) | Bounce Godo with Sink into Stupor or Chain; Horror bounce on each spell cast | Tokens are copies; bounced tokens cease to exist |
| Dualcaster Mage line: copy spell (Twinflame etc.) | Fierce Guardianship, Force of Negation, Dispel (instant), Swan Song (sorcery), Flusterstorm, Offer, Mindbreak Trap; **Commandeer** can take the copy spell (noncreature) | Counter the copy spell before Dualcaster is flashed in |
| Dualcaster Mage (creature spell, MV 3) | Force of Will, Pact, Disrupting Shoal (X=3) | Chain / Snapback after it enters |

Kinnan's own win (about turn 3–5) is no faster than Godo's (turn 2–3) **[inference]**, so it must hold a creature-capable counter (Force of Will, Pact) on turns 2–3. P's own Kinnan win loops would also beat a Godo on board if they finish first, since A and B need no creature combat. **For the Godo spec:** Godo should play around Force of Will and Pact on Godo, bounce on Godo with the trigger on the stack, and Hullbreaker Horror (uncounterable) bouncing his tokens. Most of Kinnan's counters (Fierce Guardianship, Force of Negation, Offer, Commandeer) cannot hit Godo, so Godo should expect only Force of Will, Pact, bounce and Horror. Godo should also expect Commandeer and Fierce on his Helm / copy spell.

## Decision scenarios (become AI tests)

| # | Position | Right action | Reason | Tag |
| --- | --- | --- | --- | --- |
| 1 | Turn 2, you hold Island, Forest, Birds (in play), Kinnan, Force of Will + a blue card. Opponents show two untapped blue mana | Cast Kinnan, keep Force of Will up for the next spell | Kinnan is cheap and survives one removal; the engine matters more than bait | [primer S3; P Hand 7 plays Kinnan turn 2 the same way] |
| 2 | Turn 3, Kinnan out, 8 mana, hand: Basalt Monolith, Thrasios, Fierce Guardianship, no creature counter. Opponent has three open blue mana and probably Force of Will | Do not cast Monolith yet; flip Kinnan at the end of their turn, or cast Thrasios first as bait | P: make opponents fight over other cards before the key spell; the Kinnan ability cannot be countered | [primer: NBC: bait before the key spell; Kinnan backup plan] |
| 3 | Opponent casts Null Rod while you hold Fierce Guardianship, Force of Negation, 3 mana | Counter it with Fierce Guardianship (free) | The stax hits your rocks and Monolith; Fierce is free and Kinnan is out | [primer S1: stax is the main weakness; P calls the deck resilient] |
| 4 | Opponent resolves Collector Ouphe; you hold Boseiju, Kinnan out, {G} open | Channel Boseiju ({G} with Kinnan) on the Ouphe | The channel costs one mana with a legendary creature | [inference from oracle] |
| 5 | Your end step, 8 mana open, no combo ready, Rhystic Study in play, opponents tapped low | Activate Kinnan ({5}{G}{U}); take Hullbreaker Horror if rocks are in play, otherwise Sphinx / Thrasios | Counters cannot hit the ability; top five at end step is free value | [primer S3; P: flip as backup plan] |
| 6 | Kinnan out, Hullbreaker Horror in play, Mox Amber and Lotus Petal in hand, no stax, opponents have 2 blue mana open | Start the loop only while holding a free counter or one spare zero-cost spell. Cast Lotus Petal first, then Amber. If they counter a loop spell, cast the spare spell: Horror returns their counter | A counter on a loop spell ends the loop. Horror bounces a spell on every cast, so a spare cast answers it | [primer: NBC for the loop; the backup is inference] |
| 7 | Two opponents threaten wins: Player A needs one more spell to win, Player B has a Rhystic-style engine | Counter Player A's winning spell with the best free counter; ignore B | The one closest to winning is stopped first | [inference] |
| 8 | Basalt Monolith resolved, Kinnan out, Thrasios in play, 4 colourless spare, library 40 cards | Run the loop, activate Thrasios until about 5 cards remain, keep one card in the library, then cast creatures and Finale for X | Thrasios puts lands in and draws; Finale also searches the graveyard | [primer: NBC for the outlet and Finale; library count inference] |
| 9 | You cast Pact of Negation on a combo piece; you have 6 lands next turn | Fine: cast it | Next upkeep {3}{U}{U} is 5 mana | [inference] |
| 10 | Opponent casts Godo (the Helm trigger on the stack after Godo enters); you hold Snapback, Chain of Vapor, 3 mana | Wait until Helm is attached (or in response to the attach or equip ability) and bounce Godo | Helm has nothing to copy; bounce is cheaper than a counter | [inference] |
| 11 | Mulligan: Island, Forest, Birds, Elvish Mystic, Kinnan, Force of Will, Consecrated Sphinx | Keep | Two lands, two dorks, Kinnan, a free counter; same shape as P's Hand 7 | [primer: NBC, Hand 7] |
| 12 | Mulligan: one land (Ancient Tomb), Mox Diamond, Rhystic Study, Delighted Halfling, two spells (Commandeer, Swan Song) | Mulligan | One land: "the downside of missing our first land drop is huge"; keep only at four cards | [primer: NBC, Hand 3] |
| 13 | Mulligan going last: Fyndhorn Elves, Delighted Halfling, The One Ring, Valley Floodcaller, two lands | Mulligan (keep at 5 or 6) | The turn-3 One Ring plan is slow going last | [primer: NBC, Hand 5] |
| 14 | Mulligan: Sol Ring, Talisman of Curiosity, Mystic Remora, Island, Forest | Keep, any seat | Turn 1 Sol Ring, Talisman, Remora | [primer: NBC, Hand 1] |
| 15 | An opponent casts Rhystic Study; you have Commandeer and two blue cards in hand | Cast Commandeer for free, take Study | Commandeer takes any noncreature spell; the engine pays you | [primer: NBC, Hand 2] |
| 16 | Kinnan out, Sol Ring in play, Transmute Artifact and Basalt Monolith in the library, 5 mana, opponents hold counters | Cast Transmute Artifact, sacrifice Sol Ring, pay {2}: Monolith enters without a cast | A counter on the Monolith cast is avoided; only a counter on Transmute stops it | [primer: NBC "or the ability to tutor it into play"; the cost is oracle] |
| 17 | Infinite mana from line A (colourless), Mastermind and Endurance in play, no {U} source | Do not start the Mastermind line yet: get {U}{G} from Mirage Mirror (or cast Trophy Mage for it) first | Mastermind needs {U} each time | [inference from oracle; P shows the loop] |
| 18 | Infinite mana, Finale in hand, you have 4 creatures, opponents have 3 blockers total | Do not cast Finale yet; add creatures first | P: creatures in play must equal blockers + 3 (here 6) | [primer: NBC, Finale requirements] |
| 19 | Opponent has only 1 mana open, you can start line E and have most of your counters | Start now, at instant speed on the turn of the player to your left | Their interaction is limited by mana; the order of Tithe triggers favours you | [primer: NBC, Mastermind timing] |
| 20 | Mid-game, no combo in hand, 7 mana, a Rhystic Study and Remora on your side, opponents have lots of open mana | Do not force a win attempt; draw cards and wait for the window | "We want to draw cards until the ideal window presents itself" | [primer: NBC, Strategy] |
| 21 | You hold Mindbreak Trap and the opponent has cast two spells in a turn; the third is the visible win attempt | Cast Mindbreak Trap free (three spells) and exile them | It stops an early win attempt | [primer: NBC, Hand 4] |
| 22 | The Mastermind line needs to resolve Endurance; opponents hold counters and you hold Chord and Trickster | Cast the payoff spells (Horror, Trickster, Chord) first and make them spend counters, then Endurance | P's order: trade payoffs for counterspells first | [primer: NBC, Mastermind execution] |

## Combo tests (become scripted tests)

| Line | Starting board and hand | Expected result |
| --- | --- | --- |
| A. Basalt Monolith + Thrasios | Battlefield: Kinnan, Thrasios, Island, Forest, Breeding Pool, Sol Ring, Birds of Paradise (all untapped). Hand: Basalt Monolith, Finale of Devastation. Library has creatures and lands. Opponents: no interaction | Cast Monolith (3). Tap: 4 colourless; pay 3; repeat. Net +1 per loop. Thrasios drawn to about 5 cards left. Finale with X of 10 or more: creatures +X/+X and haste. Attack with the fetched creature and any flier. Result: all three opponents dead |
| B. Hullbreaker Horror + Mox Amber + Lotus Petal | Battlefield: Kinnan, Hullbreaker Horror, Mox Amber, Thrasios, three lands. Hand: Lotus Petal, Finale of Devastation | Cast Petal: Horror returns Amber. Cast Amber: Horror returns Petal. Each loop: Amber taps for 2 (Kinnan). Net positive in {U}{G}. Use Thrasios and Finale; win as in A |
| B2. Horror + Sol Ring + Mox Amber (P's example) | Battlefield: Kinnan, Hullbreaker Horror, Sol Ring, three lands. Hand: Mox Amber, Finale | Cast Amber (Horror returns Sol Ring), tap Amber; cast Sol Ring (returns Amber), tap Sol Ring for 4; repeat. Expected: unbounded mana, then as A |
| C. Horror lock (no combat) | As B, but opponents control three permanents each, two cheap spells in hand | Each spell cast bounces an opposing permanent; after enough casts every opposing nonland permanent is in hand. The test checks that no opposing permanent remains |
| D. Finite big turn | Battlefield: Kinnan, Birds, Llanowar Elves, Elvish Mystic, Sol Ring, Grim Monolith, Mana Vault, 5 lands. Hand: Dramatic Reversal, Finale of Devastation | Tap everything, cast Reversal, tap again. Expected: about 37 mana (22 first tap, 17 more after Reversal, 2 spent), enough for Finale with X of 10 or more. The test checks the mana count and that the AI casts Reversal after tapping, not before |
| E. Mastermind decking | Line A (with Mirage Mirror for {U}{G}) or B done, infinite mana. Battlefield: Kinnan, Faerie Mastermind. Hand: Endurance, Chain of Vapor. Graveyard: 15 cards. Libraries: ours 20, each opponent 60 | Cast Endurance (graveyard to the library bottom: ours 35). Activate Mastermind until our library is near empty; recast Endurance via Chain of Vapor; opponents draw from empty libraries first. Expected: all three opponents lose; we keep at least one card in library until the last draw |
| F. Mirage Mirror colour | Battlefield: Kinnan, Basalt Monolith with the loop running, Mirage Mirror, Tropical Island | Mirror copies Tropical Island, tap for {U}/{G}; Mirror copies Monolith, untap; repeat. Expected: infinite {U}{G} as well as colourless |
| Negative 1 | A with an opposing Null Rod on the battlefield | Monolith cannot tap for mana; the AI does not cast it as a win attempt and instead removes Null Rod (Chain, Boseiju, Otawara) or waits |
| Negative 2 | D1 with 3 creatures in play and 3 opposing blockers (needs 6) | The AI does not cast Finale for the attack; it adds creatures first |
| Not testable | Coliseum, Nyxbloom, Tithe (unless an opponent has Tithe), Bribery, Oracle finishes | Cards not in our list |

## Open questions

1. **Win without a finisher.** Our list's creature finish is Finale of Devastation and Mastermind decking only. P lists a dozen creature finishers (Thassa's Oracle etc.) that we do not run, so
   if an opponent has a fog or flying blockers, only Mastermind + Endurance is left. P does not discuss this.
2. **Emrakul, the Promised End.** It is in the tournament list but not in P or any other source. This spec treats it as an unplayed 13-drop; a Kinnan flip does not trigger its cast ability.
3. **Thrasios is the only listed card-draw outlet** (plus Wan Shi Tong, which P says to use when lands must be drawn). If Thrasios is removed, line A has the Kinnan flip only. P's answer is Green Sun's Zenith or Chord to find another; we have Chord, Nature's Rhythm and Ikoria.
4. **Pace.** P says "patience" and "game might be over by turn 3"; it gives no turn-by-turn plan. The turn 3–5 target is inferred.
5. **Interaction priority and the two-threats rule.** P and every other source are silent. Scenarios 7, 10, 17 are inferred.
6. **Pact of Negation.** The deck should cast it only when it can pay or wins; no source says so.
7. **Stax.** P says "resilient to stax"; S1 lists Null Rod, Ouphe and Magistrate as the weakness. The spec keeps both and the AI should treat Null Rod as a serious threat only to lines A and B (it does not stop Kinnan's flip or draw).
8. **Mulligan hands 4 and 5 and the FAQ.** P's mulligan guide is marked "UPDATE IN PROGRESS", so it may change; its collapsed FAQ was not readable. Whether a draw-engine hand with no mana (five lands, Study, Library) is a keep is not covered; by P's order (engine first) it is a keep, but this is inference.
9. **Dramatic Reversal.** Resolved by P: it is listed only as an untapper ("Untappers: Dramatic Reversal"); no Isochron Scepter combo. The pool file's note "Basalt Monolith, Dramatic Reversal" is not a P line. Model it as ramp (line D2).
10. **Faerie Mastermind + Tithe (F)** needs an opponent's Smothering Tithe. The lead decides whether to script it (Tier 6 pools may not run Tithe).
