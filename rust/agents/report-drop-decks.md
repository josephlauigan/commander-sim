# Report: drop-decks

Removed as decks: thrasios, kraum, garland. Deleted the three lists in decklists/JD/, their rows in sim-cli decks.rs
(FILES, MINE) and the array sizes there and in args.rs (14 -> 11), their entries in data/my_decks.toml, their keys in
tests/fixtures/my_decks_parsed.json (trailing newline kept as before), deck counts in game.rs (84 -> 66 games),
http_catalog.rs, http_server.rs, server_fake.rs (12 -> 11) and the sim-cli decks unit test (14 -> 11). Re-ran
`cardgen export --offline` and `cardgen validate` (OK).

Kept: all engine and card code, tymna_partners.rs, garland.rs, key-gated hooks, audit/ and agents/.

Tests rewritten: none deleted. `sim_core::testkit` now appends three retired test decks (tier "retired", real commanders
and identity, basic-land library of 100 incl. commanders) so partners.rs, cards_tymna/garland/legends.rs, tymna_ai.rs and
tymna_fix.rs run unchanged with the seat keys still set. `tier()` and the game.rs "mine" filter ignore them.

Results: `cargo test --release` all pass (1637+; http_play flaked once, passes alone); no warnings; all_decks adaptive 3
and lookahead 1 finish; `--all-decks` lists 11 decks.

Same seed (7, 48 games, lookahead, loose): NOT identical, for a known reason. data/cards.json is regenerated per the
deck lists: `add_tags` overrides apply only to cards used by one of your decks, `pool_add_tags` otherwise. Two cards were
only in the removed decks: Talisman of Conviction (rock 1:C -> 1:RW, pain tag) and Hope-Ender Coatl (loses soft=1). Both
sit in pool decks, so those pool decks play slightly differently (sauron t3, cloud t5, sephfs t3, seph t3 all moved by a
few games). Proof: with the old cards.json restored all four runs are byte-identical to before (apart from the timing
line). New runs are deterministic (twice identical). The equivalence test requires the regenerated file, so it stays.
