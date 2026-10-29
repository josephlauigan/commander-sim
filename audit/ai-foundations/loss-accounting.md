# Goldfish loss accounting: where the turns go (2026-10-27)

Branch `ai-foundations`. Measurement only: the AI is unchanged. The counters (`// --- ai-audit`) are stats only, and
they are always on in goldfish games: `commander-sim --goldfish KEYS` prints a "Loss accounting" section per deck, and
`--trace N` prints it turn by turn after the play-by-play. Two checks show the counters change no play. The 300-game
goldfish report for scion, Kinnan and Sisay is the same line for line before and after the change. The test
`the_loss_accounting_changes_no_play` compares traced games with the counters on and off. The 2,000-game summaries
below match the 2026-10-27 baseline in `audit/scion/FINDINGS.md` exactly.

Runs: `--goldfish t6,scion --games 2000 --seed 500000 --jobs 12`, once with the look-ahead (the default) and once with
`--ai adaptive`. Then 30 traced games (`TRACE_HAND=1 --trace 0..9`) for scion, Kinnan and Sisay. Raw output is in
`raw/goldfish-lookahead.txt`, `raw/goldfish-adaptive.txt` and `raw/traces/*.txt`.

## Headline

1. **The AI doesn't sit on ready lines.** After d10bbb05, `ready_not_taken` is 0.0-0.7% of own turns in every deck.
   Once a line is ready, the deck goes for it that turn.
2. **About a fifth to a third of the look-ahead's turns are `passed_castable`.** That is 21% of turns for scion, 24%
   for Kinnan and 36% for Rograkh/Silas. On these turns the deck ends a main phase holding a card it casts later. The
   cause is "stop (hold mana)": the look-ahead chooses it 5.6 times a game for scion and 5.8 for Kinnan, when another
   play was on offer. It rates holding about 3.4 points per playout above the best play. In a goldfish, nothing needs
   that mana, and the hold bonus isn't the cause (FINDINGS: `SEARCH_HOLD_W=0` changes nothing). The adaptive AI passes
   on only 3% of turns, yet it is a turn **slower** (scion median 11 against 10). So holding costs some turns, but it
   isn't what makes the clock 10.
3. **The rest of the turns aren't lost to anything a bucket can see.** 49-61% of turns are `progressed`: the deck cast
   or played something. On scion's progressed turns, the best card cast was a fast-mana card or rock (21%), a land
   only (19%), an "other" card (18%), a tutor (17%), a combo piece (12%) or a draw card (9%). The traces show where
   these turns go. Tutors fetch cards that aren't pieces. Scion spends turns on dragons and value creatures. Sisay's
   activations fetch Mount Doom and Ragavan before the line's pieces. Draw engines that live off opponents' spells
   (Rhystic Study, Mystic Remora, Esper Sentinel) are blanks against the inert seats.
4. **`mana_short` is 10% of scion's turns, and 18% of turns 1-4.** For Kinnan it is 7% (12%), for Sisay 10% (15%).
   The traces show mostly colour screw. Exotic Orchard copies nothing from the Wastes seats, Gemstone Caverns makes
   no colour, and green dorks sit in hand with only blue sources.
5. **The crude best case is slow too, so the deck and the goldfish account for part of the clock.** Counting only
   the cards the deck saw (draws, no tutors), scion has a line and its mana by round 20 in just 23% of games (median
   own turn 7). When it does, the kill comes the same turn in 68% of them (gap median 0). Counting each seen tutor
   as one missing piece, scion has a line in 84% of games, at median turn 6 (18% by turn 3, 30% by turn 4). The kill
   then trails by a median of 3 turns (mean 4.0), and by 6 or more in 29% of games. Kinnan trails by 1 (mean 2.2),
   Kraum/Tymna and Rograkh/Silas by 2. So with perfect play on the same draws and tutors, scion would kill around
   turn 6, not turn 3. The AI loses about 3-4 turns on top of that.
