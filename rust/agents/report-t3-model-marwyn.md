# Report: Tier 3 cards for marwyn-mono-green-elves

(Saved by the main session from the agent's final message on 2026-10-24: the harness refused the agent's own write of
this file, so this is the agent's summary, not its full report. Merged into trace-fixes as a907bd55.)

Branch `t3-model-marwyn`, commit e2a65345. The audit now shows 0 Partial and 0 Unmodeled cards in the deck (it was 3
Unmodeled and 14 Partial). `cargo test --release`: 3041 passed, 0 failed, including 18 new tests in
`rust/crates/sim-core/tests/cards_t3_marwyn.rs`. `cargo build --release --all-targets` has no warnings.

### Card status
- Ability language (`dsl_parse.rs`, `dsl/mod.rs`): Emerald Medallion, Dwynen and Immaculate Magistrate now compile from
  their Oracle text. New patterns: colour cost reductions, lords that give a bonus per counter, counts of Elves (or
  attacking Elves) you control.
- Full (hand-written in `impls/t3_marwyn.rs`): Joraga Warcaller (multikicker), Galadhrim Brigade (squad), Genesis Wave,
  Dionus, Eladamri, Realmwalker, Yeva.
- Approximate, and what is left out:
  - Elvish Guidance: its {G} per Elf is its own mana source, as Wild Growth; the enchanted land isn't tracked.
  - Galadhrim Ambush: only cast when attacked, not as a trick on your own attack.
  - Legolas's Quick Reflexes: the "becomes tapped: deal damage" part is added; the AI uses it on a tapped mana Elf
    that can kill a creature, or as hexproof, but not as a combat trick.
  - Three Tree City: works like Nykthos (mana net of its {2}). It was wrongly tagged to tap for any colour; it now taps
    for {C} unless boosted.
  - Wirewood Lodge: only used to untap a big mana Elf, not an attacker.
  - Yavimaya: every land counts as a Forest for forestwalk; the extra {T}: {G} it gives other lands isn't added.
  - Umbral Mantle: a combo with Ezuri and a creature making 4+ mana, which wins before combat; its plain +2/+2 is not
    used.

### Shared files changed (small, additive)
`engine/combat.rs` now always records the turn's attackers; tap hooks in `engine/mana.rs`, `engine/combat.rs` and
`dsl/mod.rs`; other edits in `engine/cast.rs` (Yeva's flash), `cardcode.rs` (Galadhrim Ambush's fog), `ai/brain.rs`
(Ambush response), `ai/pool.rs` (Legolas added as a protection spell), `combos.rs`, `impls/mod.rs`,
`t6_legends2.rs`, `t3.rs` (Nissa's Forests through `is_forest`), `card_overrides.toml` and `audit.toml`. `cards.json`
was regenerated; only this deck's compiled abilities changed.

### Tier 3 scorecard (loose, adaptive, 4000 games, seeds 500000-503999)
"Before" is the parent commit, built from an archive.

| | seat 1/2/3/4 | median win round | combo/alt | damage | commander dmg | targets passed |
|---|---|---|---|---|---|---|
| before | 27.0 / 25.2 / 25.2 / 22.6 | 11 | 2.6% | 87.2% | 10.2% | 1/9 |
| after | 28.4 / 26.5 / 23.3 / 21.8 | 11 | 8.1% | 82.5% | 9.4% | 1/9 |
| target | 31.3 / 27.8 / 21.8 / 19.1 | 8 | 20% | 62.2% | 17.4% | |

Marwyn's share of the decisive games it played rose from 9.2% to 17.1%.

### For the main session
Llanowar Elves was a 0/1 with a stray "human" tag from `data/carddb.txt` (fixed by the main session on trace-fixes,
along with Blood Artist's).
