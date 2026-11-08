# C4 / C5: the mulligan oracle, first measurement (2026-11-07)

`commander-sim --mulligan-oracle KEYS --hands 1000 --playouts 40 --seed 500000` (adaptive AI), on main 7d4d5b35 plus the
oracle branch. The Thrasios list is the one before swap S2 (Savannah and Breeding Pool, adopted later the same day).
Raw output: the `.txt` files here. The command queue: `queue.sh`.

## What the oracle is
It deals the opening seven (and the second seven when the keep rule mulligans the first) exactly as a real game does.
For each hand it plays 40 goldfish games from KEEP and 40 from MULLIGAN. A goldfish game has three inert seats at 40
life that never act. The two arms use the same play seeds. In the KEEP arm the hand is fixed and the rest of the library
is reshuffled; the MULLIGAN arm follows the normal rule from the next draw. The score is the kill turn, with no kill by
own turn 10 counted as 11. The oracle says "keep" or "mulligan" only when the paired difference is more than 2 standard
errors; otherwise "unclear". It is trace-only and changes no game.

**How it differs from the original C4 spec.** The spec played both arms against the deck's own tiers and compared win
rates. This version is goldfish and compares kill turns, which is faster but has two limits:
- **It ignores interaction.** Counters, removal and stax are worth nothing against inert seats, so hands built on them
  look worse than they are. Most of Dack Fayden's "should mulligan" verdicts are hands like this.
- **It can't judge the fair decks.** Y'shtola, Thanos, Sheoldred, Cloud and Aragorn rarely kill three 40-life seats
  within 10 turns (mean kill turn about 9.3-10.8), so their verdicts are mostly "unclear" or noise.

## Results

| Deck | Rule keeps the first 7 | Hands settled against the rule | Turns lost per disagreement | Turns lost per hand, all hands |
|---|---|---|---|---|
| Tymna/Thrasios | 58.6% | 10.2% | 1.45 | **0.15** |
| Dack Fayden (t6) | 49.9% | 18.9% | 1.85 | 0.35 |
| Kinnan (t6) | 56.9% | 13.8% | 1.43 | 0.20 |
| Kraum/Tymna (t6) | 49.9% | 10.5% | 1.59 | 0.17 |
| Rograkh/Silas (t6) | 58.9% | 15.2% | 1.79 | 0.27 |
| Rograkh/Thrasios (t6) | 64.6% | 13.4% | 1.17 | 0.16 |
| Sisay (t6) | 59.9% | 15.2% | 1.43 | 0.22 |
| Y'shtola* | 80.1% | 13.8% | 0.73 | 0.10 |
| Thanos* | 53.0% | 10.5% | 0.91 | 0.10 |
| Sheoldred* | 79.6% | 16.9% | 0.92 | 0.16 |
| Cloud* | 67.8% | 20.3% | 0.64 | 0.13 |
| Aragorn* | 74.9% | 6.9% | 0.32 | 0.02 |

\* fair decks: little signal (see above).

## What it means
- **Mulligans are not the lever for the Tier 6 speed gap.** Perfect mulligans would save about 0.15-0.35 kill turns per
  hand. The scorecard gap is a median win turn of 8 against 4-6 for the real lists, and even with the oracle's choices
  the Tier 6 decks kill on turn 8-9 against an empty table. So the slowness is in how the lists are played after the
  keep, not in which hands are kept.
- **The disagreements mostly go one way:** the rules keep hands the oracle would mulligan more often than the reverse.
- **Thrasios patterns:**
  - **One-landers with a line get shipped.** For example, Imperial Seal + Thassa's Oracle + Mox Diamond, or Tainted
    Pact + Vampiric Tutor + Dark Ritual. Kept, these kill around turn 4.
  - **Two-landers with no line get kept.** Interaction-only hands, or Ad Nauseam without mana, kill around turn 9.
  - The keep rule counts mana only (`t6_keep`), not whether the hand holds a line.

## Next (JD, 2026-11-07)
C5 and C6 next: a fuller measurement of the keep rules, then better rules fitted to it, each confirmed by a paired sim on
the deck's own tiers. Also: practice mode's mulligan hints should explain more. Both need a plan and JD's approval.
