# Phase 4 PR 4: a learned evaluation (design)

Brief: `rust/agents/phase4-eval.md`. Branch `phase4-eval` (on `phase4-values`). Code: `ai/learned.rs` (features,
model, switch, recorder), `examples/eval_train.rs` (the fit), `data/eval/model.toml` (the fitted weights).

## What changes

`search::evaluate` scores the position at the end of a look-ahead playout. For a non-terminal position of a
generic-policy deck (`values::generic_deck`) with the learned evaluation on (`AI_EVAL`), the value becomes the
model's estimate that the evaluated seat goes on to win, mapped onto `evaluate`'s scale. Wins and losses
(`win_score`, `WIN_FLOOR`, -100) are unchanged; JD's decks and Tier 6 keep today's `evaluate`. Nothing else in the
search changes (rollouts, margin, hold and ramp bonuses).

## Positions and labels

- **Where:** in the real game (never in a playout), at the end of each turn of a generic-policy seat that is still
  alive with the game still going (`engine::turn::run_rounds`, after `play_turn`). That is the position the
  look-ahead scores: a playout ends at the end of the decider's next turn (`horizon` 1). Only the seat whose turn
  just ended is recorded (its own view), so each position appears once.
- **Label:** did that seat win the game. A draw (round cap, timeout) is a 0, as it is no win for the look-ahead
  either (they are 0.2-4% of games).
- **Weight:** a seat's positions in one game share one label, so a long game would count more. Each position is
  weighted 1 / (the seat's recorded positions in that game): every (game, seat) counts once in the fit and in the
  scores.
- **Games:** look-ahead AI (it plays the games whose outcome the evaluation predicts), loose, tiers t1-t5, seeds from
  1,000,000 (outside the paired tests' 500000-501999),
  5,000 games a tier (t1 from seed 1,000,000, t2 from 1,100,000, ... t5 from 1,400,000).
- **Split:** by game: seed % 5 == 0 held out (20%), the rest fitted. No game is on both sides.

## Recorded per position (`AI_EVAL_RECORD=<dir>`)

One fixed-size binary record (little-endian): seed (u64), seat (u8), label (u8: 1 won, 0 lost or draw), seats at
the table (u8), a pad byte, own turns (u16), round (u16), today's raw term (f32, `search::raw_term`, before the
tanh), then 16 numbers for each of 4 seats in turn order from the evaluated
seat (an empty seat is all 0): alive, life, board (`strength`'s board term: `pval` plus output), power, hand, lands,
Treasures, combo progress, `strength`, threat (PR 3's `values::threat` summed over the seat's permanents), the most
commander damage it has taken from one commander, poison, plan distance (`planner::distance`, the fewest missing
piece groups over the deck's catalogue lines, capped at 6), hand worth (PR 3's `values::worth` of each card in hand,
summed), creatures, and the commander damage it has taken from the evaluated seat. 276 bytes a
record; one file per thread under the directory (`<dir>/rec-<pid>-<n>.bin`),
written a whole game at a time. Budget: 5,000 games x 5 tiers x ~35 positions = about 0.9M records, about 0.25 GB.

The recorder only reads the game (no random numbers, no caches that play reads): the output is byte-identical with
it on or off (checked, 300 games, t1 and t5).

## Features (from the record, one function for training and play)

Per position, x (the evaluated seat is "me"; "opp" = the live opponents):
- my terms: life, board, power, hand, lands, Treasures, combo progress (7), threat, hand worth, poison, commander
  damage taken, plan distance, creatures;
- the strongest opponent's (by `strength`) same terms (13), and the mean over live opponents of life, board,
  power, hand, threat (5);
- the table: live opponents, my `strength` minus the strongest's, minus the mean, my life minus the lowest
  opponent's, the most commander damage I have dealt to a live opponent, the least life of a
  live opponent;
- time: own turns (and turns x my board, turns x my hand, turns x combo: the worth of a card or a board changes
  through the game);
- seat: my seat in turn order (0-3, as three indicators); today's raw `evaluate` term (`s - 0.6 max - 0.4 mean + 12
  dead`) and today's `evaluate` (after the tanh).

The tier is not a feature: it is the same for every option of a decision, so it would only move the constant.
Every feature is standardised by the fitted data's mean and standard deviation (stored with the model).

## Model

L2-regularised logistic regression, fitted by Newton's method (with step halving) on the weighted log loss; the
penalty is picked from 1e-5 .. 1e-1 on validation games (seed % 5 == 1) of a fit on the other fitted games, then the
model is refitted on all fitted games. Boosted depth-3 trees are tried on top of the logistic's log-odds (rounds
picked on the validation games, learning rate 0.05): they are kept only if the logistic plus trees beats the logistic
alone by more than 0.005 held-out log loss (a tree is three comparisons at play). The baseline is today's `evaluate`
through a fitted logistic (`p = σ(a + b·evaluate)`, fitted on the same games), so it is scored on the same
probability scale. Held-out scores: weighted log loss, Brier and AUC, overall and by tier.

A trial on the 600 identity-check games showed the raw terms are heavy-tailed: today's evaluation through its tanh
scored 0.500 log loss, its raw term (no tanh) 0.529, and a logistic on the raw features 0.527. So the board, power,
threat, hand worth, creatures, Treasures and the strength differences enter on a signed log scale
(`sign(x) ln(1 + |x|)`), and today's `evaluate` itself (after the tanh) is a feature beside its raw term. The trial
then gave 0.490 (logistic), 0.486 (with trees).

## The mapping onto `evaluate`'s scale

The model gives z (the log-odds that I win). The baseline fit (`p = σ(a + b·evaluate)`) says which value today's
evaluation gives to the same odds: e' = (z - a) / b. The learned value is `95 tanh(e' / 95)` (about e' for the
usual values, -40 .. 40; never 95). So:
- it is monotone in z (b > 0, checked by the loader) and stays below 95 < `WIN_FLOOR` (96);
- a position today's evaluation and the model rate equally likely to win gets about the same number, so the hold
  bonus (about +3), the -50 for a failed option and the margin's noise keep their meaning.

## The switch

`AI_EVAL`: unset = today's evaluation (Stage A: byte-identical to the base), `0` today's, `1` learned for every
generic-policy deck, `a,b` learned for those deck keys. Stage C sets the default per tier where the paired tests
are level or better.

## The model file

`data/eval/model.toml`, embedded with `include_str!` and checked when first read: the feature names match the
code's list in order, every number is finite, the scales are positive, `b_r` > 0; the commit, the data's hash (FNV-1a
of the record files), the games and positions fitted and the held-out scores are in it. A bad file panics on first
use with the reason (a test checks each refusal).

## Paired tests (Stage B)

As PR 3: per tier, the fixed halves of `audit/phase4/values-handoff.md`; `AI_EVAL=0` against `AI_EVAL=<half>`, 2000
games per arm, seeds 500000-501999, loose, look-ahead. The half's summed share with a Wald 95% CI (`half.sh`).
Level = the CI includes 0. Speed: games per second of each arm (wall clock, 22 jobs).

## The "holding mana for nothing" check (Stage C)

`AI_EVAL_DIAG=1` on t3 (1,000 games), old and learned: among decisions where the heuristic's top play is an engine,
commander or mana permanent and a hold card exists, how often "stop (hold mana)" is picked, and how often the stop
line then really kept the mana up (`kept mana up`).
