# Handoff: urdragon-approx-cards

Brief: `rust/agents/urdragon-approx-cards.md`. Worktree `.claude/worktrees/urdragon-approx-cards`.

## Done (in code, tests in `rust/crates/sim-core/tests/cards_urdragon_approx.rs`)
- Cursed Mirror: `enter_copy` replacement (`t0_stella::mirror_copy`), `CopyExc::revert_to`, `mirror_revert` from
  `engine/turn.rs step_start`; chained copy in `zones::enter` (Sculpting Steel / token copy of the Mirror).
- Astral Dragon: `is_creature` sees `exc.add_types` CREATURE; human target; legendary opponent permanent; copy value.
- Nicol Bolas: Arisen's `phys` = front card (graveyard as the Ravager); `dsl::use_loyalty` 0-loyalty uses `phys`.
- The Ur-Dragon: land cards eligible (`ur_pick`).
- Wrenn and Six: -7 emblem `wrenn6`, `t12_windgrace::emblem_retrace_options` hooked in `brain::hook_options`; +1 picks the best land.

## Next
- Done: tests pass except the known deck-rebuild failures. Report text was returned in the final message.
- Data: statuses/notes in `data/audit.toml` (Astral Dragon, The Ur-Dragon) and `data/card_overrides.toml` (Cursed Mirror,
  Nicol Bolas the Ravager, Wrenn and Six); `cardgen export`; commit generated JSON.
- Report in `rust/agents/report-urdragon-approx-cards.md`.
