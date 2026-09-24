# Opponent pool results

These results test the four main decks against a fixed field: the 25 decks in `opponents/`, five tiers of five.
In each game, one of your decks faces three decks drawn from one tier without replacement, with seats shuffled.
Pool mode uses the adaptive AI unless noted.
Games are seeded, so any cell can be reproduced with `compare.py --deck <d> --pool <tier> --games N`.

Contents:
1. Deck × tier matrix
2. Calibration: within-tier balance and tier ordering
3. Old-mode regression
4. Card coverage
5. Limitations

## 1. Deck × tier matrix

Each cell is 10,000 games. It shows the win rate of the deck in that row against three decks from that column's tier, with a 95% Wilson interval.
An even share is 25%.
Generated with `python3 compare.py --all-decks --pool all --games 10000` (1,199 s on 24 workers).

**Noise band.** At n=10,000 a cell's 95% interval is about ±0.8 pts near 25% and ±1.0 pts near 50%.
Cells in the same column and profile use the same seeds, so the opponents drawn and the seats match row to row.
The two profiles also use the same seeds, so a profile difference under about 1 pt is noise.
Two games in the Sauron vs Tier 3 cells, one per profile, hit an error and were skipped: Helm of the Host copied a legendary creature and the legend rule removed the original.
That bug is fixed.

**Conservative profile**

| deck | T1 (High B2 / Low B3) | T2 (Mid B3) | T3 (High B3) | T4 (Low B4) | T5 (High B4) |
|---|---|---|---|---|---|
| seph | 58.6% (57.6-59.5) | 58.1% (57.1-59.1) | 52.1% (51.1-53.1) | 64.7% (63.8-65.7) | 34.1% (33.1-35.0) |
| najeela | 34.4% (33.4-35.3) | 32.8% (31.9-33.7) | 33.4% (32.4-34.3) | 52.8% (51.8-53.7) | 30.3% (29.4-31.2) |
| sauron | 23.0% (22.2-23.8) | 19.9% (19.1-20.7) | 17.9% (17.2-18.7) | 31.4% (30.5-32.4) | 25.6% (24.7-26.4) |
| veyran | 20.9% (20.2-21.7) | 22.7% (21.9-23.5) | 18.6% (17.8-19.3) | 33.4% (32.5-34.4) | 18.4% (17.7-19.2) |

**Loose profile**

| deck | T1 (High B2 / Low B3) | T2 (Mid B3) | T3 (High B3) | T4 (Low B4) | T5 (High B4) |
|---|---|---|---|---|---|
| seph | 59.3% (58.3-60.2) | 58.3% (57.4-59.3) | 51.6% (50.7-52.6) | 65.6% (64.6-66.5) | 35.2% (34.3-36.1) |
| najeela | 34.4% (33.4-35.3) | 32.9% (32.0-33.8) | 32.5% (31.6-33.4) | 51.4% (50.4-52.3) | 30.5% (29.6-31.4) |
| sauron | 23.6% (22.8-24.4) | 20.5% (19.7-21.3) | 18.1% (17.4-18.9) | 31.3% (30.4-32.2) | 25.6% (24.8-26.5) |
| veyran | 21.8% (21.0-22.6) | 22.5% (21.7-23.4) | 18.9% (18.2-19.7) | 33.6% (32.7-34.6) | 18.6% (17.8-19.3) |

**Reading the matrix.**
- **Your decks keep the old-mode order against every tier:** Sephiroth > Najeela > Sauron ≈ Veyran.
  Sephiroth is above an even share everywhere, including Tier 5.
  Veyran is below an even share against every tier except Tier 4.
- **From Tier 1 to Tier 3, win rates fall a little per tier,** by about 6 pts in total for Sephiroth and about 5 for Sauron.
  Najeela stays flat at about 33%.
  Tier 5 is the hardest column for Sephiroth and Najeela and ties with Tier 3 for Veyran. Sauron does better against Tier 5 than against Tiers 1–3.
- **Do not read the T4 column as a harder field than T3.** Every deck posts its best result against Tier 4.
  This is the tier-ordering problem in section 2: the Tier 4 decks are modeled weaker than the Tier 3 decks.
  Use T4 numbers only to compare your lists with each other, not as a measure of Low-B4 strength.
- **The profile makes almost no difference in pool mode:** all differences are 1.4 pts or less.

## 2. Calibration

Commands: `python3 compare.py --calibrate all --games 2000` for both profiles, plus a cross-tier ladder that seats each pool deck alone against three decks from another tier.
Each check is n=2,000 games per tier.
For a deck, that means about 1,600 seats in the within-tier check and 400 games in the ordering check, so per-deck intervals are about ±2 pts and ±4 pts respectively.
No decklist was changed. Flags come with the likely cause:
- **modeling:** the sim does not capture what the deck does.
- **AI:** the pool AI plays it poorly.
- **list:** the deck itself is strong or weak for its tier.

### 2a. Within-tier balance (5 decks of one tier in 4-player pods; flag above 35% or below 15%)