6. **Bug 2 (the cleanup discard) hits often.** Scion discards 1.67 cards a game in cleanup and loses a combo piece
   in 18% of games (Thassa's Oracle 124 times in 2,000 games). Rates in other decks:

   | deck | games losing a combo piece |
   | --- | --- |
   | Kraum/Tymna | 33% |
   | Rograkh/Thrasios | 27% |
   | Rograkh/Silas | 24% |
   | Kinnan | 5% |
   | Sisay | 2.5% |

   **Bug 1 (fixed in d10bbb05):** the sure-win cap changed the look-ahead's pick 2,314 times, in 61% of scion's games
   (41-78% across the seven decks). Before the fix, scion's median was 13; it is now 10.

## How each bucket is detected (first match wins)

Each own turn of the deck under test is recorded from its turn start (`engine/turn.rs::step_start`). Only the real
game records: a look-ahead copy has no `Game::audit` (`search::clone` takes it out).

| bucket | detected as |
| --- | --- |
| `won` | The game's last own turn, when the deck won. A win in the end-of-turn window after turn T counts as turn T, as the goldfish report's kill turn does. |
| `ready_not_taken` | At some main-phase decision (`brain::main`), `combos::audit_ready` found a line ready **and affordable**. This is the same test as `t6_measure`, which the speed counters use: `ready` plus `affordable`, over the deck's lines and any line with a piece held. And the deck didn't go for any line that turn (`goldfish::mark_go`, which `combos::attempt` calls). |
| `passed_castable` | As a main phase ended, the deck still held cards it had been offered to cast at the last decision (Cast, Removal, Wipe, Face or Crackle from hand). One of those cards was cast on a later own turn, or discarded later (this turn's cleanup included). A card passed in main 1 and cast in main 2 doesn't count, nor does one cast in the end-of-turn window of the same round. |
| `no_land_drop` | No land was played this turn, and a land card was in hand as a main phase ended. Main 1 ends after the extra land drops, so a land drawn in main 1 is played. |
| `mana_short` | At no main-phase decision could the deck pay for any nonland card in hand or its commander (`cost_of` plus tax, then `can_pay`). Instants count only when offered, so held counterspells aren't a main-phase play. It needs a nonland card or the commander available. |
| `progressed` | It cast a spell on its own turn, played a land, or performed a main-phase play (an ability or a combo). |
| `idle` | Anything else: nothing done, and nothing it passed was ever used. |

Card roles, from tags, first match wins:

| role | detected by |
| --- | --- |
| combo piece | the card is in a group of one of the deck's catalogued lines (`lines_of`, real groups) |
| land | a land card |
| tutor | `tut` or `xtutor` tag, or the `TUTORS` name list in `gf_audit.rs` (Beseech the Mirror, Intuition, Gamble, Diabolic Intent, Ranger-Captain and others; the tags miss them) |
| ritual | a mana card (`t6stats::is_mana_card`) that is an instant or sorcery |
| fast mana / rock | any other mana card |
| draw | `draw` tag, or the `DRAWS` list (Rhystic Study, Mystic Remora, Esper Sentinel, Necropotence, The One Ring, Sylvan Library, Ad Nauseam) |
| protection | `ctr`, `prot`, or a goldfish lock |
| other | anything else |

The test `the_loss_buckets_sum_to_the_own_turns` (sim-cli goldfish) plays 16 games each of scion and Kinnan. It checks
that the bucket counts sum to the own turns (all turns, and turns 1-4). It also checks one record per own turn, in
order, and exactly one `won` turn per kill, which is the last.

## Bucket shares (2,000 games each, share of own turns)

### Look-ahead

| deck | kill median | own turns/game | won | ready_not_taken | passed_castable | no_land_drop | mana_short | progressed | idle |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **scion** | 10.0 | 10.57 | 9.1% | 0.1% | 21.2% | 1.0% | 10.1% | 48.6% | 9.9% |
| kinnan-bonder-mana-combo | 8.0 | 8.76 | 11.3% | 0.3% | 23.9% | 0.2% | 7.1% | 53.3% | 4.0% |
| sisay-legends-toolbox | 7.0 | 7.58 | 13.2% | 0.0% | 15.3% | 0.2% | 10.0% | 59.4% | 1.9% |
| dack-fayden-white-stax | 10.5 | 10.88 | 8.2% | 0.1% | 19.6% | 0.6% | 7.5% | 61.0% | 3.0% |
| kraum-tymna-blue-breach-midrange | 9.0 | 10.25 | 9.1% | 0.7% | 29.8% | 3.0% | 8.3% | 43.1% | 6.1% |
| rograkh-silas-turbo-breach | 10.0 | 11.46 | 6.9% | 0.6% | 35.5% | 0.4% | 2.6% | 40.6% | 13.3% |
| rograkh-thrasios-creature-combo | 9.0 | 9.40 | 10.6% | 0.0% | 31.7% | 0.8% | 2.7% | 50.7% | 3.3% |

On own turns 1-4 only, the shares are:

| deck | won | ready_not_taken | passed_castable | no_land_drop | mana_short | progressed | idle |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **scion** | 1.8% | 0.0% | 13.7% | 0.2% | 17.7% | 62.0% | 4.6% |
| Kinnan | 2.4% | 0.0% | 16.0% | 0.2% | 12.1% | 67.0% | 2.3% |
| Sisay | 0.4% | 0.0% | 12.1% | 0.1% | 15.1% | 71.3% | 1.0% |
| Dack | 4.8% | 0.0% | 10.2% | 0.2% | 11.4% | 72.4% | 1.0% |
| Kraum/Tymna | 1.7% | 0.2% | 16.9% | 1.2% | 15.9% | 60.5% | 3.7% |
| Rograkh/Silas | 2.9% | 0.2% | 22.8% | 0.3% | 2.9% | 65.4% | 5.5% |
| Rograkh/Thrasios | 0.3% | 0.0% | 19.2% | 0.3% | 4.7% | 73.5% | 2.0% |

### Adaptive (no look-ahead)

| deck | kill median | own turns/game | won | ready_not_taken | passed_castable | no_land_drop | mana_short | progressed | idle |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **scion** | 11.0 | 11.49 | 8.3% | 0.0% | 3.3% | 1.2% | 12.2% | 62.9% | 12.1% |
| Kinnan | 10.0 | 9.91 | 9.8% | 0.2% | 11.8% | 0.1% | 8.4% | 65.4% | 4.3% |
| Sisay | 7.0 | 8.00 | 12.5% | 0.0% | 2.8% | 0.1% | 11.4% | 71.0% | 2.2% |
| Dack | 11.0 | 11.28 | 7.7% | 0.0% | 3.4% | 0.3% | 9.0% | 76.2% | 3.3% |
| Kraum/Tymna | 10.0 | 11.02 | 8.3% | 0.0% | 14.0% | 4.9% | 9.3% | 57.2% | 6.4% |
| Rograkh/Silas | 11.0 | 12.00 | 6.3% | 0.0% | 14.8% | 0.3% | 5.2% | 59.1% | 14.3% |
| Rograkh/Thrasios | 10.0 | 10.23 | 9.7% | 0.1% | 4.8% | 0.4% | 4.1% | 75.9% | 5.0% |

The adaptive AI casts almost everything it is offered (passed_castable 3% for scion), yet it is about a turn slower
than the look-ahead in five of the seven decks. Passing isn't what sets the pace.

### What "progressed" was (look-ahead): the most useful card cast that turn

| deck | combo piece | tutor | ritual | fast mana | draw | other | land only | abilities only |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **scion** | 12% | 17% | 1% | 21% | 9% | 18% | 19% | 2% |
| Kinnan | 41% | 8% | - | 16% | 4% | 12% | 12% | 6% |
| Sisay | 38% | 10% | - | 22% | 4% | 12% | 9% | 5% |
| Rograkh/Silas | 1% | 17% | 2% | 24% | 6% | 24% | 26% | 0% |

For Kinnan and Sisay, the commander is a piece, so "combo piece" includes casting the commander. Rograkh/Silas has
26% "land only" turns: it plays a land and nothing else.

## The cards behind passed_castable, and the cards never offered

**Passed and used later** (card-turns, look-ahead):

| deck | by role | most passed |
| --- | --- | --- |
| scion | draw 23%, tutor 19%, fast mana 19%, combo piece 2%, ritual 3%, other 34% | Esper Sentinel 516, Mystic Remora 433, Tinder Wall 308, Enlightened Tutor 305, Noxious Revival 266, The Cabbage Merchant 250, Ragavan 232, Hexing Squelcher 213, Storm-Kiln Artist 212, Rhystic Study 212 |
| Kinnan | combo piece 17%, fast mana 17%, draw 16%, tutor 9%, other 41% | Sink into Stupor 433, Mystic Remora 375, The Cabbage Merchant 339, Thrasios 274, Volatile Stormdrake 247, Rhystic Study 242, Sylvan Library 212 |
| Sisay | tutor 21%, combo piece 19%, fast mana 15%, draw 13%, other 31% | Marvin 234, Gleaming Splendor 183, Agatha's Soul Cauldron 172, Mystic Remora 167, Eladamri's Call 156, Demonic Tutor 143 |

**Payable but never offered** as a main-phase play (card-turns), in 71-85% of turns. Most of these are instants
(counterspells, protection). The rest are cards a rule holds:

- **scion:** Thassa's Oracle 4,573 and Demonic Consultation 3,787 (pieces held until the line is complete), Lion's
  Eye Diamond 2,951, Crop Rotation 2,695, Rite of Flame 2,496. Rituals were never offered in 7% of the held
  card-turns.
- **Kinnan and Sisay:** almost all protection.

## Main-phase stops (look-ahead runs)

"Best other" is the best non-stop option by heuristic utility. The score gap is the look-ahead's score (per playout)
of the best other option minus the stop's.

| deck | look-ahead "stop (hold mana)" a game (main 1 + main 2) | best other utility 3-6 / 6-10 / 10+ | score gap, mean | look-ahead "stop" a game |
| --- | --- | --- | --- | --- |
| **scion** | 3.34 + 2.29 = 5.63 | 5,992 / 3,506 / 195 of 11,255 | -3.4 / -3.2 | 1.00 + 0.30 |
| Kinnan | 3.39 + 2.44 = 5.83 | 5,828 / 3,497 / 974 of 11,659 | -4.1 / -4.0 | 0.78 + 0.41 |
| Sisay | 1.41 + 0.95 = 2.36 | 2,364 / 1,382 / 300 of 4,710 | -3.5 / -2.6 | 0.60 + 0.20 |
| Kraum/Tymna | 4.44 + 3.56 = 8.00 | utility 6-10 in 10,021 of 16,005 | -4.0 / -3.7 | 0.64 + 0.20 |
| Rograkh/Silas | 6.16 + 5.87 = 12.03 | utility 6-10 in 14,433 of 24,045 | -3.7 / -3.5 | 0.88 + 0.55 |

- **Best other options for scion's "stop (hold mana)":** Scion of the Ur-Dragon (cast or activated), Mystic Remora,
  Esper Sentinel, Enlightened Tutor, Tinder Wall, Ragavan.
- **For Kinnan:** Kinnan's dig five (1,022 in main 1), Mystic Remora, The One Ring, Sylvan Library, Rhystic Study.
- **Main 1 "stop" is mostly harmless.** The traces show the same card cast in main 2.
- **Heuristic stops never have another option in the look-ahead runs.** The heuristic path runs only when stop is
  the only option (search is on for every other decision).
- **Adaptive:** scion's heuristic "stop (hold mana)" comes 5.8 times a game in main 1. It passes over Scion → Korvold
  (966), Deathrite Shaman (524+137), Dark Ritual and dorks, but 69% of those stops had nothing else offered.

## Cleanup discards to 7 (bug 2, not fixed)

| deck (look-ahead) | discards a game | games with one | games losing a combo piece | discards `discard_keep` would keep | median kill: piece lost / not |
| --- | --- | --- | --- | --- | --- |
| **scion** | 1.67 | 43.5% | 18.4% (549 pieces) | 203 | 15 / 9 |
| Kinnan | 1.32 | 35.9% | 4.8% | 48 | 11.5 / 8 |
| Sisay | 0.24 | 9.8% | 2.5% | 11 | 15 / 7 |
| Dack | 0.96 | 18.8% | 2.5% | 71 | 16 / 10 |
| Kraum/Tymna | 3.61 | 63.4% | 33.1% | 496 | 15 / 7 |
| Rograkh/Silas | 2.88 | 57.4% | 24.2% | 304 | no kill / 8 |
| Rograkh/Thrasios | 2.26 | 51.4% | 26.9% | 1,119 | 12 / 8 |

- **What scion discards:** protection 21%, combo pieces 16% and lands 13%. The most discarded cards are Mindbreak
  Trap 219, Force of Will 204, Misdirection 199, Force of Negation 173, Sevinne's Reclamation 147, Prismari 140,
  Thassa's Oracle 124, Underworld Breach 101, Beseech the Mirror 100 and Brain Freeze 93.
- **The kill-turn split is confounded.** Long games discard more, so it is no measure of the bug's cost.
- **`discard_keep` misses most pieces.** It would keep far fewer cards than the combo pieces discarded (203 against
  549 for scion), so using it as is wouldn't save most of them.
- **A trace now logs each discard:** `[audit] X discards C to hand size (...; discard_keep would keep it)`.

## The sure-win cap (bug 1, fixed in d10bbb05)

The cap changed the look-ahead's pick in this many games:

| deck | games | picks |
| --- | --- | --- |
| scion | 60.5% | 2,314 |
| Kinnan | 68.0% | 3,448 |
| Sisay | 41.0% | 2,183 |
| Dack | 18.9% | 1,856 |
| Kraum/Tymna | 71.2% | 2,574 |
| Rograkh/Silas | 56.6% | 2,134 |
| Rograkh/Thrasios | 77.8% | 5,055 |

A changed pick is a decision where a held line that also wins every playout would have beaten a sure win. The kill
turns before and after the fix are in `audit/scion/FINDINGS.md`. Scion's median went from 13 to 10, Kinnan's from 9
to 8, Kraum/Tymna's from 11 to 9 and Rograkh/Silas's from 14 to 10.

## The crude best case (look-ahead runs)

**Definition.** T is the first own turn by which the deck has seen these cards:

- Seen means the kept hand (turn 1) and every card drawn, each from the first own turn it could use it. Tutored
  cards aren't seen. The commanders are always seen.
- It must have seen a card of every group of one of its catalogued lines.
- It must have mana sources at least equal to the line's mana: the loop's own mana plus the mana value of the
  cheapest seen card of each group (lands count 0), all paid on turn T.
- Sources are the land cards seen (at most T, one land drop a turn) plus the mana cards seen (rocks, dorks, moxen and
  rituals count 1 each).

The second variant lets each seen tutor stand in for one missing group, adding both its own mana value and the
fetched piece's.

| deck | best case, no tutors: games / median turn | kill − best: median (mean); 0 / 6+ | best case, tutors as pieces: games / median turn / by T3 / by T4 | kill − best: median (mean); 0 / 6+ |
| --- | --- | --- | --- | --- |
| **scion** | 23.4% / 7 | 0 (0.96); 68% / 4% | 83.8% / 6 / 17.9% / 30.4% | 3 (3.95); 19% / 29% |
| Kinnan | 32.8% / 7 | 1 (1.49); 47% / 7% | 69.6% / 7 / 7.8% / 14.1% | 1 (2.18); 29% / 10% |
| Sisay | 0.3% / 8 | n/a | 3.1% / 10 | n/a |
| Dack | 79.3% / 5 | 6 (5.92); 16% / 51% | same | same |
| Kraum/Tymna | 29.9% / 8 | 0 (1.29); 59% / 7% | 89.3% / 6 / 12.6% / 28.9% | 2 (3.56); 17% / 24% |
| Rograkh/Silas | 24.2% / 8 | 0 (1.40); 62% / 10% | 80.6% / 6 / 18.4% / 30.6% | 2 (3.04); 22% / 20% |
| Rograkh/Thrasios | 2.9% / 10 | 0 (0.38) | 46.4% / 9 | 1 (1.44) |

With the adaptive AI, scion's gap with tutors is a median of 4 (mean 4.6, 36% at 6 or more).

How to read it:

- **Without tutors, the decks rarely see a whole line.** When they do, the AI converts it on time.
- **With tutors, the AI trails a perfect pilot of the same cards** by a median of 2-3 turns for the Oracle and Breach
  decks, with a long tail.
- **Even the perfect pilot is slow.** Scion's median is turn 6, and only 18% of games are by turn 3, against cEDH
  turbo's turn 3-4. Part of that is the goldfish itself: Rhystic Study, Mystic Remora and Esper Sentinel draw nothing
  against seats that never cast. Part of it is how crude the proxy is (see Decisions).
- **Sisay isn't measured.** Its tutor is its commander's activation, which the proxy doesn't model.
- **Dack's best case is far from its kill.** A big gap there means its "line" (Preston + Felidar Guardian + Radiant
  Solar) is assembled early but isn't how it wins (`ready_not_taken` is only 0.1%). The catalogue's ready test and
  this proxy disagree. Not chased.

## Traced games (look-ahead, seed 500000 + N, `TRACE_HAND=1`)

Each note says where the turns went as a player would judge it, then whether the counters agree.

### scion

- **0 (no kill).**
  - Turns 1-4 went well: Sol Ring, Rhystic Study, Mana Vault, Intuition, Mystic Remora, Dark Ritual.
  - Rhystic and Remora then drew nothing against the inert seats. From turn 5 the hand was Lion's Eye Diamond and
    Orim's Chant. Demonic Tutor on turn 7 found the Oracle, and the second piece never came.
  - It held Arcane Signet for "stop (hold mana)" on turns 13-15.
  - Counters agree: turns 5-6 and 10-11 are "land only", 13-15 `passed_castable`.
- **1 (no kill).**
  - Mystical Tutor sat in hand on turn 3 under "stop (hold mana)". The cleanup then discarded **Thassa's Oracle**
    (8 cards; `discard_keep` would keep it). Mystical Tutor at end of turn fetched Consultation, now useless.
  - Stuck on 2 lands, it then passed Chrome Mox and Mystic Remora under "stop (hold mana)" from turn 6 to turn 14.
  - Counters agree (6-14 `passed_castable`). The root cause is bug 2, counted separately; turn 3 shows as `idle`.
- **2 (kill turn 8).**
  - Necropotence came down on turn 2. The deck paid life at each end step to draw, then discarded Intuition and
    Beseech the Mirror (two tutors) to hand size.
  - Mystical Tutor on turn 6 put Consultation on top, but Necropotence skips the draw, so the card came a turn later.
  - Counters: turn 7 `idle`, which is right. Neither the discard waste nor the tutor timing is visible to a bucket.
- **3 (kill turn 16).**
  - Enlightened Tutor fetched Esper Sentinel on turn 2, and Ranger-Captain fetched Ragavan on turn 7: neither is a
    piece.
  - It stayed on 2-3 lands and passed Hexing Squelcher, Esper Sentinel and Beseech the Mirror (turns 12-13) under
    "stop (hold mana)".
  - Counters agree on the passes (turns 2-6 and 12-13). Fetching off-line targets reads as "progressed (tutor)".
- **4 (kill turn 10).**
  - Borne Upon a Wind was cast in main 2 on turn 4. Its draw was a land that couldn't be played.
  - On turns 5-6, Necropotence was stuck for lack of black.
  - Prismari was discarded to hand size, and The Cabbage Merchant passed on turns 7-8.
  - Counters agree: turn 4 `no_land_drop`, turns 5-6 `mana_short`, turns 7-8 `passed_castable`. The kill came about a
    turn late.
- **5 (kill turn 14, by damage).**
  - Gamble was held on turns 1-2 by a look-ahead stop, then fetched Thassa's Oracle. The exile spell never came.
  - It won by damage with the Copper Dragon.
  - Counters agree (turns 1-2 `passed_castable [tutor]`, 4-5 `idle`).
- **6 (kill turn 14).**
  - Emergence Zone was sacrificed on turn 4 for a pointless flash, leaving one land until turn 9.
  - The hand was all interaction.
  - Counters: mostly `mana_short` and `idle`, which is right. The real loss, sacrificing a land, isn't a bucket.
- **7 (kill turn 17, commander damage).**
  - It had no green for Hierarch or Tinder Wall on turns 1-2.
  - Beseech the Mirror found the Oracle on turn 6. Enlightened Tutor then fetched Esper Sentinel instead of a line
    card.
  - From turn 11 it held Prismari under "stop (hold mana)".
  - Counters agree (turns 1-2 `mana_short`, 11-15 `passed_castable`).
- **8 (kill turn 8).**
  - Black-light on turns 2-3. Necropotence came on turn 4, Beseech found the Oracle on turn 5, and Mystical Tutor at
    end of turn 6 found Consultation.
  - It won on turn 8. Under Necropotence the tutor to the top cost a turn; otherwise this is close to what the cards
    allowed.
  - Counters fair (turns 2-3 `mana_short`).
- **9 (kill turn 8).**
  - Strong mana, but turns 3-6 went to Korvold (hard-cast), Storm-Kiln Artist, Scion, Prismari and Esper Sentinel:
    dragons, not digging. Korvold's draws found the pieces.
  - Counters read "progressed", which is truthful but hides that the progress was off-plan.

### Kinnan

- **0 (kill turn 13, by damage).**
  - Kinnan came down on turn 2. After that the hand was counterspells and fetchlands, with Arcane Signet held on
    turns 6-8.
  - Counters agree: turns 3-9 are "progressed (land only)". These draws were poor.
- **1 (kill turn 11).**
  - Hullbreaker Horror came into play on turn 6 (Invasion of Ikoria). The look-ahead then chose Kinnan's dig five over
    Finale of Devastation for four turns; Finale was cast on turn 10 and the deck won on turn 11.
  - Counters read "progressed (abilities only)" on turns 7-10. Choosing a weak play over a tutor isn't
    `passed_castable`, because the mana was spent.
- **2 (kill turn 3).** Kinnan, Talisman, Springleaf Drum, Transmute Artifact, then Mirror and Monolith. A perfect
  game; counters agree.
- **3 (kill turn 9).**
  - Two lands until turn 8 and a hand of counterspells. Turn 3 went to High Fae Trickster and Flesh Duplicate. Sylvan
    Library, Sphinx and Thrasios drew into the line.
  - About what the draws allowed; counters "progressed".
- **4 (kill turn 8).**
  - No green until turn 3, so Kinnan came on turn 4. Invasion found Thrasios on turn 6 and Transmute found Monolith on
    turn 8.
  - Perhaps a turn slow; counters agree (turns 1-2 `mana_short`).
- **5 (kill turn 7).**
  - An explosive start: Sol Ring, Talisman and a dork on turn 1, Kinnan on turn 2. Invasion was then held on turn 3,
    and on turn 4 it fetched Hullbreaker, not Thrasios or Monolith.
  - A person kills on turn 4-5. Counters show turn 3 `passed_castable [tutor]`; the target choice isn't visible.
- **6 (kill turn 9).**
  - Hullbreaker came on turn 4 without the rocks. It dug and drew with Sylvan Library and The One Ring until Basalt
    Monolith arrived on turn 9.
  - Mostly the draws; counters "progressed".
- **7 (kill turn 7).**
  - Kinnan on turn 1. Invasion and Finale were held on turn 3 (`passed_castable [tutor]`). Finale fetched Consecrated
    Sphinx, a draw engine, not a piece.
  - A person wins on turn 4-5.
- **8 (kill turn 12).**
  - Kinnan on turn 2, then four mana open on turns 3-7 with nothing cast. Rhystic Study was held on turns 4-6, Sol Ring
    on turns 7-9 and Mana Vault on turn 10, all for "stop (hold mana)".
  - Chord of Calling sat in hand from turn 1, and Hullbreaker was discarded to hand size on turn 5.
  - Counters agree (turns 4-10 `passed_castable`). The worst Kinnan game, worth 5-6 turns.
- **9 (kill turn 4).** Kinnan, Signet, Basalt Monolith and Chord of Calling for Thrasios. Clean.

### Sisay

- **0 (kill turn 6).**
  - Sisay came on turn 3. On turn 5 its activation went to Selvala, not a piece. Emiel and Cradle followed on turn 6.
  - About a turn slow. Counters read "progressed"; the fetch choice is invisible.
- **1 (kill turn 8).**
  - Sisay on turn 3, then Eladamri's Call and Mystic Remora were held on turns 4-6. Turn 8 was one long turn (Demonic
    Tutor, Crop Rotation into Cradle, Derevi, Emiel).
  - Counters agree (turns 4-6 `passed_castable`).
