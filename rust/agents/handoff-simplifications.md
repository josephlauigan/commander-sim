# Handoff: simplifications

Done: items 1-4 (Urza/Triple Triad permission; Mirage Mirror lands; Sylvan Library drawn cards; loop shortcuts a+b).
Next: 5 Ragavan (hand_proper shared), 6 Goldspan, then report-simplifications.md.
Gotchas: perl s{}{} / q{} break on unbalanced braces: use the Edit tool or a heredoc'd perl script with index/substr. zsh: quote `==`.
Tests: tests/practice_simplifications.rs (mine); run with systemd-run ... cargo test --release -q -p sim-core --test <name>.
