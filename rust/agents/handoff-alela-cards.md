# Handoff: alela-cards (DONE)

Done: all seven cards Full in `cardgen audit --mine` (alela: 0 not Full), tests in cards_alela.rs (72 pass), four AI checks
answered (see report-alela-cards.md). Full `cargo test --release` passes; `cargo build --release --all-targets` has no warnings.
Smoke run: `--deck alela --pool t1` (alela is only matched to t1 in my_decks.toml) finished, 40 games.
Next: nothing; the lead checks the diff.
Gotchas: practice-mode (human) seats get the options through the normal Event::Options list, but the Page and Wall hooks skip
human owners. http_play in the practice crate is flaky under load (passes alone).
