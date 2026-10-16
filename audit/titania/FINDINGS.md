# Titania, Protector of Argoth — findings (living file)

What the simulator has taught us about this deck: strengths, weaknesses, and what was tried and why it did or didn't
work. Updated after every run. Newest evidence first; numbers are look-ahead, loose profile, vs Titania's matched
tiers (T2-T4) unless noted.

## Current state (2026-10-15)
Win rate vs T3: **29.5%** (1,000 games, seed 1015, main 0f49b7d).

## Strengths
- **Big creatures win the game:** 92% of wins by damage, almost all of it combat (38 damage a game). The highest win
  rates when cast are the big finishers: Overwhelming Stampede 63%, End-Raze Forerunners 51%, Paladin of Predation
  50%, Hornet Queen 49%.

## Weaknesses (by evidence)
1. **The plan comes late or not at all:** Titania resolves on turn 6.6 on average (by turn 6 in 49% of games), and
   the first 5/3 Elemental comes by turn 6 in only 23% of games, never in 41%.
2. **Titania is the removal target:** removed 0.71 times a game; nothing else is removed more than 0.05.
3. **Combat decks kill it at T3:** Aurelia (17.5% of games), Marwyn (14.5%), Tergrid (13.2%), all by combat.

## What was tried, and why it did or didn't work
| Change | Result | Why |
|---|---|---|
| AI: look-ahead ramp bonus (`ramp_w` 1; audit/ramp/2026-10-15) | T3 -1.3 (noise +-2) | Ramp cast sooner doesn't bring Titania or the Elementals sooner (resolve turn 6.6 -> 6.7, plan 58.7% -> 56.4%). Merged off. |
