# Handoff t6-cal-sisay
- 2026-10-18 started. Goal: why Sisay combo tries are stopped 69-87% (modeling), then plan.
- Built release OK. Baseline 600-game run (background, output /tmp/claude-1000/t6cal-sisay/base600.txt) started before the lead's PAUSE; let it finish.
- PAUSE (lead): no new cargo/sim until the lead says so. Meanwhile: reading code only.
- Findings so far: DDR plan (t6_combos_t6.rs ddr_plan) needs Ioreth/Marvin/legend from HAND; Sisay's fetch is not considered as a source. Stops come from combos.rs interrupted()/run_steps answer windows. Next: read traces from the baseline to see which step/card stops it.
- fix 1 applied (ddr_keys: only Ioreth/Marvin/one source, not every legend). running 600 -> after1.txt
- 10/18 resumed after merging t6-cedh. FOUND: newcards_b Marvin options (Kefka copy) and t6_legends1b Marvin options (Ioreth untap) both set c.options; later one overwrote the other. Fixed with marvin_both in legends1b. Next: build, baseline 600 (tmp dir recreated), traces.
- built with fix1+marvin_both; running 600 -> /tmp/claude-1000/t6cal-sisay/after1.txt
- run failed: t6-cedh broken (dup Mind Stone); lead fixing, will say when to re-merge. Meanwhile coding fetch-aware ddr/uni plan (no sims).
- coded: Plan{casts,fetches}, get_piece (hand or Sisay fetch), fetch steps in DDR/Unicorn, legends::sisay_fetch_card, go_now true when line needs no cast, marvin_both. Builds clean, UNTESTED (waiting for fixed t6-cedh merge).
