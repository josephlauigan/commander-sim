# Report: sauron-sweepers

Code: rust/crates/sim-core/src/impls/sweepers.rs (new); tests: tests/cards_sauron_sweepers.rs (16 tests).
- Crippling Fear, Widespread Brutality, Chandra's Ignition: `wipe=fear|brutal|ignite` kinds (tags in data/card_overrides.toml, dsl = []). One rule (`Sweep`) feeds the AI valuation (`wipe_eval`, `WipeRule`) and the resolution. Fear names the type that costs the caster least (Orc first; the Army counts as Orc); Brutality amasses 2 (Orc Army; Zombie type not tracked) then hits non-Army creatures, own included; Ignition: strongest own creature hits each other creature (own included, so not strictly one-sided) and each opponent; lethal player damage adds +30 per kill, +100 if it wins.
- Inferno Titan: ETB/attack 3 divided damage (all at a player if lethal, else best killable set, rest to a planeswalker or weakest player).
- Ral and the Implicit Maze: was marked Full but had no code (it was in the retired Zada module); restored from e4d8039^ into sweepers.rs, tested.
- Massacre Wurm: existing wurm tag, test shows it spares the caster's side; override note added.
- All six are Full (checked with `cardgen audit --mine --root <scratch copy of data/decklists with the six swapped into Sauron's list>`; no real list changed).
- Shared-file changes (additive): engine/removal.rs (dispatch to sweepers::resolve), ai/decks.rs (WipeRule ext field, wipe_eval early return), ai/pool.rs (WIPE_INDES += brutal, ignite), impls/mod.rs (module + register), data/scryfall_cache.json (4 cards added by hand from the Scryfall API; cardgen export would not fetch them), data/cards.json regenerated.
- Checks: cargo test --release all pass; build --all-targets 0 warnings; all_decks adaptive 3 and lookahead 1 finish.
- Smoke (t3, loose, lookahead, 96 games, swaps): A Ral+Wurm 24.0%, B Brutality+Fear 16.7%, C Ignition+Titan 25.0% (noise ~ +-4). The card report is empty at 96 games (needs 100 casts), so cast-when-drawn rates are NOT measured; a trace of 8 adaptive games with B showed 2 wipes resolving. AI use is covered by unit tests (offered when it kills more of theirs, lethal Ignition cast, held when it kills own).
