# Handoff t6-e-sba
Plan: (a) toughness<=0 check in engine/life.rs check_state (cheap prefilter), remove Eldrazi Confluence die(..,"sba");
(b) decked flag already set by zones::draw on empty library and eliminated at next check_state: verify + test.
Done: read code. Next: implement (a), tests in tests/sba.rs, full tests, report.
implemented (a); tests/sba.rs written; running tests
targeted tests pass; running full suite
