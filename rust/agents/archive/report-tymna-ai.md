# Report: tymna-ai (2026-10-13)

## What was done
- **Attack spread** (`tymna_partners::spread`, called from `engine/combat.rs` after the split, for both the normal and the look-ahead
  "all" plan). Only for keys `thrasios`/`kraum`, and only while Tymna, Reconnaissance Mission or The Aetherspark is on the battlefield.
  Every living, unshielded, untaxed opponent without a connecting attacker gets the cheapest (lowest `pval`) attacker that cannot be blocked
  by that player's untapped creatures (`can_block`; menace with one blocker counts as connecting). Not moved: the only attacker that hits its
  own opponent, anything in a group whose unblockable damage is lethal (kept lethal), held-back commanders and Tymna.
- **Priorities** (`thrasios_prio`/`kraum_prio` share `partner_prio`): Tymna 91, Thrasios 86, Kraum 84 (minus 3 per commander tax); one-mana ramp 89,
  two-mana ramp 87 (83 after turn 3), others 82 (turns <= 5); an evasive creature (flier/menace/shadow, Silhana, Exit Specialist, Spectral Lynx,
  Nightveil Predator) of cmc <= 4 is 78-80 while fewer evasive creatures are out than living opponents, else at least 66; Bastion Protector 74 once
  Tymna and the other commander are out, else 40. Everything else is `generic_prio`. Protection spells stay with the existing protect-response
  dispatch (`ai/decks.rs` -> `tymna_cards::protect`); not changed. The AI casts creatures in the first main phase, so Kraum (haste) already attacks.
- **Measurement**: new stats (`ty_*`) and an analysis section "Partner decks (Tymna)" in `sim-cli/src/report.rs`.

## Numbers (96 games, loose profile, look-ahead, seed 7, same seeds before and after)
| deck / pool | Tymna cards/game | opps hit/Tymna turn | win rate | Tymna lands | Thrasios or Kraum lands | Thrasios acts/game |
|---|---|---|---|---|---|---|
| thrasios t1 before | 1.77 | 0.59 | 9.4% | T4.76 | T4.31 | 0.51 |
| thrasios t1 after  | 2.10 | 0.81 | 11.5% | T5.32 | T4.05 | 0.75 |
| thrasios t3 before | 1.62 | 0.51 | 14.6% | T4.80 | T4.73 | 0.49 |
| thrasios t3 after  | 2.03 | 0.63 | 11.5% | T5.19 | T4.35 | 0.45 |
| kraum t1 before | 1.64 | 0.62 | 13.5% | T5.23 | T6.81 | - |
| kraum t1 after  | 1.77 | 0.77 | 14.6% | T5.09 | T6.71 | - |
| kraum t3 before | 1.31 | 0.53 | 15.6% | T5.46 | T6.66 | - |
| kraum t3 after  | 1.67 | 0.65 | 12.5% | T4.78 | T6.83 | - |

Reading: cards drawn and opponents hit rise in every cell (+10 to +28% cards, +0.12 to +0.22 hits per Tymna turn). Win rates move by
-3 to +2 points with a 95% interval of about +-7 at 96 games, so no win-rate change can be claimed; larger runs are needed.
Opponents hit per Tymna turn is still only 0.6-0.8 because Tymna usually has one or two evasive creatures when she is out and is removed often
(0.34 times per game). "Tymna lands" is the first time she enters (recasts after removal are counted in `ty_re_*`, not in the average).

## Checks
- `cargo test --release`: 1585 passed, 0 failed (7 new in `tests/tymna_ai.rs`). `cargo build --release --all-targets`: no warnings.
- `all_decks adaptive 3`: 216 games, no panic; `all_decks lookahead 1`: 72 games, no panic.
- Same-seed check (seed 7, 48 games, loose, look-ahead): `sauron --pool t3` and `cloud --pool t5` result lines identical before and after.

## Shared-file changes
- `engine/combat.rs`: two lines (`pool` clone, call to `tymna_partners::spread`); a no-op for any other key.
- `sim-cli/src/report.rs`: `Analysis::ty` map + the "Partner decks (Tymna)" section (printed only when `ty_` stats exist).
- `impls/legends.rs`: two `ty_` stat lines in Tymna's main-2 trigger. `impls/tymna_cards.rs`: one stat line in `thrasios_go` and the call
  `tymna_partners::register` (an ETB hook on Tymna, Thrasios and Kraum that only records stats).

## Open
- Win rates are within noise; a 960/cell run would show whether the plan helps. Tymna's tuning headroom: more evasive creatures per hand, protecting her.
