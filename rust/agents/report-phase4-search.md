# Report: Phase 4 PR 5, ISMCTS where it matters

Brief: `rust/agents/phase4-search.md`. Design: `audit/phase4/search-design.md`. Handoff: `audit/phase4/search-handoff.md`.
Part (a) only (main-phase, attack and counter decisions). Part (b), blocks, protection and wipe responses, is not
started: it is a later PR.

## What it does

- `ai/ismcts.rs`: single-observer ISMCTS over the look-ahead's determinized copies (`playout_copy`), for
  generic-policy decks only (`values::generic_deck`: not JD's decks, not Tier 6).
  - The tree holds the decider's own choices (UCB1, C = 15 points, progressive widening, depth 3); opponents reply
    from their policy. Leaves use the learned evaluation (PR 4).
  - Budget: 6,000 playout steps a decision, with early stopping on a clear leader or when nothing can beat the
    policy's top play. The search overrules the top play only on 6 or more shared copies.
  - Stakes test: decisions with one real option and nothing at stake play the policy's top play without playouts.
- Switch `AI_SEARCH`: unset or `1` = ISMCTS (the default after Stage C), `0` = the old look-ahead, or a list of
  deck keys. Tuning knobs `AI_SEARCH_*` for A/B runs.
- A per-seat search-cost counter (`state::SearchCost`) printed on stderr, so reports stay byte-identical.

## Results

**Stage A.** With `AI_SEARCH=0`, output equals the base byte for byte (t1, t2, t4, t5; 300 games). `AI_SEARCH=1`
is deterministic (two runs, same output).

**Stage B** (half-tier pairs, 2000 games a pair arm, seeds 500000-501999, loose; ISMCTS minus old, points):

| tier | half A | half B |
|---|---|---|
| t1 | +0.7 [-4.6, +6.0] | -1.8 [-5.5, +1.9] |
| t2 | -2.7 [-7.9, +2.5] | -1.7 [-5.7, +2.3] |
| t3 | -1.7 [-6.8, +3.4] | +2.4 [-1.8, +6.6] |
| t4 | -1.3 [-6.2, +3.6] | -2.0 [-6.3, +2.3] |
| t5 | -1.4 [-6.6, +3.8] | +1.0 [-3.1, +5.1] |

- All ten pairs are level. Seven of ten lean down; the mean is -0.85, about 1.1 standard errors from 0 when the ten
  pairs are pooled, so not significant.
- One deck's own CI is below 0: Meren -3.7 [-6.3, -1.1] (t2 half B; Kaalia in the same half is +2.0).
- Cost: with every seat on ISMCTS, playout steps per game fall 14-32% by tier (t2 the most: 398k to 270k).

**Stage C.** ISMCTS is the default for generic-policy decks: level at a lower cost, not better.

## Coordinator's checks (2026-10-30)

- Read the diff: the `brain::main` hook (`ismcts::tree_pick`) returns None outside an ISMCTS playout; `on()` is gated
  by `generic_deck`; the `search.rs` changes outside the ISMCTS dispatch are cost booking and refactors.
- Merged origin/main (#176-#178) into the branch, rebuilt with `--all-targets` (0 warnings) and reran the full suite: 3709
  passed, 0 failed.
- Spot-checked t2 half B against the raw outputs: old 23.0 + 17.9 = 40.9%, ISMCTS 25.0 + 14.2 = 39.2% (Meren 17.9 to
  14.2). All 15 Stage B runs rc=0.

## Open

- JD's call: keep ISMCTS as the default (level and cheaper) or flip `parse_mode`'s default to off until PR 5b.
- Next ablations for the lean: slower widening, and opponent nodes at their first reply.
- Pre-existing, not from this PR: t5 skips 1-7 games an arm on an internal simulator error (also in PR 4's arms).