| Tier | Deck | conservative | loose | Flag | Likely cause |
|---|---|---|---|---|---|
| T1 | Isshin | 28.1% | 28.2% | | |
| T1 | Lathril | 24.3% | 23.0% | | |
| T1 | Light-Paws | 23.1% | 23.3% | | |
| T1 | Tatyova | 21.1% | 20.8% | | |
| T1 | Teysa Karlov | 28.2% | 29.6% | | |
| T2 | Brago | 13.2% | 15.2% | LOW (cons.) | **AI + modeling.** Blink control wins by value over a long game, but a fifth of T2 games time out (see below), and the counter AI spends counters by importance rather than holding them for real threats. |
| T2 | Kaalia of the Vast | 14.9% | 16.4% | LOW (cons.) | **list + AI.** Kaalia is a single-commander plan; Tier 2 has plenty of removal for it, and the protection AI only reacts to targeted removal. |
| T2 | Lord Windgrace | 23.1% | 22.2% | | |
| T2 | Meren of Clan Nel Toth | 36.7% | 33.5% | HIGH (cons.) | **modeling.** Meren's end-step reanimation of ETB-removal creatures (Shriekmaw, Nekrataal) is Full. Opponents' graveyard hate only fires in response to reanimation spells, not to Meren's trigger, so the engine rarely gets answered. |
| T2 | Sythis | 37.2% | 37.5% | HIGH | **modeling + list.** The pillowfort taxes and caps work fully against the pool AI, which does not play around them or remove enchantments preferentially. The enchantress draw engine is Full-auto. |
| T3 | Atraxa | 36.0% | 36.5% | HIGH | **modeling.** Planeswalkers use one ability a turn and opponents under-attack them. Doubling Season is Full. |
| T3 | Aurelia | 39.5% | 38.1% | HIGH | **list + modeling.** Extra combats with Helm and Combat Celebrant are Full, and pool decks block conservatively. |
| T3 | Korvold | 15.3% | 15.7% | borderline | **AI.** Sacrifice decisions are tuned for value, not for racing. |
| T3 | Marwyn | 9.1% | 9.8% | LOW | **list for the tier.** A mono-green Elf deck with no interaction, in the tier with the most wipes; the Elf engines are Full. Hunter's Insight is never cast. |
| T3 | Tergrid | 25.1% | 24.8% | | |
| T4 | Chulane | 23.4% | 24.7% | | |
| T4 | Grand Arbiter Augustin IV | 8.5% | 7.3% | LOW | **modeling + AI.** Stax pieces tax and lock, but the deck has almost no clock: the Isochron + Reversal combo assembles in about 2% of games. Aven Mindcensor and Hushbringer are body-only. |
| T4 | Krenko | 32.6% | 32.7% | | |
| T4 | Prosper | 29.7% | 29.5% | | |
| T4 | Yuriko | 30.6% | 30.6% | | |
| T5 | Kinnan | 36.8% | 36.0% | HIGH | **modeling.** The combos (Basalt + Freed / Pemmin, Isochron) are abstracted once assembled. Opponents get one counter window and one removal window, with no real stack interaction. It assembles about a third of the time. |
| T5 | Urza | 23.0% | 23.7% | | |
| T5 | Winota | 35.1% | 35.0% | HIGH | **list + AI.** A fast aggressive deck in a combo tier. Kiki combos are abstracted; the pool AI's blockers undervalue Winota triggers. |
| T5 | Yawgmoth | 7.9% | 8.2% | LOW | **modeling.** Undying and -1/-1 cancellation are approximated, and the Yawgmoth / Gravecrawler loops need several pieces the AI rarely lines up. Necro and Ad Nauseam draw well, but the deck seldom converts. |
| T5 | Zur the Enchanter | 21.9% | 21.9% | | |

**Game length and timeouts (conservative):**

| Tier | Average game | Timeouts |
|---|---|---|
| T1 | 12.8 rounds | 3.0% |
| T2 | 16.4 rounds | 21.9% |
| T3 | 14.7 rounds | 13.3% |
| T4 | 13.6 rounds | 7.0% |
| T5 | 8.5 rounds | 0.8% |

Tier 2 times out in a fifth of games because blink, pillowfort, lands and recursion stall each other.
The pool AI lacks a closing plan for those board states. Loose is within a point of these figures.

### 2b. Tier ordering (each deck alone against three decks of the tier below; 25% = even)

A tier is ordered correctly when its decks win more than 25% against the tier below.

| Pairing | conservative | loose | Verdict |
|---|---|---|---|
| T2 into T1 | 30.5% | 30.5% | ordered |
| T3 into T2 | 25.2% | 25.2% | flat |
| **T4 into T3** | **13.0%** | **13.2%** | **INVERTED** |
| T5 into T4 | 26.7% | 26.3% | barely ordered |

**Per deck, loose profile** (an asterisk marks a result not above 25%):
- **T2 into T1:** Brago 24.4\*, Kaalia 24.3\*, Windgrace 31.1, Meren 34.5, Sythis 38.5.
- **T3 into T2:** Atraxa 28.4, Aurelia 34.6, Korvold 21.9\*, Marwyn 10.2\*, Tergrid 30.9.
- **T4 into T3:** Chulane 19.9\*, GAA 3.5\*, Krenko 12.8\*, Prosper 19.7\*, Yuriko 10.2\*.
- **T5 into T4:** Kinnan 38.0, Urza 23.4\*, Winota 34.9, Yawgmoth 10.6\*, Zur 24.7\*.

**Wider ladder** (conservative; one deck alone in pods of another tier):

| Tier alone | in T1 pods | in T2 pods | in T3 pods | in T4 pods |
|---|---|---|---|---|
| T2 | 30.5% | | 24.8% | |
| T3 | 30.6% | 25.2% | | |
| T4 | 17.0% | 14.1% | 13.0% | |
| T5 | | | 23.3% | 26.7% |

**Findings:**
1. **Tier 4 is modeled below Tier 1.** A lone Tier 4 deck wins 17% against Tier 1, 14% against Tier 2 and 13% against Tier 3.
   The cause is mostly modeling and AI, not the lists:
   - **Combos rarely assemble in the sim.** Chulane gets there in about 9% of games, Krenko about 5%, GAA about 2% and Yuriko almost never; compare Kinnan at 32%, Urza 25% and Zur 19%.
     Each needs tutor chains and top-of-library manipulation that is only approximated. Brainstorm is modeled, but Top, Scroll Rack and Ponder are not.
   - **The Tier 4 decks spend slots on interaction and card selection the sim values modestly.** Counters are spent by importance, and stax is modeled only as taxes.
   - **A lone higher-tier deck draws attacks.** Its threat value makes it the archenemy, which costs a few points in every ordering test.
   - **GAA has almost no clock**, and the Tier 4 decks durdle more often (GAA 21% of games).
2. **Tier 3 over Tier 2 is flat (25%).**
   - Atraxa, Aurelia and Tergrid are fine.
   - Marwyn (about 10%) and Korvold (about 22%) drag the tier down.
   - Tier 2's long, stalled games blunt the aggressive decks.
3. **Tier 5 over Tier 4 is carried by Kinnan and Winota.**
   - Yawgmoth is weak everywhere: 5–10%.
   - Tier 5 into Tier 3 is 23%, below even.
4. **Tiers 1 and 2 are well ordered,** and Tier 3 into Tier 1 is 30.6%.

**What this means for the matrix.** Columns T1, T2, T3 and T5 behave like increasingly strong fields, T3 and T5 far apart.
T4 is a weaker field than T1.
Until the T4 combo and selection models improve, treat "vs T4" as a sanity check, not a power reading.

## 3. Old-mode regression

