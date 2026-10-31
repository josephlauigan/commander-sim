# Phase 4 PR 5: tree search (ISMCTS) where it matters (design)

Brief: `rust/agents/phase4-search.md`. Branch `phase4-search` (on `phase4-eval`). Code: `ai/ismcts.rs` (stakes,
switch, knobs, tree, root search, the three decisions), hooks in `ai/search.rs` (dispatch, cost booking) and
`ai/brain.rs` (the decider's choices inside a playout follow the tree). Tests: `tests/ismcts.rs`.

## Today's cost (measured first)

The base (`f03f69bc`) plus a cost counter only: each seat's look-ahead decisions and the engine steps their
playouts took, by kind, printed on stderr (`Search cost per game ...`, from `realism`), so the report is unchanged.
The counter binary's stdout equals the base's byte for byte on t1-t5 (300 games, seeds 500000-500299, loose,
look-ahead; all rc=0). All seats at t1-t5 are generic-policy decks.

| tier | decisions / game (main / attack / counter) | playout steps / game (main / attack / counter) | steps / main decision (median, p90) | games/s (22 jobs) |
|---|---|---|---|---|
| t1 | 72.5 (53.9 / 17.0 / 1.7) | 226,711 (182,321 / 43,449 / 942) | 3,384 (2,127, 8,436) | 3.23 |
| t2 | 106.6 (85.1 / 19.3 / 2.2) | 398,275 (341,080 / 55,813 / 1,382) | 4,008 (2,434, 9,861) | 1.76 |
| t3 | 68.3 (52.9 / 15.3 / 0.2) | 165,907 (130,906 / 34,913 / 87) | 2,476 | 6.74 |
| t4 | 70.0 (52.2 / 15.0 / 2.8) | 203,809 (162,305 / 39,989 / 1,515) | 3,109 | 4.04 |
| t5 | 71.6 (51.3 / 12.4 / 7.9) | 181,116 (147,403 / 30,268 / 3,445) | 2,874 | 5.46 |

Main-phase decisions are 75-85% of the cost; a decision is 6 candidates (`top_k` 5 plus stop) x 6 rollouts, up to
24 when the margin hands it back (adaptive rollouts), and a playout is about 60-90 steps. The plan's "125 decisions,
270,000 steps" is the right order (t2 is above it, the rest below).

What the look-ahead does with it (`AI_SEARCH_DIAG=1`, 88,599 main decisions over the five tiers): it departs from
the policy's top play in 16.6% of main decisions (15-18% a tier), and in 4.4% of the decisions with one real option
(21% of them), 12.5% with two, 26-31% with four or more. The other facts of a decision (a wipe or removal among the
candidates, lethal on board, danger, combo distance, the utility gap) barely change the rate: the changes are spread
thin, so a stakes test saves in proportion to what it skips. Attack decisions depart from the policy's plan (the
heuristic defender, filtered) 66-70% of the time; counters are 1-11% of decisions and cheap.

## The search

