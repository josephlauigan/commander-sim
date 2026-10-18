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

### Ad Nauseam (2026-10-17 night; branch `adnaus`; JD: keep Rhystic Study)
Ad Nauseam was already modeled; the tests exposed AI misplays, fixed in shared code before the final numbers:
- **Old Ad Nauseam AI** (traces): cast in its own main phase, tapped out, flipped past a full hand (the cleanup
  discarded half) down to ~15–22 life, and died within two rounds in 4 of 8 traced casts.
- **New shared AI** (`8208586`): cast at the end of the turn before yours, mana held for it, stops at a usable hand
  (7 + next turn's mana / 3), life per card from the library's average mana value, a floor that counts every board.
- **Look-ahead hold credit** (`d973bea`, `search::hold_bonus`): the search overrode "hold mana" to tap out.
| Swap (Ad Nauseam in for) | Result over t3–t5 | Why |
|---|---|---|
| Feed the Swarm (adaptive, old AI) | -3.1 (-2.5, -3.2, -3.7) | The misplays above. |
| Hallway Heckler (adaptive, old AI) | -2.8 (-3.6, -2.0, -2.8) | |
| Hallway Heckler (look-ahead, old AI) | +0.4 (-0.7, -0.6, +2.4) | The search avoided some misplays. |
| Feed the Swarm / Hallway Heckler (adaptive, new AI) | -1.8 / -1.1 | Better, still slightly negative. |
| **Hallway Heckler (look-ahead, new AI + hold credit, 1,000 paired per tier)** | **-0.3 (t3 -1.8, t4 -1.4, t5 +2.4; ±3.1–3.5)** | Noise. Cast in only 15% of the games it's drawn (stuck 85%): Kefka keeps big hands, so there's rarely room for four cards, and `kefka::discard_choice` still discards it to the wheel (5 of 40 traced games). Wins 42% when cast. Baseline unchanged by the AI fixes (26.4% vs 26.8% over t3–t5). |
**Reading it:** Ad Nauseam is the weakest of its three homes here. Kefka's discard plan (hand-coded) treats it as fodder.

## 2026-10-18: band calibration (proposed swaps, NOT applied; see audit/bands/REPORT.md)
Band 3 (T3). Proposed: Demonic Tutor, Vampiric Tutor (to Seph FS), Mystical Tutor (to Veyran) out; Mana Leak, Expressive Iteration, Sleight of Hand (spares) in. 6 -> 3 Game Changers (Underworld Breach, Jeska's Will, Rhystic Study). Final: **26.7% vs T3** (current 27.4% at 1,000). Why: the combo is intact but found less often. Filler choice made no difference (Star Compass / Tome / Flourish gave 25.6%), nor did keeping Vampiric instead of Rhystic (26.1%).
