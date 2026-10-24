# Handoff: thrasios-deck

Done (uncommitted pieces are listed in git status):
1. Card data: Undergrowth Stadium (carddb.txt line like Spectator Seating, enters tapped unless 2+ opponents),
   Talisman of Unity and Curiosity (`add_tags = ["pain", "rock=1:xx"]` in card_overrides.toml: with `pool_add_tags` they
   lost their colours once a JD deck held them; Conviction still has the old 1:C via pool_add_tags, untouched).
   scryfall_cache.json is re-pretty-printed with `jq --indent 2` after every cardgen fetch (cardgen writes it compact).
2. Deck `thrasios`: data/my_decks.toml, decklists/JD/tymna-thrasios-partners.md (cardgen deck; prose still the Kraum
   text: rewrite), data/printings/thrasios.txt, sim-cli/practice lists (14 decks), AI gates for key `kraum` widened to
   `kraum | thrasios` (tymna_cards equip/Flail/Dark Endurance/tutor land pick, tymna_partners prio, brain/decks tutor).
3. Kuja: impls/kuja.rs + tests/cards_kuja.rs; small additive hooks in engine/cast.rs (pings), engine/combat.rs (Flare
   Star), dsl/mod.rs (Niv damage), impls/kefka.rs (prio), state.rs (DataKey::KujaWiz).

Next, in order: tests green; item 4 (Thrasios + Basalt/Rings/Ballista in impls/combos.rs power_artifact: needs the
first-loop mana check, Thrasios as a sink when Ballista is in the library; Worldly Tutor fetch test); item 5
(`commander-sim --deck thrasios --cards`); doc prose; smoke runs; report-thrasios-deck.md.

Gotchas: `cargo test` stops at the first failing binary: use `--no-fail-fast`. testkit.rs has a retired test deck
"thrasios" that the real deck now shadows (tests still pass).
