# Handoff: model-gaps-a3
Done: items 1-6 (see report-model-gaps-a3.md). Tests tests/model_gaps_a3.rs (20). ~190 existing test files re-run: pass.
Left: Marvin's NEEDS JD list (Siege-Gang, Syr Konrad mill, Face-Breaker, Tinybones, Ezuri overrun have no ability code; person uses
AI choices for borrowed hand-written abilities), the Hexing Squelcher ward audit (with JD), the Godo-line equip plan step (no stack item).
Gotchas: token creature lands are in lands AND perms; remove_land(g,p,l,Some(Zone)) is the only way a land leaves;
hand-written activated abilities go through stack::ability_run_w (effect closure, no cost inside).
