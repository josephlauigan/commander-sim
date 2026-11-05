# Handoff: model-gaps-a3
Done: item 1 (remove_land(g,p,l,Some(Zone)) is the one place a land leaves; token lands go nowhere), item 2 (Astral Dragon
land + Aura copies; `turn::enter_token_land_copy`; Enter.token), item 3 (engine/ward.rs `pay_ward`; wired in apply_removal_rest,
control::steal, dsl tap/counters/pump/grant/phase_out, card-code tap/-X sites). Tests in tests/model_gaps_a3.rs (10 pass).
Next: item 4 Mystic Forge counter from the top, 5 Peter Parker's Camera (cost/effect split of hand-written abilities), 6 Marvin.
Gotchas: token creature lands are listed in lands AND perms (remove_land, zones::leave, control keep both in step).
