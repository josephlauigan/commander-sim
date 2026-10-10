# Handoff: cardgen

Done (committed): extraction (data/*.toml|txt), crate cardgen: dsl_parse, dsl, cd, sources, cardinfo, scryfall, decks,
export, `cardgen export` (byte-identical to the Python data/cards.json + decks.json, checked 10/10).

Next, in order:
1. equivalence test in crates/cardgen/tests (export from data/ == committed data/*.json, no network)
2. `cardgen deck ...` (port of commander_sim/update_deck.py), `cardgen audit` (pool_audit.py)
3. README.md of the crate, rust/README.md update, report-cardgen.md
4. checks: cargo test --release, build --all-targets no warnings, all_decks adaptive 3 / lookahead 1

Gotchas: chrono etc. need network for first cargo build (works online). Build with
`export PATH=$HOME/.cargo/bin:$PATH` from rust/. Compare with Python: `python3 -m commander_sim.tools.export_cards --out DIR`.
