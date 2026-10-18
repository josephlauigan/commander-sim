# Handoff t6-e-combo3
Done: impls/t6_combos_extra.rs written (steps for Oracle + Kinnan/Basalt via ORACLE_STEPS / KINNAN_BASALT_STEPS referenced from
combos::BASE; 11 new combos: Thrasios/Mirage outlets, Breach (Oracle / table), Ad Nauseam, Horror, Welder+Engineer, 3 Godo
rebuilds, Basalt + copied Kinnan). t6_combos_godo helpers made pub(crate); Step derives Clone+Copy; base kinnan_basalt ready now
needs {U}{G} payable (else the outlet combos). Builds.
Next: tests/engine_t6_combo3.rs (one goldfish + one stopped per line), fix what fails, full suite, report.
