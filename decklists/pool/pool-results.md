# Opponent pool results

This file reports how the main decks do against a fixed field: the 25 decks in `decklists/pool/`, five tiers of five.
In each game, one of your decks faces three decks drawn from one tier, with seats shuffled.
Games are seeded, so any cell can be reproduced with `python3 -m commander_sim --deck <d> --pool <tier> --games N`. Since September 2026
the default AI is the look-ahead AI (`--ai lookahead`, see `commander_sim/ai/search.py`); section 0 has its results. Sections 1-3 were
measured with the heuristic AI alone (`--ai adaptive`) and are kept for comparison.

Contents:
0. Current results with the look-ahead AI
1. Deck × tier matrix (heuristic AI)
2. Calibration: within-tier balance, tier ordering, one ranking of all 25 decks (heuristic AI)
3. Why the tiers were not ordered with the heuristic AI, and what was done
4. Old mode (removed)
5. Card coverage
6. Limitations

## 0. Current results with the look-ahead AI (September 2026)

Every deck at the table picks its main-phase plays, attacks and counterspells by look-ahead: the best few candidates
are tried on copies of the game (opponents' hands re-dealt from what they could hold), each copy is played forward to
the end of the player's next turn, and the position is scored (board, cards, life, combo progress). 240 games per
cell; a 95% interval is about ±5.5 points per deck and ±2.5 points per tier average.

Pool changes since sections 1-3: Grand Arbiter Augustin IV was retired and replaced by Heliod, Sun-Crowned (Tier 4);
the Yawgmoth list was tuned (6 swaps, section 3). Both with the user's approval.

### 0a. Tier ordering (each deck alone against three decks of the tier below; conservative profile; above 25% = ordered)

| Tier into tier below | Look-ahead | Heuristic, same seeds | Heuristic, earlier (section 2b) |
|---|---|---|---|
| T2 into T1 | **27.6%** (Sythis 41, Meren 36, Kaalia 24, Windgrace 23, Brago 15) | - | 30.6% |
| T3 into T2 | **25.5%** (Aurelia 35, Tergrid 29, Atraxa 28, Korvold 21, Marwyn 14) | - | 24.4% |
| T4 into T3 | **24.1%** (Chulane 28, Heliod 25, Krenko 25, Prosper 24, Yuriko 20) | 19.5% | 15.6% (with GAA) |
| T5 into T4 | **28.7%** (Kinnan 41, Winota 33, Zur 28, Urza 25, Yawgmoth 17) | 23.2% | 25.2% (with GAA in T4) |

T2, T3 and T5 are ordered; T4 into T3 is within noise of even (the heuristic AI had it clearly inverted). The
decks below 25% in their row are the tier's weak decks in 0b as well.

### 0b. Within-tier balance (four of the tier's five decks per game; flag above 35% or below 15%)

| Tier | Avg game (rounds) | conservative | loose |
|---|---|---|---|
| T1 High B2 / Low B3 | 12.5 | Light-Paws 30, Isshin 30, Teysa 23, Tatyova 21, Lathril 21 | Light-Paws 31, Isshin 28, Lathril 24, Tatyova 23, Teysa 19 |
| T2 Mid B3 | 15.4 | **Meren 39 HIGH**, Sythis 33, Kaalia 25, Windgrace 16, **Brago 14 LOW** | **Meren 47 HIGH**, **Sythis 36 HIGH**, Windgrace 21, **Kaalia 13 LOW**, **Brago 10 LOW** |
| T3 High B3 | 13.1 | **Atraxa 38 HIGH**, Aurelia 35, Korvold 21, Tergrid 20, **Marwyn 12 LOW** | **Atraxa 40 HIGH**, Aurelia 35, Tergrid 24, Korvold 15, **Marwyn 11 LOW** |
| T4 Low B4 | 10.2 | Heliod 28, Krenko 27, Prosper 27, Yuriko 26, Chulane 18 | Heliod 30, Yuriko 28, Prosper 24, Krenko 23, Chulane 19 |
| T5 High B4 | 7.6 | Kinnan 34, Winota 29, Zur 25, Urza 22, **Yawgmoth 14 LOW** | Kinnan 35, Winota 29, Zur 24, Urza 21, Yawgmoth 17 |

After tuning five lists (Meren and Atraxa trimmed; Brago, Marwyn and Yawgmoth strengthened; see each file's notes),
look-ahead, 240 games per tier, loose / conservative:

| Tier | loose | conservative |
|---|---|---|
| T2 | Sythis 37, Meren 31, Kaalia 22, Brago 19, Lord Windgrace 17 | Sythis 32, Kaalia 30, Meren 26, Brago 23, Lord Windgrace 16 |
| T3 | Aurelia 33, Atraxa 28, Tergrid 26, Korvold 20, Marwyn 18 | Atraxa 35, Aurelia 33, Tergrid 20, Marwyn 18, Korvold 18 |
| T5 | Kinnan 37, Winota 31, Zur 23, Urza 18, Yawgmoth 16 | Kinnan 34, Winota 27, Zur 26, Urza 23, Yawgmoth 15 |

Trimming Meren made Sythis the Tier 2 outlier (40-46%), so Sythis was trimmed too (Sigil of the Empty Throne and Starfield
of Nyx out). Kinnan and Yawgmoth sit at the edges of Tier 5.

Games get shorter as the brackets go up (15 rounds in T2 to under 8 in T5), with few timeouts above T2.
Remaining outliers: Meren and Atraxa run hot; Brago, Marwyn and Yawgmoth run cold. Yawgmoth's mono-black list has
only three pieces for its undying loop (Mikaeus, Geralf's Messenger, Butcher Ghoul); an immortal Yawgmoth wins 36%, so
it loses races rather than failing to close.

### 0c. Engine and AI fixes made while calibrating

Look-ahead exposed rules and AI faults that the heuristic AI rarely hit; each was fixed and committed:
- a won position always scores above any unfinished one (a 250-Goblin board no longer "prefers" not to attack);
- step caps per game and per playout, a cap on look-ahead work per game, and look-ahead copies stop once the table
  passes 150 permanents, so every game finishes in bounded time;
- cards leave the hand or graveyard before their cost is paid (payment triggers could move them);
- Thousand-Faced Shadow copies only from a real ninjutsu; Scourge of the Throne's dethrone check;
- creature combo pieces get combo cast priority (Mikaeus, Kiki-Jiki, Felidar Guardian ...);
- Yawgmoth: Geralf's Messenger as its own loop payoff, -1/-1 counters on X/1 tokens, instant-speed use after blockers
  (a new `defend` event for defending permanents), and a second loop once a payoff is drawn.

### 0d. Your decks against the tiers (look-ahead AI, loose profile: the most interactive opponents; 240 games per cell)

Lists as updated in September 2026 (Najeela no longer tracked), after a full modeling pass: every card in the three
decks audits Full (`python3 -m commander_sim.pool_audit --mine`). Each game seats the deck against three decks of the tier;
25% is an even share. Tiers 2, 3 and 5 were re-run after the six pool lists were tuned (0b); Tiers 1 and 4 did not
change. Before the tuning those cells read: Sephiroth 49.6 / 38.8 / 22.5%, Veyran 25.0 / 17.5 / 14.2%, Sauron
23.3 / 20.4 / 18.3% (T2 / T3 / T5).

| Deck | T1 High B2/Low B3 | T2 Mid B3 | T3 High B3 | T4 Low B4 | T5 High B4 |
|---|---|---|---|---|---|
| Sephiroth | **40.8%** (35-47) | **47.5%** (41-54) | **42.1%** (36-48) | **37.9%** (32-44) | 22.1% (17-28) |
| Veyran | 20.0% (15-26) | 17.9% (14-23) | 15.0% (11-20) | 20.8% (16-26) | 12.9% (9-18) |
| Sauron | 24.2% (19-30) | 27.5% (22-34) | 21.2% (16-27) | 24.2% (19-30) | 17.5% (13-23) |

Sephiroth is favoured through Low Bracket 4 and near even against High Bracket 4. Sauron sits near an even share
through Low Bracket 4 and falls off against High Bracket 4. Veyran is below an even share at every tier, clearly so
from Tier 2 up (13-18%). These are worst-case numbers: the
loose profile has opponents counter and remove more freely than the conservative one.

## 1. Deck × tier matrix (heuristic AI)

Each cell is 5,000 games: the win rate of the deck in that row against three decks of the column's tier, with a 95% interval.
An even share is 25%. Generated with `python3 -m commander_sim --all-decks --pool all --games 5000`.

**Noise band.** Each interval is about ±1.2 points at 25% and ±1.4 at 50%.
Cells in the same column use the same seeds, so the opponents and seats match row to row.
The two profiles also share seeds, so differences under about 1.5 points are noise.

**Conservative profile**

| deck | T1 (High B2 / Low B3) | T2 (Mid B3) | T3 (High B3) | T4 (Low B4) | T5 (High B4) |
|---|---|---|---|---|---|
| seph | 58.7% (57.3-60.0) | 58.9% (57.5-60.3) | 52.4% (51.1-53.8) | 59.0% (57.6-60.4) | 38.8% (37.5-40.2) |
| najeela | 33.3% (32.0-34.6) | 29.8% (28.5-31.1) | 32.1% (30.9-33.5) | 43.4% (42.1-44.8) | 31.6% (30.3-32.9) |
| sauron | 22.3% (21.2-23.5) | 18.9% (17.9-20.0) | 17.8% (16.8-18.9) | 26.7% (25.5-27.9) | 26.9% (25.6-28.1) |
| veyran | 21.7% (20.5-22.8) | 19.8% (18.7-20.9) | 17.9% (16.8-18.9) | 29.6% (28.3-30.9) | 19.1% (18.0-20.2) |

**Loose profile**

| deck | T1 (High B2 / Low B3) | T2 (Mid B3) | T3 (High B3) | T4 (Low B4) | T5 (High B4) |
|---|---|---|---|---|---|
| seph | 58.4% (57.1-59.8) | 58.8% (57.4-60.2) | 51.6% (50.2-53.0) | 59.3% (58.0-60.7) | 39.3% (37.9-40.6) |
| najeela | 33.1% (31.8-34.4) | 29.0% (27.8-30.3) | 32.4% (31.1-33.7) | 43.9% (42.5-45.3) | 31.3% (30.0-32.6) |
| sauron | 23.8% (22.6-25.0) | 19.5% (18.5-20.7) | 18.2% (17.2-19.3) | 26.9% (25.7-28.2) | 26.6% (25.4-27.9) |
| veyran | 22.2% (21.1-23.4) | 20.2% (19.1-21.3) | 18.3% (17.2-19.4) | 29.5% (28.3-30.8) | 19.5% (18.4-20.6) |

**Reading the matrix.**
- **T1 to T3 behave like a rising difficulty.** Win rates fall from T1 to T3 for Sephiroth (58.7% to 52.4%), Sauron and Veyran. Najeela stays about flat.
- **T5 is the hardest column** for Sephiroth, Najeela and Veyran.
- **T4 is still easier than T3 for every deck,** though less than before the deck-plan work (Sephiroth vs T4 went from 64.7% to 59.0%, Najeela from 52.8% to 43.4%). Read the T4 column as "Low B4 lists as the sim can pilot them", not as a Low-B4-strength field. Section 3 explains.

## 2. Calibration (heuristic AI)

Commands: `python3 -m commander_sim --calibrate all --games 2000` for both profiles.
Per-deck intervals are about ±2 points (within tier) and ±2 points (ordering, 2,000 games per deck).

### 2a. Within-tier balance (conservative / loose; flag above 35% or below 15%)

| Tier | Deck | cons. | loose | Flag |
|---|---|---|---|---|
| T1 | Isshin | 25.3 | 25.1 | |
| T1 | Lathril | 22.4 | 22.7 | |
| T1 | Light-Paws | 25.4 | 24.3 | |
| T1 | Tatyova | 23.5 | 26.3 | |
| T1 | Teysa Karlov | 28.4 | 26.7 | |
| T2 | Brago | 11.9 | 12.3 | LOW |
| T2 | Kaalia of the Vast | 20.1 | 20.8 | |
| T2 | Lord Windgrace | 25.0 | 24.0 | |
| T2 | Meren of Clan Nel Toth | 35.0 | 35.0 | HIGH (borderline) |
| T2 | Sythis | 33.1 | 32.9 | |
| T3 | Atraxa | 36.9 | 37.7 | HIGH |
| T3 | Aurelia | 38.7 | 37.7 | HIGH |
| T3 | Korvold | 17.4 | 18.2 | |
| T3 | Marwyn | 6.9 | 6.9 | LOW |
| T3 | Tergrid | 24.9 | 24.3 | |
| T4 | Chulane | 25.1 | 23.7 | |
| T4 | Grand Arbiter Augustin IV | 8.9 | 9.0 | LOW |
| T4 | Krenko | 33.9 | 33.9 | |
| T4 | Prosper | 25.4 | 25.8 | |
| T4 | Yuriko | 31.2 | 32.1 | |
| T5 | Kinnan | 33.9 | 35.6 | HIGH (loose) |
| T5 | Urza | 20.2 | 19.4 | |
| T5 | Winota | 35.4 | 35.4 | HIGH |
| T5 | Yawgmoth | 14.9 | 14.1 | LOW |
| T5 | Zur the Enchanter | 20.4 | 20.3 | |

Tier 2 still times out in about 20% of games: blink, pillowfort and recursion decks stall each other.

### 2b. Tier ordering (each deck alone against three decks of the tier below; above 25% = ordered)

| Pairing | cons. | loose | Before this work | Per deck (cons.) |
|---|---|---|---|---|
| T2 into T1 | **30.6%** | 31.5% | 30.5% | Brago 21, Kaalia 26, Windgrace 31, Meren 35, Sythis 40 |
| T3 into T2 | **24.4%** | 24.0% | 25.2% | Atraxa 29, Aurelia 34, Korvold 19, Marwyn 8, Tergrid 32 |
| T4 into T3 | **15.6%** | 15.7% | 13.0% | Chulane 19, GAA 4, Krenko 19, Prosper 24, Yuriko 13 |
| T5 into T4 | **25.2%** | 25.5% | 26.7% | Kinnan 34, Urza 23, Winota 31, Yawgmoth 16, Zur 22 |

T2 over T1 is ordered. T3 over T2 and T5 over T4 are at parity. **T4 is still inverted.**

### 2c. One ranking of all 25 decks (random four-deck pods drawn from all tiers, 10,000 games; 25% = average)

| Tier | Average | Decks |
|---|---|---|
| T1 | 22.9% | Isshin 26, Lathril 25, Light-Paws 18, Tatyova 21, Teysa 24 |
| T2 | 29.0% | Brago 17, Kaalia 29, Windgrace 29, Meren 36, Sythis 35 |
| T3 | 28.3% | Atraxa 35, Aurelia 40, Korvold 24, Marwyn 13, Tergrid 30 |
| T4 | 19.6% | Chulane 21, GAA 7, Krenko 24, Prosper 26, Yuriko 21 |
| T5 | 24.8% | Kinnan 38, Urza 23, Winota 25, Yawgmoth 16, Zur 22 |

- **Strongest in the sim:** Aurelia, Kinnan, Meren, Sythis and Atraxa.
- **Weakest:** GAA, Marwyn, Yawgmoth, Brago and Light-Paws.

## 3. Why the tiers were not ordered with the heuristic AI, and what was done

**Card rules are not the cause.** Every card in the pool is fully modeled (section 5).
The review of the compiled cards found and fixed real errors that had distorted results, including:
- Talismans made only colourless mana.
- Signets counted as two mana instead of one net.
- Goldspan Dragon never attacked.
- Pernicious Deed wiped every creature regardless of X.
- Deathrite Shaman exiled all graveyards.
- Doubling Season doubled planeswalker ability costs.

**AI changes that helped.** These apply to every outside deck, or to one deck's play plan (`commander_sim/ai/deck_plans.py`).

| Change | Effect (lone deck vs the tier below) |
|---|---|
| Ninjutsu decks go to combat before spending the ninjutsu mana; evasive creatures attack the open player | Yuriko 8% → 13% (T3), 5.4 Yuriko triggers per game instead of 3.3 |
| Planeswalkers near their ultimate draw attacks and removal | Atraxa 49% → 37% within T3 |
| Chulane casts creatures first with its commander out; finishers when the board is wide | Chulane 16% → 19% |
| Krenko activates at the end of the turn before its own | Krenko 17% → 19% |
| Combo pieces are kept out of discards and wheels | Prosper 18% → 24% |
| Yawgmoth's tutors build the whole loop (undying pair, payoff) | Yawgmoth 10% → 16% (T4) |
| GAA: rocks and taxes early, pillowfort when attacked | GAA 4% → 4% |

**What limits Tier 4.** Several diagnostics point the same way:

- **Speed.** Against a do-nothing opponent, every tier kills around turn 8 (T4 mean 9.0, T1 8.1).
  Real Bracket 4 decks should be one or two turns faster than Bracket 2–3.
- **Head-to-head.** In pods with two T1 decks and two T4 decks, T1 wins 61% of games; T3 against T4 wins 69%.
  So it is not only that a lone deck gets focused.
- **Game Changers are cheap in the sim.** Replacing Prosper's six Game Changers with basic lands costs it 3 points.
- **Damage output.** Chulane, GAA and Prosper deal 9–20 damage per game. A lone deck needs 120.
  They mostly win by combo (Chulane, Prosper) or not at all (GAA).
- **Upgrading the lists does not fix it.** Tested with `commander_sim/tools/swaptest.py` (paired, 800 games, lone vs T3; the lists are unchanged):

| Deck | Swaps tested | Result |
|---|---|---|
| Chulane | tutors for Aluren (Enlightened, Mystical), Natural Order, Eldritch Evolution, Chrome Mox, Mox Diamond, Lotus Petal, Gaea's Cradle for eight value cards | 17.2% → 21.4% |
| GAA | eight or ten upgrades: fast mana and sweepers; more pillowfort; or a Power Artifact + Monolith package | 3.9% → at most 9.2% |
| Krenko | fast mana (Mana Vault, Chrome Mox, Mox Diamond, Lotus Petal, Grim Monolith) for five Goblins and rocks | 18.5% → 15.4% |
| Yuriko | Mystical Tutor, Imperial Seal, Force of Will, Pact, Chrome Mox, Mox Diamond | 13.2% → 10.8% |

**Conclusion.** In this sim, what separates Bracket 4 from Bracket 3 in real games does not turn into wins:
fast mana, tutors, free interaction and precise sequencing around a combo.
The sim's AI plays board-and-combat decks well. It plays interaction-heavy and stax decks, whose strength is in decisions, much less well.
Adding real Bracket 4 cards to the Tier 4 lists does not move them into place, so list edits would not order the tiers either.

**Options from here:**
1. Keep the tiers as written, and read the T4 column as a weaker field than T3.
2. Re-tier the pool by measured sim strength (section 2c), so the columns are ordered by difficulty.
3. A deeper AI with look-ahead for combo and control decisions. This is a larger project with no guaranteed result.

### GAA replaced by Heliod (September 2026)

The tables above were measured with Grand Arbiter Augustin IV in Tier 4. GAA has since been retired to `decklists/pool/retired/` and replaced by Heliod, Sun-Crowned (mono-white stax).

**Why GAA was retired.** It won about 4% against Tier 3 under every AI version, including the look-ahead AI. The diagnosis (heuristic AI, 480 games per row, GAA alone against three Tier 3 decks):

| What-if | GAA win rate |
|---|---|
| As built | 3.8% |
| Opponents never pay Rhystic Study / Mystic Remora / Smothering Tithe | 4.6% |
| GAA's own spells ignore its taxes | 4.4% |
| Isochron combo needs one less mana | 4.6% |
| GAA starts on the battlefield on turn one | 5.4% |
| GAA cannot lose life | 7.5% |
| All of GAA's stax effects switched off | 3.8% |
| Taxes on opponents doubled | 4.0% |

The card rules check out (taxes, spell limits, Drannith, and the Isochron loop's need for 3+ mana from nonland permanents). The stax slows every seat alike, and the list has no reliable kill. Even with no opponents, it had seen both Isochron Scepter and Dramatic Reversal by turn 10 in only 12% of games, and had 3+ mana from rocks in only 16%. It dealt about 9 damage per game.

**Heliod calibration** (heuristic AI, 480 games, Heliod alone against three decks of the tier):

| List | vs Tier 3 | vs Tier 4 | vs Tier 5 |
|---|---|---|---|
| With Ranger-Captain of Eos and Enlightened Tutor (High B4-level) | 44.8% | 40.0% | 39.0% |
| **As committed** (Silent Arbiter and Restoration Angel instead) | **29.6%** | **26.5%** | 30.0% |
| Also without Recruiter of the Guard and Idyllic Tutor | 23.1% | 21.7% | 27.3% |

With Heliod in Tier 4, Tier 5 into Tier 4 is 22.7% (480 games per deck): Kinnan 34, Winota 28, Urza 20, Zur 20, Yawgmoth 12.

## 4. Old mode (removed)

The original four-deck mode (the main decks against each other, `compare.py` without `--pool`) was removed in
September 2026 at the user's request; its exact-replay fixture test went with it. The four deck files are still
guarded: `tests/test_my_decks.py` checks that they parse to the recorded lists. Its last numbers (n=10,000 per
profile, conservative / loose): Sephiroth 46.9 / 47.5%, Najeela 24.9 / 24.2%, Sauron 18.2 / 18.4%, Veyran 10.0 / 9.8%.


## 5. Card coverage

`python3 -m commander_sim.pool_audit` (or `--md <file>`) regenerates this. The statuses are:
- **Full:** a hand implementation or reviewed override. For cards whose rules are complete but whose use is an AI choice, the note says how the AI uses it.
- **Full-auto:** the ability compiler handles every clause. Each of these was checked against its Oracle text; wrong compilations were replaced (`commander_sim/cards/impl/fixes.py`).
- **Approximate / Partial / Unmodeled:** none remain.

Counts are per unique card per deck.

| | Full | Full-auto | Approximate | Partial | Unmodeled |
|---|---|---|---|---|---|
| Before the pool work | 663 | 296 | 208 | 665 | 232 |
| Previous report | 1072 | 222 | 622 | 140 | 8 |
| Now | 1884 | 180 | 0 | 0 | 0 |

**Two-card combos** (Isochron + Reversal, Power Artifact + Monolith, Kinnan loops, Thassa's Oracle, Sanguine Bond + Exquisite Blood, Kiki-Jiki, Gravecrawler, Yawgmoth, Karn + Lattice, Helm + Celebrant, Aluren + Chulane, Thornbite Staff) are modeled up to the loop:
- Pieces are drawn, tutored and cast normally.
- The spells that complete the combo go through the counter window.
- Each opponent gets an instant-speed removal or Tidebinder response.
- The loop then resolves as its end result: a win, or a lock.

Where the modeling lives (in `commander_sim/cards/impl/` unless noted):
- `topdeck.py`: scry, surveil and library manipulation.
- `fixes.py`: corrected compiled cards.
- `lands.py`: utility lands.
- `partials.py`: formerly partial cards.
- `rules.py` and `rules2.py`: exact rules for formerly approximated clauses.
- `commander_sim/ai/deck_plans.py`: deck play plans.

| Tier | Deck | Full | Full-auto | Approximate | Partial | Unmodeled | Engines: full / approx. / partial / unmodeled |
|---|---|---|---|---|---|---|---|
| 1 | Isshin | 78 | 6 | 0 | 0 | 0 | 2 / 2 / 0 / 0 |
| 1 | Lathril | 73 | 7 | 1 | 0 | 0 | 4 / 0 / 0 / 0 |
| 1 | Light-Paws | 69 | 4 | 0 | 0 | 0 | 2 / 2 / 0 / 0 |
| 1 | Tatyova | 71 | 11 | 0 | 0 | 0 | 3 / 1 / 0 / 0 |
| 1 | Teysa Karlov | 58 | 17 | 4 | 0 | 0 | 2 / 2 / 0 / 0 |
| 2 | Brago | 69 | 11 | 0 | 0 | 0 | 1 / 2 / 0 / 0 |
| 2 | Kaalia of the Vast | 78 | 6 | 0 | 0 | 0 | 2 / 1 / 0 / 0 |
| 2 | Lord Windgrace | 75 | 14 | 0 | 0 | 0 | 2 / 2 / 0 / 0 |
| 2 | Meren of Clan Nel Toth | 65 | 14 | 2 | 0 | 0 | 3 / 2 / 0 / 0 |
| 2 | Sythis | 67 | 12 | 0 | 0 | 0 | 4 / 1 / 0 / 0 |
| 3 | Atraxa | 80 | 8 | 0 | 0 | 0 | 3 / 1 / 0 / 0 |
| 3 | Aurelia | 77 | 5 | 0 | 0 | 0 | 3 / 1 / 0 / 0 |
| 3 | Korvold | 75 | 9 | 3 | 0 | 0 | 1 / 3 / 0 / 0 |
| 3 | Marwyn | 69 | 7 | 0 | 0 | 0 | 3 / 1 / 0 / 0 |
| 3 | Tergrid | 64 | 11 | 0 | 0 | 0 | 2 / 3 / 0 / 0 |
| 4 | Chulane | 80 | 8 | 0 | 0 | 0 | 2 / 2 / 0 / 0 |
| 4 | Grand Arbiter Augustin IV | 79 | 4 | 0 | 0 | 0 | 3 / 2 / 0 / 0 |
| 4 | Krenko | 74 | 1 | 0 | 0 | 0 | 3 / 3 / 0 / 0 |
| 4 | Prosper | 79 | 5 | 0 | 0 | 0 | 3 / 2 / 0 / 0 |
| 4 | Yuriko | 77 | 6 | 0 | 0 | 0 | 3 / 1 / 0 / 0 |
| 5 | Kinnan | 89 | 2 | 0 | 0 | 0 | 4 / 1 / 0 / 0 |
| 5 | Urza | 80 | 0 | 0 | 0 | 0 | 3 / 4 / 0 / 0 |
| 5 | Winota | 85 | 3 | 0 | 0 | 0 | 2 / 2 / 0 / 0 |
| 5 | Yawgmoth | 71 | 6 | 2 | 0 | 0 | 3 / 2 / 0 / 0 |
| 5 | Zur the Enchanter | 90 | 3 | 0 | 0 | 0 | 3 / 2 / 0 / 0 |

**Isshin — Mardu Attack Triggers** (Tier 1)

| Engine | Status | What is missing |
|---|---|---|
| Isshin's attack-trigger doubling | Full | - |
| token-on-attack creatures | Approximate | attack_triggers: Approximate (attack triggers: hooks for the key cards, compiled text for the rest) |
| Hellrider / Brutal Hordechief per-attacker damage | Approximate | attack_triggers: Approximate (attack triggers: hooks for the key cards, compiled text for the rest) |
| extra combats | Full | - |

**Lathril — Golgari Elves** (Tier 1)

| Engine | Status | What is missing |
|---|---|---|
| Elf-scaling mana | Full | - |
| Lathril's tokens and tap-ten drain | Full | - |
| lords and overruns | Full-auto | - |
| Skullclamp | Full | - |

**Light-Paws — Mono-White Aura Voltron** (Tier 1)

| Engine | Status | What is missing |
|---|---|---|
| Light-Paws Aura chain | Approximate | auras: Approximate (Auras attach to the best creature (the commander in voltron decks); bonuses, keywords, protection; hostile Auras modeled as exile) |
| enchantress draw | Full-auto | - |
| totem armor | Approximate | auras: Approximate (Auras attach to the best creature (the commander in voltron decks); bonuses, keywords, protection; hostile Auras modeled as exile) |
| commander damage | Full | - |

**Tatyova — Simic Landfall Ramp** (Tier 1)

| Engine | Status | What is missing |
|---|---|---|
| Tatyova landfall draw | Full-auto | - |
| extra land drops | Full | - |
| landfall token output | Full-auto | - |
| land recursion | Approximate | land_recursion: Approximate (Crucible / Ramunap / Greenwarden land drops from the graveyard; Loam dredge when short on lands) |

**Teysa Karlov — Orzhov Aristocrats** (Tier 1)

| Engine | Status | What is missing |
|---|---|---|
| Teysa's death-trigger doubling | Full | - |
| drain payoffs | Approximate | Blood Artist: Approximate; Zulaport Cutthroat: Approximate; Falkenrath Noble: Approximate |
| forced sacrifice | Approximate | edicts: Approximate (the victim sacrifices its lowest-value creature) |
| Skullclamp | Full | - |

**Brago — Azorius Blink Control** (Tier 2)

| Engine | Status | What is missing |
|---|---|---|
| flicker ETB re-use | Approximate | flicker: Approximate (blink re-triggers ETBs (Brago, Soulherder, Closet, Displacer, Deadeye, Ephemerate, Resto); Deadeye soulbond read as a paid blink) |
| counterspell decisions | Approximate | counter_ai: Approximate (counters by importance; spells that complete a combo count as critical) |
| monarch | Full | - |

**Kaalia of the Vast — Mardu Angels, Demons & Dragons** (Tier 2)

| Engine | Status | What is missing |
|---|---|---|
| Kaalia's put-onto-battlefield-attacking | Full | - |
| commander protection | Approximate | protection_ai: Approximate (outside decks answer targeted removal and wipes with their protection cards (and sacrifice creatures in response for value)) |
| big flier combat | Full-auto | - |

**Lord Windgrace — Jund Lands** (Tier 2)

| Engine | Status | What is missing |
|---|---|---|
| Windgrace loyalty and land recursion | Approximate | planeswalkers: Approximate (hand-written walkers use one ability a turn; attackers go after valuable walkers (pool games); walkers without an implementation use compiled loyalty abilities) |
| landfall token makers | Full-auto | - |
| Titania elementals | Full | - |
| Life from the Loam dredge | Approximate | land_recursion: Approximate (Crucible / Ramunap / Greenwarden land drops from the graveyard; Loam dredge when short on lands) |

**Meren of Clan Nel Toth — Golgari Recursion** (Tier 2)

| Engine | Status | What is missing |
|---|---|---|
| Meren's end-step reanimation | Full | - |
| ETB removal re-use | Full-auto | - |
| sac outlets | Full | - |
| Grave Pact | Approximate | edicts: Approximate (the victim sacrifices its lowest-value creature) |
| Survival / Birthing Pod | Approximate | survival_pod: Approximate (Survival discards the worst creature for the best; Pod upgrades by one MV) |

**Sythis — Selesnya Enchantress Pillowfort** (Tier 2)

| Engine | Status | What is missing |
|---|---|---|
| enchantress draw | Full-auto | - |
| attack taxes and caps | Approximate | attacker_caps: Approximate (Crawlspace and Silent Arbiter attack caps; Silent Arbiter's block cap missing) |
| Sigil of the Empty Throne | Full-auto | - |
| Starfield of Nyx | Full | - |
| Sythis cast trigger | Full-auto | - |

**Atraxa — Four-Color Superfriends** (Tier 3)

| Engine | Status | What is missing |
|---|---|---|
| planeswalker loyalty and ultimates | Approximate | planeswalkers: Approximate (hand-written walkers use one ability a turn; attackers go after valuable walkers (pool games); walkers without an implementation use compiled loyalty abilities) |
| proliferate | Full | - |
| Doubling Season | Full | - |
| attack taxes | Full | - |

**Aurelia — Boros Extra Combats** (Tier 3)

| Engine | Status | What is missing |
|---|---|---|
| additional combat phases | Full | - |
| attack triggers per combat | Approximate | attack_triggers: Approximate (attack triggers: hooks for the key cards, compiled text for the rest) |
| equipment | Full-auto | - |
| Helm of the Host + Combat Celebrant | Full | - |

**Korvold — Jund Sacrifice & Treasure** (Tier 3)

| Engine | Status | What is missing |
|---|---|---|
| Korvold sacrifice draw | Full | - |
| Treasure / Food / Clue tokens | Approximate | food: Approximate (Food tokens: sacrifice fodder (Korvold, Trail of Crumbs, Goose mana, Savvy Hunter); eaten when low) |
| per-sacrifice damage | Approximate | Blood Artist: Approximate; Zulaport Cutthroat: Approximate |
| Gravecrawler + Phyrexian Altar loop | Approximate | gravecrawler_loop: Approximate (combo with Phyrexian Altar, another Zombie and a death payoff) |

**Marwyn — Mono-Green Elves** (Tier 3)

| Engine | Status | What is missing |
|---|---|---|
| Elf-scaling mana | Full | - |
| Gaea's Cradle | Full | - |
| Craterhoof overrun | Full | - |
| creature tutors | Approximate | creature_tutors: Approximate (Natural Order, GSZ, Finale, Eldritch Evolution, Chord onto the battlefield; the target is the best card (Craterhoof on a wide board)) |

**Tergrid — Mono-Black Discard & Edicts** (Tier 3)

| Engine | Status | What is missing |
|---|---|---|
| Tergrid's steal | Full | - |
| edicts | Approximate | edicts: Approximate (the victim sacrifices its lowest-value creature) |
| discard | Approximate | discard: Approximate (targeted discard picks the fullest hand; random discard at random; opponents choose their worst card for symmetric discard) |
| Grave Pact / Dictate | Approximate | edicts: Approximate (the victim sacrifices its lowest-value creature) |
| Cabal Coffers + Urborg | Full | - |

**Chulane — Bant Value & Aluren** (Tier 4)

| Engine | Status | What is missing |
|---|---|---|
| Chulane cast trigger | Full | - |
| Aluren free casting | Approximate | aluren: Approximate (free creature spells MV 3 or less; the Chulane loop is a combo) |
| self-bounce creatures | Full | - |
| creature tutors | Approximate | creature_tutors: Approximate (Natural Order, GSZ, Finale, Eldritch Evolution, Chord onto the battlefield; the target is the best card (Craterhoof on a wide board)) |

**Grand Arbiter Augustin IV — Azorius Stax** (Tier 4)

| Engine | Status | What is missing |
|---|---|---|
| static cost increases | Full | - |
| one-spell-per-turn limits | Full | - |
| Drannith lock | Full | - |
| Isochron + Reversal | Approximate | isochron_reversal: Approximate (combo: needs 3+ mana from nonland permanents and a mana sink; loop abstracted) |
| counterspells | Approximate | counter_ai: Approximate (counters by importance; spells that complete a combo count as critical) |

**Krenko, Mob Boss — Mono-Red Goblins** (Tier 4)

| Engine | Status | What is missing |
|---|---|---|
| Krenko's token activation | Full | - |
| ETB damage | Full | - |
| Goblin Bombardment | Full | - |
| Kiki-Jiki + Conscripts | Approximate | kiki_combo: Approximate (combo: Kiki / Reflection + Conscripts / Felidar / Resto; loop abstracted) |
| Thornbite Staff loop | Approximate | thornbite_untap: Approximate (combo: Krenko + Thornbite Staff + sacrifice outlet) |
| Goblin tutors | Approximate | creature_tutors: Approximate (Natural Order, GSZ, Finale, Eldritch Evolution, Chord onto the battlefield; the target is the best card (Craterhoof on a wide board)) |

**Prosper — Rakdos Exile & Treasure** (Tier 4)

| Engine | Status | What is missing |
|---|---|---|
| Prosper Treasure on exile-cast | Full | - |
| impulse draw | Approximate | impulse_draw: Approximate (exile-and-play cards go to hand until they expire) |
| Treasure payoffs | Full | - |
| Bolas's Citadel | Full | - |
| Sanguine Bond + Exquisite Blood | Approximate | sanguine_bond: Approximate (combo: Sanguine Bond + Exquisite Blood; loop abstracted) |

**Yuriko — Dimir Ninja Tempo** (Tier 4)

| Engine | Status | What is missing |
|---|---|---|
| ninjutsu | Full | - |
| Yuriko reveal damage | Full | - |
| top-of-library manipulation | Approximate | top_manipulation: Approximate (Brainstorm puts back the two worst cards; Top / Scroll Rack / Ponder not modeled) |
| free counterspells | Full | - |

**Kinnan — Simic Mana Combo** (Tier 5)

| Engine | Status | What is missing |
|---|---|---|
| Kinnan's extra mana | Full | - |
| Basalt / Freed / Pemmin loops | Full | - |
| Kinnan activation | Full | - |
| Isochron + Reversal | Approximate | isochron_reversal: Approximate (combo: needs 3+ mana from nonland permanents and a mana sink; loop abstracted) |
| free counterspells | Full | - |

**Urza, Lord High Artificer — Mono-Blue Artifacts** (Tier 5)

| Engine | Status | What is missing |
|---|---|---|
| artifact fast mana | Approximate | artifact_mana: Approximate (rocks and Monoliths tap for mana; untap loops are combos) |
| Urza tapping artifacts, Construct | Full | - |
| Isochron + Reversal | Approximate | isochron_reversal: Approximate (combo: needs 3+ mana from nonland permanents and a mana sink; loop abstracted) |
| Power Artifact / Rings loops | Approximate | power_artifact: Approximate (combo with Basalt/Grim Monolith (Rings + Basalt too); loop abstracted) |
| Karn + Lattice lock | Approximate | karn_lattice: Approximate (Karn shuts off opponents' artifact mana; with Lattice a lock (no mana from permanents)) |
| taxing artifacts | Full | - |
| free counterspells | Full | - |

**Winota — Boros Humans & Hatebears** (Tier 5)

| Engine | Status | What is missing |
|---|---|---|
| Winota's attack trigger | Full | - |
| Kiki-Jiki copy combos | Approximate | kiki_combo: Approximate (combo: Kiki / Reflection + Conscripts / Felidar / Resto; loop abstracted) |
| stax Humans | Full | - |
| protection spells | Approximate | protection_ai: Approximate (outside decks answer targeted removal and wipes with their protection cards (and sacrifice creatures in response for value)) |

**Yawgmoth — Mono-Black Undying Combo** (Tier 5)

| Engine | Status | What is missing |
|---|---|---|
| Yawgmoth sac / -1/-1 / draw | Full | - |
| undying and counter cancellation | Full | - |
| death-trigger drains | Approximate | Blood Artist: Approximate; Zulaport Cutthroat: Approximate |
| Gravecrawler + Altar | Approximate | gravecrawler_loop: Approximate (combo with Phyrexian Altar, another Zombie and a death payoff) |
| Necropotence / Ad Nauseam / Citadel | Full | - |

**Zur the Enchanter — Esper Control & Oracle** (Tier 5)

| Engine | Status | What is missing |
|---|---|---|
| Zur's attack trigger | Full | - |
| Necropotence | Full | - |
| Thassa's Oracle + Consultation / Pact | Approximate | thoracle_combo: Approximate (combo: both spells cast through the counter window; win abstracted) |
| free counterspells | Full | - |
| graveyard exile | Approximate | graveyard_hate: Approximate (Rest in Peace keeps graveyards empty, Bojuka Bog, Dauthi void exile, Crypt / Lantern in response to reanimation or proactively) |


## 6. Limitations

- **The AI is heuristic, one decision at a time.** It has no look-ahead. It plays creature and value decks well. Stax, control and multi-step combo turns are where it is weakest: see section 3.
- **Politics are not modeled.** Attack and removal targets come from threat value (board value, planeswalkers near an ultimate, combo pieces). There are no deals.
- **Tier 2 games stall.** About 20% time out; the leader by life and board is then scored the winner.
- **Old mode is untouched.** The four main decks keep their own AI, and in pool games they play by the pool rules.
