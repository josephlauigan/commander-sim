# Veyran (key veyran): findings

The deck's living list of strengths, weaknesses and what was tested.

## 2026-10-18: band calibration (proposed swaps, NOT applied; see audit/bands/REPORT.md)
Band 4 (T1-T3). Proposed: +Consecrated Sphinx (from Seph/Atraxa), +Mystical Tutor (from Kefka) for Evacuation, Venser, Shaper Savant (its two lowest win rates when cast). 2 Game Changers. Final: **25.8% vs T1, 28.6% vs T2, 24.5% vs T3** (avg 26.2%; current 23.4 / 26.0 / 23.5 at 1,000). Why: Mystical finds Thousand-Year Storm / Aetherflux, cast 83-87%; the Sphinx draws when cast. Model gap: Triple Triad is never cast (untagged).

## 2026-10-18: AI audit re-measure (branch ai-audit; 2,000 games per profile, look-ahead)
**29.5 / 32.6 / 27.7% vs T1 / T2 / T3** (before 25.8 / 28.6 / 24.5): **+3 to +4 at every tier, beyond noise.** Triple Triad cast (was never), Greaves before Veyran, Recruiter/Mystical/Solve pick by state, engine creatures valued. Veyran now matches Kefka (band 3) at T3: band 3/4 gap gone, JD to decide.

## 2026-10-19: Underworld Breach back from Tymna/Kraum's spares (paired, loose, look-ahead, 1,000 games per tier)
Raw: `2026-10-19-breach/` (card reports `analyze-adaptive-<tier>.txt`, swaps `breach-for-greaves-<tier>.txt`). The cut,
Lightning Greaves, is among the three lowest win rates when cast in all three tiers (28-33%, adaptive card report).
The first try was stopped by the preflight: Veyran's priority had no Breach rule (0, never cast); fixed in
`ai/veyran.rs` (cast with two spells worth escaping and 5 mana, like Kefka), with a test.
| Tier | Current | Breach for Greaves | Change |
|---|---|---|---|
| T1 | 29.2% | 31.1% | +1.9 (±3.6) |
| T2 | 34.0% | 35.9% | +1.9 (±3.8) |
| T3 | 26.1% | 26.7% | +0.6 (±3.7) |
| **Average over t1-t3** | **29.8%** | **31.2%** | **+1.5** |

Why: Breach is cast in 41-53% of the games it's drawn (first cast round 10.6-11.6: it waits for a stocked graveyard)
and wins 45-46% when cast, against Greaves' 31-38%. The cost is protection: Veyran is removed 0.09-0.13 more times per
game without Greaves. Positive in every tier but inside the noise. Breach is a Game Changer: Veyran goes to 3 (Bracket 3).
