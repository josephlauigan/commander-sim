# Handoff t6-e-combo1
Feature: plan row 18 part 1: combo framework + Godo combos.
Done: combos::all() registry (BASE + t6_combos_godo::COMBOS), Step/StepKind (Spell, Trigger, Ability, Permanent, Engine),
run_steps, 9 Godo combos (impls/t6_combos_godo.rs), tests/engine_t6_combo1.rs (22 tests, passing).
Next: rustfmt my files, full cargo test --release, build --all-targets (no warnings), write report-t6-e-combo1.md.
Gotchas: combo_options gate lets step combos through when `ready` is true (Godo starts in cmd zone/library);
godo_wants COPIERS now also lists Jaxis and Sculpting Steel.
