# C5: the mulligan oracle against real tables (2026-11-08)

`commander-sim --mulligan-oracle aragorn,cloud,sheoldred,thanos,thrasios,yshtola --vs-tier --hands 100 --playouts 40
--seed 500000 --recheck 20 --out c5-hands.csv`, on main 502f7ddc (after the mana-colour fixes #305 and #308; Thrasios on
the S2 list). Adaptive AI, loose profile, each deck only against its own tiers from `data/my_decks.toml`. Raw output:
`c5-report.txt` (the full report) and `c5-hands.csv` (one row per hand and tier, 20 columns). It ran in 57 minutes.
The keep-rule proposals that come out of it: `C6-PROPOSALS.md`.

## What it measures
The C4 oracle (2026-11-07, `../c4-2026-11-07/REPORT.md`) played goldfish games and couldn't judge fair decks. This one
plays real pod games. For each dealt hand there are 40 games that keep it and 40 that mulligan it, against three
opponents drawn from the deck's own tiers. Both arms use the same seeds, so they see the same opponents and draws. The
mulligan arm follows the deck's normal keep rule from the next hand on. The score is the win rate.

The oracle says KEEP or MULLIGAN only when the paired difference is more than 2 standard errors; otherwise UNCLEAR.
With 40 games per arm one hand's standard error is about 8 points, so only large gaps settle. Most hands come out
unclear. The patterns below come from averaging many hands that share a feature, not from single verdicts.

**The first mulligan is free** (multiplayer Commander), so mulliganing a first seven deals a fresh random seven. That is
why the mulligan arm's win rate sits near the deck's average for every hand. A hand is worth keeping when it beats the
deck's average hand, not when it beats zero.

## Results (every row: the deck over its own tiers)

| Deck (tiers) | Rule keeps the first 7 | Hands settled against the rule | Win points lost per hand | Look-ahead recheck agrees |
|---|---|---|---|---|
| Y'shtola (t5) | 86% | 5.3% | **1.32** | 15 of 20, 0 opposite |
| Aragorn (t4-t5) | 76% | 5.6% | 0.97 | 15 of 20, 1 opposite |
| Tymna/Thrasios (t6) | 65% | 3.7% | 0.83 | 10 of 20, 0 opposite |
| Sheoldred (t3-t4) | 87% | 3.1% | 0.70 | 19 of 20, 0 opposite |
| Cloud (t1-t3) | 66% | 3.5% | 0.66 | 18 of 20, 0 opposite |
| Thanos (t5) | 50% | 2.0% | 0.47 | 15 of 20, 0 opposite |

"Win points lost per hand" is the win rate the rule gives up on the hands the oracle settles against it, spread over
all hands. A perfect mulligan judge would gain at most about that many points of win rate. That is under 1.5 points for
every deck, and under 1 for five of six. **The keep rules are close to right against real tables;** mulligans are not
where these decks lose games.

The recheck replays 20 random hands with the look-ahead AI, the one the normal sims use. In nearly all of them the two
AIs either agree or one of them is unclear; they disagree outright on one hand in 120. So the adaptive AI's verdicts
can stand for the look-ahead AI's. For Thrasios, 10 of the 20 settle under one AI and are unclear under the other: noise
at 40 games, not opposite answers.

## The one shared pattern: two lands and no fast mana

Grouped by land count, the hands the rules keep do worse than a fresh seven at two lands, and better at three or more:

| Deck | Kept 2-land hands: mulligan minus keep | 3 lands | 4 lands |
|---|---|---|---|
| Y'shtola | **+5.2** (32 hands) | -4.9 | -4.7 |
| Sheoldred | **+3.8** (76) | -2.3 | -5.1 |
| Aragorn | **+2.2** (80) | -3.8 | -1.0 |
| Cloud | -0.6 (99) | -1.1 | -5.7 |
| Tymna/Thrasios | -2.6 (48) | -6.3 | -6.1 |
| Thanos | -3.2 (36) | -4.9 | -8.0 |

(Positive means mulliganing the hand wins more often. These are means over the hands; one hand's noise is about ±8.)

Split by fast mana (Sol Ring, Signets, mana dorks and the like, the `fast` count in the keep rules), the cost sits in
the **two-land hands with no fast mana**. In first sevens, mulliganing beats keeping by:

| Deck | Hands | Mulligan minus keep | ±2 standard errors |
|---|---|---|---|
| Y'shtola | 22 | **+7.5** | ±4.6 |
| Sheoldred | 52 | **+3.8** | ±2.8 |
| Aragorn | 44 | +2.4 | ±2.2 |
| Cloud | 45 | +2.1 | ±2.2 |
| Thrasios | 19 | -1.4 | ±6.6 |
| Thanos | 12 | -1.9 | ±6.5 |

Two-land hands with fast mana are about as good as a fresh seven or better for Y'shtola (-4.3, 7 hands), Sheoldred
(+1.0, 18) and Cloud (-4.1, 42). Aragorn's are not (+2.2, 28 hands, about ±3.5), so for Aragorn the weak spot is two lands
whatever the hand holds. The worst kept hands are all of this shape:
- Y'shtola: Caves of Koilos, Hallowed Fountain, Anguished Unmaking, Displacer Kitten, Exsanguinate, Path, Propaganda:
  keep 7%, mulligan 40%. Plains, Watery Grave, Debt to the Deathless, Exquisite Blood, Exsanguinate, Prayer of Binding,
  Teferi's Protection: 28% vs 50%.
- Sheoldred: Swamp x2, Call of the Ring, Exquisite Blood, Gix, Grave Pact, Triskelion: 22% vs 45%. Swamp x2,
  Exsanguinate, Grave Pact, Overkill, Swiftfoot Boots, Yawgmoth: 18% vs 38%.
- Aragorn: Sunpetal Grove, Verdant Catacombs, Mana Drain, Path, Smothering Tithe, Sylvan Library, Young Pyromancer: 10%
  vs 28%.

**Why it hurts these three and not the others:** their kept two-landers are hands of 4-6-mana payoffs and interaction
with nothing to bridge the gap. At t3-t5 a deck that misses its third land drop falls behind tables that are deploying
by turn 3. Thanos's and Thrasios's two-landers without fast mana are not worse than a fresh seven (on 12 and 19 hands,
too few to say why; both decks are built on cheap tutors and cantrips). Cloud already keeps two-landers only with a
`keep2` card (its table), which may be why its number is smaller.

## Other patterns, none strong enough to act on
- **Zero or one land:** every rule already mulligans these, and the oracle agrees (mulligan minus keep +4 to +13 in
  every deck). The few one-landers the oracle would keep had Sol Ring or a dork and a cheap engine (Aragorn: Jungle
  Shrine, Sol Ring, Counterspell, Fact or Fiction, Niv-Mizzet, Smothering Tithe, Windborn Muse: 48% vs 20%), too rare to
  write a rule for.
- **Thanos keeps only 50% of first sevens**, the fewest. Its kept hands beat a fresh seven at every land count, and the
  hands it mulligans are mostly unclear (43 of 50). The rule may be a little strict, but nothing settles.
- **Second sevens (after the free mulligan)** follow the same shape, on too few hands to read separately.

## Limits
- 100 hands per deck and tier, 40 games per arm: patterns over 20-80 hands are about ±2-5 points. They show where to
  look; the paired sims in C6 decide.
- The adaptive AI plays the games (cheap enough for up to 160,000 games in under an hour). The look-ahead recheck agrees, but on only 20 hands
  per deck.
- The mulligan arm follows today's keep rule from the next hand, so a hand's MULLIGAN number includes that rule's own
  mistakes on later hands.
