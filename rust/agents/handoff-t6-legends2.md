# Handoff t6-legends2
Cards: Magda, Brazen Outlaw; The Queen of Dale; Tataru Taru; Teferi, Who Slows the Sunset; Spider-Woman, Stunning Savior; Ertai Resurrected; Legolas's Quick Reflexes; Colossal Skyturtle; Wan Shi Tong, Librarian
Done: ALL 9 cards committed (Magda P, Queen F, Tataru F, Teferi P, Spider-Woman P, Ertai P, Legolas P, Skyturtle F, Wan Shi Tong P)
Next: full cargo test --release, build --all-targets warnings, report-t6-legends2.md
Gotchas: after editing data/card_overrides.toml run `rust/target/release/cardgen export --offline` (tests read data/cards.json). Test table: tbl() in tests/cards_t6_legends2.rs. Trigger hooks: at_once(c, Event, false) when the hook calls trigger_window; ManaTapped runs at once. Scryfall Oracle text in scratchpad o.json.