Old mode (`compare.py` without `--pool`) is unchanged by this work. Three guards enforce that:
- `engine.POOL_RULES` is False outside pool games, so new rules such as Boots haste, fetch cracking and walker attacks never apply there.
- A hook on any card in the four main decks is inactive outside pool games.
- A fixture test (`tests/test_old_mode.py`) fingerprints 500 games per profile and requires an exact match.

n=10,000 per profile on this branch gives:

| Deck | conservative | loose | Reference in the prompt |
|---|---|---|---|
| Sephiroth | 46.9% | 47.5% | 50.8 / 51.8 |
| Najeela | 24.9% | 24.2% | 27.8 / 26.6 |
| Sauron | 18.2% | 18.4% | 13.9 / 14.0 |
| Veyran | 10.0% | 9.8% | 7.4 / 7.6 |

These numbers are identical, game for game, to the starting commit `db6149a`, so this branch changed nothing.
The reference numbers come from an older state of the repo:
- `73a7a77` gives 48.2/49.3 for Sephiroth.
- `9cddd67` gives 49.2/50.8.
- The Erebos modeling and the decklist updates (`c255d7d` / `db6149a`) moved the numbers to their current values.

The pool matrix keeps the same order: Sephiroth > Najeela > Sauron ≈ Veyran.

## 4. Card coverage

`python3 pool_audit.py` (or `--md`) regenerates this. The statuses are:
- **Full:** a hand implementation or reviewed override.
- **Full-auto:** the ability compiler handles every clause.
- **Approximate:** the effect is modeled but simplified; each card lists how.
- **Partial:** some abilities are missing.
- **Unmodeled:** the card is cast as a body or not at all.

Counts are per unique card per deck.

| | Full | Full-auto | Approximate | Partial | Unmodeled |
|---|---|---|---|---|---|
| Before this work | 663 | 296 | 208 | 665 | 232 |
| Now | 1072 | 222 | 622 | 140 | 8 |

Many Full-auto and Partial cards moved to Approximate, because the audit now reports honestly where the compiler reads a card but misses a clause.

Every deck engine is at least Approximate except those listed as Partial below: Isshin's Hordechief, Light-Paws' enchantress draw, Sythis's Starfield, Atraxa's proliferate (Tekuthal) and Krenko's Goblin tutors (Muxus).
No engine is Unmodeled. Each deck's engines and what they are missing:

| Tier | Deck | Full | Full-auto | Approximate | Partial | Unmodeled | Engines: full / approx. / partial / unmodeled |
|---|---|---|---|---|---|---|---|
| 1 | Isshin | 45 | 7 | 25 | 7 | 0 | 2 / 1 / 1 / 0 |
| 1 | Lathril | 48 | 13 | 17 | 3 | 0 | 2 / 2 / 0 / 0 |
| 1 | Light-Paws | 35 | 4 | 23 | 10 | 1 | 1 / 2 / 1 / 0 |
| 1 | Tatyova | 46 | 13 | 17 | 5 | 1 | 2 / 2 / 0 / 0 |
| 1 | Teysa Karlov | 37 | 20 | 18 | 4 | 0 | 2 / 2 / 0 / 0 |
| 2 | Brago | 40 | 13 | 23 | 4 | 0 | 1 / 2 / 0 / 0 |
| 2 | Kaalia of the Vast | 43 | 7 | 31 | 3 | 0 | 1 / 2 / 0 / 0 |
| 2 | Lord Windgrace | 51 | 16 | 13 | 9 | 0 | 2 / 2 / 0 / 0 |
| 2 | Meren of Clan Nel Toth | 43 | 18 | 19 | 1 | 0 | 1 / 4 / 0 / 0 |
| 2 | Sythis | 42 | 12 | 22 | 3 | 0 | 3 / 1 / 1 / 0 |
| 3 | Atraxa | 50 | 8 | 27 | 3 | 0 | 2 / 1 / 1 / 0 |
| 3 | Aurelia | 41 | 5 | 31 | 5 | 0 | 1 / 3 / 0 / 0 |
| 3 | Korvold | 45 | 13 | 26 | 3 | 0 | 2 / 2 / 0 / 0 |
| 3 | Marwyn | 33 | 11 | 25 | 6 | 1 | 3 / 1 / 0 / 0 |
| 3 | Tergrid | 34 | 12 | 20 | 9 | 0 | 1 / 4 / 0 / 0 |
| 4 | Chulane | 47 | 9 | 27 | 5 | 0 | 1 / 3 / 0 / 0 |
| 4 | Grand Arbiter Augustin IV | 44 | 6 | 29 | 4 | 0 | 1 / 4 / 0 / 0 |
| 4 | Krenko | 32 | 4 | 27 | 11 | 1 | 2 / 3 / 1 / 0 |
| 4 | Prosper | 47 | 5 | 28 | 4 | 0 | 2 / 3 / 0 / 0 |
| 4 | Yuriko | 44 | 6 | 21 | 9 | 3 | 2 / 2 / 0 / 0 |
| 5 | Kinnan | 48 | 3 | 30 | 10 | 0 | 3 / 2 / 0 / 0 |
| 5 | Urza | 36 | 3 | 29 | 11 | 1 | 3 / 4 / 0 / 0 |
| 5 | Winota | 47 | 3 | 31 | 7 | 0 | 1 / 3 / 0 / 0 |
| 5 | Yawgmoth | 39 | 7 | 31 | 2 | 0 | 2 / 3 / 0 / 0 |
| 5 | Zur the Enchanter | 55 | 4 | 32 | 2 | 0 | 3 / 2 / 0 / 0 |

**Isshin — Mardu Attack Triggers** (Tier 1)

| Engine | Status | What is missing |
|---|---|---|
| Isshin's attack-trigger doubling | Full | - |
| token-on-attack creatures | Approximate | Brimaz, King of Oreskos: Approximate; Legion Warboss: Approximate; Goblin Rabblemaster: Approximate; Tilonalli's Summoner: Approximate; attack_triggers: Approximate (attack triggers: hooks for the key cards, compiled text for the rest) |
| Hellrider / Brutal Hordechief per-attacker damage | Partial | Brutal Hordechief: Partial; attack_triggers: Approximate (attack triggers: hooks for the key cards, compiled text for the rest) |
| extra combats | Full | - |

**Lathril — Golgari Elves** (Tier 1)

