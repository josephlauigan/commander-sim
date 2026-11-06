# Handoff: model-gaps-b2 (Rings for every person ability)

## Done
- Probe `rust/crates/sim-core/tests/practice_rings_probe.rs` (sweep over karn/thrasios/sauron: `sweep_report` prints
  every non-offered ability; `PROBE_SHOW="karn: Staff of Domination #1"` prints a case's questions and log).
- stack.rs: `ability_copies` logs "the copy of X resolves" (the probe reads it).
- First sweep: 22(33) offered, 46 NOT OFFERED, 18 not exercised.

## Next
- convert human/ sites (list in the report table), then impls `*_go` paths the sweep names, then assert mode in the probe.

## Gotchas
- ability_run for an AI seat == ability_window + eff (copy_worth 0.0 means the AI never Cameras it).
