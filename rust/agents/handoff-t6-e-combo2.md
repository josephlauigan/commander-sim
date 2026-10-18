# Handoff t6-e-combo2
Feature: plan row 18 part 2: Dack/Sisay/Rograkh-Thrasios combos in impls/t6_combos_t6.rs.
Done: nothing yet (reading framework).
Next: Dack, Sisay, Rograkh combos + tests/engine_t6_combo2.rs.
Done 2: impls/t6_combos_t6.rs written (6 combos: Dack x2, DDR, Flash Unicorn, Rograkh Plan A, Witness+Snap), registered (mod.rs + combos::all), builds.
Next: tests/engine_t6_combo2.rs, FINDINGS/engine-needs notes, full test, report.
Gotchas: Preston cap and Ioreth cap KEPT (real play reaches loops without the combo); Dack 'running' combo keys off the cap being hit this turn. Two-banisher variant not built.
Done 3: tests/engine_t6_combo2.rs (26 pass), report written. Next: full cargo test result, commit (git add -A rust data), SubagentHandback.
