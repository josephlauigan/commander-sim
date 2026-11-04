# Handoff: Garth One-Eye

Done so far: four missing cards added (cardgen `extra` list in export.rs; Scryfall fetched), overrides in
data/card_overrides.toml (Garth, Black Lotus, Braingeyser, Regrowth, Shivan Dragon, Terror fixed to tgt=cna nonblack),
`cardgen export` run; impls/garth.rs (ability, options hook, cast_copy); DataKey::Garth; rules::petal_sac made pub.
Next: tests/cards_garth.rs, cargo test, report-garth-one-eye.md, commit.
Gotchas: the dsl "target_player" means an opponent, so Braingeyser is overridden to "you draw"; cast_card with zone
"copy" moves no card (a countered copy is removed from the graveyard by hand in garth::cast_copy).

Update: tests/cards_garth.rs written (16 pass); full `cargo test --release --no-fail-fast`: only plans
`every_plan_file_fits_its_deck` (known) and two `play_urdragon_human` tests (Godo search, camera) fail, both also fail on
base e6243581. Report written; commit last.
