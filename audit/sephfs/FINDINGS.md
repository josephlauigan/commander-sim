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

## 2026-10-18 night: Zombify => Dread Return (coarse: loose only, 500 games per list, T5, look-ahead)
Modeled for Seph FS first (JD asked): cast from hand by the Zombify rule (a creature worth 3+ in the yard), own
graveyard only (`rean_targets("dread")`; Seph FS's rule had taken from any graveyard), and the flashback as a deck
option (sacrifice three: tokens and Reassembling Skeleton first, never Sephiroth or a body worth more than the target;
the deaths' drains count in its value), exiled after. Brain fix: Seph FS may cast hand-tagged reanimation spells.
Result: **22.6% -> 27.4% (+4.8, noise ±4.1): just past noise, the best coarse result of the day.** Raw:
`2026-10-18-new-cards/dread-return.txt`.
Why: cast in 48% of the games drawn (Zombify 64%: it waits for a target), wins 69% when cast vs Zombify's 45%. The
plan came online more often (26.0 -> 29.4%), games ran a little longer (8.6 -> 8.8 rounds), and fewer losses to
Kinnan/Urza/Zur/Winota combo. The flashback makes it a second reanimation from the graveyard, and its three deaths
are drains with Sephiroth or a payoff out. Coarse sample: confirm with a bigger run before trusting the size.

## 2026-10-19: Fellwar Stone => The Cruelty of Gix, full run (T5, loose, 2,000 games per list, look-ahead)
Result: **32.8% -> 31.9%, -0.9 (noise ±2.5): neutral.** Raw: `2026-10-19-cruelty-of-gix/t5.txt`. Fellwar Stone was
the cut (lowest-win-rate rock in the 10/19 card report; in mono-black it only makes the opponents' colours).
Why: Cruelty is cast in only 52% of the games it is drawn (stuck 48%), around round 8.3, against games that last 9.8
rounds; Fellwar Stone was cast in 78% around round 5.3, and the plan comes online a little less often without it
(34.2% -> 32.9%). The deck still dies to the same fast combo (Urza 17-18%, Zur 14%, Winota). When Cruelty is cast the
deck wins 51% (deck average 32%): strong when it lands, too slow for T5. Not worth the slot.

## 2026-10-19: The Cruelty of Gix modeled (smoke run only: Mind Stone => Cruelty, T5, loose, 50 games, adaptive)
Modeled (impls/sephfs_cards.rs): read ahead picked by the AI (III when any graveyard holds a creature worth 3+, II when
the deck has a wanted tutor target and 14+ life, else I); I takes the best creature or planeswalker card from an
opponent's hand, II is the shared "any" tutor and 3 life, III takes the best creature from any graveyard. Cast priority 56.
Smoke result: 36.0% -> 42.0% (+6.0, noise ±6.8): no clear effect at 50 games, this was a functionality check, not a
measurement. Why: drawn in 16% of games, cast in only 37.5% of those (stuck 62.5%: five mana, and it ranks below
the wipes and the commander); Mind Stone was cast in 90%. Wins when cast 66.7% (3 games: no signal). The real
test is a full-size run; the likely strength is chapter III after one of the deck's own wipes (an opponent's yard too).

**Retired 2026-10-27 (JD).** Its cards went to other decks, none of it simulated: Sephiroth, Fabled SOLDIER, Vampiric Tutor, Grave Researcher // Reanimate, Living Death, Rune-Scarred Demon, Dark Ritual, Phyrexian Altar, Dread Return and Phyrexian Tower to Seph/Atraxa; The One Ring, Demonic Tutor, Gray Merchant of Asphodel, Call of the Ring and Bolas's Citadel to Sheoldred; the rest to bulk. The list is kept in `decklists/pool/retired/sephiroth-fs-wipe-steal.md` so the Rust tests can seat it. This file is kept as history.
