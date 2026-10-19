# Handoff: practice-hints
Done (uncommitted until tests pass): sim-core engine/turn.rs `keeps_hand` + `bottom_pick` (extracted, AI behaviour identical);
practice advisor/ is now a folder: mod.rs (catch_unwind + normalise wrapper, falls through to other::hint), mulligan.rs
(hint, bottom, verdict), other.rs (discard/scry/sacrifice real picks; tips for block/target/search/choose/continue/unknown);
shadow.rs records a "mulligan" entry (keep/mulligan vs the AI, ai_note saved); app.js hint box never prints null;
tests/hint_every_decision.rs (3 tests) + session_shadow.rs updated.
Next: run all `-p practice` tests, add a review test for the mulligan entry, write report-practice-hints.md (inventory,
null causes, real-pick vs tip kinds), check `commander-practice --no-browser` starts, commit.
Gotcha: no python even in a trailing command (hook blocks the whole command).
