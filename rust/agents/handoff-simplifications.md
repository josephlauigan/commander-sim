# Handoff: simplifications

Done: item 1 (Urza / Triple Triad permission), item 2 (Mirage Mirror copies any land; Perm::real_card; legend rule covers lands that are copies).
Next: 3 Sylvan Library, 4 loop shortcuts, 5 Ragavan, 6 Goldspan, then report-simplifications.md.
Gotchas: perl s{}{} with big replacement text breaks on braces: use the Edit tool for multi-line edits. zsh: quote `==`.
Tests: tests/practice_simplifications.rs (mine); run with systemd-run ... cargo test --release -q -p sim-core --test <name>.
Old practice_limits test (Mirror not offering fetch lands) was flipped to expect it offered.
