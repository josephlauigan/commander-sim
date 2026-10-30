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
- [x] Stage A: byte-identical with AI_VALUES unset on t1, t2, t4, t5 (300 games, all rc=0): base `bin/commander-sim-base` vs
      `bin/commander-sim-headA` (a92b02a2), outputs `A/base-tN.txt`, `A/head-tN.txt` (`stageA.sh`).
- [x] Diagnostic, fixes (design, "Diagnostic findings"; 21861ab2). Overrides: Archon of Cruelty, Sheoldred, the
      Apocalypse.
- [ ] Stage B pairs: `stageB.sh` (binary `bin/commander-sim-B`, outputs `B/tN-old.txt`, `B/tN-A.txt`,
      `B/tN-B.txt`; it skips arms already done with rc=0, so rerun it to resume). Read a pair with
      `half.sh B/tN-old.txt B/tN-A.txt "Name;Name;Name"` (the display names on the share line).
- [ ] Stage C.

## Stage B log

- First run (binary `bin/commander-sim-B`, 21861ab2; derived arms kept in `B1/`): t1 half A -0.4 [-5.7, +4.9], half
  B +0.9 [-2.8, +4.6]; t2 half A -3.8 [-9.0, +1.4] (Sythis -6.0 [-9.4, -2.6], Meren +2.8), half B -3.0 [-7.0, +1.0]
  (Kaalia -3.1). Level by the CI, but the t2 point estimates were negative and Sythis' loss showed two general faults
  (removal with no target cast at nothing; mana sources under development creatures): fixed in 18b443f9, every
  derived arm rerun from t1 with `bin/commander-sim-B2`. The `AI_VALUES=0` arms (t1-old, t2-old, t3-old) are kept:
  the fixes are behind the switch (checked: B2 with `AI_VALUES=0` on t1, 300 games, = the Stage A head byte for
  byte). `chain2.sh` runs that check and then `stageB.sh`.
  Gotcha: `cargo build -p sim-cli -p sim-core --example X --test Y` builds only the named example and test, not the
  `commander-sim` binary; the first B2 copy was the old binary (its t1 arms repeated the first run byte for byte, so
  they were discarded). Build `-p sim-cli` on its own and `cmp` the copy against the previous one.
- Second run (B2, 18b443f9; identity check passed): t1 A -1.7 [-7.0, +3.6], B +0.3 [-3.4, +4.0]; t2 A -1.9
  [-7.1, +3.3] (Sythis -4.9 [-8.4, -1.4]), B -2.9 [-6.9, +1.1] (Kaalia -2.8 [-5.9, +0.3]); t3 A -1.4 [-6.6, +3.8],
  B +1.9 [-2.2, +6.0]. (Table below when all tiers are in.)
