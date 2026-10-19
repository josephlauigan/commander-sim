# Handoff t6-cal-dack
Branch t6-cal-dack. Task: Dack too strong (brief rust/agents/t6-calib.md, task dack).
## Status
- [ ] build release + cardgen export
- [ ] 1 answer windows in shortcut (impls/t6_dack.rs)
- [ ] 2 pile table
- [ ] 3 archenemy view
- [ ] 4 plan tune
## Notes
(none yet)
- built OK. Finding 1: the shortcut gave ONE trigger window (first flicker) + one removal check after the flip; real loop has Dack's 2nd trigger, Preston copies, Solar ventures, repeated flickers. Added steps S_FLIP_DACK2/PRESTON/SOLAR + extra pieces checks in t6_combos_t6.rs (window fix). Next: measure 600 vs baseline.
- Item 2 (pile table) / 3 (archenemy view: line [QUIET,DACK] is public-ready when Dack castable) read; no change so far.
- run1 (windows only) 600: Dack 32.7/35.0 (base 34.8/36.3@2000), no real change. Now tuning dack_bar tighter (.2/.3/.4/.5/.8), measuring 600.
