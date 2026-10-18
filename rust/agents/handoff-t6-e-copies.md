# Handoff t6-e-copies
Started. Next: read engine copy code (zones.rs enter/Tag::Clone, enter_token_copy, t6_copies.rs).
Design: state.rs CopyExc (exceptions) + Perm.exc; hooks CardImpl.enter_copy slot (fn(&mut Game,PlayerId,CardId)->Option<CopyOf>) read by zones::enter before new_perm; Enter.exc for token copies (enter_token_copy_with). values::ptypes = card types|exc.add_types used in removal/dsl/decks. t6_copies clones -> enter_copy slot; Mockingbird target via g.cast_target.
Done: (nothing yet)
Done: engine slot/exc/ptypes, t6_copies, Preston, Helm, Copy Enchantment rewired, data toml+cards.json. Next: run tests, add tests (Soul Warden once, Mockingbird Bird Wizard, Copy Artifact enchantment, Preston Human Cleric not Human, Molten artifact, Steel->Mirror->Godo test), report
Tests added and passing (copies, dack). Next: full cargo test, build all-targets, update engine-needs-copies/dack md, report, commit
ALL DONE; committing.
