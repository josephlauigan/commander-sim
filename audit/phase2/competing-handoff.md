# Phase 2 PR 5a, the competing and shadowed doubles: handoff

Brief: `rust/agents/phase2-competing.md`. Branch `phase2-competing` (based on `phase2-coverage`, origin/main merged
after #163 and #164), not pushed.

## Findings (2026-10-29)
- **Lands' compiled activated abilities never run.** Lands are not permanents in `perms`; `dsl::ability_options`
  and `dsl::activate` take permanents only, and DSL targets (`select`) are permanents, so "destroy target land" could
  not work either. The 15 competing lands' only live path was their `land_options` card code.
- **Dragon's Hoard**: its compiled draw has an `other` cost ("remove a gold counter"), which `pay_ability_cost`
  refuses: it never ran.
- **Weathered Wayfarer**: a real competing pair; the compiled search lacked the activation condition.
- **Demon of Fate's Design, Gingerbrute, Staff of Compleation, Staff of Domination**: the `options` entry and the
  compiled activations are different abilities (Staff of Domination: an infinite-mana shortcut).
- **Damn**: not a double; `damn_resolve` calls `dsl::resolve_spell` for the targeted mode.
- **The other 24 shadowed spells**: every compiled spell is partial or wrong; none could replace its hand resolve.
- **The 9 doubles from the splits pass** (coordinator, 2026-10-29): enters tags beside card code, and Doubling Season.

## Decisions
- `dsl = []`: the 15 lands, Dragon's Hoard, Weathered Wayfarer, Desperate Ritual (compiled added any colour).
- `paths_ok`: Demon, Gingerbrute, both Staffs, Damn, and 23 shadowed spells (compiled = the AI's value, as PR #161).
- Tags removed: Battle Angels (treas), Requiem Angel (tags override: no deck plays it), Sifter, Pawn, Springheart
  (tok), Karn Scion of Urza (tags override: JD's karn deck), Sowing Mycospawn (lr), Sakura-Tribe Elder (lr, in
  data/carddb.txt: hand-tagged cards ignore overrides). Doubling Season: impls/t3.rs etb hook deleted.
- `data/paths_known.txt` is empty (header only).

## Status
- [x] overrides, paths_known, export / validate, merge of origin/main
- [x] tests (`sim-core/tests/paths_competing.rs`, 21)
- [x] `audit/phase2/paths-after-competing.md`; `documents/realism.md`
- [x] cargo test 3616 passed, 0 failed; build --all-targets 0 warnings (after the merge, b1604152)
- [ ] sims: before running (scratchpad/phase2-competing/before, base = origin/phase2-coverage); after to run
