# Handoff: veil-oracle

Branch veil-oracle (worktree .claude/worktrees/veil-oracle). Brief: rust/agents/veil-oracle.md.

## State
- Code done and building: Veil of Summer as its Oracle text (state, AI response, resolve, counter paths, hexproof from
  blue/black for permanents and the player, person path).
- New tests: crates/sim-core/tests/veil_oracle.rs (9 tests, pass).
- To do if resumed: run the named test files (see report), `cardgen export` check, `cardgen audit` for Veil, report.

## Where things are
- State: `Player::veil_turn` (state.rs); `values::veil_shield`, `values::player_hexproof_from` (engine/values.rs).
- Card: `impls/rules.rs` `veil_respond` (AI answer, called from `stack::ai_respond` when a counterspell is aimed at q's
  spell) and `veil_resolve` (registered as the card's `resolve`; draws, sets `veil_turn`, `hx_U`/`hx_B` eot keywords on
  the permanents and lands there are as it resolves). The old cast-time `veil_response` is gone.
- Counter paths: `stack::counterable` (Veil-aware) / `counterable_but_veil` (Mindbreak Trap), `counter_window`,
  `resolve_counter_with` (does nothing under Veil unless Mindbreak Trap), `settle_stack`, `ai/search.rs playout_counter`.
- Hexproof: permanents use the existing Skrelv `hx_*` keywords (`tymna_cards::hexproof_from`); the player uses
  `player_hexproof_from` at the burn / face / "target player" sites that know the source's colours.
