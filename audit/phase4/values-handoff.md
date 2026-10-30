# Handoff: Phase 4 PR 3, values from abilities

Brief: `rust/agents/phase4-values.md`. Design: `audit/phase4/values-design.md`. Branch `phase4-values` (on
`phase4-cedh`). Scratch: `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase4-values/`
(`sim.sh BIN POOL GAMES OUT AI_VALUES|-`, `half.sh OLD NEW "Name;Name"`: a half's summed share both ways with CIs;
`bin/commander-sim-base` = the base binary, built at 42375806 before any change).

## Code

- `rust/crates/sim-core/src/ai/values.rs`: profile (from tags + compiled abilities + card code), `worth`, `cast_prio`,
  `card_worth`, `threat` (the derived bomb), the switch `on` (`AI_VALUES`: unset = `DEFAULT`, `0` none, `1` every
  generic-policy deck, `a,b` those keys), `pval_for` / `read_as` (pval by a reader), the override table
  (`data/values_overrides.toml`).
- Reads switched (each identical when the reader is not on derived values): `pool::generic_prio` (-> `derived_prio`),
  `tutors::card_worth`, `decks::tutor_pick` / `tutor_value` / `gy_worth` / `rean_options`, `brain::card_utility`
  (counter-risk bomb), `brain::rem_fodder`, `brain::removal_options` / `cast_removal` / `spare_creature` (pval by the
  caster), `engine::values::pval` (bomb via `values::pval_bomb`), `cast::sac_fodder`, `zones::edict`,
  `cast::discard_worst`, `turn::bottom_pick`.
- Diagnostic: `cargo run --release -p sim-core --example values_diag [all]`. Tests: `tests/values.rs`.

## Halves (fixed before any Stage B run: alternate decks in file order)

| tier | half A | half B |
|---|---|---|
| t1 | isshin-mardu-attack-triggers, light-paws-aura-voltron, teysa-orzhov-aristocrats | lathril-golgari-elves, tatyova-simic-landfall |
| t2 | brago-azorius-blink-control, lord-windgrace-jund-lands, sythis-selesnya-enchantress | kaalia-mardu-creature-cheat, meren-golgari-recursion |
| t3 | atraxa-infect, korvold-jund-sacrifice, tergrid-mono-black-disruption | aurelia-boros-extra-combats, marwyn-mono-green-elves |
| t4 | chulane-bant-value-combo, krenko-mono-red-goblins, yuriko-dimir-ninjas | edgar-markov-mardu-vampires, prosper-rakdos-exile-treasure |
| t5 | kinnan-simic-mana-combo, urza-mono-blue-artifacts, zur-esper-enchantment-control | miirym-temur-dragons, yawgmoth-mono-black-aristocrats |

## State

- [x] Design (2bfa0390).
- [ ] Stage A: identical with AI_VALUES unset on t1, t2, t4, t5 (300 games).
- [ ] Diagnostic, fixes.
- [ ] Stage B pairs.
- [ ] Stage C.
