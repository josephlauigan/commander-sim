# Tymna the Weaver & Kraum — findings (living file)

What the simulator has taught us about the Tymna/Kraum partner deck: strengths, weaknesses, and what was tried and why
it did or didn't work. Updated after every run. Newest evidence first; numbers are look-ahead, loose profile.

## Current state (2026-10-16, evening)
**Tuned list: 31.2% at T3** (4,000 games, ±1.4; `audit/kraum/2026-10-16-iter/L8-analyze-t3.txt`), the matched tier. Dies in
67% of games (was 86%), deals 37 damage a game (was 22), Tymna lands on turn 3.7 and Kraum on 6.0. Best when cast:
Sunblast Angel, Rune-Scarred Demon, Elspeth, Grave Titan, Terror of the Peaks, Lyra (55–57%). Still stuck in hand
40–48% of the time: the 5–6 drops (Sun Titan, Rune-Scarred Demon, Elspeth, Grave Titan, Dream Trawler, Gilgamesh) and
Diabolic Tutor, Fact or Fiction. Remaining losses: creature decks in combat (Aurelia 16%, Tergrid 12%, Marwyn 11%).

### The morning's list (2026-10-16)
List: the 10/16 build (Gilgamesh, Diabolic Tutor, eleven Equipment including Shield and Helm of Kaldra); every card
modeled. Matched tiers not set yet (provisional t1, t2 in `data/my_decks.toml`).
**Best so far: defense package E, T3 21.1% (+8.5, 1,000 paired) and E + C 20.4% (+7.4, 2,000 paired); not applied to the list.**
Win rate of the current list, 1,000 games per tier, **all five tiers (run with `--any-tier`, JD's OK)**: **T1 12.7%, T2 12.3%, T3 12.6%,
T4 13.4%, T5 12.1%; 12.6% over t1–t5** (±1.0 per tier). Fair band: 22–32%. Below it in every tier, and flat across
tiers: the deck doesn't beat weak tables more often than strong ones, which points at its own engine, not at matchups.
Files: `audit/kraum/2026-10-16/analyze-t1.txt` … `analyze-t5.txt`.

## Strengths
- **Tymna lands on time:** turn 3.8–3.9 on average, in 83–98% of games.
- **The finishers win when they resolve** (win rate when cast, T1): Lyra 37%, Terror of the Peaks 33%, Nightveil
  Predator 32%, Lightning 31%, Elenda 31%, Sphinx of New Prahv 30%, Marchesa 30%. Correlation, not causation, but
  these show up in the top ten in three or more tiers.
- **Conqueror's Flail** is in the top ten win rate when cast in four of five tiers (29% at T1): +4/+4 and no opponent
  spells on your turn.

## Weaknesses (by evidence)
1. **Tymna's engine barely runs.** She hits only **0.51–0.72 opponents per turn** (ideal: 2–3), so she draws **1.9–2.6
   cards per game**. The deck is under attack all game (eliminated in 86–88% of games), so the AI keeps creatures home
   to block ("holds back attackers that would die to blocks"), attacks planeswalkers, and loses its small creatures to
   wipes. Too few cheap evasive bodies reach the board early.
2. **Kraum is late:** turn 6.2–6.4 on average (he costs 5), and in T5 he lands in only 52% of games (the deck is dead
   by round 7.3). The mana is slow: twelve lands always enter tapped, and four colours with double pips strand cards.
   Chromatic Lantern sits in hand in 39% of the games it's drawn (T1).
3. **Expensive cards get stuck in hand** (cast in under half the games they're drawn, all five tiers): Gilgamesh
   (~36%), Shield of Kaldra (~37%), Elenda (~37%), Diabolic Tutor (~32%), Fact or Fiction (~26%), Dissipate (~25%,
   reactive), Fate of the Sun-Cryst (~35%, four tiers).
4. **It loses races:** T1–T3 losses are creature decks in combat (Light-Paws commander damage 17%, Isshin 16%, Brago
   18%, Aurelia 26% of games); T4–T5 losses are combos (Heliod 20%, Kinnan 24%, Urza 14%). Too little cheap
   interaction for either.
5. **The commanders are the removal magnets:** Kraum removed 0.37 and Tymna 0.35 times a game (T1).
6. **The evasion Equipment underperforms:** Medic's Kitesail and Nim Deathmantle are in the bottom ten win rate when
   cast in three tiers. They cost mana twice (cast and equip) in a deck that is already mana-starved, the same
   bottleneck the Cloud deck has. Toxic Deluge (four tiers) and the Talismans are also low, but cheap early cards
   always look low on this measure.

## What was tried, and why it did or didn't work
| Change | Result | Why |
|---|---|---|
| 10/16 build vs the 10/13 comparison build (13 swaps + 2 Kaldra) | 15.3% (10/13, older code) → 12.6% over t1–t5 | Not a paired test: the engine changed since 10/13 (land-mana fixes, double strike, Flail +1/+1 per colour). |
| **A mana** (Orzhov Signet, Spire of Industry for Star Compass, Spectacle Summit), T3, 1,000 paired | -0.5 (±2.7) | Commander by turn 4 +2.7 pts, mana on turn 4 +0.03: too small a change to matter. |
| **B cheap evasion** (Cloudfin Raptor, Callous Inspector, Thrummingbird, Rescue, Speed for Fate of the Sun-Cryst, Medic's Kitesail, Nim Deathmantle, Urza, Hero's Blade), T3 | +0.3 (±2.9) | **Damage +2.6 a game (significant)**, spells cast +0.8, but eliminations unchanged (86%): more pressure, no more wins. |
| **C cheap interaction** (Icy Reception, Destroy Evil, Rip Apart, Suplex for Dissipate, Make Your Move, Silverquill Charm, Bedevil), T3 | +1.0 (±3.0) | Eliminated 86 → 85%. The new 2-mana removal sits in hand 20–54% of the games it's drawn (Suplex 46%, Icy Reception 54%). |
| **D ~$30 buys** (Faerie Mastermind, Dauthi Voidwalker, Whispersilk Cloak, Spectral Sailor, Flawless Maneuver, Swords, Arcane Signet, Fellwar Stone, Talisman of Dominance, three checklands), T3 | -0.1 (±2.8) | Damage +2.2, mana turn 4 +0.13; eliminations unchanged. The 2-mana rocks are cast in only 62–70% of the games they're drawn. |

**Reading A–D together (2026-10-16):** every package adds pressure (damage +2 to +2.6 a game) but none changes how often the
deck dies (85–87%, mostly to creature decks in combat: Aurelia ~25%, Tergrid, Marwyn, Atraxa). The hand also stays full
(~6 cards on turns 4 and 8), so more cards aren't the limit. The deck loses the combat race; the Cloud deck had the same
problem and only attack taxes + wipes moved it (+6.4). Next: a defense package (E).

| **E defense** (~$26 buys: Ghostly Prison, Propaganda, Norn's Annex, Crawlspace, Windborn Muse, Elspeth, Sun's Champion, Aetherize, Baird for Medic's Kitesail, Nim Deathmantle, Fate of the Sun-Cryst, Urza, Hero's Blade, Dissipate, Make Your Move, Exit Specialist), T3, 1,000 paired | **+8.5 (±3.2): 12.6 → 21.1%** | **The first real gain.** Eliminated 86 → 77%, rounds survived +0.8, damage +4.0, Aurelia's share of the deaths 26 → 20%. The attack taxes are cast in 75–84% of the games they're drawn. Aetherize was never cast (reactive, the AI never finds the moment): replaced by Archon of Absolution. Elspeth sits in hand 58% (six mana) but wins 50% when cast. |
| **E + C** (E with Archon of Absolution for Aetherize, plus Rip Apart, Suplex, Destroy Evil for Silverquill Charm, Bedevil, Elemental Mascot), T3, 2,000 paired | **+7.4 (±2.3): 13.1 → 20.4%** | Same as E alone within noise: the extra removal adds nothing. Eliminated 86 → 78%. Still short of the fair band (22%+). The expensive defensive cards get stuck (Elspeth 64%, Baird 48%, Norn's Annex 46%), and mana on turn 4 is unchanged (3.5): the defense comes down late. |

### Iterating toward 30% at T3 (2026-10-16, JD: budget $50, keep Gilgamesh, Diabolic Tutor, Shield/Helm of Kaldra, Toxic Deluge, Fact or Fiction)
Baselines are re-measured in each paired run (same seeds), so a baseline number differs slightly from run to run.

| Step (on the list of the step before) | Result at T3 | Why |
|---|---|---|
| **L1 = E applied** (`0a7f5b8`) | 22.9% (1,000) / 20.6% (2,000 analysis) | Still dies in 75–77% of games; Tymna hits 0.56 opponents a turn; the big threats win 42–49% when cast but sit in hand. |
| R2a cheap mana (Arcane Signet, Fellwar Stone, Talisman of Dominance, 3 checklands) | -1.8 (±3.5) | Turn-4 mana +0.17; not enough to show. |
| R2b owned wipes (Austere Command, Vanquish the Horde) | -1.9 (±3.4) | Cast late (round 10–11), stuck 54–57%. |
| R2c owned cheap evasion (4 creatures) | -0.1 (±3.6) | Cast 86–93% of the time, no win change. |
| R2d more taxes (Archangel of Tithes, Sphere of Safety) | -1.0 (±3.5) | The first set of taxes is enough; more adds nothing. |
| **L2 = threat package T1** (Adeline, Hero of Bladehold, Vampire Nighthawk, Skyclave Apparition, Glorybringer, Baneslayer for the Equipment JD didn't ask for + Spectral Lynx, Silverquill Charm) (`11d3bfd`) | **+1.9 (±3.5)**; L2 = 24.4% (2,000) | Damage +2.5, eliminations +0.07 a game. Keeping the Equipment instead (T2) was -0.5. |
| R4a owned bodies (Callous Inspector, Cloudfin Raptor, Thrummingbird, Rescue for Chromatic Lantern, Vanish from Sight, Quick Study, Vibrant Outburst) | +1.4 (±2.5, 2,000) | Merged into the curve package L. |
| R4b Kaalia of the Vast package | -2.1 (±2.5, 2,000) | Dropped. |

Traces (read-only agent, 8 games): the AI mostly plays sensibly; the deck is mana- and curve-limited (36 lands, 12 always
tapped, 4 rocks, average mana value 3.39 with no one-drops). One small AI weakness: the look-ahead sometimes picks a
creature over a 2-mana rock (`ai/search.rs` choose_inner; `ramp_bonus` is off). Tymna dies and is recast every 2–3 turns.
| R5 L curve (avg MV 2.94; cut Baneslayer, Glorybringer, Norn's Annex, The Aetherspark too) | -0.5 (±2.5) | Key permanents removed -0.12 (significant), damage +2.3, but cutting the finishers cost the wins. Take Up the Shield never cast. |
| R5 M1 basics for the 12 always-tapped lands | +1.4 (±2.6) | Mana turn 4 +0.39, turn 8 +0.37. |
| **L3 = M2 checklands + basics** (Isolated Chapel, Drowned Catacomb, Sulfur Falls, Dragonskull Summit + 8 basics for the 12 tapped lands) (`2112c91`) | **+2.1 (±2.5)** | Mana turn 4 +0.33, eliminated -2.0. |
| **L4 = L′ curve, finishers kept** (Callous Inspector, Cloudfin Raptor, Thrummingbird, Rescue, Lightning Greaves, Masterful Flourish, Spectral Sailor, Dauthi Voidwalker, Swords, Path, Rip Apart for Chromatic Lantern, Vanish from Sight, Quick Study, Vibrant Outburst, Garland, Photon, Caparocti, The Aetherspark, Wander Off, Norn's Annex, Glorybringer; avg MV 2.98) (`75b0132`) | **+3.3 (±2.6): 23.9 → 27.1%** | Eliminated -3.5, damage +4.6, key permanents removed -0.12 (significant). |
| L4 analysis, T3, 4,000 games | **25.7%** (±1.4) | Tymna draws 2.74 a game (hits 0.66 opponents a turn); Elspeth 50%, Baneslayer 49%, Terror 49%, Marchesa 48% win when cast; Force of Will, Star Compass, Talisman of Hierarchy, The Wasp, Rescue lowest. |
| **L5 = R7a** (Restoration Angel for Force of Will, Arcane Signet and Fellwar Stone for Talisman of Hierarchy and Star Compass; avg MV 2.97) (`ba8c23c`) | +2.0 (±2.6) | Eliminated -2.0; Restoration Angel 39% win when cast. Force of Will gone: no Game Changers left. |

The AI's ramp setting (`SEARCH_RAMP_W`) can only be set for every seat at once (opponents too), so it was not used.
| R8a Selfless Spirit, Tithe Taker for Rescue, The Wasp | +0.4 (±2.5) | No effect. (R8c Lingering Souls stopped by JD before it finished.) |
| R9a Glorybringer, Sun Titan for Spectral Sailor, Cloudfin Raptor | +2.5 (±2.5) | JD lifted the average-mana-value cap (now: under 4). |
| **L6 = R9b finishers** (Glorybringer, Sun Titan, Grave Titan, Thundermaw Hellkite for Spectral Sailor, Cloudfin Raptor, Callous Inspector, Thrummingbird) (`18ef902`) | **+3.4 (±2.7): 25.8 → 29.1%** | Each finisher wins 52–58% when cast; eliminated -2.4, damage +2.3. |
| R10a Rune-Scarred Demon, Serra Angel for Rescue, The Wasp | +0.1 (±2.6) | No effect; L6 measured 29.1% again. |
| **L7 = R10b** (R10a + Dream Trawler, Sunblast Angel for Laelia, Abigale) (`24b4bd0`) | +1.8 (±2.7): 29.1 → 30.9% | Within noise; the gain is from Dream Trawler and Sunblast Angel (R10a alone did nothing). Upgrade buys now ~$48.90. |

**What moved the deck (2026-10-16):** defense (attack taxes, +8.5), then a lower curve with protection for Tymna (+3.3),
then finishers that win when cast (+3.4), with untapped mana (+2.1) and small cheap swaps on top. Cheap evasive creatures
alone, more cards, more removal, and more taxes did not move it.
| L7 analysis, T3, 4,000 games | 29.7% (±1.4) | Expensive finishers stuck in hand 38–46%: mana again. |
| R11a Orzhov Signet for Potioner's Trove, Plains for Scorching Dragonfire (37 lands) | +2.1 (±2.7) | Mana turn 8 +0.17. |
| **L8 = R11b** (R11a + Island for Harsh Annotation, 38 lands) (`75db0a5`) | **+3.1 (±2.7): 29.5 → 32.6%** | Mana turn 8 +0.29, damage +3.3, eliminated -2.3. |
| **L8 analysis, T3, 4,000 games** | **31.2% (±1.4)** | JD's goal (30% at T3) reached. |
| **S1 substitutes** (Austere Command, Dark Endurance, The Wasp, Swamp for Toxic Deluge, Masterful Flourish, Moseo, Shattered Sanctum: JD doesn't own them) (applied) | +1.8 (±2.7): 30.9 → 32.6% | No loss; Toxic Deluge was the weakest card when cast. |

### Candidate buys JD asked about (2026-10-16 night, T3, 2,000 paired games each; the list measured 34.0% in these runs)
| Candidate (in for) | Result | Why |
|---|---|---|
| Urza, Lord High Artificer, $17.55 (Elemental Mascot) | +1.1 (±2.6) | Within noise. |
| Winota, Joiner of Forces, $3.96 (Bedevil) | -2.1 (±2.6) | Winota fine (49% when cast); losing Bedevil (54% when cast, removal against the creature decks) cost more. |
| Winota redo (Tocasia's Welcome) | -0.9 (±2.5) | No gain with a fair cut either. |
| Purphoros, God of the Forge, $26.55 (Disenchant) | +0.7 (±2.7) | Its triggers add 3.4 damage a game (significant), but total damage only +0.6: the damage lands on players who were dying anyway. |
| Isshin, Two Heavens as One, $3.63 (Elemental Mascot) | -1.7 (±2.5) | Doubling needs Adeline, Hero or a Titan already attacking; a non-flying 3/4 adds nothing for Tymna. |
| Isshin + Winota (Elemental Mascot, Tocasia's Welcome) | -0.1 (±2.7) | No effect. |
| All three (Urza, Winota, Purphoros) | stopped by JD | It cut Bedevil too. |
None of them is worth adding. Lightning, Army of One is not modeled; on hold (JD: wait for these results).

### Phyrexian packages (2026-10-17 early morning, T3, 2,000 paired games each; the list measured 34.0–34.4%)
| Package (in for) | Result | Why |
|---|---|---|
| A: Elesh Norn, Grand Cenobite + Norn's Annex, $26.85 (Elemental Mascot, Tocasia's Welcome) | -0.1 (±2.6) | Elesh Norn wins 62% when cast (the best card tested) but sits in hand 63% of the time at 7 mana; a seventh attack tax adds nothing. |
| B: Elesh Norn, Mother of Machines, $27.35 (Elemental Mascot) | -0.3 (±2.6) | Stuck in hand 53% at 5 mana; 54% when cast. |
| E: Dismember, Porcelain Legionnaire, Vault Skirge, Apostle's Blessing, ~$5.50 (Memory Trap, Elemental Mascot, Tocasia's Welcome, Disenchant) | -1.6 (±2.8) | The cheap cards are cast 84–96% of the time, but the cards they replace were already doing their jobs; Apostle's Blessing is cast only 21% (reactive timing). Cards modeled for this test (Full, branch kraum-phyrexian). |
**Reading it:** nothing tried since the 31% list moves it. The deck now dies in about 64% of games, nearly all in combat to
Aurelia, Tergrid, Marwyn and Atraxa, and no single card or package changes that; the remaining gains would have to come from
the AI (attack and blocking decisions), not the list.

**2026-10-17 land audit** (branch `land-audit2`, `rust/agents/report-land-audit2.md`): 12 lands across the decks now make only the mana their Oracle text allows (filter lands, Rugged Prairie, Mystic Gate, Spire of Industry, Exotic Orchard, Cavern of Souls, Spectator Seating, Hidden Grotto and others; Sulfur Falls and Fabled Passage were too strict). Same-seed check (adaptive AI, 300 games per tier, matched tiers): 40.0 → 41.7, inside the noise (about ±5). No measurable change to this deck.

### Phyrexian round 2 (2026-10-17, T3, 2,000 paired games each; the list measured 34.9%)
Aimed at the real loss (creature decks in combat) with cheap cards, cutting non-combat slots instead of removal.
| Package (in for) | Result | Why |
|---|---|---|
| P1: Sheoldred's Edict, Dismember, Porcelain Legionnaire, Vault Skirge, ~$7.50 (Tocasia's Welcome, Elemental Mascot, Arcane Denial, Expressive Iteration) | -1.4 (±2.8) | Spot removal cast +0.20 a game (significant), but deaths unchanged (64%): the T3 creature decks have enough bodies that one more removal spell doesn't stop the attack. Lost the only counterspell (0.17 cast a game). |
| P2: Phyrexian Obliterator, Sheoldred's Edict, ~$11.50 (Elemental Mascot, Tocasia's Welcome) | -0.6 (±2.7) | Obliterator wins 59% when cast but its {B}{B}{B}{B} is cast in only 28% of the games it's drawn in a four-colour deck. |
Not modeled, so not tested: Archfiend of the Dross, Skrelv, Gut Shot, White Sun's Twilight, Phyrexian Unlife, Mondrak ($46).

### Phyrexian round 3 (2026-10-17, T3, 2,000 paired games each; five new cards modeled for it, 1,975 tests pass)
| Package (in for) | Result | Why |
|---|---|---|
| R3a: Archfiend of the Dross, $0.38 (Elemental Mascot) | -1.9 (±2.6) | Cast 75% (4 mana) but wins 49% when cast, the same as Elemental Mascot; a 6/6 flier adds pressure, not defense. Its oil-counter loss isn't among the deck's death causes. |
| **R3b: Skrelv, Gut Shot, White Sun's Twilight, ~$8 (Tocasia's Welcome, Expressive Iteration, Memory Trap)** | **+0.8 (±2.9)** | Best of the Phyrexian rounds, still noise: deaths -1.5, Marwyn's share of the deaths 12 → 9% (Gut Shot cast 83%, kills 1-toughness elves). White Sun's Twilight sits in hand 52% (the AI waits for X ≥ 5). |
| R3c: all five incl. Phyrexian Unlife (+ Arcane Denial out) | +0.1 (±2.8) | Phyrexian Unlife is cast in only 28% of the games it's drawn (the AI waits for 20 life or less) and wins 33% when cast. |
**Phyrexian conclusion (three rounds, eight packages):** none beats the list beyond noise. If JD wants one Phyrexian change, R3b
(Skrelv, Gut Shot, White Sun's Twilight) is the only candidate that points up, and it is specifically good against elves.

### Resplendent Angel for Dream Trawler (2026-10-17, applied: JD got Resplendent Angel instead of Dream Trawler)
| Swap | Result | Why |
|---|---|---|
| Resplendent Angel in, Dream Trawler out | -1.8 (±2.6), list 34.9 → 33.0% | Inside the noise. Resplendent Angel gets cast far more often (75% of the games it's drawn vs 45%; 3 mana vs 6) but wins less when cast (52% vs 63%): Dream Trawler was a hard-to-answer finisher when it landed, the Angel is an early flier whose 4/4 tokens need 5 life gained in one turn. Deaths +1.8, game shape otherwise unchanged. |

### Godo, Bandit Warlord for Serra Angel (2026-10-17, not applied: JD picked up Godo; modeled for this test)
| Swap | Result | Why |
|---|---|---|
| Godo in, Serra Angel out | -0.1 (±2.3), list 31.7 → 31.6% (`2026-10-17-godo/G1-godo.txt`) | No change. Godo is a 6-drop that gets stuck: cast in 43% of the games he's drawn vs Serra's 55%, first cast on round 11 like her. When he lands he fetches Helm of Kaldra (haste, first strike, trample; Hero's Heirloom, his best pick, is no longer in the list) and gets two combats that turn, and he wins a little more when cast (57% vs 50%, correlation). But the extra combat gives Tymna no extra cards (she counts opponents hit once per turn), and the deck's losses are defensive: deaths (67%), damage dealt and the killers (Aurelia, Tergrid, Marwyn in combat) are unchanged. A one-shot 3/3 tutor for an Equipment adds nothing a 5-mana 4/4 flyer doesn't. His real home is Helm of the Host (infinite combats), which JD doesn't own. |

### Avenge for Rip Apart (2026-10-17, APPLIED: JD's card from bulk)
| Swap | Result | Why |
|---|---|---|
| Avenge in, Rip Apart out | **+3.1 (±2.6)**, list 31.7 → 34.8% (`2026-10-17-godo/A1-avenge.txt`) | A second creature wipe beside Austere Command answers the deck's main way of dying, creature decks in combat: eliminated 67.0 → 63.7%, rounds survived +0.29, Aurelia kills 16.2 → 14.0%, Tergrid 13.1 → 11.9%. Surviving longer converts: damage +2.4 a game, opponents eliminated +0.05. The AI holds it (cast in 37% of the games it's drawn vs Rip Apart's 77%) until the wipe is worth losing the deck's own creatures; the attack taxes and rocks survive it. |

### Godo, Bandit Warlord for Sun Titan (2026-10-17, not applied; measured on the list before Avenge)
| Swap | Result | Why |
|---|---|---|
| Godo in, Sun Titan out | -0.2 (±1.9), list 31.7 → 31.5% (`2026-10-17-godo/G2-godo-suntitan.txt`) | A wash: two 6-drops that get stuck the same way (cast in 43% vs 45% of the games drawn, first cast round 10.5 vs 10.7) and win about as often when cast (60% vs 57%, correlation). Game shape unchanged to a tenth. With both Godo tests (Serra Angel -0.1, Sun Titan -0.2), Godo is a fair card here but no upgrade: his extra combat doesn't feed Tymna, and the deck's limit is surviving combat, not dealing damage. |

### Smaug the Magnificent (2026-10-17 evening, T3; modeled Full in `impls/smaug.rs`, branch `smaug`)
Smaug ({2}{R}{R}, 4/3 flying, haste; upkeep: a Treasure; attacks: damage equal to your Treasures to any target).
Quick adaptive passes (2,000 paired) picked the cut; the look-ahead run decides. Files: `audit/kraum/2026-10-17-smaug/`.
| Swap (Smaug in for) | Result | Why |
|---|---|---|
| Elemental Mascot (adaptive) | +0.1 (±1.3) | No effect. |
| Serra Angel (adaptive) | -0.1 (±1.2) | No effect: a 4-mana flier for a 4-mana flier. |
| Tocasia's Welcome (adaptive) | +1.8 (±1.2) | Best of the three on the quick AI. |
| **Tocasia's Welcome (look-ahead, loose, 2,000 paired)** | **+1.1 (±2.7): 33.5 → 34.6%** | Within noise. Smaug is cast in 64% of the games he's drawn (stuck 36%: {R}{R} in a four-colour deck with 3 Mountains) and wins 56% when cast vs Tocasia's Welcome's 44%. Damage +1.1 a game, eliminations -1.1 pts; the death causes (Aurelia, Tergrid, Marwyn in combat) are unchanged. His Treasure trigger adds almost nothing (about 0.1 burn a game on the quick run): the deck makes no Treasure of its own, so he attacks with 0–2. He is a hasty 4-mana flier here, not an engine. |
**Reading it:** a fair swap that doesn't hurt, but not a measurable gain. If JD wants Smaug in the deck, Tocasia's Welcome is the slot.
