# Handoff: Phase 4 PR 7, hand AI behind a switch (branch phase4-hand-off)

Brief: `rust/agents/phase4-hand-off.md`. Design and survey: `audit/phase4/hand-off-design.md` (sites per deck, deck
AI vs card code, the recalibration list). No sims were run (JD, 2026-10-31): cargo tests only.

## State

- [x] Survey and design.
- [x] Switch `AI_HAND` (unset/0 = generic policy for every switchable deck, the default; 1 = all hand AI; a key list
      = those keep it). `ai::hand_ai_key` / `hand_ai` / `hand_key` (the key, or "" when off) in `ai/mod.rs`;
      `ai::force_hand(Some(b))` returns a guard (thread-local, restored on drop). `ai::HAND_DECKS` (14 switchable)
      and `ai::HAND_OFF_PLANS` (13 plan files written here, read only with the hand AI off; `plan_unread`).
- [x] Every deck-AI site reads `hand_key` (see the design); card-code sites keep `g.player(p).key`.
- [x] `values::generic_deck`: a JD deck with its hand AI off is a generic-policy deck (Tier 6 stays out).
- [x] `plans::effective` / `plays_plan` / `config`: the new files are always loaded, read only while the hand AI is
      off (AI_PLAN_DATA does not touch them).
- [x] Plan files: veyran, sauron, yshtola, galadriel, alela, kefka, sheoldred, cloud, karn, thrasios,
      brago-azorius-blink-control, aurelia-boros-extra-combats, dack-fayden-white-stax (each header says what it
      was translated from and what the schema can't express).
- [x] Tests: `tests/hand_off.rs` (one switch test per switchable deck + defaults + plan files present); 151 existing
      tests of hand AI now start with `let _hand = sim_core::ai::force_hand(Some(true));`.
- [x] Checks: `cargo test --release --no-fail-fast -j 8` 3750 passed, 0 failed; build 0 warnings; cardgen validate;
      plan test passes.

## Gotchas

- `force_hand` is thread-local: a test that plays games on worker threads would not see it (none of the forced tests
  does).
- The sauron Sword + Aggravated Assault extra-combat line (`engine/combat.rs`) and Aurelia's Sword/Assault code are
  card code (the only way those cards are played): not switched.
- Plan files approximate: the hand AI's board-reading rules (Army power, Kefka flipped, Dack's cost bar, Chalice's
  net, Alela's +priority bonus) have no schema; expect the off arm to be worse for some decks until their plans are
  improved. That is the recalibration's question.

## Next (after this PR)

- The recalibration's on/off paired tests (`AI_HAND=<key>` vs default) per the design's table; delete the hand AI
  where off is level or better.
