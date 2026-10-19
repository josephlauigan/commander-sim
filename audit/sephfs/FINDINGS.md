# Sephiroth FS (key sephfs): findings

The deck's living list of strengths, weaknesses and what was tested.

## 2026-10-18: band calibration (proposed swaps, NOT applied; see audit/bands/REPORT.md)
Band 1 (T5). Proposed: +Demonic Tutor, +Vampiric Tutor (Kefka), +Rune-Scarred Demon (Tymna/Kraum), +Bolas's Citadel (Y'shtola), +Gray Merchant of Asphodel (Sheoldred) for Star Compass, Tome of Legends, Masterful Flourish, Bitter Triumph, Rabid Attack. Final: **23.7% vs T5** (4,000 per profile; current list 21.5% at 1,000). Why: the tutors (cast ~75-80% of games drawn) find the wipes and Living Death sooner; Citadel (62% win when cast) and Gray Merchant (46%) turn the drain life into cards and reach. Rune-Scarred Demon is stuck in hand 61% (six mana). Weakness that stays: fast T5 combo; card quality alone moves it only ~2 points.

## 2026-10-18: AI audit re-measure (branch ai-audit; 2,000 games per profile, look-ahead)
**23.6% vs T5** (before 23.7): no change. Board-state tutor pick, Gray Merchant devotion, Altar+Skeleton+Pawn loop, Citadel gate: play fixes, win rate flat. Fixed wish lists measured 0.8-1.7 points worse than the generic pick (60,000 games each).

## 2026-10-18 evening: Mind Stone => The Soul Stone (quick pass: loose only, 700 games per list, T5, look-ahead)
Result: **24.7% -> 24.9% (+0.1, ±2.5): no effect.** Raw: `2026-10-18-soul-stone/soul-stone-quick.txt`.
Why: as a rock it matches Mind Stone (mana on turns 4 and 8 unchanged); it is cast in 83% of the games it's drawn,
around round 5, but the {6}{B} + exile-a-creature harness comes too late: the deck's plan is online in only ~27% of
games and it dies to fast combo (Kinnan 23%, Urza 18%, Zur 10%, Winota 9%). A slow reanimation engine can't answer
that. JD decided to wait on buying it.
