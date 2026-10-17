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
| Change | Result | Why |
|---|---|---|
| AI: look-ahead ramp bonus (`ramp_w` 1; audit/ramp/2026-10-15) | T4 -1.9 (noise +-2) | Rocks come down first, and the late payoffs are cast less (Howling Mine 50% -> 43% of games drawn, Peer into the Abyss 31% -> 26%); Sheoldred's timing is unchanged (6.0). Merged off. |

**2026-10-17 land audit** (branch `land-audit2`, `rust/agents/report-land-audit2.md`): 12 lands across the decks now make only the mana their Oracle text allows (filter lands, Rugged Prairie, Mystic Gate, Spire of Industry, Exotic Orchard, Cavern of Souls, Spectator Seating, Hidden Grotto and others; Sulfur Falls and Fabled Passage were too strict). Same-seed check (adaptive AI, 300 games per tier, matched tiers): 40.1 → 39.2, inside the noise (about ±5). No measurable change to this deck.
