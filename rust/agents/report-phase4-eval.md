# Report: Phase 4, PR 4: a learned evaluation (branch `phase4-eval`)

Brief: `rust/agents/phase4-eval.md`. Design: `audit/phase4/eval-design.md`. Handoff: `audit/phase4/eval-handoff.md`.

## What was built
- **`ai/learned.rs`:**
  - 46 features computed by existing code: for me and for the strongest opponent, life, board, power, hand, lands,
    Treasures, combo progress, PR 3 threat and hand worth, poison, commander damage and plan distance; the mean over
    opponents; table gaps; the turn; the seat; and today's raw term and `evaluate`.
  - The model: a logistic regression plus 600 boosted trees of depth 3.
  - The `AI_EVAL` switch, gated by the deciding deck through `values::generic_deck`.
  - A recorder, `AI_EVAL_RECORD`, that does not change play.
- **`search::evaluate`** calls the learned value when the switch is on. Today's term moved into `raw_term`, with
  identical output.
- **Mapping:** the model's log-odds go through the baseline's own logistic fit onto `evaluate`'s scale,
  `95 tanh(e/95)`. This is monotone and stays below `WIN_FLOOR`, so the hold bonus and the margin keep their meaning.
- **`data/eval/model.toml`** (embedded, loader-validated) records the commit, the data hash, the sample size and the
  held-out scores. It was trained by `examples/eval_train.rs` in Rust. No new crates.

## Data and fit
- **Data:** 24,992 look-ahead games on t1-t5 (seeds from 1,000,000), 830,119 positions, 220 MB. The raw data is
  deleted.
- **Weighting and split:** each (game, seat) counts once. The split is by game.
- **Held-out scores** (4,999 games):

| model | log loss | Brier | AUC |
|---|---|---|---|
| today's `evaluate` (fitted logistic) | 0.4963 | 0.1608 | 0.7168 |
| logistic | 0.4781 | 0.1547 | 0.7424 |
| logistic + 600 trees (kept) | 0.4617 | 0.1492 | 0.7675 |

## Stage B (2000 games per arm, seeds 500000-501999, look-ahead; each half's summed share, learned - old, 95% CI)
| tier | half A | half B |
|---|---|---|
| t1 | +0.5 [-4.8, +5.8] | +1.2 [-2.6, +5.0] |
| t2 | +3.3 [-1.9, +8.5] | -0.6 [-4.7, +3.5] |
| t3 | -0.6 [-5.8, +4.6] | +2.1 [-2.1, +6.3] |
| t4 | +2.1 [-2.8, +7.0] | +1.8 [-2.5, +6.1] |
| t5 | +3.7 [-1.5, +8.9] | -0.8 [-5.0, +3.4] |

All ten pairs are level, and no learned deck's CI is below zero. Zur is +3.4 [+0.6, +6.2]. The slowest arm, t3
half B, is 8% slower, which is within the 10% budget.

## Stage C
The learned evaluation is now the default for generic-policy decks on t1-t5. `AI_EVAL=0` brings back today's
evaluation as a measurement arm.

The holding-mana-for-nothing case is rare and essentially unchanged: on t3, stop is picked in 2.9% -> 3.4% of
main-1 hold decisions.

JD's decks and Tier 6 keep today's evaluation. Their opponents on t1-t5 now play differently; the recalibration
after Phase 4 covers that.

## Next
- Refit on games played with the learned evaluation. The recorded games were played with today's.
- Then richer features.

## Coordinator checks
- Reran `cargo test --release --no-fail-fast`: 3692 passed, 0 failed, 0 warnings.
- Read the `search.rs` diff and `learned::on`. The gate goes by the deciding deck, and with `AI_EVAL=0` it is off
  for everyone.
- Recomputed t5 half A from the sim outputs: 75.7 -> 79.4.
- All 15 Stage B runs rc=0.
- `F/head-tN` = `F/arm-tN` byte for byte on t1-t5.
- No Co-Authored-By lines.
