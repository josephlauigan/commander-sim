# Handoff: ai-rules (Phase C2)

Branch ai-rules, worktree .claude/worktrees/ai-rules. Brief: rust/agents/ai-rules.md. STATUS: finished, everything committed.

Items 1-8 done (see the report). Tests: tests/ai_rules_c2.rs (27). Existing tests of the touched cards, practice_opponent_*,
practice_*_probe, ai, ai_flow, generic_paths: all pass (three existing tests were updated for the Oracle piles/keeps:
cards_t3 liliana_of_the_veil_ultimate_keeps_the_better_pile, model_gaps_a3 mass_sacrifice_of_lands..., practice_opponent_fixes
liliana_of_the_veil_the_person_picks_the_pile_they_sacrifice). `cargo build --release --all-targets`: no warnings.
