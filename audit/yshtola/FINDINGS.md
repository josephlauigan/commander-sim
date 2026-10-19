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
### 2026-10-19: Orcish Bowmasters for three cards (quick read: T5, loose only, 700 games per list, look-ahead)
| Cut (=> Orcish Bowmasters) | T5 (noise ±4.0-4.1) |
|---|---|
| Curiosity | +3.1 |
| Mystic Remora | +2.0 |
| Lightning Greaves | +3.3 |
**Leans good: about +2.8, every cut positive, none beyond noise alone.** The three share one baseline (30.9%), so they
are not independent; a full run of one cut is needed to confirm. Raw: `2026-10-19-bowmasters/<cut>-t5.txt`.
Why: Bowmasters is cast in 79-86% of the games it is drawn, early (round 5.4-5.8), and wins 45-56% of the games it
is cast in (the deck averages about 31%). Damage to opponents rises 1.8-3.6 a game in all three, and the plan comes
online slightly more often (+0.9 to +3.3). The ping kills T5's mana creatures and 1-toughness pieces, and the fast
tables' own extra draws feed it. JD would buy a second copy (the Sauron copy stays).

### 2026-10-19: Talisman of Progress => The Mind Stone (T5, 2,000 games per list per profile, look-ahead)
Result: **+2.5 conservative (±2.0) / -1.0 loose (±1.9): about +0.75 averaged, no clear effect** (the tool's
"leans better" rests on one profile, and the other goes the other way). Raw: `2026-10-19-mind-stone/t5.txt`.
Why: a rock for a rock. Mana on turns 4 and 6 and the plan (online in about 61% of games, turn 6.9) are unchanged.
The Mind Stone is cast in 67-73% of the games it is drawn (Talisman 67-68%), around round 5.2-5.5, and stuck in hand
in 27-33%, as much as the Talisman: her rocks are often skipped for better plays, so it is not a Mind Stone priority
problem. Her losses are the same fast combo either way (Urza 15%, Zur 13%, Winota 11-20%), which ends T5 games at
about 9.4 rounds, before the {5}{W} harness matters. The 10/18 quick test (Chromatic Lantern out, loose only) was
also noise (+0.4). Verdict: not an upgrade for her.

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

## 2026-10-18: band calibration (proposed swaps, NOT applied; see audit/bands/REPORT.md)
Band 1 (T5). Proposed: +Force of Will, +Farewell, +Teferi's Protection (all from Seph/Atraxa) for Dakkon, Thought Vessel, Bolas's Citadel (to Seph FS). 9 Game Changers. Final: **24.3% vs T5** (current list 23.2%). Why: Force of Will is cast 76% of the games drawn and answers T5 combo; Farewell and Teferi's Protection are mostly held (stuck 68-75%) but win 45-60% when cast. Citadel was cast in only 25% of the games Y'shtola drew it, so it was the cheapest card to give up.

## 2026-10-18: AI audit phase 2 (branch audit-ysh; problems 3-6 of audit/ai-audit/yshtola.md)
Measured at T5 (her tier), adaptive AI, 4,000 games per list, conservative/loose win rate: before 32.3 / 31.6; after
31.7-32.0 / 31.9-32.0. No change beyond noise (one run's SE is 0.7); the fixes make the plays right, they do not move the
win rate. Weakness this leaves: she still loses 35% of games to Kinnan/Urza combos, which spot removal cannot reach in time.
- **Removal and her trigger.** `removal_adjust` (impls/yshtola.rs): a noncreature spell of mana value 3+ with her out
  (Anguished Unmaking, Generous Gift, Your Fate Ends Here) earns `trigger_bonus / 4` and needs a target worth 1; every
  other removal needs a target worth 3 (before: Path on Ornithopter, Push on Esper Sentinel). `trigger_bonus` adds 3 when
  an opponent already lost 2-3 life this turn (one more trigger makes four and she draws). The same check applies to the
  attack-response removal. The unplayed 4-player test: choosing the single best target instead of sampling cost -1.3
  points (31.0 vs 32.3), so the sample stays, restricted to targets at least half as valuable as the best.
- **Farewell / Austere Command.** The wipe also earns the trigger bonus. With Teferi's Protection in hand and its
  {2}{W} left over, the wipe is valued without her own losses and cast with Teferi's in response (her permanents phase
  out, the opponents' are exiled): `teferis_first`, `cast_wipe`. The opponent-end-step version of the second trigger
  is not possible: the AI's end-of-turn window runs after the end-step triggers have been checked.
- **Mystic Remora 66 -> 40, Esper Sentinel 62 -> 46** (early casts; the two were in the bottom group by win rate when
  cast). Neutral on the win rate (31.6 / 32.0 with both); kept because Remora was being cast and sacrificed unpaid.

## 2026-10-18: AI audit re-measure (branch ai-audit; 2,000 games per profile, look-ahead)
**24.5% vs T5** (before 24.3): no change. Fixes changed play (trigger timing, removal targets, Teferi's Protection as the full card, Mystical Tutor finishers), not the win rate. Details: audit/ai-audit/yshtola.md, REPORT.md.

## 2026-10-18 evening: Chromatic Lantern => The Mind Stone (quick pass: loose only, 700 games per list, T5, look-ahead)
Result: **24.4% -> 24.9% (+0.4, ±3.3): noise.** Raw: `2026-10-18-mind-stone/mind-stone-quick.txt`.
Why: a better rock than the Lantern (cast 70% of the games drawn vs 51%; mana on turn 8 +0.13), but her losses are
fast combo (Kinnan 20%, Urza 16%, Zur 11%, Winota 11%) and the 6-mana harness (blink Hostage Taker, Massacre Wurm,
Gonti) comes too late. Open AI question: The Mind Stone sits in hand in 30% of the games it's drawn; her rules
don't name it (generic priority), so a re-test should check that first. JD decided to wait on buying it.

## 2026-10-18 evening: JD's new cards, coarse check (loose only, 500 games per list per tier, look-ahead; only big downgrades show, noise ±4-5 per tier)
**Prison Sentence => Stroke of Midnight: T5 -2.6 (noise ±4.3, no big downgrade).** Cast in 80% of draws (Prison
Sentence 67%), but it doesn't change her losses (fast combo: Kinnan, Urza, Zur, Winota). Cloud is the other home
(-1.4 there); JD has one copy. Raw: `2026-10-18-new-cards/`. JD plans to add the cards unless one is a big downgrade.
