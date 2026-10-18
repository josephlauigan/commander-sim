# Kefka, Court Mage — findings (living file)

What the simulator has taught us about the Kefka Grixis discard deck: strengths, weaknesses, and what was tried and why
it did or didn't work. Updated after every run. Newest evidence first. Matched tiers: t3–t5 (`data/my_decks.toml`).

## Current state (2026-10-17, evening)
Card report (adaptive AI, loose, 1,000 games per tier; `audit/kefka/2026-10-17-smaug/K0-analyze-adaptive.txt`):
T3 24.3%, T4 32.5%, T5 26.4% on the quick AI. Look-ahead baselines from the swap run below: **T3 26.9%, T4 30.5%,
T5 23.1% (26.8% over t3–t5).**

## Strengths
- **The combo finishers win when they land** (win rate when cast, every tier): Underworld Breach, Brain Freeze,
  Psychosis Crawler, Niv-Mizzet, the Firemind (38–68%).

## Weaknesses (by evidence)
1. **The combo pieces sit in hand:** Grapeshot, Jin-Gitaxias, Yawgmoth's Will, Brain Freeze, Underworld Breach and
   Massacre Wurm are cast in only 9–25% of the games they're drawn; Deflecting Swat and Slip Out the Back almost never.
2. **Kefka flips late:** online in about 34% of games, 6–7% by turn 8. Kefka resolves by turn 6 in 58% of games.
3. **Deaths are creature decks in combat** (T3: Aurelia 20%, Tergrid 15%, Marwyn 10%).
4. **Lowest win rate when cast (quick AI):** Bitter Triumph (bottom ten in all three tiers), Feed the Swarm (t3, t4),
   Rank Rat, Hallway Heckler, Grim Servant (t4).

## What was tried, and why it did or didn't work
### Smaug the Magnificent (2026-10-17 evening; modeled Full in `impls/smaug.rs`, branch `smaug`)
| Swap (Smaug in for) | Result over t3–t5 | Why |
|---|---|---|
| Rank Rat (adaptive, 1,000 paired per tier) | +1.0 (t3 +0.4, t4 +1.7, t5 +1.0; ±1.5–1.9 each) | Best of four on the quick AI. |
| Hallway Heckler (adaptive) | +0.9 (+0.6, +2.1, +0.1) | |
| Feed the Swarm (adaptive) | +0.5 (+0.8, +0.9, -0.1) | |
| Grim Servant (adaptive) | +0.4 (-1.6, +2.1, +0.6) | |
| **Rank Rat (look-ahead, loose, 1,000 paired per tier)** | **-0.1 (t3 +0.2, t4 -1.0, t5 +0.4; ±3.1–3.4 each)** | No effect. Smaug is stuck in hand 41% of the games he's drawn ({R}{R} with 5 Mountains in a mostly black deck) and is first cast in round 8.0 vs Rank Rat's 6.1. Wins 42% when cast vs Rank Rat's 32% (correlation). Eliminations -1.1 pts, flip +1.0 pt, both noise. |
**Reading it:** Smaug doesn't move Kefka. The red mana is the problem, and the deck's Treasure makers (Big Score,
Seize the Spoils, Contract Killing, Skullport Merchant) aren't enough to make his attack trigger matter.
