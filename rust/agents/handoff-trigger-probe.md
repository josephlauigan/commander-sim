# Handoff: trigger probe
Done: practice_trigger_probe.rs passes; lists filled (3 TO_FIX, 6 FALSE_ALARMS, 8 UNPROBED); report-trigger-probe.md written.
Next (coordinator): decide on the 3 TO_FIX fixes; review the PROBE_ACTIVATE=1 leads and, if wanted, make that pass default.
Gotchas: cargo test --release -q -p sim-core --test practice_trigger_probe -- --nocapture (under the memory cap); PROBE_DUMP / PROBE_ALL="<card>" show the requests.
