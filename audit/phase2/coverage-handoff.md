# Handoff: Phase 2, PR 3 (export refuses doubles; coverage per deck), branch `phase2-coverage`

Brief: `rust/agents/phase2-coverage.md`. Base: origin/main 2bdfead0.

## Done
- `cardgen export` guard (`cardgen::paths::guard`, called in `export::run` before any write). Refuses hard,
  shadowed, competing doubles and undeclared splits without `paths_ok`, unless listed in `data/paths_known.txt` with
  the same class (never a hard double). Uses `audit::engine_db_from` (the real sim-core registry, as `audit --paths`).
- The current data had 25 shadowed, 21 competing and 161 undeclared splits with no `paths_ok` (the brief assumed 0),
  so they are frozen in `data/paths_known.txt` (207 "class | card" lines). A ratchet: the test fails on a stale line.
  Coordinator agreed (2026-10-29): don't claim the gate passed; the list is what's left to clear.
- Tests: `cardgen/tests/paths.rs` `the_export_guard_passes_the_committed_data`,
  `the_export_guard_refuses_an_undeclared_double`.
- Coverage: `sim-cli/src/coverage.rs`, printed before results in pool mode (main.rs, before preflight), `--realism`
  (realism.rs), `--goldfish` (goldfish.rs). `--quiet` keeps it. No `--no-coverage` flag (not needed). `--fit`,
  `--trace`, `--cards` print none.
- Docs: `documents/realism.md` Phase 2 status, `--help`, sim-cli README.

## Checks (2026-10-29)
All pass: tests 3589/0/2, 0 warnings, export no change, validate OK, t3 realism byte-identical apart from the
coverage block, 12 deck runs rc=0. Details: the agent's report (the harness refused the report file). Nothing left to do.
