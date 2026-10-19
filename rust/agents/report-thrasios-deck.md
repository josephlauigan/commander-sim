# Report: thrasios-deck

Done
- Data: Undergrowth Stadium (carddb.txt, like Spectator Seating; enters tapped unless 2+ opponents), Talisman of Unity and
  Curiosity (`add_tags` pain + rock=1:GW / 1:GU; with `pool_add_tags` they fell back to a colourless rock once a JD
  deck held them). Breeding Pool already has `enters_rule pay 2` (engine supports shocks). Kuja in carddb.txt.
  scryfall_cache.json re-pretty-printed with jq after cardgen fetches (cardgen writes it compact).
- Deck `thrasios`: my_decks.toml, decklists/JD/tymna-thrasios-partners.md, data/printings/thrasios.txt (9 placeholders),
  decks.json/cards.json exported, fixtures, sim-cli/practice lists (14 decks), tests updated. AI gates of the kraum seat
  (equip/Flail, Dark Endurance, tutor land pick, tutor pick, Baird prio) widened to `kraum | thrasios`. The Breach
  rules stay kraum-only (no Breach in the list). Shared Tymna code (spread, commander-first) already covered both keys.
- Kuja: impls/kuja.rs (end-step token, pings via cast.rs hook, transform at 4 Wizards, Flare Star in combat.rs, DSL damage
  for Niv-Mizzet), 9 tests in tests/cards_kuja.rs. `--swap "A=>Kuja, Genome Sorcerer"` now accepts a front-face name.
- Combo: power_artifact now needs {5} for the first Rings loop (Python had no mana check), and Thrasios on the
  `thrasios` seat counts as a sink when the Ballista is in the library. Thrasios mana sink (eot / main 2) and Worldly
  Tutor -> Ballista already worked; tests in tests/thrasios_deck.rs (9).
- `--deck thrasios --cards`: nothing Partial or Unmodeled.

Python/old-code notes: Talisman of Conviction in JD decks is a colourless rock (pool_add_tags only); untouched.
CLAUDE.md band list still names kraum only (not edited).
Checks: cargo test --release --no-fail-fast all ok (168 result lines); build --all-targets no warnings.
Smoke: thrasios vs t5, 100 games adaptive: 31.0% win (CI 22.8-40.6), wins 81% damage / 19% combo. Kefka t3 swap Rank Rat ->
Kuja (100 games, lookahead): 34.0 -> 27.0 (noise +-10.7, no clear effect), Kuja drawn 38.5%, cast 62.3% of those (dry run).
