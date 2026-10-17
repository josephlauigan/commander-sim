# Report: kraum-godo
Godo, Bandit Warlord modeled Full (impls/tymna_cards.rs: godo_etb, godo_attack, godo_gear_score; registered in register_gear).
- Card was missing from data: fetched from Scryfall into data/scryfall_cache.json, data/cards.json re-exported (only Godo added).
- Override in data/card_overrides.toml: dsl = [] (the Oracle compiler produced a wrong "search any card" ETB), Full + note.
- ETB: searchable Equipment, scored by size + keywords (haste worth most while Godo is summoning sick; Heirloom counts as trample+haste on a legendary), entered, attached to Godo (also job select pieces), library shuffled. AI choice order with both in library: Heirloom > Helm > Flail > Shield.
- Attack: once per turn (once_per_turn key), untaps Godo + Samurai (is_samurai), extra_combats += 1.
- Tests (cards_tymna.rs, 6 new, 89 total in file): fetch+attach, job select attach, empty library, untap+one extra combat+no second, Heirloom preference, hasty Godo gets exactly 2 combats (combat_no == 2).
- Full cargo test --release passes. cardgen audit: Full. No sims run. all_decks smoke runs not done.
- Swap syntax: commander-sim --deck kraum --swap "Serra Angel=>Godo, Bandit Warlord" (OUT=>IN, from sim-cli args.rs; not executed).
