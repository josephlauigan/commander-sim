# Handoff: c1b-engines (2026-11-06)

Branch c1b-engines (from c1-t6-lines 8a7650e8), everything committed. Brief: `rust/agents/c1b-engines.md`. The report file
could not be written by the agent (the harness refuses report files): the full report is in the agent's reply; the
coordinator saves it as `audit/realism/c1b-2026-11-06/REPORT.md`.

Done: trace-only `brain::stop_diag`; `brain::engine_rule` + `ENGINE_MARGIN` (Tier 6 draw engines cast before "stop", early
turns 1-4 without the reserve penalty, archenemy `combo_near` only); Ad Nauseam hold needs room (`plans::adnaus_room`);
Tier 6 wheel refill (`partials::t6_refill`, wheel_prio, windfall_prio, Timetwister in `pool::generic_prio_base`).
Tests: `rust/crates/sim-core/tests/c1b_engines.rs` (11; 9 fail on the old rules). 74 test binaries / 1211 tests pass.
Open: see NEEDS JD in the reply (Sheoldred's Ad Nauseam hold, `cmdr_combo` for other casts, decklist cantrips).
