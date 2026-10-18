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
## Update 10/18 (resumed)
- Lock saved: rust/agents/perf-lock-t3.txt (progress/timing lines stripped; compare with the same grep). Pre-merge branch.
- perf is BLOCKED (perf_event_paranoid=4, no sudo). Using gdb poor-man's sampler (scratchpad mk.sh), 40 games jobs 1.
- 40-game jobs-1 timing: mainbase 127s/145s, branch 183s/217s (~+45%; another session was loading the box).
- First hot spot seen: combos::is_piece_name (linear scan with slice eq) via piece_threat <- pval <- threat.
- Lead: merge t6-cedh (Karlach double-run fix da52e57) before final timing; re-save lock after the merge.
- Opt 1 (committed): is_piece_name -> OnceLock HashSet. Opt 2 (committed): dsl::pt one pass over statics. Lock identical after both.
  t3 300 games jobs 12 CPU: branch 1236 s vs mainbase 1094 s (+13%, box shared with another session, load ~8).
- Next: merge t6-cedh (Karlach fix), re-save lock, more profile rounds (sampler: scratchpad mk.sh; stats via awk on mine.log).
