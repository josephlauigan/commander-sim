# Handoff retire-kraum
Done (committed): physical cards, docs, my_decks.toml/decks.json/fixture/printings.
In progress: rust key lists + test seats (kraum -> thrasios) uncommitted; full `cargo test --release` running (log: worktree t.log, delete after).
Next: fix failing tests (Kraum-only cards -> hand-built seat), commit, build no-warnings, CLI checks (--deck kraum refused; --deck thrasios --pool t5 --games 50), report-retire-kraum.md.
Kept: kraum_prio / `"kraum" | "thrasios"` gates in impls/tymna_*.rs, ai/decks.rs, ai/brain.rs, report.rs (harmless dead arms).
