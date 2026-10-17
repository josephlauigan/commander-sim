# Handoff: kraum-phyrexian3
Done: all five cards added (data fetched, overrides, cardgen export), code in impls/tymna_cards.rs (register_phyrexian2),
tests in tests/cards_tymna.rs (20 new), full `cargo test --release` passes, build has no warnings, all five Full in audit.
Next: smoke run (kraum t3, 40 games, five swaps), then rust/agents/report-kraum-phyrexian3.md.
Gotchas: the compiled Archfiend/Gut Shot DSL was wrong (override dsl = []); Mite tokens need g.dsl_on for "cant_block".
