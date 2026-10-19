# Sheoldred, the Apocalypse — findings (living file)

What the simulator has taught us about this deck: strengths, weaknesses, and what was tried and why it did or didn't
work. Updated after every run. Newest evidence first; numbers are look-ahead, loose profile, vs Sheoldred's matched
tiers (T3-T5) unless noted. Older tests: README.md in this folder.

## Current state (2026-10-15)
Win rate vs T4: **30.1%** (1,000 games, seed 1015, main 0f49b7d).

## Strengths
- **Drain wins:** 76% of the opponents it eliminates go by drain (30 of its 46 damage a game). Card-draw punishers
  and draw engines have the highest win rates when cast: Memory Jar 53%, Painful Lesson 50%, Read the Bones 49%,
  Howling Mine 49%, Underworld Dreams 48%.

## Weaknesses (by evidence)
1. **Combo decks at T4:** Heliod's combo kills it in 19% of games, Prosper's in 9%; Sheoldred has few ways to stop a
   combo.
2. **Sheoldred comes late:** resolves on turn 6.0 on average (by turn 6 in 53% of games, never in 19%); the first
   drain comes by turn 6 in 52% of games.
3. **Combat decks:** Krenko (12%) and Yuriko (7%) by combat.

## What was tried, and why it did or didn't work
### 2026-10-19: Stir Up Trouble => Orcish Bowmasters, full run (T3-T4, loose, 2,000 games per list, look-ahead)
Result: **T3 +2.0 (±2.6), T4 +0.5 (±2.4): about +1.2 over t3-t4, positive but within noise.** Raw:
`2026-10-19-bowmasters/full-stir-up-trouble-t3.txt`, `t4.txt`.
Why: Bowmasters is cast in 88-91% of the games it is drawn, around round 5.4-6.1, against Stir Up Trouble's 79-86%
(stuck 14-21%). Damage +0.9 to +1.7 a game; plan timing and game length unchanged. At T3 the Atraxa losses drop
(8.2 -> 6.4%); at T4 the deck still dies to Heliod combo (17%) and Krenko (12%) as before, which a 1/1 pinger doesn't
stop. A small, real-looking edge over a weak card, smaller than the quick read suggested.

### 2026-10-19: Orcish Bowmasters for three cards (quick read: T3-T4, loose only, 700 games per list, look-ahead)
| Cut (=> Orcish Bowmasters) | T3 | T4 | Average |
|---|---|---|---|
| Stir Up Trouble | +3.3 | +1.9 | +2.6 |
| Fetid Imp | +3.3 | +1.3 | +2.3 |
| Soul Enervation | +1.4 | +2.1 | +1.8 |
Noise ±3.9-4.3 per cell. **Leans good: about +2.2 over t3-t4, all six cells positive, none beyond noise alone.** The
cuts share one baseline per tier (30.9% T3, 29.4% T4), so they are not independent; a full run of one cut is needed
to confirm. Raw: `2026-10-19-bowmasters/<cut>-<tier>.txt`.
Why: Bowmasters is cast in 84-90% of the games it is drawn, early (round 5.5-6.5), and wins 38-48% of the games it is
cast in. Damage to opponents rises in every cell (+0.7 to +3.7 a game): Howling Mine and Seizan hand opponents
extra draws, each a ping and a bigger Orc Army. The plan (Sheoldred out) comes no sooner, so the gain is extra
pressure, not speed. JD would buy a second copy (the Sauron copy stays).

| Change | Result | Why |
|---|---|---|
| AI: look-ahead ramp bonus (`ramp_w` 1; audit/ramp/2026-10-15) | T4 -1.9 (noise +-2) | Rocks come down first, and the late payoffs are cast less (Howling Mine 50% -> 43% of games drawn, Peer into the Abyss 31% -> 26%); Sheoldred's timing is unchanged (6.0). Merged off. |

**2026-10-17 land audit** (branch `land-audit2`, `rust/agents/report-land-audit2.md`): 12 lands across the decks now make only the mana their Oracle text allows (filter lands, Rugged Prairie, Mystic Gate, Spire of Industry, Exotic Orchard, Cavern of Souls, Spectator Seating, Hidden Grotto and others; Sulfur Falls and Fabled Passage were too strict). Same-seed check (adaptive AI, 300 games per tier, matched tiers): 40.1 → 39.2, inside the noise (about ±5). No measurable change to this deck.

### Ad Nauseam (2026-10-17 night; branch `adnaus`; would be the deck's first Game Changer, bracket 3 allows three)
Card report (adaptive, 1,000 games per tier, `audit/sheoldred/2026-10-17-adnaus/K0-analyze-adaptive.txt`): lowest win
rate when cast: Stir Up Trouble (bottom two at T3 and T4), Fetid Imp, Soul Enervation, Restless Bloodseeker (T3, T5).
The shared Ad Nauseam AI was fixed during these tests (see audit/kefka/FINDINGS.md: own-turn casts, flipping past a
full hand, a one-board floor; and the look-ahead overriding "hold mana").
| Swap (Ad Nauseam in for) | Result over t3–t5 | Why |
|---|---|---|
| Stir Up Trouble / Restless Bloodseeker (adaptive, old AI) | -1.5 / -2.0 | Cast on its own turn, down to 16–18 life, discarded back to seven. |
| Stir Up Trouble (look-ahead, old AI) | +0.7 (+2.7, +0.9, -1.4) | |
| Stir Up Trouble / Restless Bloodseeker (adaptive, new AI) | -0.9 / -0.5 | |
| **Restless Bloodseeker (look-ahead, new AI + hold credit, 1,000 paired per tier)** | **+1.6 (t3 +2.0, t4 +0.7, t5 +2.2; ±3.1–3.7)** | The best Ad Nauseam result, positive in every tier but still noise. Cast in 30% of the games it's drawn (twice Kefka's and Y'shtola's), wins 47% when cast vs 28–36% for Restless Bloodseeker; eliminations -1.8 (T3), -0.6 (T4). The lowest curve of the three decks (average mana value 2.98) and no hand-size problem. Baseline unchanged by the AI fixes (29.0% vs 29.5%). |
**Reading it:** the best home for Ad Nauseam among JD's decks. A 2,000-paired run would settle it. **Applied (JD, 2026-10-17): Ad Nauseam in, Restless Bloodseeker out (to bulk).**

## 2026-10-18: band calibration (proposed swaps, NOT applied; see audit/bands/REPORT.md)
Band 2 (T3-T4). Proposed: Gray Merchant of Asphodel to Seph FS, Necrosquito (spare) in. Final: **30.6% vs T3, 30.8% vs T4** (avg 30.7%; current 32.5 / 33.1 at 1,000). Why: Sheoldred was at the top of band 2; Gray Merchant is a better fit in Seph FS (32 Swamps), and losing it costs Sheoldred ~2 points, which tightens band 2.

## 2026-10-18: AI audit re-measure (branch ai-audit; 2,000 games per profile, look-ahead)
**31.9% vs T3, 31.6% vs T4** (before 30.6 / 30.8): +1, within noise. Protection held for Sheoldred; Ad Nauseam in the second main; Memory Jar waits for a punisher; Seizan, Necrosquito valued; Rooftop Assassin condition.