- **2 (kill turn 7).**
  - Sisay on turn 2 off Mana Vault. Then Enlightened Tutor for Marvin, Lavinia, Chord, and Sisay fetching Emiel, Mount
    Doom and Cradle.
  - Mount Doom cost a turn; counters "progressed".
- **3 (kill turn 7).**
  - Three green dorks were stuck in hand with only Otawara and Sol Ring on turns 1-3.
  - Counters agree: turns 1 and 3 `mana_short` (turn 2 cast Sol Ring). A Mount Doom fetch on turn 6 is invisible.
- **4 (kill turn 11).**
  - **No white source** from turn 3 to turn 7: Exotic Orchard copies nothing from the Wastes seats, and Gemstone
    Caverns makes no colour. Sisay ({2}{W}) came on turn 8.
  - Counters agree: turns 3-7 `mana_short`. Part of this is a goldfish artifact.
- **5 (kill turn 7).**
  - Sisay on turn 3. Its activations went to Mount Doom (turn 4), then Gaea's Cradle, then Derevi, plus Eladamri's
    Call for Emiel.
  - About a turn slow (Mount Doom). Counters "progressed".
- **6 (kill turn 8).**
  - Sisay on turn 2 off fast mana. The activations then went Mount Doom, Cradle, Ragavan, Marvin and Shang-Chi, then
    Ioreth.
  - A person wins on turn 5-6. Counters "progressed (abilities only)", which is truthful but blind to the order.
