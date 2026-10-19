# Sisay, Weatherlight Captain (Legends Toolbox) — Behaviour Spec (Tier 6)

Sources (all opened 2026-10-17; full links in the Sources list at the end). **The dedicated primer [TE] outranks all the
others and wins any conflict:** [TE] "Oath of Sisay" by Tolaria_East, Moxfield primer, updated 2026-08-29 (read from JD's
printout, extract in `/home/joseph/projects/mtg_sims/primers/sisay-tolaria-east.md`; it is the author's own list, which differs
from ours). Tag **[primer: Tolaria_East]** = said by [TE].
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
[TE] is the best source but is written for the author's "Oath of Nicol" list. It names these cards that **our list does not
have**: Oath of Teferi, Nicol Bolas, Dragon-God, Aminatou, Saheeli Rai, Orcish Bowmasters, Faeburrow Elder, Jegantha, Chromatic
Orrery, Dihada, Wan Shi Tong, Dockside, Strix Serenade, Orim's Chant, Suspend, Into the Flood Maw, Dispel, Assassin's Trophy.
Lines and rules that need them are marked "not in our list". Cards of [TE] that we do have: Bloom Tender, Derevi, Emiel, Ertai,
Marvin, Ioreth, Shang-Chi, Selvala, Kinnan, Tyvar, Agatha's Soul Cauldron, Gaea's Cradle, Mount Doom, Enduring Vitality, Mana
Vault, Deadpool, Teferi (Time Raveler), Abrupt Decay, Cyclonic Rift, Fierce Guardianship, Mental Misstep, Mindbreak Trap, Force of
Will, Silence, An Offer You Can't Refuse, Flusterstorm, Swan Song, Legolas's Quick Reflexes, Deflecting Swat, Redirect Lightning,
Colossal Skyturtle, Chord of Calling, Eladamri's Call, Worldly, Vampiric, Enlightened and Demonic Tutor, Green Sun's Zenith.

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
**[primer: Tolaria_East]** [TE] adds: the deck is a five-colour adaptive combo toolbox, an "inevitable bringer of doom": as long
as land drops and legends keep landing, the win gets nearer. "Putting value on the table is the baseline of the gameplan." It
does not aim to combo on turn 2 but must ramp hard: the target is **double the mana each turn (turn 1: two, turn 2: four, turn 3:
six)**. "Counterspells typically do not affect us" (tutored permanents are not cast); the author's #1 enemies are Opposition
Agent and Cursed Totem, and "protection is but one spin away" (Sisay can fetch it). Post-Dockside, the build focuses on mana and
on putting win attempts on the stack efficiently. The author calls it a consistent top-10 deck. Longer games favour Sisay: "the
longer the game goes on the more inevitable we become."

Sisay's numbers **[primer, Oracle]**: she is 2/2 for {2}{W}. She gets +1/+1 for each colour among *other* legendary permanents
you control, so at most 7/7. Her ability fetches a legendary permanent with mana value *less than* her power, so power 2 finds
MV 0-1 only, power 3 finds MV 2, power 4 finds MV 3, power 5 finds MV 4. Each activation costs a full WUBRG (5 mana).
**Oracle check, 2026-10-17 (Scryfall):** "Search your library for a legendary permanent card with mana value **less than Sisay's
power**." [TE]'s own words say "based on Sisay's power"; its aside "MV <= number of legendary permanents" is the author's
shorthand and wrong. Power = 2 + colours among *other* legendary permanents. [TE]'s chain rule "Sisay power >= 4 finds Derevi /
Tyvar (MV3)" fits the Oracle. Mount Doom, Gaea's Cradle and Mox Amber (all colourless, MV0) are reachable at power 1-2;
colourless legends add no power.

## Win lines
Mana is counted from an empty board with every piece cast from hand. Pieces fetched by Sisay cost her activation (5 coloured
mana each) instead of their cost, but need her power high enough (see above).

