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
- Diagnostic: `AI_VALUES=0 cargo run --release -p sim-core --example values_diag [all]`. Tests: `tests/values.rs`.

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
- [x] Stage B pairs: `stageB.sh` (binary `bin/commander-sim-B`, outputs `B/tN-old.txt`, `B/tN-A.txt`,
      `B/tN-B.txt`; it skips arms already done with rc=0, so rerun it to resume). Read a pair with
      `half.sh B/tN-old.txt B/tN-A.txt "Name;Name;Name"` (the display names on the share line).
- [x] Stage C (below).

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

## Stage B results (binary B2 = 18b443f9; 2000 games per arm, seeds 500000-501999, loose; all 15 arms rc=0)

The half's summed share of the decisive games its decks sat in, `AI_VALUES=0` against `AI_VALUES=<half>` (one shared
old arm per tier); Wald 95% CIs, the decks' variances summed (conservative). Level = the CI includes 0.

| tier | half | old (95% CI) | derived (95% CI) | derived - old (95% CI) | verdict |
|---|---|---|---|---|---|
| t1 | A (Isshin, Light-Paws, Teysa) | 90.6% [86.8, 94.4] | 88.9% [85.2, 92.6] | -1.7 [-7.0, +3.6] | level |
| t1 | B (Lathril, Tatyova) | 34.5% [31.9, 37.1] | 34.8% [32.2, 37.4] | +0.3 [-3.4, +4.0] | level |
| t2 | A (Brago, Lord Windgrace, Sythis) | 82.7% [79.0, 86.4] | 80.8% [77.1, 84.5] | -1.9 [-7.1, +3.3] | level |
| t2 | B (Kaalia, Meren) | 42.3% [39.5, 45.1] | 39.4% [36.6, 42.2] | -2.9 [-6.9, +1.1] | level |
| t3 | A (Atraxa, Korvold, Tergrid) | 79.3% [75.6, 83.0] | 77.9% [74.2, 81.6] | -1.4 [-6.6, +3.8] | level |
| t3 | B (Aurelia, Marwyn) | 45.6% [42.7, 48.5] | 47.5% [44.5, 50.5] | +1.9 [-2.2, +6.0] | level |
| t4 | A (Chulane, Krenko, Yuriko) | 62.0% [58.6, 65.4] | 64.0% [60.6, 67.4] | +2.0 [-2.8, +6.8] | level |
| t4 | B (Edgar Markov, Prosper) | 63.3% [60.3, 66.3] | 60.4% [57.4, 63.4] | -2.9 [-7.2, +1.4] | level |
| t5 | A (Kinnan, Urza, Zur) | 76.7% [73.0, 80.4] | 76.6% [73.0, 80.2] | -0.1 [-5.3, +5.1] | level |
| t5 | B (Miirym, Yawgmoth) | 48.2% [45.3, 51.1] | 51.5% [48.5, 54.5] | +3.3 [-0.9, +7.5] | level |

Deck level (`pair.sh`): one deck's own CI excludes 0, Sythis on t2 (half A) -4.9 [-8.4, -1.4]; Kaalia (t2 B) -2.8
[-5.9, +0.3] and Edgar Markov (t4 B) -2.9 [-6.4, +0.6] lean down. Sythis' worth comes from her own trigger (a card
for each enchantment cast): the derived value reads each cheap enchantment alone (an unread engine, 29-43, against the
old 55 "hooked"), so the deck casts fewer of them early. What would close it: a commander-synergy term (the cards the
commander's own triggers key on: enchantment, creature type, spell kind), read from the commander's abilities.

## Stage C

All five tiers level in both halves: derived values are the default for every generic-policy deck (8adc9209). The
old tables are **not deleted**: JD's decks (`kefka`, `sheoldred`, `cloud`, `karn`, `yshtola`, `alela`, `garland`,
`sephfs`, `titania`, `tymna_partners`) and every Tier 6 deck call `pool::generic_prio` as their fallback, and
`card_worth`, `tutor_value`, `pval` and the other `bomb` reads serve them too; they go when those decks move (later
Phase 4 points). `AI_VALUES=0` keeps the old reading for the generic decks (a measurement arm).

## Final checks (head)

- `cargo build --release --all-targets -j 22`: 0 warnings. `cargo test --release --no-fail-fast -j 22`: 3685 passed,
  0 failed. `cardgen validate`: all decks passed. `all_decks adaptive 3` and `lookahead 1`: rc=0.
- Defaults (no env, 300 games) = the derived arm (B2, `AI_VALUES=1`) byte for byte on t1, t2, t3, t4, t5, all rc=0
  (`F/head-tN.txt` vs `F/arm-tN.txt`).
- Diagnostic (`AI_VALUES=0`): cast-priority rank correlation 0.74 / 0.73 / 0.75 / 0.78 / 0.85 (t1-t5).

## Next (not in this PR)

- Sythis (-4.9 on t2): a commander-synergy term from the commander's own triggers.
- Move JD's decks and Tier 6 off `generic_prio`'s table, then delete the table, the held-interaction constants in
  `card_worth`, and the `bomb` reads (`spell_importance` needs a per-reader counter decision first).
