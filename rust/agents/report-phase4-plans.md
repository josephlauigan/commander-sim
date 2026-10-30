# Report: Phase 4, PR 1: deck plans as data (branch `phase4-plans`)

Brief: `rust/agents/phase4-plans.md`. Design: `audit/phase4/plans-design.md`. Handoff (deck table with paired
numbers): `audit/phase4/plans-handoff.md`. CI script: `audit/phase4/pair.sh`.

## What was built
- `data/plans/<deck-key>.toml`, one per deck; `sim-core/build.rs` embeds the folder, so a new deck needs a file, no
  Rust. `ai/plan_data.rs` loads them (toml + serde, unknown fields refused; fails with the file and reason on a key
  mismatch, an unknown combo or condition, a priority rule that selects nothing, bad knob values).
- `tests/plans.rs`: every card a plan names is in its deck (`data/decks.json` or a retired deck's import list);
  every line group has a card in the deck; a cited Spellbook variant's cards are all in the line's pieces; each
  refusal is tested.
- Schema: `deck`, `retired`, `default = "hand" | "plan"`, `engines`; static knobs moved verbatim (`[style]`,
  `[commander]`, `[knobs]`, `[key_cards]`); `[[line]]` (catalogue combo or piece groups, optional Spellbook id,
  kind, ordered, when); `[[prio]]` rules (selectors, conditions, value); `[[hold]]`; `[mulligan]`.
- Derived tutor priority: the mana a line needs when it is complete except for mana; then lines from closest to
  furthest (missing groups, +1 when its `when` fails); then engines.
- Switch: `AI_PLAN_DATA=1` all plans, `=0` all hand code, or a comma-separated list of deck keys; unset, each file's
  `default` decides.

## Stage A: results identical
7 plan files hold the static knobs. Old binary vs branch with `AI_PLAN_DATA=1`, 300 games: byte-identical on t1,
t2, t4, t5 (rc=0).

## Stage B: paired results (2000 games per arm, seeds 500000-501999, rc=0; plan arm vs hand arm)
| Deck | Tier | Plan | Hand | Delta [95% CI] | Outcome |
|---|---|---|---|---|---|
| chulane | t4 | 15.7% | 13.2% | +2.5 [+0.1, +4.9] | plan better: hand code deleted |
| yawgmoth | t5 | 15.7% | 17.6% | -1.9 [-4.5, +0.7] | level by the rule: hand code deleted (**JD to confirm**) |
| kaalia | t2 | 26.8% | 26.8% | byte-identical | hand code deleted |
| light-paws | t1 | knobs only | | identical | hand entry deleted |
| yuriko | t4 | 18.1% | 20.9% | -2.8 [-5.5, -0.1] | plan worse: hand code kept (its derived wish list loses) |
| gaa, heliod | | retired from the pool | | no pair possible | knobs-only plans, hand code kept |

Other decks in each tier move by less than their CIs. CIs are Wald intervals with unpaired variance (conservative for
same seeds). Coordinator check: Chulane's arms in the sim outputs are 15.7% and 13.2% of the same 1622 games.

## Deleted
`chulane_prio`, `chulane_wish`, `BOUNCERS`, `board`; `yawg_prio`, `yawg_wish`, `UNDYING`, `undying_count`;
`t2::kaalia_prio` and its wrapper; the CONFIG entries for chulane, yawgmoth, kaalia, light-paws. CONFIG holds only
yuriko, gaa and heliod.

## Can't be expressed yet
- A group that counts only on the battlefield; engines ranked ahead of a line's pieces; a value depending on the rest
  of the hand; "N cards of a group"; the cEDH slots (fast-mana keep rule, hold-up utility, go-now gates); colour
  fixing in mulligans and mana up for holds (in the schema, not read yet).

## Also found
The old hand wish lists named cards no longer in the lists (Draco, Enter the Infinite, Avenger of Zendikar,
Consecrated Sphinx, Kor Skyfisher, Man-o'-War). The plan test refuses that drift.

## Checks
- `cargo test --release --no-fail-fast`: 3671 passed, 0 failed (rerun by the coordinator). Build 0 warnings.
  `cardgen validate` passes. `all_decks adaptive 3` and `lookahead 1` rc=0.
- With no env set, the head binary matches the plan arms byte for byte (t4, t5) and the old binary (t1).
