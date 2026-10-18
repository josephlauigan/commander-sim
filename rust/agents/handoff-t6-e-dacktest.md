# Handoff t6-e-dacktest
Feature: plan row 19: Dack first-flip shortcut combo (impls/t6_combos_t6.rs) + 35-pile table test + e2e + Monte Carlo (tests/engine_t6_dacktest.rs).
Design: flip_and_donate records a snapshot (pile mask, card left in library) in grudge keys; a QUIET combo reads it. Table fn pure (testable).
Done: read traces. Next: snapshot in t6_dack.rs, table + combo, tests.
Done: snapshot (t6_dack.rs), table+combo (t6_combos_t6.rs), tests/engine_t6_dacktest.rs 7 pass. Next: swords/class-a checks ok; full cargo test, commit, extra spec combo tests if time, report.