| Line | Pieces (in our list) | Mana (total, colours) | Steps | Where it can be answered (step, by what) |
| --- | --- | --- | --- | --- |
| **1. Mana loop, Bloom Tender + Derevi + Emiel ("Flash Unicorn" in [TE])** **[primer: Tolaria_East]** [TE][A][G] | Bloom Tender (non-legendary: draw or tutor it), Derevi, Emiel, Sisay as the tutor. Finisher: Mount Doom. Kinnan adds +1 mana per Bloom Tender tap **[inference]** | Cast from hand: Bloom Tender 2 (1G) + Derevi 3 (GWU) + Emiel 4 (2WW) = 9, then 3 more per loop. Five colours among your permanents make Bloom Tender tap for WUBRG (5). Via Sisay: Sisay 3 (2W) + one 5-mana activation per piece | 1. Have Bloom Tender, Derevi, Emiel and five colours among permanents. 2. Tap Bloom Tender for 5. 3. Pay {3}: Emiel blinks Derevi. 4. Derevi's enter trigger untaps Bloom Tender. 5. Repeat: net +2 per loop (+3 with Kinnan). Pay the 5 for Sisay activations. Finish: see below | Before Sisay's activation: kill Sisay or the mana creature. In response to Emiel's ability: kill Derevi or Emiel (the loop dies if Derevi is exiled). On Derevi's trigger: Ertai-style counters or Stifle effects. Afterwards: graveyard hate, search hate, activated-ability hate (Cursed Totem) [A] |
| **2. Cradle loop, Derevi + Emiel + Gaea's Cradle + Agatha's Soul Cauldron** **[primer: Tolaria_East]** [TE][A] | Derevi, Emiel, Cradle, Cauldron (Cradle and Cauldron are legendary, so Sisay fetches them). Needs 4+ creatures | Cast: Derevi 3 + Emiel 4 + Cauldron 2 = 9, plus creatures to feed Cradle. Cradle makes G per creature. Cauldron lets green mana pay creature abilities (Sisay's) as any colour. 3 per loop | Tap Cradle for 4+ G. Pay {3}: Emiel blinks Derevi. Derevi untaps Cradle. Repeat. Surplus G pays Sisay via Cauldron | Same as line 1, plus: kill creatures in response (Cradle shrinks), artifact removal on Cauldron, land removal on Cradle |
| **3. Untap loop ("DDR"), Marvin + Ioreth + a legendary mana source** **[primer: Tolaria_East]** [TE][A][E][J] | Marvin, Ioreth, plus one other legendary creature that makes mana: Selvala, Shang-Chi, or any legend with Enduring Vitality [A]. **[inference]** Cryptolith Rite (same text as Vitality) and Relic of Legends (taps legends for mana) also work; Sisay is a legend too. Tyvar or Shang-Chi give haste to abilities | Cast: Marvin 2 + Ioreth 3 (2U) + Selvala 3 (1GG) = 8; with Shang-Chi instead: 2+3+2 = 7 and the combo works the turn they land (Shang-Chi lets abilities ignore summoning sickness). Shang-Chi's mana only pays creature abilities **[inference]**, so Mount Doom needs other mana | Marvin has Ioreth's ability. Ioreth (tap) untaps Marvin and the mana source; Marvin (tap, Ioreth's ability) untaps Ioreth and the mana source. The source taps for mana each time. Selvala needs {G} and makes X (greatest power, at least 2) so net +1 or more | Needs Marvin and Ioreth free of summoning sickness [E]. Answered by killing or bouncing Marvin or Ioreth with the loop's first activation on the stack, or a Cursed Totem effect. It uses no spells, so counterspells do nothing |
| **4. Finisher: Mount Doom** **[primer: Tolaria_East]** [TE][A][G] | Mount Doom (legendary land; Sisay fetches it with power 1+) | {1}{B}{R} and tap per 1 damage to each opponent. Needs an untap per ping: Derevi (extra blink = 3) or Marvin/Ioreth | With infinite mana, each ping = 3 for Mount Doom + one extra untap (3 for another Emiel blink). 40 pings for a 40-life opponent | Kill the untap source (Derevi/Emiel or Marvin/Ioreth) before the next ping. Damage already dealt stays. Lifegain from opponents raises the number of pings. Mount Doom itself can be destroyed or bounced in response to an activation |
| **5. Finisher: Deadpool + Forbidden Orchard** **[primer]** [A] | Deadpool, Trading Card, Forbidden Orchard, Emiel, infinite mana | Deadpool 4 (2BR) | With infinite mana tap Orchard repeatedly: each tap gives an opponent a 1/1 Spirit. Emiel blinks Deadpool; as it enters it swaps text boxes with a Spirit. The Spirit now says "at the beginning of your upkeep, you lose 3 life". Repeat per Spirit. Opponents lose 3 life per Spirit at their upkeep [A] | Slow: it wins at an opponent's upkeep, so it gives them a turn. Kill the Spirits, or win before upkeep. **[inference]** a faster finisher is Mount Doom |
| **6. Last resort: Sisay beats** **[primer]** [H] | Sisay with 5 coloured legends is 7/7 | n/a | Attack | Chump blockers, any removal. [H] says this means the deck is near the bottom of its win options |
| **7. Selvala chain: end-step tutor, then Derevi + Emiel** **[primer: Tolaria_East]** [TE] | Sisay (power 5+), Selvala, Derevi, Emiel, rocks and dorks for the seed | Sisay 3 (2W) + two activations 10 (WUBRG each) + {G} to start Selvala + {3} per Emiel blink. Mana needed from other sources: about 14 over two turns **[inference]** | [TE]: at the opponents' end step, activate Sisay, fetch Selvala. Your turn: activate again, fetch Derevi; Derevi untaps Selvala; Selvala makes X mana (X = greatest power, 5 with Sisay at 5/5) for {G}; activate again, fetch Emiel (MV4 needs power 5: Selvala G + Derevi GWU = three colours, Sisay 5/5); Emiel blinks Derevi ({3}) and Derevi untaps Selvala. Net about +1 per loop (5 - {G} - 3) **[inference from Oracle]**: infinite. Finish: Mount Doom | Same windows as line 1. Extra: removal on Selvala during the opponent's end-step window (she is tutored a turn early so she is not summoning-sick on your turn) |
| **8. Rainbow-dork chain** **[primer: Tolaria_East]** [TE] | A dork that makes WUBRG (Bloom Tender; with Enduring Vitality or Cryptolith Rite every creature taps for any one colour **[inference]**), Sisay power 4+, Derevi or Tyvar | Dork 2 + Sisay 3 + Derevi (fetched, 5 mana) | Tap the rainbow dork for WUBRG. Pay for Sisay: fetch Derevi (untaps the dork) or Tyvar (+1 untaps a creature; gives abilities haste). Tap again. Chain to Emiel and then line 1 | Kill the dork or Sisay in response; Cursed Totem; Opposition Agent (Sisay's search is taken over) |
| **9. Mana Vault + Kinnan + Cauldron starter** **[primer: Tolaria_East]** [TE] | Mana Vault, Kinnan, Agatha's Soul Cauldron, "3 extra mana" | [TE] gives only the pieces. Vault taps for {C}{C}{C} and Kinnan adds one more mana of that type (nonland permanent tapped), so 4 colourless, plus the "3 extra mana" [TE] mentions to reach 5; Cauldron lets colourless pay Sisay's WUBRG **[inference from Oracle; Kinnan and Cauldron checked, Vault from memory]** | Tap Vault, Kinnan doubles, Cauldron converts: one free Sisay activation. Then fetch the missing loop piece | Kill Kinnan or Cauldron in response; artifact removal on Vault/Cauldron |
| **10. Flash Unicorn on an opponent's turn** **[primer: Tolaria_East]** [TE] | Lines 1 or 2 with Emiel + Derevi; both abilities are instant speed | As line 1 or 2 | Emiel's {3} and Derevi's enter trigger have no timing limit, so the loop can run in response to an opponent's spell or at their end step. Finish with Mount Doom pings (Derevi or Ioreth untaps it) | Only abilities: counterspells do nothing. Ertai, Stifle effects, removal on Emiel/Derevi |
| **11. "Ertai lock" (not a win)** **[primer: Tolaria_East]** [TE] | Emiel + Derevi + infinite mana + Ertai Resurrected (all in our list). [TE]: "author not currently equipped" | Ertai blinks cost {3} each | Each blink of Ertai counters a spell or ability (its controller draws a card **[Oracle]**), so every response is countered. A lock, not a win | The cards drawn give opponents more outs; finish with Mount Doom as soon as possible **[inference]** |
| **12. "Oath of Bolas": Oath of Teferi + Nicol Bolas, Dragon-God + Aminatou** **[primer: Tolaria_East]** [TE] | **Not in our list** (no Oath of Teferi, Nicol Bolas or Aminatou) | n/a | [TE]'s infinite planeswalker-activation line. Our Teferi, Time Raveler and Teferi, Who Slows the Sunset do not loop. Do not model | n/a |
| **13. Finisher variants: Saheeli +1 pings, Orcish Bowmasters flickers** **[primer: Tolaria_East]** [TE] | **Not in our list** (no Saheeli Rai, no Orcish Bowmasters) | n/a | Our finishers are Mount Doom and Deadpool (lines 4-5). With infinite mana Kinnan's {5}{G}{U} can be repeated to put any non-Human creature from the library onto the battlefield (Deadpool among them) **[inference from Oracle; extends [TE]'s Kinnan-to-OBM idea]** | n/a |

[TE]'s DDR (line 3) names Shang-Chi for haste and "a legend that taps for mana: Sisay with Enduring Vitality, Esika, Jegantha". In
our list: Sisay or Selvala with Enduring Vitality; Cryptolith Rite or Relic of Legends **[inference]**. Esika and Jegantha are not
in our list. [TE] also notes that "15 mana with a big enough Sisay (three activations)" starts a chain without any loop piece.
Line 5 (Deadpool + Orchard) comes from [A] only; [TE] does not list it as a win.

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
**From [TE] (these outrank the [inference] lines above):**
- **[primer: Tolaria_East]** The ideal hand plays "a bunch of dorks out as fast as possible" and doubles mana each turn (two on
  turn 1, four on turn 2, six on turn 3). It does not need to combo on turn 2.
- **[primer: Tolaria_East]** "If it makes a mana, it is probably good enough. Mana is the most important thing." So a hand with
  two or more mana makers is a keep; a hand of legends and tutors with no mana is not.
- **[primer: Tolaria_East]** Tutors count as ways in: any rainbow dork (Bloom Tender) plus tutors gets "up the chain". A hand with
  a rainbow dork and one tutor is a keep.
- **[inference]** [TE] says nothing about hands to ship or about mulligan to what count. Ship rules above stay [inference from H].
- **[inference]** Do not count Force of Will as a keep reason: [TE] doubts it (needs about 30% blue cards; pitching Teferi,
  Derevi or other blue engine pieces can leave you unable to win).

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
**Tutor timing [primer: Tolaria_East].** "Tutor on the end step before your turn, then another on your own turn." Fetched pieces
are then not summoning-sick on your turn and Sisay untaps first. Example (line 7): end step, fetch Selvala; own turn, fetch
Derevi, untap Selvala, tutor Emiel, blink Derevi. Chord of Calling at an end step is the other end-step play [TE] names (haste
matters on your own turn; Tyvar and Shang-Chi give it).
**Roles in [TE] for cards in our list [primer: Tolaria_East]:** Teferi, Time Raveler is "a Silence through one spin" (opponents
only at sorcery speed; -3 bounces anything). Smothering Tithe is stax and ramp in draw-heavy tables. Rhystic Study, Mystic Remora
and Esper Sentinel are draw engines, but drawing combo pieces is awkward since they are tutored onto the battlefield. Lavinia is
stax against turbo decks. Marvin gives pseudo-haste (DDR). Deadpool copies a creature's text (steals commander abilities).
Loki's Scepter gives haste and steals. Lotho is two coloured pips and a Treasure. Agatha's Soul Cauldron goes up the Unicorn or
Oath line with Cradle (5 creatures); worst case it lets rocks (Mana Vault, Sol Ring) pay for spins; if Bloom Tender dies the
Cauldron can give its ability to another creature. Kinnan is "hidden Kinnan": usually the first low spin, and its own infinite
mana is a backup to Sisay. Enlightened Tutor finds Cauldron, Vitality, Rhystic. Orcish Bowmasters, Dihada, Wan Shi Tong, Jegantha,
Faeburrow Elder, Orrery are not in our list.
**First fetch [inference, with [TE]]:** [TE] calls Kinnan (MV2) the first low spin, which needs Sisay power 3, i.e. one other
coloured legend in play. In our list the only coloured legend that Sisay can reach naked (power 2, MV 0-1) is Ragavan (scenario 3).
[TE]'s Cultist of the Absolute (+4 power) is not in our list.

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
aimed at the engine), saving Silence for the turn it combos. **[Superseded by [TE], below: Mental Misstep is not for fast mana.]** Ertai for a trigger or a
creature that threatens the engine. The sources do not give a spend table.
**Lets resolve [inference]:** opponents' ramp and non-threatening permanents, since the card-draw enchantments (Rhystic Study,
Remora) tax them anyway.
**What [TE] says about each card in our list [primer: Tolaria_East]:**
| Card | Rule from [TE] |
| --- | --- |
| Mental Misstep | "Reserve this card to counter other missteps, other counterspells, Demonic Consultation, or 1-cmc tutors." NOT for Sol Ring, Mana Vault or dorks. **[inference]** In our pod: opponents' MV1 tutors (Vampiric Tutor, Worldly Tutor) and MV1 counters; the other Mental Misstep is the only one that matters |
| Force of Will | The author doubts it: needs about 30% blue cards; pitching blue enablers (Teferi, Derevi) can leave you unable to win. Pitch only spare blue cards |
| Fierce Guardianship | Stop others' wins or protect your own (noncreature spells only; free only with Sisay on the battlefield) |
| Silence | Stop a win on their turn, or protect yours |
| Mindbreak Trap | "Sisay needs to get back to my turn." Free when the stack gets big or the opponent casts a multi-spell turn |
| An Offer You Can't Refuse | Counters noncreature spells; can counter your own spell to make mana |
| Legolas's Quick Reflexes | Split-second protection; untaps to continue a chain; also removal |
| Abrupt Decay | Removes Opposition Agent and Cursed Totem; cannot be countered |
| Swan Song, Flusterstorm, Deflecting Swat, Redirect Lightning, Colossal Skyturtle | Listed as interaction; no rule given |
| Ertai Resurrected | Counter, removal or "Stifle" and the lock piece |
| Teferi, Time Raveler | Protection through one spin of the engine |

