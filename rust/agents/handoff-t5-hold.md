# Handoff: t5-hold (measure Tier 5 counter-mana holding)

Done, committed on branch t5-hold:
- `sim-core/src/holdstats.rs`: trace-only `hd|...` counters in `Player::stats` (key format in its header). Hooks: `engine/turn.rs`
  (`snapshot` after `mana_held` in main 2; `turn_start` before `flow_turn_start`), `engine/cast.rs` `on_cast`, `engine/mana.rs`
  `spend_unit`, `t6stats::counter_hit`, `engine/removal.rs` (`apply_removal_rest`, the "protects" branch). `Player::hw` is the
  snapshot being followed; `Settings::hold_trace` (default true) turns it off for the identity test.
- `sim-cli/src/holdreport.rs` + call in `realism.rs::run`: the "Held mana at the end of the turn" blocks, after the flow tables.
- `tests/t5_hold.rs`: identical game (result, log, other counters) with the counters off, and a scripted counter hit.
- `rust/agents/t5-hold-queue.sh`: the coordinator's run (t5 then t3, 2000 games, seed 900000).
Todo: nothing but the coordinator's queue run and reading the two output files.
