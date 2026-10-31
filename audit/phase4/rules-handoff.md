# Handoff: Phase 4 PR 6, rule checks in the policy

Brief: `rust/agents/phase4-rules.md`. Design: `audit/phase4/rules-design.md`. Branch `phase4-rules`. No sims (JD,
2026-10-31): cargo tests only; cargo `-j 8` (three agents in parallel).

## State

- [x] Survey (design section 1; 36da5f57).
- [x] `ai/rules.rs`: switch `AI_RULES` (+ `AI_RULES_DECKS` for a half), the three guards, hooks (c886c9b1).
- [x] Tests `rust/crates/sim-core/tests/rules_guards.rs` (14, all pass).
- [x] Design, switch, tests, recalibration list (design sections 2-5).
- [x] Final checks at c886c9b1: `cargo build --release --all-targets -j 8` 0 warnings; `cargo test --release
      --no-fail-fast -j 8` 3748 passed, 0 failed; `cardgen validate` all decks passed. No sims (JD).

## Hooks (all one-liners, `--- phase4-rules`)

`ai/brain.rs` (2: end of `main_options`, `end_of_turn_window`), `engine/stack.rs::pick_counter`, `ai/mod.rs`
(`cleanup_discard`), `ai/topdeck.rs::scry`, `engine/cast.rs` (`discard_worst`, `additional_cost` discard),
`impls/kefka.rs::discard_n`, tutor marks in `engine/tutors.rs` (3), `dsl/mod.rs`, `impls/{t1,t2,t6_blue,rules2}.rs`;
additive: `state.rs` (`Player::tutored`), `impls/combos.rs` (`deck_lines_with`).

## Gotchas

- Card ids are per name: three Islands in hand are one id three times (a test that diffs hands needs distinct cards).
- Next: the recalibration after Phase 4 runs design section 5; nothing else is pending here.
