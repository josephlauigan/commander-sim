# Handoff t6-cal-rthras4
Task: rthras4 (keep Thrasios alive), rust/agents/t6-calib.md. Work dir /tmp/claude-1000/t6cal-rthras4/.
## Status
- t6-cedh merged. Next: rebuild, export, baseline 600, then (1) check removal targeting, (2) plan: don't recast into open removal / hold protection.
- Baseline 600 (cons): Thrasios 16.1. 60 traces (43 with the deck): removals taken per seated game similar to Sisay/Kinnan (rt 49/43 games vs Sisay 51, Kinnan 46): NOT over-targeted. Commander casts 156 in 43 games (3.6/game): Rograkh recast at R12-14.
- Experiments (env flags in t6_plan_rograkh_thrasios.rs): RT4_A (Rograkh tax>=4 -> prio 40), RT4_B (Thrasios tax>=2 and open removal -> prio 60), RT4_B0 (->0). Runs in /tmp/claude-1000/t6cal-rthras4/{RT4_A,RT4_B,RT4_B_RT4_B0}-600.txt via runseq.sh (seq.done). Then make the winner permanent (remove env), 2000 confirm, tests, t3 diff, report.
- 600 results (cons/loose): base 16.1/15.7; A 18.3/15.7; B(prio 60) 17.6/15.9; B0 (hold 0) 15.2/14.2 (worse, dropped). A+B made permanent (commit "rthras4: ..."), no env flags.
- Running /tmp/claude-1000/t6cal-rthras4/final.sh: build, final2000.txt (seed 500000), t3 lock (t3.diff), tests (tests.txt); final.done when finished. Then write report-t6-cal-rthras4.md, commit.