- **7 (kill turn 6).** Mount Doom on turn 4 and Ragavan on turn 5 before Derevi and Emiel on turn 6. About a turn
  slow; counters "progressed".
- **8 (kill turn 5).** Sisay on turn 2. Deadpool and Tyvar, then Derevi, Emiel and Mount Doom on turn 5. Near the
  deck's pace.
- **9 (kill turn 6).** Sisay on turn 3. Mount Doom on turn 4 and Ragavan on turn 5, then Derevi and Emiel on turn 6.
  About a turn slow.

### Do the counters tell the truth?

**Yes, at the level they look.** In the 30 traces:

- Every `passed_castable` turn was a real pass of a card used later.
- Every `mana_short` turn after the fix (see Decisions) was real screw, colour screw included.
- `no_land_drop` caught the main-2 cantrip land.
- `ready_not_taken` never fired, and no trace showed a ready line skipped.

**What they can't see:**

1. **Tutor targets.** Enlightened Tutor fetched Esper Sentinel in 2 of 10 scion games, Ranger-Captain fetched Ragavan,
   Finale fetched Consecrated Sphinx, and Invasion fetched Hullbreaker without its rocks.
2. **Sisay's activation order.** Mount Doom and Ragavan came before the line's pieces in 6 of 10 Sisay games, costing
   about a turn each.