Scope: the look-ahead decisions of the generic-policy decks (`values::generic_deck`) when `AI_SEARCH` turns them
on: main-phase plays, the attack plan, counters. JD's decks and Tier 6 keep today's search (Phase 4's last step).

**Determinizations.** As today (`search::playout_copy`): each copy re-deals every opponent's hand from their hand
and library (a commander in hand stays known) and shuffles the decider's own library below the cards it put on top
itself. That is what the decider knows: its own hand, every board, graveyards, counts. Copy j of a decision is seeded
from the decision's seed and j, so all candidates' j-th playouts see the same hidden cards and the same draws, and
they compare paired.

**The tree (single-observer ISMCTS).** Nodes are the decider's own choices, from its point of view (an option's
label and occurrence, the same key today's search uses to find an option in a copy). The root's children are the
decision's candidates: the policy's top `top_k` (5) real options and stop for a main-phase decision, today's plans
for an attack (filtered and all-out at each opponent, none), counter or not for a counter. Below the root, each of
the decider's later main-phase choices inside the playout (the rest of this phase, its next main phases until the
horizon) is a node: `brain::main` in a copy asks `ismcts::tree_pick`. Opponents' replies come from their policy
(the heuristic at its top play, as in today's playouts): no opponent nodes, since four players' replies would split
the budget four ways at each level and the budget is a few dozen playouts.

- Selection at a node: UCB1 over the children available in this copy, with ISMCTS's availability count in place
  of the parent's visits: mean + C sqrt(ln(avail) / visits), values on `evaluate`'s scale capped at a win (100).
  **C = 15 points** (a playout's value has a standard deviation of 12-18 points, eval-fix.md).
- Progressive widening below the root: a node visited v times may try its best ceil(0.5 sqrt(v)) candidates by
  the policy's utility, so with few visits a playout follows the policy (as today's playouts do) and alternatives are
  tried only where the tree has been many times.
- Expansion: one new node a playout; past it, or past the depth (3 levels: the root's candidates and two more
  choices), the policy plays.
- The root: every candidate first gets 2 playouts (copies 0 and 1, paired), then UCB picks the candidate (C as
  above, over all the root's playouts). When UCB picks the leader and the leader is not the policy's top play, the
  top play is played instead while it has fewer playouts: the pick is held to the margin against the top play (as
  today), and the two must be compared on the same copies.

**The horizon.** Today's: the rest of the decider's turn and everyone's turns to the end of the decider's next turn
(`horizon` 1), i.e. one to two of its turns. Per-playout step cap (`playout_work` 2,000) and the per-game caps are
unchanged.

**The leaf value.** `search::evaluate`: `win_score` for a win (100 this turn, less 4 a later own turn, never below
96), -100 for a loss, else the learned evaluation (PR 4, the default for generic decks). The main-phase bonuses are
today's: the kept-hold credit to each open playout that ends the decider's turn with the held card's mana up, and
the ramp bonus (0 by default). A candidate that can't be played in a copy scores -50 (today's rule).

**The budget.** A per-decision limit in **playout steps** (the engine's work counter, `search_work`), not wall
clock: seeded runs stay reproducible. **6,000 steps** a decision (today's main decisions average 2,500-4,000 and
reach 8,000-10,000 at the 90th percentile, when adaptive rollouts run to 24). The playout that crosses the budget
finishes. A main-phase candidate gets at most 24 playouts (today's adaptive cap); an attack plan or a counter choice at
most 6 (today's count: a first trial at 24 doubled the attack cost on t1, 43k to 84k steps a game, as the seven
plans are close and rarely separate).

**Early stopping.** Once every root candidate has its 2 playouts, after each playout:
- a candidate clearly ahead: the leader's paired difference with every other candidate (on their shared copies)
  is above z standard errors (`search::margin_holds`, z = 2.36, today's adaptive z for repeated looks): stop, and
  the leader is the pick;
- no candidate can be expected to beat the policy's top play: every other candidate's paired gain over the top play
  is at or below 0.5 standard errors with 3 or more shared copies (`search::hopeless`): stop, and the top play is
  the pick.

**The pick.** The leader by capped mean, held to the margin against the policy's top play (z = 2.36 on their shared
copies), as today: the look-ahead overrules the policy only on evidence. Attacks: the leader, with today's near-tie
rule (a "none" that wins by less than 1 point goes to the best attack). Counters: counter when its mean is higher.

**The stakes test (which decisions get search).** From the measurement, a main-phase decision is searched when:
- a board wipe or a removal spell is among the candidates (5% and 11% of decisions);
- lethal on board: the decider's creatures' power reaches the lowest opponent's life (17%);
- danger: the table's possible attack reaches the decider's life (`Situation::danger` >= 1, 5%);
- a combo turn: the decider is at most one piece group from a catalogued line (`planner::distance` <= 1, 26%), or
  an opponent's line is assembled (distance 0, 11%);
- a close call: two or more real options and the policy's top two within 0.5 utility (17%).
The rest (one real option and nothing at stake, or a clear policy favourite) play the policy's top play without a
search. Over the five tiers this searches 61% of main decisions (47-89% by tier: t5's combo decks are mostly near
a line), 67% of their steps, and catches 65% of the decisions where today's look-ahead departs from the policy.
Attacks are searched when lethal is on board, the decider is in danger (>= 0.5) or an opponent has an untapped
creature (92% of attack decisions; else the policy's attack). Counters (counter wars) are always searched.

**Switch.** `AI_SEARCH`: unset or `0` today's look-ahead (Stage A), `1` ISMCTS for every generic-policy deck, a key
list for those decks. Knobs for A/B runs: `AI_SEARCH_BUDGET`, `_C`, `_DEPTH`, `_WIDEN`, `_MIN`, `_MAX`, `_CLOSE`,
`AI_SEARCH_GATE=0` (search every decision). `AI_SEARCH_DIAG=1`: the per-decision stakes line (stats only).

## Expected cost

Main decisions: 61% searched, each stopping at 12 playouts at the least (2 per candidate) and 6,000 steps at most;
attacks 92% with early stopping. If a searched decision costs about what today's does, the per-game cost falls to
about two thirds; early stopping and the budget's cap on the long tail should take it lower. Measured in Stage B.

## Stages

- **A:** `AI_SEARCH` unset and `0` = the base byte for byte on t1, t2, t4, t5 (300 games); `AI_SEARCH=1` twice with
  the same seed and `--jobs` = the same output.
- **B:** each tier t1-t5, both halves (values-handoff.md's halves), `AI_SEARCH=0` against ISMCTS on that half
  (`AI_SEARCH=<half's keys>`), 2000 games a pair arm, seeds 500000-501999, look-ahead, loose; games/s and playout
  steps a game for both arms.
- **C:** default where level or better; else today's stays, with why.

## Revisions after the first Stage B arms (t1, 2000 games, paired against `AI_SEARCH=0`)

The first binary (cfbd9ec3: the stakes test above, a close call at 0.5) leaned down on both t1 halves: A -3.2
[-8.5, +2.1], B -3.6 [-7.2, +0.0], at 21-25% fewer playout steps a game. Ablations on t1 half B (Lathril,
Tatyova; the table's steps a game, all seats; today's 228,753):

| arm | half B delta (95% CI) | steps / game |
|---|---|---|
| first run (close 0.5, overrule on 2 copies) | -3.6 [-7.2, +0.0] | 171,542 |
| + overrule only on 6 shared copies (3868adee) | -3.7 [-7.3, -0.1] | 172,852 |
| + no stakes test (`AI_SEARCH_GATE=0`) | -1.5 [-5.2, +2.2] | 205,423 |
| + no tree below the root (`AI_SEARCH_DEPTH=1`, stakes test on) | -4.5 [-8.1, -0.9] | 170,987 |
| + close call unlimited (`AI_SEARCH_CLOSE=1e6`) | -1.8 [-5.5, +1.9] | 204,871 |

- The stakes test's close-call rule cost about 2 points: a clear policy favourite among several real options is
  still a decision the look-ahead improves (Tatyova lost most: -2.9 [-5.4, -0.4]). **Changed:** every decision with
  two or more real options is searched; one real option is searched only when something is at stake (the facts
  above). That skips 4.4 main decisions a game on t1 (6%), cheap ones, at the same cost as searching everything.
- **Kept:** the look-ahead overrules the policy's top play only on 6 or more shared copies (`AI_SEARCH_PAIRS`, the
  pick and the clear-leader stop against the top): today's margin never decided on fewer, and a t-test on 2 copies
  passes noise.
- The tree below the root helps a little (depth 1 is 2.7 points under depth 3 with the same stakes test, inside the
  noise): kept.
- Stage B's second run (binary D, 02b729d2) uses these defaults; its t1 half B arm is the `CLOSE=1e6` arm above (the
  same rule: a finite gap is always below 1e6).

## Stage C

Level on t1-t5 in both halves (`search-handoff.md`): ISMCTS is the default for generic-policy decks; `AI_SEARCH=0`
keeps today's. The switch's default changed accordingly (unset = ISMCTS).
