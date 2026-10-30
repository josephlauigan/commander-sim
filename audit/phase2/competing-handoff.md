# Phase 2 PR 5a, the competing and shadowed doubles: handoff

Brief: `rust/agents/phase2-competing.md`. Branch `phase2-competing` (based on `phase2-coverage`), not pushed.

## Findings (2026-10-29)
- **Lands' compiled activated abilities never run.** Lands are not permanents in `perms`; `dsl::ability_options`
  and `dsl::activate` take permanents only, and DSL targets (`select`) are permanents, so "destroy target land" could
  not work either. The 15 competing lands' only live path was their `land_options` card code.
- **Dragon's Hoard**: its compiled draw has an `other` cost ("remove a gold counter"), which `pay_ability_cost`
  refuses: it never ran.
- **Weathered Wayfarer**: real competing pair; the compiled search lacked the activation condition.
- **Demon of Fate's Design, Gingerbrute, Staff of Compleation, Staff of Domination**: the `options` entry and the
  compiled activations are different abilities (or, for Staff of Domination, an infinite-mana shortcut).
- **Damn**: not a double; `damn_resolve` calls `dsl::resolve_spell` for the targeted mode.
- **The other 24 shadowed spells**: every compiled spell is partial or wrong; none could replace its hand resolve.

## Decisions
- `dsl = []`: the 15 lands, Dragon's Hoard, Weathered Wayfarer, Desperate Ritual (compiled added any colour).
- `paths_ok`: Demon, Gingerbrute, both Staffs, Damn, and 23 shadowed spells (compiled = the AI's value, as PR #161).
- All 46 lines deleted from `data/paths_known.txt`.

## Status
- [x] overrides written, paths_known trimmed
- [ ] export / validate
- [ ] tests (`sim-core/tests/paths_competing.rs`)
- [ ] `audit/phase2/paths-after-competing.md`
- [ ] `documents/realism.md` Phase 2 paragraph
- [ ] cargo test / build 0 warnings
- [ ] sims before (running: scratchpad/phase2-competing/before) / after
