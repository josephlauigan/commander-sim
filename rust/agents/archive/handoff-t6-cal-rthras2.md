# Handoff t6-cal-rthras2
Task: rthras2 (see rust/agents/t6-calib.md): model lines C, D, G as combos, check tutors, measure 600/2000 seed 500000.
Branch t6-cal-rthras, t6-cedh merged (cc69788 base).
## Status
- 18 Oct: merged, starting rebuild + export. Nothing else done yet.
- Found: `cradle_out`/`on_bf` in t6_plan_rograkh_thrasios.rs never saw lands (lands are not perms): Cradle/Gates on bf invisible, so the previous Crop Rotation/Gamble Gates change was dead code. Fixed.
- Added combos C (Cub+Mockingbird/Image+Oboro+Cradle, N>=3/4) and D (Cub+clone+Witness+Snap), both in t6_combos_t6.rs; line G not unbounded (one renewable R per loop), skipped. Clones held (prio 0); wish list has Cub/clones; T6-only wish-ranked put_creature (t3.rs) and Mycospawn pick (t6_green2.rs).
- Next: 600 games -> /tmp/claude-1000/t6cal-rthras2/r600.txt, then 2000, then tests, report.
