# Report: karn-threats

All 19 cards are coded in `rust/crates/sim-core/src/impls/karn_cards.rs` (registered in `impls/mod.rs`), with
`dsl = []` and hand tags in `data/card_overrides.toml` (the compiled Oracle abilities were wrong or unsafe: Disk
destroyed creatures only, Ratchet Bomb and Steel Hellkite destroyed every nonland permanent for free, O-Stone ignored
fate counters). `cardgen export --offline` was run; only the 19 cards changed in `data/cards.json`.
Tests: `rust/crates/sim-core/tests/cards_karn_threats.rs` (22 tests).

## Per card (status in card_overrides.toml)
- All Is Dust: Full. Tag `wipe=dust`; the `dust` kind of `sweepers.rs`; sacrifices every coloured permanent; the usual wipe options cast it.
- Blightsteel Colossus: Approximate. Infect (poison instead of life loss, 10 kills; `engine/combat.rs`), shuffle-back from the battlefield (`to_zone_card`). Not modeled: -1/-1 counters on creatures (damage instead), mill/discard still reaches the graveyard.
- Cityscape Leveler: Approximate. Cast trigger (`self_cast`) and attack trigger destroy the best opposing nonland permanent (value 3+), tapped Powerstone for its controller; unearth {8} (haste, exiled at end step). Not modeled: "exile instead of leaving".
- Darksteel Forge: Full (`grant_kw` indestructible for our artifacts, tokens typed artifact too).
- Duplicant: Full. Exiles best nontoken creature (value 3+), takes its P/T and types.
- Kuldotha Forgemaster: Full. Sacrifices the 3 cheapest other artifacts for the best artifact in the library (AI weighs card vs fodder).
- Liberator, Urza's Battlethopter: Approximate. Flash grant for artifact/colourless spells via `grants_flash` (the AI casts them in its end-of-turn window), +1/+1 counter when mana spent > power.
- Mishra's Self-Replicator: Full. Pay {1} on historic casts for a token copy; copies trigger too.
- Nevinyrral's Disk: Full. Enters tapped; AI activates when opposing loss clearly beats ours (`Wipe::Disk`, respects indestructible).
- Oblivion Stone: Full. Fate counter on our best permanent (when it improves the wipe), wipe when the swing is clear, counters cleared.
- Platinum Angel: Full. `t0_giada::cant_lose` also true with a Platinum Angel (life, poison, decking).
- Portal to Phyrexia: Full. Edict of three (least valuable) and upkeep reanimation of the best creature card in any graveyard, as a Phyrexian.
- Ratchet Bomb: Full. AI winds it up when a higher count hits more, detonates at the best mana value (tokens are 0).
- Spine of Ish Sah: Full. ETB destroy (value 2+); `self_dies` returns it to hand.
- Steel Hellkite: Approximate. {X} destroy for players it dealt combat damage to this turn, once a turn, AI picks X. Not modeled: {2}: +1/+0.
- Steel Overseer: Full. Counters on each artifact creature, precombat.
- Triplicate Titan: Full. Three Golems (flying, vigilance, trample) on death.
- Ugin, the Ineffable: Approximate. Cost reduction (`cost` hook), +1 Spirit (card to hand when that Spirit dies, or when Ugin leaves), -3 on the best coloured permanent. Loyalty 4 from Scryfall.
- Unstable Obelisk: Full. Tap for {C} (`rock` tag); {7} sacrifice destroys the best opposing permanent (value 4+).

## Shared-file edits (all small, additive)
- `engine/combat.rs`: infect branch (poison counters) after the Szadek replacement.
- `engine/zones.rs` `to_zone_card`: Blightsteel goes to the library instead of the graveyard.
- `engine/cast.rs` `grants_flash`: `|| karn_cards::liberator_flash`.
- `impls/t0_giada.rs` `cant_lose`: `|| karn_cards::angel_holds`.
- `impls/sweepers.rs`: `dust` kind (KINDS, hits, eval, resolve).
- `impls/combos.rs` `combo_options`: the `karn` key is allowed (the other JD decks stay excluded), so the AI plays the Basalt Monolith + Rings of Brighthearth combo with Walking Ballista as the outlet (tested with a hand Ballista: Rings is cast, the combo wins). If karn-core makes `karn` a main deck, keep this exception.
- `state.rs`: `DataKey::Fate`; `Game::is_creature`: a token whose card is not a creature (the Powerstone) is no longer a creature. Before, `make_powerstone` tokens (Static Net, Urza) died at the next state check as "toughness 0 or less". This fixes an existing bug.

## Notes for the merge
- Karn, the commander, and the AI plan (karn-core) are not touched. Karn's restricted mana: my abilities use the normal `pay`.
- Not run: sims (only unit tests). Smoke runs noted in the final line of the report.

## Checks
- `cargo build --release -j 7 --all-targets`: no warnings. `cargo test --release -j 7 --no-fail-fast`: all pass except 8 tests that count the decks (13 -> 14 with the new `karn` entry in decks.json / my_decks.toml): cardgen `deck_guard_fixture_is_current` (tests/fixtures/my_decks_parsed.json needs regenerating), practice `app_entry`, `http_catalog`, `http_server::test_options`, `server_fake::options_and_saves_shapes`, `session_partners` (deck lists), sim-cli `the_repository_decks_have_their_tiers`, sim-core `game::every_deck_finishes_games_at_every_tier`. They belong to the deck wiring (karn-core), not to these cards.
- `all_decks adaptive 3` (75 games) and `all_decks lookahead 1` (25 games): no panic.