| Engine | Status | What is missing |
|---|---|---|
| Elf-scaling mana | Full | - |
| Lathril's tokens and tap-ten drain | Full | - |
| lords and overruns | Approximate | Ezuri, Renegade Leader: Approximate |
| Skullclamp | Approximate | Skullclamp: Approximate |

**Light-Paws — Mono-White Aura Voltron** (Tier 1)

| Engine | Status | What is missing |
|---|---|---|
| Light-Paws Aura chain | Approximate | auras: Approximate (Auras attach to the best creature (the commander in voltron decks); bonuses, keywords, protection; hostile Auras modeled as exile) |
| enchantress draw | Partial | Eidolon of Countless Battles: Partial |
| totem armor | Approximate | Felidar Umbra: Approximate; auras: Approximate (Auras attach to the best creature (the commander in voltron decks); bonuses, keywords, protection; hostile Auras modeled as exile) |
| commander damage | Full | - |

**Tatyova — Simic Landfall Ramp** (Tier 1)

| Engine | Status | What is missing |
|---|---|---|
| Tatyova landfall draw | Full-auto | - |
| extra land drops | Approximate | Dryad of the Ilysian Grove: Approximate |
| landfall token output | Full-auto | - |
| land recursion | Approximate | Life from the Loam: Approximate; land_recursion: Approximate (Crucible / Ramunap / Greenwarden land drops from the graveyard; Loam dredge when short on lands) |

**Teysa Karlov — Orzhov Aristocrats** (Tier 1)

| Engine | Status | What is missing |
|---|---|---|
| Teysa's death-trigger doubling | Full | - |
| drain payoffs | Full-auto | - |
| forced sacrifice | Approximate | edicts: Approximate (the victim sacrifices its lowest-value creature) |
| Skullclamp | Approximate | Skullclamp: Approximate |

**Brago — Azorius Blink Control** (Tier 2)

| Engine | Status | What is missing |
|---|---|---|
| flicker ETB re-use | Approximate | Deadeye Navigator: Approximate; Eldrazi Displacer: Approximate; flicker: Approximate (blink re-triggers ETBs (Brago, Soulherder, Closet, Displacer, Deadeye, Ephemerate, Resto); Deadeye soulbond read as a paid blink) |
| counterspell decisions | Approximate | counter_ai: Approximate (counters by importance; spells that complete a combo count as critical) |
| monarch | Full | - |

**Kaalia of the Vast — Mardu Angels, Demons & Dragons** (Tier 2)

| Engine | Status | What is missing |
|---|---|---|
| Kaalia's put-onto-battlefield-attacking | Full | - |
| commander protection | Approximate | Lightning Greaves: Approximate; Swiftfoot Boots: Approximate; Mother of Runes: Approximate; Giver of Runes: Approximate; Selfless Spirit: Approximate; Deflecting Swat: Approximate; protection_ai: Approximate (outside decks answer targeted removal and wipes with their protection cards (and sacrifice creatures in response for value)) |
| big flier combat | Approximate | Baneslayer Angel: Approximate; Terror of the Peaks: Approximate |

**Lord Windgrace — Jund Lands** (Tier 2)

| Engine | Status | What is missing |
|---|---|---|
| Windgrace loyalty and land recursion | Approximate | planeswalkers: Approximate (hand-written walkers use one ability a turn; attackers go after valuable walkers (pool games); walkers without an implementation use compiled loyalty abilities) |
| landfall token makers | Full-auto | - |
| Titania elementals | Full | - |
| Life from the Loam dredge | Approximate | Life from the Loam: Approximate; land_recursion: Approximate (Crucible / Ramunap / Greenwarden land drops from the graveyard; Loam dredge when short on lands) |

**Meren of Clan Nel Toth — Golgari Recursion** (Tier 2)

| Engine | Status | What is missing |
|---|---|---|
| Meren's end-step reanimation | Full | - |
| ETB removal re-use | Approximate | Shriekmaw: Approximate; Nekrataal: Approximate |
| sac outlets | Approximate | Viscera Seer: Approximate; Carrion Feeder: Approximate; Ashnod's Altar: Approximate |
| Grave Pact | Approximate | edicts: Approximate (the victim sacrifices its lowest-value creature) |
| Survival / Birthing Pod | Approximate | Survival of the Fittest: Approximate; survival_pod: Approximate (Survival discards the worst creature for the best; Pod upgrades by one MV) |

**Sythis — Selesnya Enchantress Pillowfort** (Tier 2)

| Engine | Status | What is missing |
|---|---|---|
| enchantress draw | Full-auto | - |
| attack taxes and caps | Approximate | Norn's Annex: Approximate; Silent Arbiter: Approximate; attacker_caps: Approximate (Crawlspace and Silent Arbiter attack caps; Silent Arbiter's block cap missing) |
| Sigil of the Empty Throne | Full-auto | - |
| Starfield of Nyx | Partial | Starfield of Nyx: Partial |
| Sythis cast trigger | Full-auto | - |

**Atraxa — Four-Color Superfriends** (Tier 3)

| Engine | Status | What is missing |
|---|---|---|
| planeswalker loyalty and ultimates | Approximate | Jace, the Mind Sculptor: Approximate; Teferi, Hero of Dominaria: Approximate; Elspeth, Sun's Champion: Approximate; Karn Liberated: Approximate; Oko, Thief of Crowns: Approximate; planeswalkers: Approximate (hand-written walkers use one ability a turn; attackers go after valuable walkers (pool games); walkers without an implementation use compiled loyalty abilities) |
| proliferate | Partial | Atraxa, Praetors' Voice: Approximate; Tekuthal, Inquiry Dominus: Partial |
| Doubling Season | Full | - |
| attack taxes | Full | - |

**Aurelia — Boros Extra Combats** (Tier 3)

| Engine | Status | What is missing |
|---|---|---|
| additional combat phases | Approximate | Port Razer: Approximate |
| attack triggers per combat | Approximate | Legion Warboss: Approximate; attack_triggers: Approximate (attack triggers: hooks for the key cards, compiled text for the rest) |
| equipment | Approximate | Embercleave: Approximate; Shadowspear: Approximate |
| Helm of the Host + Combat Celebrant | Full | - |

**Korvold — Jund Sacrifice & Treasure** (Tier 3)

