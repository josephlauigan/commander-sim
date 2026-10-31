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

## Part (b): blocks, protection and wipe responses (PR 5b)

Brief: `rust/agents/phase4-search-b.md`. Branch `phase4-search-b` (on PR 5). Handoff: `audit/phase4/search-b-handoff.md`.

### Where the three decisions are made today

- **Blocks**: `engine::combat::ai_blocks`, inside `resolve_combat_inner` (after the declare-attackers priority). One
  blocker an attacker (the engine has no double blocks; a menace attacker takes two, only the first fights). Per
  attacker, biggest first: the cheapest blocker that kills it and survives; else a trade when the attacker is worth
  at least the cheapest trading blocker; else a spare token chump (Veyran); else, for a 2+ power attacker, a chump at
  random with `chump_prob` (a sigmoid of the incoming damage over life, 50% at 35%). A 1-power attacker is never
  chumped. Lethal combat damage after blocks fires `last_chance` (Teferi's Protection) only.
- **Protection**: `engine::removal::apply_removal` asks `ai::protect_response` before the removal lands. For a
  generic-policy deck that is `decks::protect_main`: a few deck hooks first (Light-Paws' Auras, Chulane/Kinnan's
  Spellskite, Tergrid's Malakir Rebirth, the partners' white spells), then `pool::protect`: if `worth_protecting`
  (pval 4+, the commander, a key card) the cheapest saving entry of `PROTECTORS` (tapping permanents, then cards,
  then self-sacrificing ones; Teferi's Protection only for the commander or 40% of the board), else a sacrifice in
  response (`sac_in_response`).
- **Wipe responses**: `removal::apply_wipe_inner` asks `ai::wipe_response` for each victim in turn before the wipe
  hits its board (the three sweepers of `impls::sweepers` do the same). For a generic deck: `pool::wipe_response`:
  nothing if the wipe costs under 6 value (`wipe_loss`), else Golgari Charm's regeneration for a destroy wipe, else
  the first board-wide `PROTECTORS` entry that saves (self-sacrificing ones first; Ainok only when the tokens carry
  the board).

### How often they come up (PR 5's head, 300 games a tier, seeds 500000-500299, loose, look-ahead)

The survey counters (`SearchCost::BLOCK` / `PROTECT` / `WIPE`, fields `seen` / `hot` / `acted`, printed on stderr
after the cost line) count the generic seats' decisions in the real game; the counter binary's stdout = PR 5's head
byte for byte on t1-t5 (all rc=0). Per game, all generic seats together:

| tier | blocks: seen / attack reaches half the life / heuristic's blocks leave lethal | protection: targeted removal at a generic seat / a protector would save it / one used | wipes: wipe costs 6+ / a protector would save / one used |
|---|---|---|---|
| t1 | 8.10 / 2.03 / 0.87 | 4.57 / 0.09 / 0.01 | 0.85 / 0.07 / 0.03 |
| t2 | 7.42 / 1.97 / 0.77 | 12.01 / 1.32 / 0.28 | 2.59 / 0.20 / 0.07 |
| t3 | 6.25 / 1.66 / 0.55 | 2.66 / 0.25 / 0.05 | 0.85 / 0.05 / 0.03 |
| t4 | 6.68 / 1.81 / 0.77 | 3.56 / 0.24 / 0.05 | 0.76 / 0.03 / 0.01 |
| t5 | 4.49 / 0.75 / 0.22 | 3.71 / 0.07 / 0.03 | 1.15 / 0.01 / 0.01 |

- Blocks are the only one of the three with volume: 4.5-8 a game, about a quarter of them against an attack that
  reaches half the defender's life, and the heuristic's blocks leave lethal through 0.2-0.9 times a game (some of
  those are unavoidable). For scale, a game has 57-88 main / attack / counter decisions.
- Protection with a real option is rare except on t2 (1.3 a game: Brago and Sythis' tables, Teferi's Protection and
  blink), and the heuristic uses it a quarter of those times; elsewhere 0.07-0.25 a game.
- Wipe responses with an option: 0.01-0.2 a game. Their effect on a tier's win rate can only be small.

### The design

All three are searched only for a generic-policy deck on ISMCTS (`ismcts::on`), in the real game, with the
look-ahead on and within its per-game caps (`search::enabled`), behind `AI_SEARCH_B` (unset or `0`: PR 5's
behaviour; `1` every such deck; a key list). A decision is a root search (`search_root`, PR 5's): the candidates
get 2 playouts each on shared copies, then UCB, at most 6 playouts a candidate (`rollouts`, as attacks and
counters, for cost) and the 6,000-step budget, early stopping as PR 5's. The pick is held to the margin against the
heuristic's own answer on 6 shared copies (`AI_SEARCH_PAIRS`): the search overrules the heuristic only on evidence.
Below the root the decider's later main-phase choices are tree nodes as in PR 5. Leaves: `search::evaluate`.

**Playing on from inside a step.** These decisions come in the middle of a combat or a spell's resolution, which a
copy can't resume. Each is split where the decision is made so a copy can play the rest of it:
`combat::resolve_blocks` (from the blocks' card code to combat damage), `removal::apply_removal_rest` (ward and the
removal itself), `removal::wipe_victim` / `wipe_finish` (the wipe on one victim's board, then Farewell's graveyards
and state-based actions; the wipe's modes, X and the victims still to come are read from `WIPE_CTX`, set by
`apply_wipe_inner` around each response). The real game calls the same halves, so play is unchanged when the search
is off. After the option, the copy finishes what is left of the decision (the combat's damage; the removal; the wipe
on this and the later victims, who answer by the heuristic), then `search::play_on_resume`: the stack settles and the
active player's turn goes on from the step after the one in progress where re-running it would repeat it (a combat
resumes at the second main phase; the end step goes to the next turn), then everyone's turns to the end of the
decider's next turn. Lost in the copy: the rest of the spell that dealt the removal or wipe, later defenders of a
split attack and extra combats. They are the same for every candidate, so the comparison stays paired.

**Blocks** (`ismcts::choose_blocks`, `block_plans`). Candidates, duplicates dropped: the heuristic's blocks; no
blocks; kill-and-survive blocks only; the heuristic's blocks plus chumps on the biggest attackers left until the rest
is not lethal; every attacker blocked (kill-and-survive, else a trade, else a chump). Menace attackers are blocked only
as the heuristic blocks them. Stakes: searched when the attack, unblocked, reaches half the defender's life
(`AI_SEARCH_B_BLOCK`, 0.5: 0.75-2 decisions a game) and there is a second plan; else the heuristic's blocks stand.
Kept out: double blocks (the engine has none), blocks for planeswalkers.

**Protection** (`ismcts::choose_protect`). Searched when one of the `PROTECTORS` entries would save the target
(0.07-1.3 a game). Candidates: let it resolve (with the heuristic's sacrifice in response) and up to 3 saving
protectors, cheapest first, without the heuristic's own limits (`worth_protecting`, Teferi's Protection's 40%): the
search decides whether the target is worth it, which is the "key permanent targeted" test. The deck hooks before
`pool::protect` (Light-Paws, Chulane/Kinnan, Tergrid, the partners) keep the heuristic.

**Wipe responses** (`ismcts::choose_wipe`). Searched when a board-wide protector or Golgari Charm would save
something from a wipe costing 6+ value, inside an ordinary wipe (the three `impls::sweepers` keep the heuristic, as
does a board with Lena). Candidates: no answer, regeneration, up to 3 protectors.

**Cost.** Blocks are the bulk: 0.75-2 searched a game at up to 5 candidates x 6 playouts within 6,000 steps, i.e.
about 2-5% of PR 5's 140k-270k playout steps a game when every seat searches; protection and wipes add well under
1%. Inside the 10% the brief allows.

### The lean (PR 5's Stage B: 7 of 10 pairs down, mean -0.85; Meren -3.7)

Ablations first, each a 2000-game arm on t2 half B (Kaalia, Meren; ISMCTS for the half, the rest on the old
look-ahead), paired against PR 5's arms with the same binary and env: slower widening below the root
(`AI_SEARCH_WIDEN`); attack plans and counter choices with all their 6 playouts before UCB (`AI_SEARCH_ATK_MIN=6`: PR 5
gives a plan 2 and can stop on a "clear" leader after 2 shared copies, and picks the leader by its mean however few
its playouts); and a decision-by-decision look at Meren's games (`AI_SEARCH_DIAG`). Results in the handoff.

### Results so far, and what is left (JD, 2026-10-31: no sims of any kind until Phase 4's recalibration)

**Ablations on t2 half B** (Kaalia, Meren on ISMCTS, the rest on the old look-ahead; 2000 games, seeds
500000-501999, binary b1 = a0a547ff with `AI_SEARCH_B` unset; old = `AI_SEARCH=0`):

| arm | half B (95% CI) | vs old | vs PR 5's ISMCTS (ref) | Meren vs old | steps / game (table) |
|---|---|---|---|---|---|
| old (`AI_SEARCH=0`, 12 jobs) | 40.9% [38.1, 43.7] | | | 17.9% | 393,538 |
| ref: PR 5's ISMCTS (12 jobs) | 39.2% [36.4, 42.0] | -1.7 [-5.7, +2.3] | | -3.7 [-6.3, -1.1] | 371,881 |
| `AI_SEARCH_WIDEN=0.25` (8 jobs) | 38.1% [35.3, 40.9] | -2.8 [-6.8, +1.2] | -1.1 [-5.0, +2.8] | -2.3 [-4.9, +0.3] | 371,876 |
| `AI_SEARCH_ATK_MIN=6` | killed (JD) | | | | |
| `AI_SEARCH_OPP=1` (opponent nodes) | not run | | | | |
| `AI_SEARCH_MIN=6` (old allocation) | not run | | | | |

- The ref arm reproduces PR 5's Stage B arm exactly (Meren 17.9% to 14.2%, Kaalia 23.0% to 25.0%, half -1.7): the
  data changes since PR 5 do not touch t2.
- Slower widening does not help (-1.1 against PR 5's ISMCTS, inside the noise): not kept.
- Meren's decisions (`AI_SEARCH_DIAG`, 2000 games each arm): main-phase decisions 14.4 a game on the old look-ahead,
  departing from the policy's top play 19.6% of the time at 3,733 playout steps a decision; on ISMCTS 14.1 a game,
  departing 14.5% at 2,626 steps (with two or more real options: 25.6% vs 19.6%). Attacks: 3.7 vs 3.2 a game,
  departing 68% vs 71%. So ISMCTS overrules Meren's policy a quarter less often in the main phase, on 30% fewer
  steps: the lean looks like lost main-phase look-ahead (the early stops and the 6-shared-copies rule against the
  policy's top play), not a worse tree. Kaalia shows the same shift (13.2% to 9.8%) and gained. The `AI_SEARCH_MIN=6`
  arm (every root candidate 6 playouts before UCB, as the old look-ahead allocated) tests that directly.
- The `--trace` of games where the arms differ was not done.

**Stage A** (before the no-sims order): `AI_SEARCH_B` unset = PR 5's head byte for byte on t1, t2, t4, t5 (300
games); `AI_SEARCH_B=1` twice on t1 = the same output. The later `AI_SEARCH_OPP` / `AI_SEARCH_ATK_MIN` knobs are
default-off by code reading and `tests/ismcts_b.rs::off_by_default_is_pr5` (no sim).

**Stage B and C: not run.** Part (b) stays behind `AI_SEARCH_B`, default off; PR 5's ISMCTS stays the default.

**To run at recalibration** (2000 games an arm, seeds 500000-501999, loose, look-ahead, one `--jobs` for all arms):
1. Stage B, each tier t1-t5: PR 5's head (no env) against `AI_SEARCH_B=<half A keys>` and `AI_SEARCH_B=<half B
   keys>` (the values-handoff halves); half share deltas with CIs, games/s, CPU seconds, playout steps a game.
2. The lean, t2 half B against `t2-old` / `t2-B-ref`: `AI_SEARCH_ATK_MIN=6`, `AI_SEARCH_OPP=1`, `AI_SEARCH_MIN=6`;
   whichever helps, again on t1 half B (Lathril, Tatyova).
3. Stage C from 1: `AI_SEARCH_B` on by default where level or better at about 10% more cost or less; then with no
   env the head = the chosen arm byte for byte on each tier (300 games); `all_decks adaptive 3` / `lookahead 1`.
