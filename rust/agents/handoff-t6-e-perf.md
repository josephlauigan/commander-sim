# handoff t6-e-perf (stopped early by lead: CPU needed for overnight run)
## Done
- Both builds done (release has debug=1 line tables already): t6-e-perf/rust/target/release/commander-sim and t6-mainbase/rust/target/release/commander-sim.
- Behaviour-lock run started on the unchanged branch; the 300-game t3 run (12 jobs) took ~46 s wall but was killed
  mid-run, so the lock output is NOT complete (scratchpad lock.txt only has progress lines). No code changes made.
## Not done
- Behaviour lock, profile, optimisations, timing, report.
## Next steps
1. Rerun: `commander-sim --calibrate within --pool t3 --games 300 --seed 500000 --jobs 12 > lock.txt` (nice 19, systemd-run MemoryMax=18G); save as the byte-for-byte reference (~1 min).
2. `perf record -g` on `--jobs 1 --games 40` for both builds (perf exists at /usr/bin/perf; no valgrind); diff top functions.
3. Optimise (check_state legend/toughness, HAND_MANA Spirit Guide scan, cast gates, combo readiness prefilter, would_be_cast_countered, each_combat); re-run lock after each; commit each.
4. Time --games 1500 t3 on mainbase vs branch back to back; write rust/agents/report-t6-e-perf.md.
