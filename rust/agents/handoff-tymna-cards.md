# Handoff: tymna-cards

## Done (committed)
- cardgen `audit --cards FILE [--fetch]`; before-audit saved; Scryfall data fetched (cache + cards.json exported).
- `impls/tymna_cards.rs` (registered in impls/mod.rs): Thrasios, Kraum, Black Widow, Abigale (both names), Cuboid
  Colony, Elemental Mascot (impulse part), Falcon and Redwing, Photon, Faeburrow Elder, Paradox Surveyor, Spectral Lynx,
  Twisted Fates, Razia redirect + King Darien save (`protect` / `darien_wipe`, one-line hooks at the top of
  ai/decks.rs protect_response / wipe_response).
- data/card_overrides.toml entries for all of them (Sphinx = ward 2). Audit: all Full except Razia (Approximate, combat
  damage not redirected) and Full-auto cards to hand-check.
- Tymna checked: legends.rs tymna_hit counts any of your creatures' hits per opponent; test exists. No fix needed.

## Next
1. tests/cards_tymna.rs: one test per card above (not written yet unless the file exists).
2. Spot-check Full-auto: Dakkon, Elenda, Lyra, Make Your Move, Mana Leak, Memory Trap, Moss Diamond, Potioner's Trove,
   Silverquill Charm, Embrace the Paradox, Star Compass, Wander Off; plus evasion/combat-damage cards from the brief.
3. cargo test --release (all), build warnings, all_decks adaptive 3 / lookahead 1, audit-after file, report file.

## Gotchas
- zsh: quote globs; cardgen export --offline after changing overrides.
- Falcon/Photon/Black Widow are also in pool deck captain-america-avengers-assemble: its games change (Falcon used to
  make 2 tokens on ETB by a bad autotag).
- scryfall_cache.json / cards.json will conflict with partner-engine's branch (both fetch cards).
