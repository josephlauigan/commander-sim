# Handoff: c3-casual-speed

State (committed on branch c3-casual-speed):
- F1 combat.rs: `redirect_dropped`, `sendable_power`; attackers a tax/cap keeps off their defender attack another opponent;
  `brain::choose_defender` uses `sendable_power`.
- F2 poison: `combat::poison_hit`, `poison_share`; `ai_blocks` (poison chump), `ai::chump_prob_poison`,
  `brain::attack_response` danger, `Situation::new` danger, `values::pval` (infect/toxic worth), `choose_defender` kill check;
  infect damage is now combat damage dealt (lifelink, triggers, commander damage) in `resolve_blocks`.
- F3 `brain::COMMANDER_FIRST` (+1.5 on a first commander cast, outside decks except Tier 6).
- tests/c3_casual_speed.rs (see its header). Diagnostics patch (trace lines "C3DIAG": mana left at end step, able vs
  declared attackers): scratchpad c3diag.patch, not committed.

Todo: run the listed test files, `cargo build --release --all-targets` warnings, REPORT.md in audit/realism/c3-2026-11-06/.
