# Report: cloud-pkg2 (claude-sonnet-5-5)

Part 1 (commit 990dccf): Package D applied with `cardgen deck cloud` (CHANGELOG entry carries the buys and prices); the 8
cuts went to bulk-cards.txt (Rootborn Defenses became 2x); doc prose (Strategy, Doubled triggers, Free equips, Removal,
Weak spots, Mana) rewritten so nothing names a cut card; Equipment count 16 -> 14. validate OK, full tests pass.

Part 2: three packages on top of the D list in `audit/cloud/packages/` (s-speed, c-draw, b-carriers `.swaps`, tables and
test plan in README.md). Disjoint cuts, all cards Full. Smoke runs (100 games, T1, look-ahead): all load and finish.
- S (5 swaps, $52.01; lite = first 3, $5.70): Brass Squire, Lightning Greaves, Danitha, Sigarda's Aid, Stoneforge Mystic.
- C (5 swaps, $14.93): Sram, Mentor of the Meek, Welcoming Vampire, Skullclamp, Wall of Omens.
- B (5 swaps, $4.73): Fencing Ace, Mirran Crusader, Kor Skyfisher, Selfless Spirit, Adeline.
Modeling: Fencing Ace and Mirran Crusader were Partial; now Full (override entries, tests in
`rust/crates/sim-core/tests/cards_cloud_pkg2.rs`, which also tests Wall of Omens). Stonehewer Giant was dropped: its
compiled text fetched Sol Ring (not an Equipment) in a test. Esper Sentinel dropped for price ($55).
data/scryfall_cache.json and data/cards.json grew (the candidate cards are cached/exported so swaps can load them).
Nothing shared was edited (only data/card_overrides.toml additions).
