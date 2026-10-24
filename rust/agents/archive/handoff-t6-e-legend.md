# handoff t6-e-legend
Plan: (a) legend rule SBA in engine/life.rs check_state + test (Godo Twinflame); (b) Blood Moon layer effect
(nonbasic lands lose abilities: Tomb damage, land_options, land_etb/saga, manlands) + Damping Sphere colourless.
Status: started, nothing done yet.
2026-10-17: engine done (legend SBA life.rs sba_legend; mana.rs moon_land/moon_card + gates in lands.rs, cardcode.rs, turn.rs, zones.rs,
common.rs saga_step, galadriel.rs; new Event LandColorless for Damping Sphere). Tests: tests/engine_t6_legend.rs (13 pass).
Next: full cargo test, cardgen export, engine-needs notes, report, commit.
