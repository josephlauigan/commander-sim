# Handoff t6-cal-sisay
- 2026-10-18 started. Goal: why Sisay combo tries are stopped 69-87% (modeling), then plan.
- Built release OK. Baseline 600-game run (background, output /tmp/claude-1000/t6cal-sisay/base600.txt) started before the lead's PAUSE; let it finish.
- PAUSE (lead): no new cargo/sim until the lead says so. Meanwhile: reading code only.
- Findings so far: DDR plan (t6_combos_t6.rs ddr_plan) needs Ioreth/Marvin/legend from HAND; Sisay's fetch is not considered as a source. Stops come from combos.rs interrupted()/run_steps answer windows. Next: read traces from the baseline to see which step/card stops it.
