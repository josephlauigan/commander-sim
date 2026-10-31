# Handoff: Phase 4 PR 6, rule checks in the policy

Brief: `rust/agents/phase4-rules.md`. Design: `audit/phase4/rules-design.md`. Branch `phase4-rules`. No sims (JD,
2026-10-31): cargo tests only; cargo `-j 8` (three agents in parallel).

## State

- [x] Survey (design section 1).
- [ ] `ai/rules.rs`: switch `AI_RULES`, the three guards, hooks.
- [ ] Tests `tests/rules_guards.rs`.
- [ ] Recalibration list (design section 5).
