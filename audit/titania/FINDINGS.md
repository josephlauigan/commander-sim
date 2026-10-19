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

**2026-10-17 land audit** (branch `land-audit2`, `rust/agents/report-land-audit2.md`): 12 lands across the decks now make only the mana their Oracle text allows (filter lands, Rugged Prairie, Mystic Gate, Spire of Industry, Exotic Orchard, Cavern of Souls, Spectator Seating, Hidden Grotto and others; Sulfur Falls and Fabled Passage were too strict). Same-seed check (adaptive AI, 300 games per tier, matched tiers): 32.2 → 33.8, inside the noise (about ±5). No measurable change to this deck.

## 2026-10-18: band calibration (proposed swaps, NOT applied; see audit/bands/REPORT.md)
Band 4 (T1-T3). Proposed: Hornet Queen, Harmonize, Tireless Tracker (to Seph/Atraxa), Environmental Scientist out; Cactarantula, Penumbra Spider, Paradise Druid (spares), Worldly Tutor (from Seph/Atraxa; 1 Game Changer) in. Final: **19.1% vs T1, 27.6% vs T2, 25.5% vs T3** (avg 24.1%, 2,000 per profile; current 20.0 / 32.2 / 28.3 at 1,000). Why: Hornet Queen and Harmonize were among her best cards; losing them takes ~3-5 points at T2-T3. Weakness that stays: Tier 1's fast aggro (19% vs T1, below the fair line, as before the swaps).

## 2026-10-18: AI audit re-measure (branch ai-audit; 2,000 games per profile, look-ahead)
**17.5 / 25.9 / 25.5% vs T1 / T2 / T3** (before 19.1 / 27.6 / 25.5, with Penumbra Spider): within noise, stays band 4. AI fixes: Fblthp X cost, Silhana coloured mana, Worldly Tutor pick, Arachnogenesis timing, protection held.

## 2026-10-18 evening: JD's new cards, coarse check (loose only, 500 games per list per tier, look-ahead; only big downgrades show, noise ±4-5 per tier)
**Ghalta the Unstoppable => Primeval Titan: T1 +1.0, T2 +1.6, T3 +3.6 (about +2.1 over t1-t3; noise, no downgrade).**
Why: the Titan is cast in 85-90% of the games it's drawn, about a round earlier than Ghalta (8-9 vs 9-10), and wins
36-48% when cast vs Ghalta's 33-35%. It fetches fetch lands first (a landfall now, a 5/3 Elemental under Titania when
cracked), so it is ramp and threat in one where Ghalta needs a big board first. Raw: `2026-10-18-new-cards/`. JD plans to add the cards unless one is a big downgrade.
**Correction (JD, 10/18): Primeval Titan is BANNED in Commander.** The test above should never have been proposed; the
swap is not made. See CLAUDE.md "Check every swap before proposing it".