3. **The weaker of two plays.** Kinnan's dig chosen over Finale was spent mana, so it reads as progress.
4. **Off-plan development.** Scion casts dragons and value creatures while a piece short.
5. **Self-inflicted mana loss.** Emergence Zone was sacrificed, and Borne Upon a Wind was cast in main 2.
6. **Dead draw engines in the goldfish.** Rhystic Study, Mystic Remora and Esper Sentinel draw nothing here.

These sit inside `progressed`, which is half the turns. A next step would count tutor targets: is the card found a
piece of a deck line, or a card of the plan? That needs one hook at the tutor resolution sites (`engine/tutors.rs`
and about six card-code sites).

## Decisions

- **Ready means ready *and affordable*,** the same test as the Tier 6 speed counters (`t6_measure`).
  `combos::ready` alone counts a line whose mana isn't there. The Tier 6 table's `go_ok` gate isn't applied, so a
  plan that says "wait" counts as not taken.
- **"Went for it" is `goldfish::mark_go`,** called by `combos::attempt`. Lines played out by normal actions (Scion
  becoming Prismari) are `won` if they win that turn. Otherwise they could read as `ready_not_taken`. Scion has 11
  `ready_not_taken` turns in 2,000 games, all Oracle + Consultation; I didn't inspect them.
