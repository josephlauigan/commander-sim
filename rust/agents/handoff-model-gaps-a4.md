# Handoff: model-gaps-a4
Done: item 1 AI + person code for the five abilities (impls/model_gaps_a4.rs, human/model_gaps_a4.rs, registered; t1's dead
Ezuri overrun and rules.rs ezuri_none removed); item 3 (StepKind::Equip in impls/combos.rs run_steps, used by the two Godo
Helm {5} equip steps).
Next: item 2 (Marvin person choices), item 4 (overrides: five cards Full, Marvin Full), tests/model_gaps_a4.rs, cardgen
export + audit --mine, report.
Gotchas: no Python (the hook blocks even a version check); cargo only under the systemd-run cap from rust/.
