# Handoff: t6-ai (worktree .claude/worktrees/t6-ai, branch t6-ai)

Brief: rust/agents/t6-ai.md. Report: audit/realism/t6-ai-2026-11-06/REPORT.md.

## Done (see git log)
1. Finale for Witness: cause = the Rograkh/Thrasios plan file holds Finale and GSZ at prio 0, and `planner::plays`
   skipped any card the plan slot holds (`castable_at`). Fix: the X-tutor loop ignores the hold (`castable_with(.., true)`);
   line B (`t6_combos_t6::plan_b`) counts Finale in the graveyard as a finisher and a new step returns it
   (`witness_returns_finale`).
2. Whir of Invention: tagged `xtutor` (data/card_overrides.toml, cards.json re-exported), `xtutor_rule` knows it
   (artifacts), `planner::pay_x` subtracts the artifacts improvise taps, `rules2::improvise_arts`, `whir` takes the
   planned card.
4. `brain.rs`: `hold_of` and `main_options` read `combo_near` alone for Tier 6.
5. `brain.rs` `hold_value`: the Ad Nauseam room test also for key `sheoldred`.
3. Investigation only, in the report.

## Left
- run the test list from the brief, report.
