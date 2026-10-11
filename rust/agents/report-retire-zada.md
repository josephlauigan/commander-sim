# Report: retire-zada (branch retire-zada, base titania-deck 57ef99b)
Result: DONE. Eight decks removed (zada, torbran, glissa, jenova, gix, yawgmoth, lordofpain, smaug).
Checks: `cargo test --release` all green; `build --all-targets` no warnings; `all_decks adaptive 3` and `lookahead 1` exit 0;
`cardgen audit --mine`: nine decks, not-Full counts titania 0, yshtola 1, alela 2, jodah 4, rest 0; quoted-key grep in rust/crates empty.
Invariance: 9 decks, `--pool t3 --profile loose --games 48 --seed 500000 --ai lookahead --jobs 22 --brief`: all identical to the base
(apart from progress/timing lines). Extra: seph, sauron, titania, kefka at t1, t2, t4, t5 (same flags): identical to a build of the base.

## Removed
Deck docs (zada doc, candidates/), my_decks.toml entries, fixture entries, `cardgen export`; the keys from every registration site
(sim-cli decks/args/report, settings, rules CTHRESH, cast.rs, ai/mod MAIN_STYLES, brain GENERIC_PLAYS and options, decks.rs arms,
plans.rs response_only, zones token_color, practice catalog and test counts, game.rs 85 -> 45).
Modules and tests deleted: smaug, lordofpain, torbran, gix, yawgmoth, jenova, zada, zada_cards and their cards_* test files (cards_rakdos too).
Engine branches removed (keeping the other side exactly): Torbran's +2 (`torbran_plus`, `ping_floor` = 3.0 inlined) in life/combat/removal/cast/dsl/brain;
Smaug's Treasure pings (cast, removal, dsl); Manaworker mana filter (mana.rs `fixed_plan`) and `Player::bloods`; Gix's tuned equip scoring (dsl),
`key == "gix"` in t3.rs and partials.rs (pool Gix keeps life > 10 and no seven-land rule); yawgmoth.rs wrappers (the pool deck's t5 code runs
directly), `returns_when_dies` in zones, `dk == "yawgmoth"` in cardcode; Mutant sac worth (zones); Glimpse the Impossible spawns and
Generator Servant haste (`glimpse_exiled`, `haste_next`).
## Kept (card code that Titania, Sauron or pool decks use)
- `glissa.rs` -> `bites.rs`: bites and fight (Bite Down, Hard-Hitting Question, Compel Brutality, Diplomatic Relations, Knockout Maneuver,
  Chelonian Tackle), Track Down, Hardbristle Bandit (crime), Lashwhip Predator, Bugenhagen, Tenured Concocter (+ `becomes_target`), Paladin of
  Predation. Tests `cards_glissa.rs` -> `cards_bites.rs` (19 tests, on the titania deck; Bugenhagen rebuilt on tokens).
- Topiary Lecturer (increment, mana) and Vinelasher Adept (landcycling) from jenova.rs -> `green_cards.rs`, with `gained_life` (Titania's Follow the
  Lumarets); four tests added to `cards_green.rs`. The Efflorescence part of a Hardened Scales test was dropped (no deck plays it).
- Relic of Legends (Sauron) -> `mine.rs`. mine.rs PREPARED lost the Goblin Glasswright and Strife Scholar rows (Zada only); Firesinger stays.
- Yshtola's Champion's Helm hook now keys on "yshtola" only.
## Left alone
card_overrides.toml verdict entries for removed cards (harmless); history docs (pool-results.md etc.) still mention the decks.
