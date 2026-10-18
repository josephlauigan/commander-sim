# Report t6-e-control
Mechanism: `engine/control.rs`: `gain_control(g, new, m, Duration::{EndOfTurn,Permanent})`, `return_borrowed` (end-step return, was inline in turn.rs), `goad(g, goader, m, GoadFor::{NextTurn,Forever})` (stamp -1 = forever; `t0_zurgo::goaded_by` reads it, now pub).
`marchesa::steal` delegates to gain_control (so all other callers unchanged). Rewired directly: Dack, Loki's Scepter, Volatile Stormdrake, Sudden Substitution (substitute), Commandeer (t6_counters), t5 steal, Grenzo goad.
Dack Fayden: Full (flip enters under controller, triggers his, then donate, goad forever). Loki already Full. Stormdrake still Partial (hexproof from abilities only). Sudden Substitution still Partial (cast in response / resolving under new controller).
Behaviour for other decks: none intended (a permanent steal now also drops the creature from the old controller's borrowed list; same outcome).
Not built: a card-facing "controller changed" trigger event (no card needs it yet).
Base had EVENTS length error; fixed upstream, merged t6-cedh.
Tests: 4 new in cards_t6_dack.rs; full cargo test --release passes, no warnings.
