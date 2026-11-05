# Handoff: simplifications

Done: items 1-5 (5: values::hand_proper / hand_count, sites listed in the report).
Next: 6 Goldspan, then report-simplifications.md.
Gotchas: perl s{}{} / q{} break on unbalanced braces: use the Edit tool or a heredoc'd perl script with index/substr. zsh: quote `==`.
Tests: tests/practice_simplifications.rs (mine); run with systemd-run ... cargo test --release -q -p sim-core --test <name>.
