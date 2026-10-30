# Handoff: Phase 2, PR 5b (the 161 undeclared splits), branch `phase2-splits`

Brief: `rust/agents/phase2-splits.md`. Base: `phase2-coverage` (PR #163).

## Done
- Each of the 161 `split` lines judged against the Oracle text, `data/cards.json` and the card code (registration and
  hook bodies read).
- 152 real splits: `paths_ok` added in `data/card_overrides.toml` (81 on existing entries, 71 new entries holding only
  `paths_ok`, at the end of the file; `paths_ok` alone is no override). Their lines were deleted from
  `data/paths_known.txt`.
  - 45 lands: their compiled abilities never run (a land is a `Land`, not a permanent: `dsl::fire`, `dsl::statics` and
    `dsl::ability_options` read `player.perms` only). The engine runs Temples' scry and gain lands' life from
    `land_etb_fx` (`zones::lands_entered`) and cracks fetches by the `f` tag (`turn::crack_fetch`). The card code is
    City of Traitors' `land_etb` check, which `t6_lands.rs` sets on every land that has none. Fidelity gaps (not
    doubles): the surveil lands (Commercial District, Hidden Grotto, Undercity Sewers, Underground Mortuary) surveil
    never; the lands' activated abilities are never offered.
  - 107 others: two paths for different parts of the card (each note names both).
- 9 real doubles left in the list (no fix: behaviour changes need measured runs). Probe results (a throwaway test,
  not committed): Battle Angels of Tyr +1 Treasure on entering; Requiem Angel, Sifter of Skulls, Springheart Nantuko
  +1 token on entering; Pawn of Ulamog and Karn, Scion of Urza a 0/0 token on entering (dies at once);
  Sakura-Tribe Elder +1 land on resolving; Sowing Mycospawn +2 lands on casting; Doubling Season + Karn Liberated
  (6) enters with 24 loyalty.
- Test: `rust/crates/sim-core/tests/paths_splits.rs` (gain lands gain once, lands' compiled abilities never offered,
  Cryogen Relic draws once entering, Canopy Tactician taps for 3).
- `documents/realism.md` Phase 2: list now 21 competing, 25 shadowed, 9 splits (55 cards).

## Checks
- `cardgen export`: `data/cards.json` unchanged; `cardgen validate`: all decks passed.
- `cargo test --release --no-fail-fast -j 8`: 3593 passed, 0 failed, 2 ignored; `cargo build --release --all-targets -j 8`: 0 warnings.
