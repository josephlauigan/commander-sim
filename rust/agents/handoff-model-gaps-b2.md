# Handoff: model-gaps-b2 (Rings for every person ability)

## Done
- All 67 `human/` call sites converted except two listed in the report (Basalt loop shortcut, Soul-Guide Lantern enter
  trigger); the `impls/` functions the person's menus reach are converted too (list in the report).
- `tests/practice_rings_probe.rs` (sweep of the 11 decks, two boards, hand/graveyard ways) passes: 257 offered, 11 exceptions.
- `tests/play_rings.rs`: 9 new spot tests. Regression files (32 + 53 binaries) pass; `cargo build --release --all-targets`
  has no warnings.
- Report: `rust/agents/report-model-gaps-b2.md` (classification table, NEEDS JD).

## Next
- Nothing for the agent; JD decides the NEEDS JD list.

## Gotchas
- `ability_run` for an AI seat is `ability_window` plus the effect (copy worth 0.0, copies always 0).
- A copy of a walker ability re-runs the effect's own choice function (`Ready::Go` is `Fn`).
