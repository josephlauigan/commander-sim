# Report: Phase 4, PR 3: values from abilities (branch `phase4-values`)

Brief: `rust/agents/phase4-values.md`. Design: `audit/phase4/values-design.md`. Handoff (halves, Stage B log, checks):
`audit/phase4/values-handoff.md`.

## What was built
- `ai/values.rs` reads each card's profile once from its tags, compiled abilities and card code: mana, cards,
  removal, damage, board presence, engines, tutors and protection.
- **Worth** reads that profile at the current state: turn, hand, the opponents' boards and the card's combo role. Cast
  priority is `20 + 8 x worth`. Card rules that are code still come first: key cards, plan rules, card code `prio`,
  the commander, combo floors and timing.
- **Threat** (0-10) is the derived `bomb`. A card is a bomb at 5 or more, and the deck reading the card decides.
- `data/values_overrides.toml` holds two rows, each with a reason: Archon of Cruelty and Sheoldred, the Apocalypse.
  Their triggers live in tag-keyed engine code that the profile can't read.
- The switch is `AI_VALUES`. Unset or `1` puts every generic-policy deck on derived values; `0` keeps the old tables;
  a list of keys puts only those decks on. It is gated by the deciding player, so JD's decks and Tier 6 keep the
  old tables.
- `tests/values.rs` checks a rock, a draw engine, a wipe, a bomb and a combo piece against their Oracle text.
  `examples/values_diag.rs` prints old against derived values. Cast-priority rank correlation by tier: 0.74, 0.73,
  0.75, 0.78, 0.85 (t1-t5).

## Stage A
With `AI_VALUES` unset, the output was byte-identical to the base on t1, t2, t4 and t5 (300 games, rc=0).

## Stage B (2000 games per arm, seeds 500000-501999; each half's summed share, old -> derived, 95% CI)
| tier | half A | half B |
|---|---|---|
| t1 | 90.6 -> 88.9, -1.7 [-7.0, +3.6] | 34.5 -> 34.8, +0.3 [-3.4, +4.0] |
| t2 | 82.7 -> 80.8, -1.9 [-7.1, +3.3] | 42.3 -> 39.4, -2.9 [-6.9, +1.1] |
| t3 | 79.3 -> 77.9, -1.4 [-6.6, +3.8] | 45.6 -> 47.5, +1.9 [-2.2, +6.0] |
| t4 | 62.0 -> 64.0, +2.0 [-2.8, +6.8] | 63.3 -> 60.4, -2.9 [-7.2, +1.4] |
| t5 | 76.7 -> 76.6, -0.1 [-5.3, +5.1] | 48.2 -> 51.5, +3.3 [-0.9, +7.5] |

All ten pairs are level. One deck is worse on its own: **Sythis** on t2, 44.6% -> 39.7%, -4.9 [-8.4, -1.4]. Her value
comes from her trigger on each enchantment cast, which a per-card value can't see. Whether to add a
commander-synergy term is JD's call.

The first Stage B run found two general faults: removal was cast with no target, and mana sources ranked below
development creatures. Both are fixed (18b443f9), and every derived arm was rerun.

## Stage C
Derived values are the default for generic-policy decks. **Nothing was deleted**, because JD's decks and Tier 6 still
fall back on `generic_prio` and the shared `bomb` reads. Those go when those decks move, in the last Phase 4 step.

## Effect on JD's decks
Their own code is unchanged, but their generic opponents on t1-t5 now play differently, so JD-deck win rates will
shift. The one recalibration after Phase 4 covers this.

## Coordinator checks
- Reran `cargo test --release --no-fail-fast`: 3685 passed, 0 failed, 0 warnings.
- Read the switch: `values::on` is gated by `generic_deck`.
- Recomputed the t2 half A and t4 half B sums from the sim outputs.
- `F/head-tN` = `F/arm-tN` byte for byte for t1-t5.
- All 25 runs have rc=0, and there are no Co-Authored-By lines.
