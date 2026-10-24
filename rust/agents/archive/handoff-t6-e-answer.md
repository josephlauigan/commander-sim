# Handoff t6-e-answer
Started. Next: read stack.rs ai_respond / partials.rs answer_ability; wire ertai_answer + louisoix; Camera copy; tests.
Plan: Game.trig_copies (ids of orig items with a resolved copy); stack.rs copy_item_window + trigger_window/resolve_batch run copies first;
ai_respond: controller copy via cardcode::copy_ability -> t6_copies::camera_copy; answer_ability chain adds ertai_answer + louisoix_answer (t6_counters);
Helm trigger imp 5->6 (t3.rs); Thassa's Oracle win via trigger_window imp 9 (combos.rs); ABILITY_ANSWERS + abilities_answered(camera).
Done so far: nothing committed.
Engine+card code written & builds (stack.rs copy_trigger, cardcode answer chain, camera_copy, louisoix_answer, Helm 6, Oracle window). Next: tests (new file tests/engine_t6_answer.rs), overrides, commit.
Tests written (tests/engine_t6_answer.rs, 12 pass), overrides done. Next: full cargo test, commit, report.
