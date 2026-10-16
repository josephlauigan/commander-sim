# Tymna the Weaver & Kraum — findings (living file)

What the simulator has taught us about the Tymna/Kraum partner deck: strengths, weaknesses, and what was tried and why
it did or didn't work. Updated after every run. Newest evidence first; numbers are look-ahead, loose profile.

## Current state (2026-10-16)
List: the 10/16 build (Gilgamesh, Diabolic Tutor, eleven Equipment including Shield and Helm of Kaldra); every card
modeled. Matched tiers not set yet (provisional t1, t2 in `data/my_decks.toml`).
Win rate, 1,000 games per tier, **all five tiers (run with `--any-tier`, JD's OK)**: **T1 12.7%, T2 12.3%, T3 12.6%,
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
