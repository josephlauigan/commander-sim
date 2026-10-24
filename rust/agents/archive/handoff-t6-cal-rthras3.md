# Handoff t6-cal-rthras3
Task: rthras3 (rust/agents/t6-calib.md). Work dir /tmp/claude-1000/t6cal-rthras3/.
## Results at 600 (seed 500000, conservative/loose)
- baseline 14.5/14.5; E2 (Thrasios dig on own main2) 13.3/13.5 -> dropped; E1 (keep finders) 15.7/14.7 -> KEPT; E4 (Rhystic/Remora/Library prio 90) 14.2/12.5 -> dropped; go_ok/go_now read: not a blocker.
## Status
- E1 is now permanent in t6_plan_rograkh_thrasios.rs (FINDERS, keep()). 2000-game confirm -> final2000.txt. Then: sim-core tests (ai_t6_rograkh_thrasios etc), t3 lock diff, report.
