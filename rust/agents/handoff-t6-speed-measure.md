# Handoff t6-speed measure
Status: started 2026-10-19. Building release baseline; then add ready/reachable/go/win stats in t6stats.rs + t6report.rs.
## Update
- Code done (uncommitted->committed next): t6stats speed_* fns, combos.rs t6_measure/one_tutor_away (called from combo_options), explain() hook for chosen label, t6report render_speed + test.
- Running: build + sim-cli t6report test + calibrate t6 1000 games -> /tmp/claude-1000/t6speed/cal1000.txt
- Next: read speed lines, lock t3 diff (agents/lock.sh uses /tmp/claude-1000/t6ai-report dir; use own copy), report.
