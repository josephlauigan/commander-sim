# Handoff: t0-zurgo

DONE. All 22 cards Full, tests and checks pass; see rust/agents/report-t0-zurgo.md.
Gotcha for anyone extending it: hook bodies run twice (a probe up to the first trigger_window, then for real), so
nothing may change state before the first trigger_window.
