# Sauron, the Dark Lord — findings (living file)

What the simulator has taught us about the Sauron Grixis amass deck: strengths, weaknesses, and what was tried and why
it did or didn't work. Updated after every run. Newest evidence first. Matched tiers: t3–t5 (`data/my_decks.toml`).
Older tests: the dated folders here.

## Current state (2026-10-17, night)
Look-ahead, loose, current list (baselines of the Urza run, with the hold-mana search credit): **T3 28.2%, T4 31.6%,
T5 26.4% (28.7% over t3–t5).**

## Weaknesses (by evidence)
- **Combo tables at T4–T5:** the killers there are Heliod (combo 16% at T4), Kinnan (20% at T5), the Urza deck (16%)
  and Zur. At T3 it's creature decks in combat (Aurelia 17%, Marwyn 13%, Tergrid 12%).
- **Lowest win rate when cast** (adaptive card report, `audit/sauron/2026-10-17-adnaus/K0-analyze-adaptive.txt`):
  Disdainful Stroke, Arcane Denial, Spell Pierce (t3, t4), Inexorable Tide, Relic of Legends.

## What was tried, and why it did or didn't work
### Urza, Lord High Artificer (2026-10-17 night; branch `adnaus`; Full, from the pool's Urza deck)
| Swap (Urza in for) | Result over t3–t5 | Why |
|---|---|---|
| Spell Pierce / Disdainful Stroke (adaptive) | -0.3 / -0.3 | No effect. |
| Spell Pierce (look-ahead, before the hold credit) | -1.1 (t3 -0.6, t4 -0.3, t5 -2.5) | Noise. |
| **Spell Pierce (look-ahead, with the hold credit, 1,000 paired per tier)** | **+0.5 (t3 +2.1, t4 0.0, t5 -0.6; ±3.3–3.7)** | Noise. Urza cast in 65% of the games he's drawn (52% at T5), wins 36–55% when cast. T3 eliminations -2.6 (an extra body and the Construct against creature decks); at T4–T5 losing a cheap counter to combo decks costs a little (T5 damage -2.0). Mana on turn 8 unchanged: Sauron's artifacts are mostly rocks that already make mana, so Urza's {U} ability adds almost nothing. |
**Reading it:** a fair swap, not an upgrade. Urza wants cheap non-mana artifacts and artifact tokens; Sauron has few.

## 2026-10-18: band calibration (proposed swaps, NOT applied; see audit/bands/REPORT.md)
Band 2 (T3-T4). Proposed: +Force of Will (spare) for Slaughter Pact; 3 Game Changers. Final: **30.2% vs T3, 31.6% vs T4** (avg 30.9%; current 28.9 / 31.6 at 1,000). Why: a free counter protects Sauron and stops T3-T4 combo turns; Slaughter Pact had the lowest win rate when cast.
