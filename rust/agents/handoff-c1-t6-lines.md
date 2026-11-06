# Handoff: c1-t6-lines (2026-11-05)

Branch c1-t6-lines, everything committed. Brief: `rust/agents/c1-t6-lines.md`. Report: `audit/realism/c1-2026-11-06/REPORT.md`.

Done: trace line `[X passes (...)]` (t6stats.rs `hold_diag`, trace only); planned sacrifice tutors may eat the commander
(`brain::planned_fodder`) and the planner casts a fodder creature (`planner::fodder_cast`); Transmute Artifact is a planner
tutor (`planner::tutor_rule`, `rules2::transmute_fodder`, `tutor_extra`); Sisay's fetch and Kinnan + Monolith honour ability
locks; `cast_card` keeps the countering card's name; the speed measure counts only tutors that can find the missing piece
(`combos::tutor_finds`, `useful_tutors`). Tests: `rust/crates/sim-core/tests/c1_t6_lines.rs` (5 fail on the old code, 1 control).

Open (NEEDS JD / next session): Rograkh/Thrasios holds Finale + Snap with 8-11 mana when Eternal Witness is the missing
piece (g139, seed 900000, `--trace 139`), not diagnosed; Kraum/Tymna ready-hold rule (Esper Sentinel / Kraum) is the plan
file's choice, left; Lion's Eye Diamond and Moxen held at the end of turns 1-3 not examined; Whir of Invention not a planner
tutor.
