# Report: Phase 2, PR 5a: the competing, shadowed and last split doubles (branch `phase2-competing`)

Brief: `rust/agents/phase2-competing.md`. Handoff: `audit/phase2/competing-handoff.md`. Paths report:
`audit/phase2/paths-after-competing.md`. Commits: 9a3ffff1 (the 46 overrides), 969f2d38 (tests, paths report,
realism.md), b863a968 (the 9 split doubles from PR #164), 8b29198b (merge of origin/main), b1604152 (test fixes after
the merge), 2d548c1b and 515c00a4 (handoff, gate wording).

## Result
`data/paths_known.txt` is empty (header only). `cardgen audit --paths`: 0 hard, 0 shadowed, 0 competing, 0
undeclared split doubles; 209 cards (158 that decks play) carry a `paths_ok` reason. **The Phase 2 gate "no
double-modeled cards" passes.**

Main finding: lands are not in `perms`, so `dsl::ability_options` never offered a land's compiled activated ability,
and DSL targets can't be lands. For all 15 competing lands, the card code was the only path that ran. Phase 3's
permanent arena addresses this.

## Cards
| Card | Class | Side kept | Deleted / overridden | Test (`sim-core/tests/paths_competing.rs`) |
|---|---|---|---|---|
| 15 lands: Boiling Rock Prison, Buried Ruin, Castle Embereth, Castle Locthwain, Ghost Quarter, Horizon Canopy, Karn's Bastion, Mount Doom, Silent Clearing, Slayers' Stronghold, Sunbaked Canyon, Sunhome, Tectonic Edge, Vault of the Archangel, Waterlogged Grove | competing | card code (the only path that ran) | `dsl = []` | `the_competing_lands_keep_only_their_land_code`, Ghost Quarter, Tectonic Edge, the sacrifice lands, the Castles; existing tests for the rest |
| Dragon's Hoard | competing | card code | `dsl = []` (the counter cost can't be paid, so the compiled draw never ran) | `dragons_hoard_has_no_compiled_draw_and_draws_once_per_counter` |
| Weathered Wayfarer | competing (both live) | card code (has the "more lands" condition) | `dsl = []` | `weathered_wayfarer_searches_once_...` |
| Demon of Fate's Design, Gingerbrute, Staff of Compleation, Staff of Domination | competing, false positive | both | `paths_ok` (two abilities, one path each) | one test each |
| Damn | shadowed, false positive | both | `paths_ok` (the resolve runs the compiled spell unless overloaded) | `damn_overloaded_is_a_wrath_and_targeted_destroys_one` |
| Desperate Ritual | shadowed | hand resolve (RRR) | `dsl = []` (the compiled spell gave any colour) | `desperate_ritual_adds_exactly_three_red` |
| 23 shadowed spells (Drown in Ichor, Eldrazi Confluence, Get Lost, Kozilek's Command, Time Wipe and others) | shadowed | hand resolve | `paths_ok` naming the missing part; the compiled spell stays as the AI's value, as in PR #161 | `the_shadowed_spells_keep_their_resolve_and_a_compiled_value` |
| Requiem Angel, Sifter of Skulls, Pawn of Ulamog | real double (tok tag on enter + dies code) | card code | tok tag removed | `the_dies_token_makers_make_nothing_as_they_enter_and_one_token_per_death` |
| Springheart Nantuko | real double (tok + landfall) | card code | tok tag removed | `springheart_nantuko_makes_its_insect_on_landfall_only` |
| Battle Angels of Tyr | real double (treas tag on enter) | card code (combat damage) | treas tag removed | `battle_angels_of_tyr_makes_a_treasure_only_on_combat_damage` |
| Karn, Scion of Urza | real double (0/0 Construct on enter) | card code (-2) | `tags = "leg"` (tok/tokp dropped; leg kept) | `karn_scion_of_urza_enters_without_a_construct_...` |
| Doubling Season | real double (loyalty doubled twice) | `zones::enter` | the `impls/t3.rs` enter hook deleted | same test: 5 becomes 10, not 20 |
| Sakura-Tribe Elder | real double (lr on resolve + on sacrifice) | card code (sacrifice) | `lr=1 lrt` removed in `data/carddb.txt` (hand-tagged cards ignore overrides) | `sakura_tribe_elder_finds_one_basic_when_sacrificed_...` |
| Sowing Mycospawn | real double (cast trigger + lr) | card code | lr tag removed | `sowing_mycospawn_puts_one_land_onto_the_battlefield` |

## Follow-ups for the ability language
- Land activated abilities: the interpreter handles permanents only; it needs land sources and land targets (Phase 3 arena).
- `other` costs (remove a counter) can't be paid; no activation conditions; `add_mana` has no colour.
- The shadowed spells' missing parts are named in each `paths_ok` note.
- Ghost Quarter's card code puts the basic in tapped; the Oracle text says untapped.
- Sakura-Tribe Elder lost its `lr` AI ramp hint (JD's galadriel and seph): a non-executing ramp hint would restore it.

## Checks
- `cargo test --release --no-fail-fast`: 3616 passed, 0 failed (rerun by the coordinator).
- `cargo build --release --all-targets`: 0 warnings. `cardgen export --offline`: no data change; `cardgen validate` passes.

## Measurement
No before/after sims were run, by JD's choice (no sims until Phase 4). For the post-Phase-4 recalibration:
- **Rules changed:** t1 teysa (Sifter, Pawn), tatyova (STE); t2 windgrace, meren (STE); t3 atraxa (Doubling Season),
  aurelia (Battle Angels), korvold (STE); t5 yawgmoth (Sifter, Pawn); t6 rograkh-thrasios (Springheart, Mycospawn).
  JD's decks: galadriel and seph (STE), karn (Karn, Scion of Urza).
- **`dsl = []` only** (rules unchanged; card worth may shift slightly): t0 zurgo; t3 atraxa, aurelia; t4 krenko;
  t5 kinnan, urza, miirym; t6 kinnan-bonder, sisay. JD's alela, karn, seph.
