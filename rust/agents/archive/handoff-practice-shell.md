# Handoff: practice-shell (command line part only; iOS deferred by the user)

Done: `commander-practice` binary (main.rs, flags of play/__main__.py; --text accepted but exits "not ported"),
`common::start_rust()` (in-process server, temp saves/images, symlinked data, fake Scryfall), autosave race fix in
`server/hub.rs::autosave` (state lock held across check+write), rust/README.md "Practice mode".
HTTP tests ignored with "needs practice-core": http_server 4, http_play 6, http_lan 4 (only the mulligan is asked).
Note: http_lan::test_hint_is_for_the_seat_deciding assumes the host's mulligan comes first; seats are random in
two-player, so it may be seat-order dependent once practice-core lands (Python behaves the same).
Next: when practice-core/cards land, run `cargo test --release -p practice -- --ignored` and remove the ignores.
NOT done: ios-shell/, lib.rs practice_start/stop/set_lan staticlib entry points + deleting the M6 proof page, Xcode,
practice-mode.md / architecture.md docs, crates/practice/README.md, --text terminal table.
