# Y'shtola, Night's Blessed — findings (living file)

What the simulator has taught us about the Y'shtola Esper drain deck: strengths, weaknesses, and what was tried and
why it did or didn't work. Updated after every run. Newest evidence first. Matched tier: T5 (`data/my_decks.toml`).

## Current state (2026-10-17, night)
Look-ahead, loose, current list: **T5 23.3–23.6%** (the baselines of the swap runs below). Adaptive: 30.2–31.6%.

## Weaknesses (by evidence)
- **Lowest win rate when cast** (adaptive card report, `audit/yshtola/2026-10-17-adnaus/K0-analyze-adaptive.txt`;
  nearly every card is above the deck average): Mystic Remora (39%), Lightning Greaves, Luminous Bonds, Prison
  Sentence, Esper Sentinel, Curiosity, Chromatic Lantern, Thought Vessel (43–44%).

## What was tried, and why it did or didn't work
### AI fix: cards her rules don't name (2026-10-17, `9b7e19a`)
`yshtola_prio` returned 0 (never cast by priority) for any card it didn't name: a swapped-in Ad Nauseam was cast in
0.5% of the games it was drawn, and **Dakkon, Shadow Slayer in her list was never cast**. Unnamed cards now get the
pool AI's generic priority (`yshtola_named_prio` holds her rules). T5 baseline 31.1 → 31.6% (adaptive, noise).

### Ad Nauseam (2026-10-17 night; branch `adnaus`; JD: keep Esper Sentinel)
The shared Ad Nauseam AI and the look-ahead's hold-mana credit were fixed during these tests (audit/kefka/FINDINGS.md).
| Swap (Ad Nauseam in for), T5 | Result | Why |
|---|---|---|
| Mystic Remora / Curiosity (adaptive, new AI) | -0.9 / +1.0 | |
| Curiosity (look-ahead, before the hold credit) | -2.0 (±3.2) | The search tapped out: Ad Nauseam cast in 5.7% of the games it was drawn. |
| **Curiosity (look-ahead, hold credit)** | **-0.2 (±3.2)** | Cast 15%, wins 56% when cast vs Curiosity's 39% (cast 80%). |
| **Thought Vessel (look-ahead, hold credit)** | **-1.4 (±3.2)** | Cast 17%, wins 58% when cast; Thought Vessel cast 70% (no maximum hand size, useful with Necropotence and Citadel). |
| **Mystic Remora (look-ahead, hold credit)** | **-1.9 (±3.1)** | Cast 14%, wins 54% when cast; Mystic Remora cast 75%. |
**Reading it:** Ad Nauseam wins when it's cast but is rarely cast: T5 games are short (about 8.5 rounds) and the deck
pays life already (Necropotence, Bolas's Citadel), so the safe window seldom opens. Not a fit; Sheoldred is its home.