| Engine | Status | What is missing |
|---|---|---|
| Korvold sacrifice draw | Full | - |
| Treasure / Food / Clue tokens | Approximate | Gilded Goose: Approximate; Tireless Provisioner: Approximate; food: Approximate (Food tokens: sacrifice fodder (Korvold, Trail of Crumbs, Goose mana, Savvy Hunter); eaten when low) |
| per-sacrifice damage | Full | - |
| Gravecrawler + Phyrexian Altar loop | Approximate | Phyrexian Altar: Approximate; gravecrawler_loop: Approximate (combo with Phyrexian Altar, another Zombie and a death payoff) |

**Marwyn — Mono-Green Elves** (Tier 3)

| Engine | Status | What is missing |
|---|---|---|
| Elf-scaling mana | Full | - |
| Gaea's Cradle | Full | - |
| Craterhoof overrun | Full | - |
| creature tutors | Approximate | Finale of Devastation: Approximate; creature_tutors: Approximate (Natural Order, GSZ, Finale, Eldritch Evolution, Chord onto the battlefield; the target is the best card (Craterhoof on a wide board)) |

**Tergrid — Mono-Black Discard & Edicts** (Tier 3)

| Engine | Status | What is missing |
|---|---|---|
| Tergrid's steal | Full | - |
| edicts | Approximate | Liliana's Triumph: Approximate; Plaguecrafter: Approximate; edicts: Approximate (the victim sacrifices its lowest-value creature) |
| discard | Approximate | Liliana of the Veil: Approximate; Tinybones, Trinket Thief: Approximate; discard: Approximate (targeted discard picks the fullest hand; random discard at random; opponents choose their worst card for symmetric discard) |
| Grave Pact / Dictate | Approximate | edicts: Approximate (the victim sacrifices its lowest-value creature) |
| Cabal Coffers + Urborg | Approximate | Cabal Coffers: Approximate; Urborg, Tomb of Yawgmoth: Approximate |

**Chulane — Bant Value & Aluren** (Tier 4)

| Engine | Status | What is missing |
|---|---|---|
| Chulane cast trigger | Full | - |
| Aluren free casting | Approximate | Aluren: Approximate; aluren: Approximate (free creature spells MV 3 or less; the Chulane loop is a combo) |
| self-bounce creatures | Approximate | Shrieking Drake: Approximate; Whitemane Lion: Approximate; Kor Skyfisher: Approximate |
| creature tutors | Approximate | Finale of Devastation: Approximate; Survival of the Fittest: Approximate; creature_tutors: Approximate (Natural Order, GSZ, Finale, Eldritch Evolution, Chord onto the battlefield; the target is the best card (Craterhoof on a wide board)) |

**Grand Arbiter Augustin IV — Azorius Stax** (Tier 4)

| Engine | Status | What is missing |
|---|---|---|
| static cost increases | Approximate | Lavinia, Azorius Renegade: Approximate |
| one-spell-per-turn limits | Approximate | Archon of Emeria: Approximate |
| Drannith lock | Full | - |
| Isochron + Reversal | Approximate | Isochron Scepter: Approximate; Dramatic Reversal: Approximate; isochron_reversal: Approximate (combo: needs 3+ mana from nonland permanents and a mana sink; loop abstracted) |
| counterspells | Approximate | Mana Drain: Approximate; Flusterstorm: Approximate; counter_ai: Approximate (counters by importance; spells that complete a combo count as critical) |

**Krenko, Mob Boss — Mono-Red Goblins** (Tier 4)

| Engine | Status | What is missing |
|---|---|---|
| Krenko's token activation | Full | - |
| ETB damage | Approximate | Purphoros, God of the Forge: Approximate |
| Goblin Bombardment | Full | - |
| Kiki-Jiki + Conscripts | Approximate | Zealous Conscripts: Approximate; Fable of the Mirror-Breaker // Reflection of Kiki-Jiki: Approximate; kiki_combo: Approximate (combo: Kiki / Reflection + Conscripts / Felidar / Resto; loop abstracted) |
| Thornbite Staff loop | Approximate | Thornbite Staff: Approximate; thornbite_untap: Approximate (combo: Krenko + Thornbite Staff + sacrifice outlet) |
| Goblin tutors | Partial | Goblin Recruiter: Approximate; Muxus, Goblin Grandee: Partial; creature_tutors: Approximate (Natural Order, GSZ, Finale, Eldritch Evolution, Chord onto the battlefield; the target is the best card (Craterhoof on a wide board)) |

**Prosper — Rakdos Exile & Treasure** (Tier 4)

| Engine | Status | What is missing |
|---|---|---|
| Prosper Treasure on exile-cast | Full | - |
| impulse draw | Approximate | Light Up the Stage: Approximate; Outpost Siege: Approximate; Valakut Exploration: Approximate; impulse_draw: Approximate (exile-and-play cards go to hand until they expire) |
| Treasure payoffs | Approximate | Storm-Kiln Artist: Approximate |
| Bolas's Citadel | Full | - |
| Sanguine Bond + Exquisite Blood | Approximate | sanguine_bond: Approximate (combo: Sanguine Bond + Exquisite Blood; loop abstracted) |

**Yuriko — Dimir Ninja Tempo** (Tier 4)

| Engine | Status | What is missing |
|---|---|---|
| ninjutsu | Approximate | Silver-Fur Master: Approximate |
| Yuriko reveal damage | Full | - |
| top-of-library manipulation | Approximate | Brainstorm: Approximate; Scroll Rack: Approximate; Sensei's Divining Top: Approximate; top_manipulation: Approximate (Brainstorm puts back the two worst cards; Top / Scroll Rack / Ponder not modeled) |
| free counterspells | Full | - |

**Kinnan — Simic Mana Combo** (Tier 5)

| Engine | Status | What is missing |
|---|---|---|
| Kinnan's extra mana | Full | - |
| Basalt / Freed / Pemmin loops | Approximate | Basalt Monolith: Approximate; Freed from the Real: Approximate; Pemmin's Aura: Approximate |
| Kinnan activation | Full | - |
| Isochron + Reversal | Approximate | Isochron Scepter: Approximate; Dramatic Reversal: Approximate; isochron_reversal: Approximate (combo: needs 3+ mana from nonland permanents and a mana sink; loop abstracted) |
| free counterspells | Full | - |

**Urza, Lord High Artificer — Mono-Blue Artifacts** (Tier 5)