**Spend interaction first on these ("uh oh" cards) [primer: Tolaria_East]:** Opposition Agent, Blood Moon / Magus of the Moon /
Back to Basics, Cursed Totem, Orcish Bowmasters, Suppression Field, Weathered Runestone, Drana / Linvala, Containment Priest,
Grafdigger's Cage, Torpor Orb, Null Rod (somewhat), Drannith Magistrate and the Otawara / Boseiju channel lands (all as listed
in [TE]). Lesser (let them resolve, "they hinder others more"): Rule of Law, Sphere of Resistance, Gaddock Teeg, Lavinia,
Trinisphere, Defense Grid. "Sisay can usually tutor in response to someone attempting to win."
**Our answers per target [inference from our list and Oracle, checked against [TE]'s rule]:** Opposition Agent (flash creature, MV3):
Force of Will or Pact of Negation on the spell, Ertai (destroy), Abrupt Decay (MV3, uncounterable), Cyclonic Rift. Cursed Totem
(artifact, MV2): An Offer You Can't Refuse, Force of Will, Pact on the spell; Abrupt Decay, Cyclonic Rift, Boseiju (channel) on
the permanent. Blood Moon (MV3, enchantment): the same, and it is the worst for us because the list has **28 nonbasic lands and
no basic** (every land would make only {R}); the answer must come before our turn. Strix Serenade, Orim's Chant, Suspend, Into the
Flood Maw, Dispel are [TE] cards **not in our list**.
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
- **From [TE] [primer: Tolaria_East]:** the win attempt should be "put on the stack efficiently": the deck uses ability-only
  loops (Flash Unicorn, DDR) that no counterspell touches, and can run them on an opponent's turn (line 10) when the opponent has
  tapped out. It tutors at the opponents' end step first (line 7) so the loop starts on its own turn with no summoning sickness.
  It also uses Sisay's ability in response to an opponent's win attempt (fetch the answer). The longer the game, the more it
  favours waiting: with a hate piece out that is a "concerning but not a blowout" (Rule of Law, Sphere of Resistance, Lavinia,
  Trinisphere) it keeps going; with an "uh oh" piece (Opposition Agent, Cursed Totem, Blood Moon, Bowmasters) it first spends
  interaction on the piece.

## What it fears
| Fear | Source | Play around |
| --- | --- | --- |
| Multiple layers of stax (Opposition Agent, Cursed Totem, Weathered Runestone) | [H] | Default to card draw and removal [H]; Abrupt Decay hits MV3 or less; Cyclonic Rift; Touch the Spirit Realm |
| Removal on Sisay or on a mana engine | [A] | "Protect the engine, not the commander": hold counters/Silence; commander tax makes recasting costly |
| Activated-ability, search and graveyard hate | [A] | Pieces arrive without spells; keep Ertai and Swan Song for hate pieces |
| Mana-base strain (WUBRG) | [J] | The deck runs 28 lands, many duals, fetches, Mana Confluence, City of Brass, Exotic Orchard |
| Fragile utility creatures (Orcish Bowmasters) | [J] | [J] cut the Queen of Dale for this; ours keeps her **[inference: more fragile engine]** |
| Chump blockers vs the beat-down line | [G] | Use loops, not combat |
| Opposition Agent, Cursed Totem ("our #1 enemies") | [TE] | Spend the first interaction on them: Abrupt Decay (uncounterable, hits both), Rift, counters on the spell; Ertai on the Agent |
| Blood Moon / Magus of the Moon / Back to Basics | [TE]; our 28 nonbasic lands make it worse **[inference]** | Counter or Decay on sight; do not tap out into it |
| Orcish Bowmasters, Suppression Field, Weathered Runestone, Drana / Linvala, Containment Priest, Grafdigger's Cage, Torpor Orb, Null Rod, Drannith Magistrate, Otawara, Boseiju | [TE] "uh oh" list | Interaction goes here before other threats |
| Rule of Law, Sphere of Resistance, Gaddock Teeg, Lavinia, Trinisphere, Defense Grid | [TE] "concerning, not blowouts" | Ignore; they hinder others more and the game gets longer for Sisay |
| Thassa's Oracle / Demonic Consultation from opponents | [J] | Gleaming Splendor is [J]'s still-being-tested answer; Spider-Woman does not stop it |

## Against Godo
How this deck stops Godo's lines (Oracle text checked):
- **Godo cast (creature spell, {5}{R}).** Force of Will and Pact of Negation can counter it. Fierce Guardianship, An Offer You
  Can't Refuse and Swan Song cannot (not creature spells), and Flusterstorm cannot (instant/sorcery only). Mental Misstep
  is [TE]-reserved for counters and 1-mana tutors, so it is not spent on the 1-mana rocks that pay for Godo. Lavinia counters any spell cast with no mana spent (Godo's free spells, a Force of
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
   second turn-1 mana spell. Action: play Birds-line, keep FoW for a combo piece, do not Misstep Sol Ring or any mana rock. Reason: [TE] reserves Misstep
   for counters, Demonic Consultation and 1-mana tutors, not for Sol Ring, Vault or dorks. Tag **[primer: Tolaria_East]**
   (replaces an earlier [inference] that allowed Misstep on fast mana).
2. **Sisay on turn 2.** Hand: Sisay, two lands plus a Mox. Open table. Action: cast Sisay with the Mox (3 mana), no counter
   backup needed. Reason: [A] puts Sisay on turns 2-3. Tag **[primer: A]**.
3. **First activation target.** Sisay in play (power 2), WUBRG available. Action: fetch Ragavan (MV1, red, raises power to 3), then use the next activation at power 3 for an MV2 piece (Kinnan, Lavinia, Marvin, Shang-Chi). Reason: power 2 only reaches MV 0-1 (Oracle). Tag **[inference from Oracle]**.
4. **Mana loop available.** Bloom Tender, Derevi and Emiel on board, five colours present, opponents have 2 open Islands.
   Action: wait one turn unless Silence is in hand; otherwise cast Silence and go. Reason: the three answer windows [A]. Tag
   **[primer: A]**.
5. **Protect the engine.** Sisay is targeted by removal with a loop piece on board. Action: counter the removal, not save the
   commander if a Rift/Decay only hits Sisay and the engine is complete. Reason: "protect the engine, not the commander." Tag
   **[primer: A]**.
6. **Stax in play.** Opponent has Cursed Totem out. Action: remove Totem first (Cyclonic Rift, or Abrupt Decay if its MV is 3 or less), then start creature-ability loops. Reason: [H] lists Totem as a hard lock and [TE] names it enemy #1 (Abrupt Decay is the clean answer). Tag **[primer: Tolaria_East]** (was [inference from H]).
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

12. **Misstep target.** An opponent casts Vampiric Tutor (MV1) with Mental Misstep in hand and a Sol Ring also on the stack
    earlier in the turn. Action: Misstep the tutor (or an opposing Misstep aimed at our counter), let Sol Ring resolve. Reason:
    [TE] reserves it for counters and 1-mana tutors. Tag **[primer: Tolaria_East]**.
13. **Opposition Agent.** An opponent casts Opposition Agent (flash) in our end step with Sisay and WUBRG up. Action: counter it
    with Force of Will (pitching a spare blue card) or Pact of Negation; otherwise destroy it (Ertai, Abrupt Decay). Do not
    activate Sisay while it is on the battlefield. Reason: Agent takes over our search; [TE] names it enemy #1. Tag
    **[primer: Tolaria_East]** (the actions are **[inference]** from our cards).
14. **Blood Moon.** Opponent casts Blood Moon on turn 2; we hold Abrupt Decay (3 mana up) and Force of Will. Action: counter it
    or Decay it at once, before our lands change. Reason: Blood Moon is an "uh oh" card and all 28 of our lands are nonbasic.
    Tag **[primer: Tolaria_East]** + **[inference]** (land count).
15. **End-step tutor.** Our turn is over, Sisay power 5 (enough coloured legends in play), WUBRG available from rocks at
    the opponents' end step. Action: activate Sisay, fetch Selvala (or Derevi). Then untap and fetch Derevi/Emiel on our turn.
    Reason: no summoning sickness, two activations before the loop. Tag **[primer: Tolaria_East]**.
16. **Tutor in response.** An opponent has just cast a spell that will win. We have no counter in hand, Sisay power 5, WUBRG up.
    Action: activate Sisay at instant speed and fetch Ertai Resurrected (MV4); its enter trigger counters the spell. Reason:
    [TE]: "Sisay can usually tutor in response to someone attempting to win"; the fetch cannot be countered. Tag
    **[primer: Tolaria_East]** for the idea; the Ertai choice is **[inference from Oracle]**.
17. **Force of Will pitch.** Hand: Force of Will, Teferi, Time Raveler, Derevi, one spare blue card, no mana. An opponent
    casts a winning spell. Action: pitch the spare blue card and pay 1 life; never Teferi or Derevi. Reason: pitching engine
    pieces can leave us unable to win. Tag **[primer: Tolaria_East]**.
18. **Rainbow dork chain.** Bloom Tender, Sisay (power 4+), five colours among permanents, WUBRG tapped. Action: fetch Derevi,
    untap the dork, tap it again, fetch Emiel (power 5) and start the Flash Unicorn loop. Reason: a rainbow dork plus Sisay at
    power 4 chains upward. Tag **[primer: Tolaria_East]**.
19. **Flash Unicorn on their turn.** Emiel and Derevi on the battlefield, Gaea's Cradle making 4+ (or Bloom Tender untapped),
    opponents tapped out at their end step. Action: run the loop then, finish with Mount Doom pings, with Ertai blink to counter
    any response. Reason: abilities have no timing limit and counterspells do not stop them. Tag **[primer: Tolaria_East]**.
20. **Ertai lock vs finish.** Infinite mana, Emiel, Derevi, Ertai and Mount Doom on board; an opponent has an instant. Action:
    blink Ertai to counter it, then ping with Mount Doom to win; do not hold the lock for turns. Reason: each counter draws the
    opponent a card, so the lock is not a win. Tag **[primer: Tolaria_East]** + **[Oracle]**.
21. **Teferi first.** Opponents hold open blue mana and we want to start a loop next turn. Action: Teferi, Time Raveler on our
    turn, then loop on the next. Reason: "a Silence through one spin": opponents can only act at sorcery speed. Tag
    **[primer: Tolaria_East]**.
22. **Line not in our list.** The primer's Oath of Bolas line (Oath of Teferi, Nicol Bolas, Aminatou) must not be started: the
    pieces are missing from the 100 cards. Action: use lines 1-3. Reason: the list differs from the author's. Tag
    **[primer: Tolaria_East]** + our list.

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

6. **Selvala chain [TE].** Board: Sisay, rocks and dorks making about 14 mana over two turns, library with Selvala, Derevi, Emiel,
   Mount Doom. Opponents at 40, no interaction. Expected: end-step fetch Selvala; own turn fetch Derevi and Emiel; the loop gains
   net mana; Mount Doom wins.
7. **Flash Unicorn on the opponent's turn.** Board: Emiel, Derevi, Cradle with 4+ creatures, Mount Doom, all untapped, during an
   opponent's end step. Expected: loop runs at instant speed; Mount Doom pings kill all opponents before our turn.
8. **DDR with Sisay + Enduring Vitality + Shang-Chi.** Board: Marvin, Ioreth, Shang-Chi, Enduring Vitality, Sisay, all cast this
   turn. Expected: infinite mana the same turn (Shang-Chi lets abilities ignore summoning sickness; Vitality makes Sisay a mana
   source); Mount Doom wins.
9. **Ertai lock.** Board: infinite mana, Emiel, Derevi, Ertai Resurrected; opponent casts a spell. Expected: Emiel blinks Ertai,
   spell countered, that opponent draws a card; repeatable. Not a win.
10. **Rainbow dork chain.** Board: Bloom Tender, Sisay power 4, WUBRG; library holds Derevi and Emiel. Expected: two fetches, then
    line 1 runs.
11. **Misstep restraint.** Opponent casts Sol Ring, then Demonic Tutor-class MV1 tutor. Expected: Misstep only hits the tutor.
12. **Missing pieces.** Run with no Oath of Teferi/Nicol Bolas/Aminatou in the deck. Expected: the AI never plans the Oath line.

## Open questions
- [TE] resolves "Sisay's ability" only as "based on Sisay's power". The Oracle (checked) says MV *less than* power, used above.
- [TE] "Drannith, Otawara and Boseiju" in the "uh oh" list: reads as Drannith Magistrate and the channel lands used against us;
  this is our reading. [TE] says nothing on whether Boseiju in our own list is used as an answer to Cursed Totem; that is ours.
- [TE] "Mindbreak Trap: Sisay needs to get back to my turn" is unclear (possibly: survive a multi-spell turn). Not modeled beyond
  "free when the opponent has cast three spells".
- [TE] gives no mulligan rule beyond the double-mana plan; no pod-politics rule either.
- [TE]'s line pieces we lack (Oath of Teferi, Nicol Bolas, Aminatou, Saheeli, Orcish Bowmasters, Orrery) mean our deck wins by
  Flash Unicorn, DDR and the Mount Doom / Deadpool finishers. Whether the Cookout list really drops the Oath lines is unknown;
  the EDHTop16 page has no write-up. Also unchecked: the exact net mana of line 7 (our maths, [inference]).
- Line 9 (Mana Vault + Kinnan + Cauldron): [TE] lists the pieces and "3 extra mana"; the detailed count is ours.
- Mana Vault and Boseiju Oracle text were not re-fetched in this pass (written from memory); Legolas's Quick Reflexes is only
  described as [TE] describes it.
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
- **Tolaria_East, "Oath of Sisay", Moxfield primer, updated 2026-08-29 (the primary source, [TE]):**
  https://moxfield.com/decks/4_8wod4y7ECmFZi-d_drpA/primer (read from JD's printout; extract
  `/home/joseph/projects/mtg_sims/primers/sisay-tolaria-east.md`)
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
