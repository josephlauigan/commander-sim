# Handoff: sheoldred-upgrade
Done: cache entries for the 4 cards (data/scryfall_cache.json), overrides (Full), code in impls/sheoldred.rs (register_upgrade, seizan/dreams/deal/lesson, Helm-on-Seizan), 11 tests in cards_sheoldred.rs, cardgen export.
Next: full checks (cargo test, warnings, all_decks, audit, 48-game smoke, swap smoke), report-sheoldred-upgrade.md.
Gotcha: an override `tags = ""` wipes autotag pow/tgh/leg; Seizan uses tags = "pow=6 tgh=5 leg".
