# Handoff t6-legends2
Cards: Magda, Brazen Outlaw; The Queen of Dale; Tataru Taru; Teferi, Who Slows the Sunset; Spider-Woman, Stunning Savior; Ertai Resurrected; Legolas's Quick Reflexes; Colossal Skyturtle; Wan Shi Tong, Librarian
Done: Magda (Partial); Queen of Dale (Full; pub fn recruit); Tataru Taru (Full); Teferi (Partial: -7 emblem unmodeled)
Next: Spider-Woman
Gotchas: after editing data/card_overrides.toml run `rust/target/release/cardgen export --offline` (tests read data/cards.json). Test table: tbl() in tests/cards_t6_legends2.rs. Trigger hooks: at_once(c, Event, false) when the hook calls trigger_window; ManaTapped runs at once. Scryfall Oracle text in scratchpad o.json.