- **The castable cards at the end of a main phase are the options at its last decision.** At the 18-play cap, that
  list can be one play stale, so cards no longer in hand are dropped. "Cast on a later turn" means a later own-turn
  count. A card passed on turn T and cast in the end-of-turn window before turn T+1 isn't a pass.
- **`mana_short` changed after the first traces.** "No offered spell" made held counterspells a screw (scion game 0,
  turns 5-6). It now reads "no nonland card payable", where instants count only when offered. With that, scion
  game 0 reads "land only" and Sisay game 4 reads `mana_short`, as a player would judge them.
- **Roles take the first match, in this order:** combo piece, land, tutor, ritual, fast mana, draw, protection,
  other. Demonic Consultation is a piece, not a tutor. Tags miss many tutors (Beseech the Mirror, Gamble, Intuition)
  and opponent-dependent draw engines, so two name lists were added in `gf_audit.rs` (`TUTORS`, `DRAWS`).
- **Commanders count as combo pieces when they are in a line** (Kinnan, Sisay, Scion's Prismari line).
- **Best-case proxy:**
  - Every source makes 1 mana: Sol Ring 1, a ritual 1. Lands are capped at one per turn. All the line's mana is
    paid on turn T. Dorks have no summoning sickness, and colours are ignored.
  - In the tutor variant, a tutor to the top of the library delivers the same turn.
  - These roughly cancel: the proxy is optimistic on colours, sickness and timing, and pessimistic in paying every
    piece on one turn.
  - "Seen" counts draws (`zones::draw`) only. Cards put into hand by other effects (Necropotence's end-step cards,
    Ad Nauseam, impulse effects) are excluded, as tutors are. This makes the no-tutor best case later than it should
    be for Necropotence games.
- **The cleanup-discard bug isn't fixed.** It is counted (`gf_audit::discard`, with a trace line). The kill-turn split
  by "a piece was discarded" is reported, but it is confounded by game length.
- **The audit is always on in goldfish games,** not behind a separate flag. It changes no play (two checks), costs
  about nothing (237 s for 14,000 look-ahead games, as before), and the goldfish report is where it belongs. It is
  never on outside goldfish games.
- **Heuristic stops show "nothing else offered" in look-ahead runs.** That isn't a bug in the counter: the heuristic
  only decides when stop is the only option. Its real stops show in the adaptive run.
- **The `stop` records hold the look-ahead's scores** from `LAST_SCORES`, after the sure-win cap.

## Files

- `rust/crates/sim-core/src/gf_audit.rs` (new): the recorder, the buckets, the roles and the best case.
- `rust/crates/sim-cli/src/gf_audit.rs` (new): the report section and the per-turn trace view.
- Hooks, each marked `// --- ai-audit`:
  - `state.rs` (`Game::audit`)
  - `lib.rs`
  - `engine/turn.rs` (turn start, main ends, land drop, cleanup discard, setup)
  - `engine/zones.rs` (draws)
  - `engine/cast.rs` (casts)
  - `ai/brain.rs` (main-phase decisions, plays, stops)
  - `ai/search.rs` (copies keep no audit; the cap counter)
  - `impls/combos.rs` (read-only accessors)
  - `goldfish.rs` (went for a line)
  - `t6stats.rs` (`RITUALS` made public)
  - `sim-cli/src/goldfish.rs`
  - `sim-cli/src/main.rs`
- Tests (sim-cli `goldfish::tests`): `the_loss_buckets_sum_to_the_own_turns`, `the_loss_accounting_changes_no_play`.
