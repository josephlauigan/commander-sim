# Report: elrond-plan

## Changes
- **Milestone `elrond`** (first Ring tempt from Elrond): `impls/elrond.rs` `elrond_enters` sets `milestone["elrond"]`; `sim-cli/src/report.rs`
  (`plan_text`, `plan_turn`). Also a `ring_tempts` stat and a "Ring tempts per game" line in the pool report
  (`sim-cli/src/poolmode.rs`, elrond only).
- **Entry planner** (`impls/elrond.rs`: `entries_of`, `entry_set`, `tempt_set`, `elrond_prio`): on the elrond seat's turn, finds the cheapest
  set (up to 3) of entry cards (creatures, Coral Barrier = 2, Muse's Encouragement / Retrieve the Esper / Unfathomable Truths = 1) that
  reaches two entries this turn within the mana available, Elrond (if not out) always in it. Members get prio 90 (Elrond 98), so Elrond
  goes first and the rest follow.
- **Elrond is held** while another entry card is in hand and no tempting set exists, until turn 8 (+2 per tax step): alone he is removed
  for free. With no entry cards in hand, or later, he is cast as before (recast at prio 45).
- **Search gate** (`ai/search.rs`, 4 lines, elrond only): when the best main-phase option has utility >= 8.9 (a planner pick) the
  look-ahead search is skipped; its evaluation cannot see the Ring and was undoing the plan.
- Test: `the_planner_casts_the_pair_that_tempts_and_holds_elrond_alone` (35 tests in tests/cards_elrond.rs, all pass).

## Measurements (lookahead, loose, 240 games, default seeds; win rate / Ring tempts per game)
| change | t1 | t3 | t5 | avg win |
|---|---|---|---|---|
| baseline | 5.0 / 0.60 | 11.2 / 0.72 | 8.3 / 0.30 | 8.2 |
| planner + hold Elrond (solo turn 6), no gate | 6.2 / 1.00 | 7.5 / 0.88 | 5.4 / 0.41 | 6.4 |
| + search evaluation bonus for Elrond/tempts (kept out) | 11.7 / 1.17 | 6.7 / 1.04 | 8.3 / 0.47 | 8.9 |
| planner, solo turn 6, no bonus, no gate | 10.4 / 0.89 | 12.1 / 0.80 | 9.6 / 0.41 | 10.7 |
| **final: planner + hold to turn 8 + search gate** | 11.2 / 1.16 | 8.3 / 1.02 | 8.8 / 0.41 | 9.4 |

t3 only, 480 games: gate off 9.2% / 0.77 tempts, gate on 7.3% / 1.08. Plan online (t3, 240): 41% -> 59% of games.
Adaptive AI, 2,400 games per cell, solo turn 0 / 4 / 6 / 9 / never (tempts t3): 0.83 / 0.82 / 1.02 / 1.23 / 1.28; win rates stayed
within 10.8-11.2% (t3), 12.3-13.6% (t1), 13.2-14.8% (t5). Without the planner: 0.51 tempts, 11.9 / 12.5 / 12.4%.
Aggression/caution style (0.3-0.8 / 0.4-0.9) changed nothing beyond noise.

## Findings
- Tempts per game roughly doubled (0.5 -> 1.0-1.3), but the win rate did not move measurably (240-game cells are +-4 points;
  the 2,400-game adaptive cells are flat). One or two tempts only reach Ring level 1-2; the deck needs four (the 3-life drain) and
  the games are decided by combat against Aurelia/Tergrid/Atraxa/Marwyn (87% eliminated, mostly by combat) and by flooding with
  held counterspells (7 cards in hand, 11 lands, one spell a turn).
- Elrond is removed 0.39 times a game; a wipe (Toxic Deluge) or cheap removal usually answers him before the next turn. Protection
  (`elrond_protect`, unchanged) only helps when a counter or bounce is in hand with mana up.
- Ring-bearer pick (`ring_pick`) checked in traces: evasive creature, Elrond only when alone; the bearer attacks every turn in the
  traces. Not changed.
- Not done: tuning a late-game mana sink for the flood (hand full of Dissipate/Dismiss/Gandalf with 9+ lands).
