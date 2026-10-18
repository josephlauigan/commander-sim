# Handoff audit-ysh
Done: all Y'shtola (3-6) and Kraum (1-6, 8) fixes, tests in crates/sim-core/tests/audit_ysh.rs, FINDINGS for both decks,
Kraum doc tier text. Not done: Kraum 3 (bodies, not code), Kraum 7 (mana), Y'shtola 1, 2, 7 (core agent).
Gotchas: zsh does not word-split `$R` command prefixes; use a function. Games are fast (about 25 per second at 2 jobs).
