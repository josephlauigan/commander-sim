# Handoff: Phase 4 PR 7, hand AI behind a switch (branch phase4-hand-off)

Brief: `rust/agents/phase4-hand-off.md`. Design and survey: `audit/phase4/hand-off-design.md`.

## State

- [x] Survey and design (`hand-off-design.md`).
- [ ] Switch `AI_HAND` (`ai::hand_ai_key`, `hand_ai`, `hand_key`, `force_hand`), every deck-AI site reads it.
- [ ] Plan files for the switched decks without one.
- [ ] Tests: hand-AI tests forced on; one switch test per deck.
- [ ] Checks: cargo test, build 0 warnings, cardgen validate.
