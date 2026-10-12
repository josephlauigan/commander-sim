# Handoff: cloud-core

## Done (committed)
- cloud.rs: Cloud doubling (stack::queue_triggers -> cloud::doubles), attach/equip_cost/holder_bonus, all cards of the brief
  (hooks in `register`), AI: cloud_prio, cloud_protect, reenter_value. Overrides + verdicts in card_overrides.toml.
- tests/cards_cloud.rs: one+ test per card (40 pass).
- Shared-file edits (all small): engine/stack.rs (queue_triggers doubling), cardcode.rs (evasion_blocked),
  dsl/mod.rs (equip_options cost/bonus, pt base P/T, has_kw loses flying), engine/combat.rs (can_block flying,
  force_blocks, jocasta_attack), impls/common.rs (blink_value Cloud), impls/mine.rs (etb_value Cloud).

## Next
1. tests for cloud_protect / cloud_prio / reenter_value; smoke games; watch traces for the plan working.
2. Checks of LASTDECK-RULES (full cargo test, all-targets no warnings, all_decks adaptive 3 / lookahead 1, audit --mine, 48-game smoke).
3. report-cloud-core.md.

## Gotchas
- Never run Python (hook). Library top is `.last()`.
- Cho-Manno's Blessing / Shardmage's Rescue Auras are white-rest's; cloud_protect casts them with g.attach_to and only
  counts a save if the result is verified (untargetable / protected_from).
