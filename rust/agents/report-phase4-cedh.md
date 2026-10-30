# Report: Phase 4, PR 2: plan files for Yuriko, Yawgmoth and the cEDH decks (branch `phase4-cedh`)

Brief: `rust/agents/phase4-cedh.md`. Design: `audit/phase4/plans-design.md` ("PR 2"). Handoff (deck table, reasons,
final checks): `audit/phase4/plans-handoff.md`. Stacked on `phase4-plans` (#171).

## What was built
Schema extensions in `ai/plan_data.rs`, each validated by the loader and unit-tested in `tests/plans.rs`:
- `bf:` groups (held only on the battlefield), `?cond` group entries, `counts` (N cards of a group), `engines_ahead`;
- named `[sets]` (`hand:`/`bf:`/`gy:<set>`);
- nested `|` and parentheses, and sums and differences in conditions;
- hand, mana, board and table quantities (stop chance, open mana, archenemy score and others);
- f64 priority values and `per_tax`;
- cEDH slots: `[[wish]]` scored rules, a mulligan rule table, `[[hold_up]]`, `[[go_now]]`.

`impls/t6_plans.rs` routes a Tier 6 deck's slots to its plan file when the plan is on; `t6_plans::HAND` lists the
decks that still have hand code.

## Paired results (2000 games per arm, seeds 500000-501999, `AI_PLAN_DATA=0` vs `=<deck>`, every run rc=0)
| Deck | Tier | Hand | Plan | Delta [95% CI] | Outcome |
|---|---|---|---|---|---|
| yuriko | t4 | 18.8% | 18.1% | -0.7 [-3.4, +2.0] | level: hand code deleted |
| yawgmoth | t5 | 17.6% (PR 1 hand arm) | 17.6% | +0.0 [-2.7, +2.7] | PR 1's -1.9 closes |
| scion (`--deck scion`) | t6 | 30.1% | 30.1% | byte-identical | deleted |
| kinnan | t6 | 24.6% | 24.6% | byte-identical | deleted |
| sisay | t6 | 31.1% | 31.1% | byte-identical | five slots deleted (`fetch` stays code) |
| kraum/tymna | t6 | 29.8% | 29.8% | byte-identical | deleted |
| rograkh/silas | t6 | 18.6% | 18.3% | -0.3 [-3.3, +2.7] | level: deleted |
| rograkh/thrasios | t6 | 21.1% | 20.8% | -0.3 [-3.4, +2.8] | level: deleted |
| dack | t6 | | | | not expressible as data; hand code kept |
| godo | | | | | skipped (not seated) |

Yuriko's hand arm is 18.8% here against 20.9% in PR 1. That's because PR 1's hand arm still seated Chulane's hand code
(13.2%). Chulane now plays only its plan (15.6%) in both arms, so the table is stronger, and this pair is the fair
one. Yawgmoth's hand code was deleted in PR 1, so its pair uses PR 1's hand arm, measured on that binary.

## Bug found and fixed (5aa32540)
The Tier 6 slots first answered every deck with an active plan (Kaalia, Chulane, Yawgmoth, Yuriko's plan arm).
Through them, priority bypassed the key cards and the wish list lost its mana step. They now answer Tier 6 players
only, and a regression test covers it. The Yuriko and Yawgmoth pairs were rerun after the fix with the same numbers.
t2 matches PR 1 byte for byte.

## Not expressible yet
- **Dack Fayden.** Its slots read its own cost and tax, a per-round risk bar over the table's stop chances, Chalice's
  net, the opponents' creature share and counter likelihoods.
- **Sisay's `fetch`.** It is a card choice, not a plan slot.

## Coordinator checks
- Reran `cargo test --release --no-fail-fast`: 3678 passed, 0 failed, 0 warnings.
- Read the diff of 5aa32540.
- Pulled every share in the table above from the sim outputs: all 35 runs rc=0.
- The t6 hand arm and Kinnan's plan arm are byte-identical files.
- The `--deck scion` arms differ only in the wall-clock line.
- No Co-Authored-By lines.