| Engine | Status | What is missing |
|---|---|---|
| artifact fast mana | Approximate | Grim Monolith: Approximate; Basalt Monolith: Approximate; Mox Opal: Approximate; artifact_mana: Approximate (rocks and Monoliths tap for mana; untap loops are combos) |
| Urza tapping artifacts, Construct | Full | - |
| Isochron + Reversal | Approximate | Isochron Scepter: Approximate; Dramatic Reversal: Approximate; isochron_reversal: Approximate (combo: needs 3+ mana from nonland permanents and a mana sink; loop abstracted) |
| Power Artifact / Rings loops | Approximate | Power Artifact: Approximate; Rings of Brighthearth: Approximate; power_artifact: Approximate (combo with Basalt/Grim Monolith (Rings + Basalt too); loop abstracted) |
| Karn + Lattice lock | Approximate | Karn, the Great Creator: Approximate; Mycosynth Lattice: Approximate; karn_lattice: Approximate (Karn shuts off opponents' artifact mana; with Lattice a lock (no mana from permanents)) |
| taxing artifacts | Full | - |
| free counterspells | Full | - |

**Winota — Boros Humans & Hatebears** (Tier 5)

| Engine | Status | What is missing |
|---|---|---|
| Winota's attack trigger | Full | - |
| Kiki-Jiki copy combos | Approximate | Zealous Conscripts: Approximate; Restoration Angel: Approximate; kiki_combo: Approximate (combo: Kiki / Reflection + Conscripts / Felidar / Resto; loop abstracted) |
| stax Humans | Approximate | Thalia, Heretic Cathar: Approximate |
| protection spells | Approximate | Flawless Maneuver: Approximate; Teferi's Protection: Approximate; Deflecting Swat: Approximate; Selfless Spirit: Approximate; protection_ai: Approximate (outside decks answer targeted removal and wipes with their protection cards (and sacrifice creatures in response for value)) |

**Yawgmoth — Mono-Black Undying Combo** (Tier 5)

| Engine | Status | What is missing |
|---|---|---|
| Yawgmoth sac / -1/-1 / draw | Full | - |
| undying and counter cancellation | Approximate | Mikaeus, the Unhallowed: Approximate; Geralf's Messenger: Approximate; Butcher Ghoul: Approximate |
| death-trigger drains | Approximate | Syr Konrad, the Grim: Approximate |
| Gravecrawler + Altar | Approximate | Phyrexian Altar: Approximate; gravecrawler_loop: Approximate (combo with Phyrexian Altar, another Zombie and a death payoff) |
| Necropotence / Ad Nauseam / Citadel | Full | - |

**Zur the Enchanter — Esper Control & Oracle** (Tier 5)

| Engine | Status | What is missing |
|---|---|---|
| Zur's attack trigger | Full | - |
| Necropotence | Full | - |
| Thassa's Oracle + Consultation / Pact | Approximate | Thassa's Oracle: Approximate; Demonic Consultation: Approximate; Tainted Pact: Approximate; thoracle_combo: Approximate (combo: both spells cast through the counter window; win abstracted) |
| free counterspells | Full | - |
| graveyard exile | Approximate | Rest in Peace: Approximate; Dauthi Voidwalker: Approximate; graveyard_hate: Approximate (Rest in Peace keeps graveyards empty, Bojuka Bog, Dauthi void exile, Crypt / Lantern in response to reanimation or proactively) |


### Partial and Unmodeled cards, and the decks they affect

Land utility abilities that are ignored (manlands, channel lands, sacrifice-to-draw lands) are listed too; these lands still make their mana.

| Card | Status | Decks | What is missing |
|---|---|---|---|
| Academy Ruins | Partial | Urza | land utility ignored: {1}{U}, {T}: Put target artifact card from your graveyard on top of yo |
| Archangel Avacyn // Avacyn, the Purifier | Partial | Kaalia of the Vast | flash, ETB indestructible; the transform is not modeled |
| Archfiend of Sorrows | Partial | Kaalia of the Vast | ETB -2/-2 to opposing creatures; unearth not modeled |
| Aven Mindcensor | Partial | Grand Arbiter Augustin IV | body only: search restriction not modeled |
| Bloodline Keeper // Lord of Lineage | Partial | Tergrid | {B}: Transform ~. Activate only if you control five or more Vampires. |
| Boseiju, Who Endures | Partial | Marwyn, Chulane, Kinnan | land utility ignored: Channel — {1}{G}, Discard this card: Destroy target artifact, enchantm |
| Brutal Hordechief | Partial | Isshin | drain per attacker modeled; the forced-block activation is not |
| Castle Embereth | Partial | Krenko | land utility ignored: {1}{R}{R}, {T}: Creatures you control get +1/+0 until end of turn. |
| Castle Locthwain | Partial | Tergrid, Yawgmoth | land utility ignored: {1}{B}{B}, {T}: Draw a card, then you lose life equal to the number of |
| Celestial Mantle | Partial | Light-Paws | +3/+3; the life-doubling combat trigger is modeled in a hook |
| Chain of Vapor | Partial | Kinnan, Urza | Return target nonland permanent to its owner's hand. Then that permanent's contr |
| Chaos Warp | Partial | Isshin, Kaalia of the Vast, Lord Windgrace, Aurelia, Korvold, Krenko, Prosper, Winota | shuffle-away modeled; the revealed-permanent replacement ignored |
| Conduit of Worlds | Partial | Lord Windgrace | land drops from the graveyard; casting permanents from the graveyard not used |
| Conspicuous Snoop | Partial | Krenko | body only |
| Crypt Ghast | Partial | Tergrid | Swamps tap for an extra B; extort not modeled |
| Dakmor Salvage | Partial | Lord Windgrace | land utility ignored: Dredge 2 |
| Dig Through Time | Partial | Yuriko | Delve; keywords not modeled: delve |
| Doom Whisperer | Partial | Tergrid | body only, abilities not modeled: Pay 2 life: Surveil 2.  |
| Draco | Partial | Yuriko | body only, abilities not modeled: Domain — This spell costs {2} less to cast for each basic land type am; Domain — At the beginning of your upkeep, sacrifice ~ unless you pay { |
| Druid Class | Partial | Tatyova | landfall life and the level-2 extra land drop; level 3 (land creature) not modeled |
| Eidolon of Countless Battles | Partial | Light-Paws | cast as a creature with its +1/+1 per creature and Aura; bestow not modeled |
| Elvish Spirit Guide | Partial | Kinnan | cast as a 2/2; the exile-for-G is not used |
| Emeria, the Sky Ruin | Partial | Light-Paws | land utility ignored: At the beginning of your upkeep, if you control seven or more Plains,  |
| Faerie Mastermind | Partial | Yuriko | Whenever an opponent draws their second card each turn, you draw a card. |
| Faerie Seer | Partial | Yuriko | body only, abilities not modeled: When this creature enters, scry 2.  |
| Ghost Quarter | Partial | Lord Windgrace | land utility ignored: {T}, Sacrifice ~: Destroy target land. Its controller may search their |
| Gitaxian Probe | Partial | Kinnan, Urza, Zur the Enchanter | Look at target player's hand. |
| Gix, Yawgmoth Praetor | Partial | Tergrid | draw for combat damage; the discard-to-play ability is not used |
| Goblin Cratermaker | Partial | Krenko | body only, abilities not modeled: {1}, Sacrifice ~: Choose one —; • This creature deals 2 damage to target creature.; • Destroy target colorless nonland permanent. |
| Golgari Charm | Partial | Meren of Clan Nel Toth | • regenerate each creature you control |
| Grim Hireling | Partial | Korvold, Prosper | two Treasures per player hit; the -X/-X ability is not used |
| Gryff's Boon | Partial | Light-Paws | +1/+0 and flying; graveyard recursion not modeled |
| Heartless Act | Partial | Lathril | • remove up to three counters from target creature |
| Hero of Iroas | Partial | Light-Paws | Aura discount; heroic not modeled |
| Hissing Quagmire | Partial | Lord Windgrace | land utility ignored: {1}{B}{G}: Until end of turn, ~ becomes a 2/2 black and green Elementa |
| Hive of the Eye Tyrant | Partial | Tergrid | land utility ignored: {3}{B}: Until end of turn, ~ becomes a 3/3 black Beholder creature wit |
| Hope of Ghirapur | Partial | Yuriko, Winota | body only, abilities not modeled: Sacrifice ~: Until your next turn, target player who was dealt combat  |
| Horizon Canopy | Partial | Sythis | land utility ignored: {1}, {T}, Sacrifice ~: Draw a card. |
| Hushbringer | Partial | Grand Arbiter Augustin IV | body only: stopping ETB and dies triggers is not modeled |
| Jagged-Scar Archers | Partial | Lathril, Marwyn | P/T = Elves you control; the flier-shooting ability is not modeled |
| Jaxis, the Troublemaker | Partial | Isshin | Blitz {1}{R}; keywords not modeled: blitz |
| Karn's Bastion | Partial | Atraxa | land utility ignored: {4}, {T}: Proliferate. |
| Kaya's Wrath | Partial | Teysa Karlov | Destroy all creatures. You gain life equal to the number of creatures you contro |
| Kor Haven | Partial | Light-Paws | land utility ignored: {1}{W}, {T}: Prevent all combat damage that would be dealt by target a |
| Loran of the Third Path | Partial | Winota | destroys an artifact/enchantment on entry; the draw ability is not used |
| Mardu Charm | Partial | Isshin | • target opponent reveals their hand; • you choose a noncreature, nonland card from it; • that player discards that card |
| Massacre Girl | Partial | Tergrid | body only, abilities not modeled: When ~ enters, each other creature gets -1/-1 until end of turn. Whene |
| Mishra's Bauble | Partial | Urza | {T}, Sacrifice ~: Look at the top card of target player's library. Draw a card a |
| Mogg Fanatic | Partial | Krenko | body only, abilities not modeled: Sacrifice ~: It deals 1 damage to any target. |
| Mortuary Mire | Partial | Teysa Karlov | land utility ignored: When ~ enters, you may put target creature card from your graveyard on |
| Mosswort Bridge | Partial | Tatyova, Lord Windgrace | land utility ignored: Hideaway 4; {G}, {T}: You may play the exiled card without paying its mana cost if |
| Multani, Yavimaya's Avatar | Partial | Lord Windgrace | +1/+1 per land on the battlefield and in the graveyard; the graveyard recursion is not used |
| Mutavault | Partial | Tergrid | land utility ignored: {1}: This land becomes a 2/2 creature with all creature types until en |
| Muxus, Goblin Grandee | Partial | Krenko | Goblins from the top six onto the battlefield; the attack pump is not used |
| Mystic Sanctuary | Partial | Brago, Grand Arbiter Augustin IV | mana only; the instant/sorcery return is ignored |
| Needle Spires | Partial | Aurelia, Winota | land utility ignored: {2}{R}{W}: Until end of turn, ~ becomes a 2/1 red and white Elemental  |
| Noble Hierarch | Partial | Chulane | Exalted |
| Oath of Teferi | Partial | Atraxa | When ~ enters, exile another target permanent you control. Return it to the batt; You may activate the loyalty abilities of planeswalkers you control twice each t |
| Oran-Rief, the Vastwood | Partial | Tatyova | land utility ignored: {T}: Put a +1/+1 counter on each green creature that entered this turn |
| Paradise Druid | Partial | Kinnan | This creature has hexproof as long as it's untapped. |
| Pashalik Mons | Partial | Krenko | a Goblin dying deals 1; the token-making activation is not used |
| Pendelhaven | Partial | Marwyn | land utility ignored: {T}: Target 1/1 creature gets +1/+2 until end of turn. |
| Pia Nalaar | Partial | Isshin | {1}{R}: Target artifact creature gets +1/+0 until end of turn.; {1}, Sacrifice an artifact: Target creature can't block this turn. |
| Ponder | Partial | Grand Arbiter Augustin IV, Yuriko, Kinnan, Urza, Zur the Enchanter | Look at the top three cards of your library, then put them back in any order. Yo |
| Quirion Ranger | Partial | Marwyn, Kinnan | body only, abilities not modeled: Return a Forest you control to its owner's hand: Untap target creature |
| Rabble Rousing | Partial | Isshin, Aurelia | a Citizen per attacker; hideaway ignored |
| Raging Ravine | Partial | Lord Windgrace, Korvold | land utility ignored: {2}{R}{G}: Until end of turn, ~ becomes a 3/3 red and green Elemental  |
| Ranger-Captain of Eos | Partial | Aurelia, Chulane, Winota | Sacrifice ~: Your opponents can't cast noncreature spells this turn. |
| Rapid Hybridization | Partial | Tatyova, Kinnan, Urza | Destroy target creature. It can't be regenerated. That creature's controller cre |
| Reforge the Soul | Partial | Krenko, Prosper | Each player discards their hand, then draws seven cards.   [partly: each player ; Miracle {1}{R} |
| Reliquary Tower | Partial | Brago | land utility ignored: You have no maximum hand size. |
| Sai, Master Thopterist | Partial | Urza | Thopter per artifact spell; the sacrifice-for-cards ability is not used |
| Sakashima's Protege | Partial | Yuriko | body only, abilities not modeled: Cascade; You may have ~ enter as a copy of any permanent that entered this turn; keywords not modeled: cascade |
| Sentinel's Eyes | Partial | Light-Paws, Sythis | +1/+1 vigilance; escape not modeled |
| Silent Clearing | Partial | Teysa Karlov | land utility ignored: {1}, {T}, Sacrifice ~: Draw a card. |
| Simic Growth Chamber | Partial | Tatyova | land utility ignored: When ~ enters, return a land you control to its owner's hand. |
| Skyclave Apparition | Partial | Brago, Chulane | exile on entry; the token when it leaves ignored |
| Snap | Partial | Urza | Return target creature to its owner's hand. Untap up to two lands.   [partly: un |
| Sokenzan, Crucible of Defiance | Partial | Krenko | mana only; channel ignored |
| Starfield Mystic | Partial | Light-Paws | enchantment discount; +1/+1 counters not modeled |
| Starfield of Nyx | Partial | Sythis | returns an enchantment each upkeep; animating enchantments at five is not modeled |
| Sunbaked Canyon | Partial | Winota | land utility ignored: {1}, {T}, Sacrifice ~: Draw a card. |
| Sunfall | Partial | Light-Paws | Exile all creatures. Incubate X, where X is the number of creatures exiled this  |
| Tajic, Legion's Edge | Partial | Isshin | Prevent all noncombat damage that would be dealt to other creatures you control. |
| Tectonic Edge | Partial | Lord Windgrace | land utility ignored: {1}, {T}, Sacrifice ~: Destroy target nonbasic land. Activate only if  |
| Tekuthal, Inquiry Dominus | Partial | Atraxa | proliferate twice; its indestructible ability is not used |
| Temur Sabertooth | Partial | Chulane | body only (re-buying ETB creatures is not used) |
| The One Ring | Partial | Kinnan, Urza | When ~ enters, if you cast it, you gain protection from everything until your ne; At the beginning of your upkeep, you lose 1 life for each burden counter on ~.; {T}: Put a burden counter on ~, then draw a card for each burden counter on ~.   |
| Tolaria West | Partial | Urza | land utility ignored: Transmute {1}{U}{U} |
| Treasure Cruise | Partial | Yuriko | keywords not modeled: delve |
| Tyvar the Bellicose | Partial | Lathril | attacking Elves gain deathtouch; the +1/+1 counters on mana creatures are not modeled |
| Urza's Bauble | Partial | Urza | {T}, Sacrifice ~: Look at a card at random in target player's hand. You draw a c |
| Vault of the Archangel | Partial | Teysa Karlov | land utility ignored: {2}{W}{B}, {T}: Creatures you control gain deathtouch and lifelink unt |
| Venser, Shaper Savant | Partial | Brago | bounces a permanent on entry; the bounce-a-spell mode is ignored |
| War Room | Partial | Light-Paws, Tergrid, Krenko, Yawgmoth | land utility ignored: {3}, {T}, Pay life equal to the number of colors in your commanders' c |
| Waterlogged Grove | Partial | Kinnan | land utility ignored: {1}, {T}, Sacrifice ~: Draw a card. |
| Wheel of Fortune | Partial | Aurelia, Krenko, Prosper, Winota | Each player discards their hand, then draws seven cards.   [partly: each player  |
| Whirler Rogue | Partial | Yuriko | Tap two untapped artifacts you control: Target creature can't be blocked this tu |
| Wirewood Symbiote | Partial | Marwyn | body only, abilities not modeled: Return an Elf you control to its owner's hand: Untap target creature.  |
| Yavimaya Hollow | Partial | Marwyn | land utility ignored: {G}, {T}: Regenerate target creature. |
| Coat of Arms | Unmodeled | Krenko | the shared-type anthem is not modeled (too costly to compute per creature) |
| Enter the Infinite | Unmodeled | Yuriko | Draw cards equal to the number of cards in your library, then put a ca |
| Hunter's Insight | Unmodeled | Marwyn | the combat-damage draw is not modeled; never cast |
| Lim-Dûl's Vault | Unmodeled | Yuriko | Look at the top five cards of your library. As many times as you choos |
| Retreat to Coralhelm | Unmodeled | Tatyova | tap/untap and scry on landfall not modeled |
| Sigarda's Aid | Unmodeled | Light-Paws | You may cast Aura and Equipment spells as though they had flash.; Whenever an Equipment you control enters, you may attach it to target  |
| Tishana's Tidebinder | Unmodeled | Yuriko | hand tags (tide) are only acted on by the main decks' AI |
| Voltaic Key | Unmodeled | Urza | {1}, {T}: Untap target artifact. |

## 5. Limitations

- **Combos are abstracted.** Once all pieces are on the battlefield and the remaining spells resolve through the counter window, opponents get one instant-speed removal (or Tidebinder) answer. Then the loop resolves as a win or a lock.
  Assembly is modeled properly (tutors chase missing pieces). The loops themselves are not played out.
- **Top-of-library manipulation is thin.** Brainstorm puts back the two worst cards; Top, Scroll Rack and Ponder do nothing extra. This hurts Yuriko, GAA and the Tier 5 blue decks.
- **The pool AI is generic.** It uses shared priorities plus per-deck settings in `pool_decks.py`. Thin spots:
  - counter timing (it counters by importance and doesn't hold mana up)
  - blocking against extra combats and Winota
  - racing with sacrifice decks (Korvold)
  - closing out stalled boards (Tier 2 timeouts)
  - attacking planeswalkers
- **The four main decks keep their own AI in pool games.** They play by the pool-only rules (Boots haste, fetch cracking, walker attacks); old mode never uses them.
- **Politics is not modeled.** There are no deals or kingmaking. Target choice is by threat value, which makes a lone stronger deck the archenemy in ordering tests.
