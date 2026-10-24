# Report retire-kraum (10/19)
Deck key `kraum` retired (Tymna/Thrasios replaces Tymna/Kraum); decks 14 -> 13.

## Done
- data/my_decks.toml (kraum removed), data/decks.json (cardgen export --offline; also flips one `rock` tag in cards.json, export's own output), tests/fixtures/my_decks_parsed.json (kraum block removed, format kept), data/printings/kraum.txt removed. thrasios.txt already carries Kraum's printings for every shared card (diffed: no differences).
- Key lists 14 -> 13: practice catalog.rs + setup.rs MY_DECKS, sim-cli decks.rs (FILES, MINE, test), args.rs DECK_KEYS, poolmode.rs (tier table + test), ai/mod.rs JD list, tests jd_zero_prio, server_fake, app_entry, http_catalog, http_server, game.rs (98 -> 91).
- Tests: kraum seats moved to thrasios (thrasios_ai, tymna_ai, audit_ysh, play_partners, cards_tymna, cards_land_audit2, tymna_fix, session_partners). Changes: Thrasios costs {G}{U} (tax 2 generic), fund() uses Forest instead of Mountain, session policy casts Thrasios with UG. `the_kraum_seat_does_not_cast_blossoming_defense` became `the_thrasios_seat_casts_blossoming_defense_to_save_a_commander` (Thrasios's list has the card). DROPPED: audit_ysh `breach_candidates_include_the_decks_wipe_only_when_the_table_is_ahead` (the wipe rule is keyed to key "kraum", no longer seatable).
- Docs: tymna-kraum-partners.md removed; CHANGELOG (list line, both Kraum sections get "Retired 2026-10-19"), CLAUDE.md tiers (thrasios t5), audit/kraum/FINDINGS.md retired line, tymna-thrasios-partners.md built-from note (and its Simulator row), omnath-landfall.md "Cards to confirm" drops Terror of the Peaks. No README/documents list named Kraum.
- Kept (harmless dead arms): kraum_prio, `"kraum"` in `"kraum" | "thrasios"` gates (impls/tymna_cards.rs, tymna_partners.rs, ai/decks.rs, ai/brain.rs, sim-cli report.rs), `kraum_wipe` in breach_candidates. Tier 6 deck kraum-tymna-blue-breach-midrange untouched.

## Physical cards (bulk-cards.txt / binder-cards.txt)
Bulk added: Kraum Ludevic's Opus (c16), Underworld Breach (mb2), Gilgamesh (fin), Expressive Iteration (soa), Lightning Army of One (fin), Bedevil (brc), Marchesa (2x2), Dragonskull Summit (m11), Sulfur Falls (isd), 3x Mountain (mdd), Hallway Heckler (fra).
Bulk count +1 on existing lines: Talisman of Conviction 2x, Lightning Security Sergeant 2x, Black Widow 2x, Firebird 2x, Battlefield Forge 2x (fic, ori), Clifftop Retreat 3x, Furycalm Snarl 3x, Rugged Prairie 2x, Spectator Seating 2x.
Bulk removed: Llanowar Wastes (blc), Deathcap Glade, Faeburrow Elder (ppeld), Avacyn's Pilgrim (list), Witherbloom Charm, Coiling Oracle, Paradox Surveyor; Forest 18x -> 15x (mdd).
Binder removed: Brushland, Dragonlord Dromoka, Yavimaya Coast. Binder still has Llanowar Wastes and Marchesa (second copies; brief put Marchesa to bulk anyway).
Not touched: Glorybringer, Thundermaw, Smaug, Terror (in Omnath); Basalt Monolith is in neither file (JD decides: one copy also listed in Godo).

## Checks
cargo test --release --workspace: all green (169 test binaries ok). cargo build --release --all-targets: no warnings. `--deck kraum` refused (usage error). `--deck thrasios --pool t5 --games 50 --ai adaptive --jobs 6` ran.
