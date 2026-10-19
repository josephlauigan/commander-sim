# Handoff t6-speed-sisay
Task: sisay-speed (brief rust/agents/t6-speed.md). PAUSED by lead (CPU); resume only on "resume".
## Done (uncommitted code: legends.rs, t6_combos_t6.rs; audit/t6/speed-2026-10-19/sisay-v1-600.txt, sisay-v2-600.txt)
- legends.rs `sisay`: Tier 6 offers the fetch in the end-of-turn window (post None, is_t6) -- per-deck gate, no shared hook added.
- t6_combos_t6.rs: uni_plan(g,p,in_line) and ddr_plan(g,p,in_line): in-line steps ignore a tapped Bloom Tender (fixed Unicorn
  stop after first fetch, trace game 40) and skip the mana re-check (DDR); fetch_next logs "line no longer ready".
- Results 600 games seed 500000 (cons/loose): baseline cal1000 26.3/20.7, v1 (EOT) 26.2/23.7, v2 (+fix) 26.4/24.0; median win round 12-13 unchanged,
  ready 8 in 13% of games. EOT fetch fires ~1 in 60 games (leftover mana is rarely WUBRG): no speed-up by logic, mana pool is one per cycle.
- Unicorn unnamed stops still 11 (of 600) after fix; DDR unnamed 5->8. DDR stop at step 3 in trace game 20 (seed 500000 --trace 20): plan None
  with src Ragavan "unusable" though haste=true; cause not found.
## Next (after resume)
1. trace2.sh (traces 0-199 into /tmp/claude-1000/tr/s*.txt) was killed part way (partial); tally "stops at step" per line to find remaining Unicorn stops.
2. 2000-game run (/tmp/claude-1000/run3.sh), t3 lock diff, FULL cargo test --release, commit (git add -A rust data audit), report rust/agents/report-t6-speed-sisay.md, SubagentHandback.
## Paused again (lead, 2nd time). Resumed work killed mid-way.
Lead's wrap-up order: keep changes, run /tmp/claude-1000/final.sh (build, 2000-game t6 run to audit/.../sisay-v2-2000.txt, t3 lock diff -> /tmp/claude-1000/t3.diff, full cargo test -> tests.out), write rust/agents/report-t6-speed-sisay.md (include remaining unnamed stops + Ragavan "unusable" DDR stop), commit, final message <=12 lines. Nothing from final.sh finished.
