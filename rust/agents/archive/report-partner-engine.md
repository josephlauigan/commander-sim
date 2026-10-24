# Report: partner-engine (two commanders per seat)

Model: claude-sonnet-5-5. Base: tymna-thrasios (e70961e). Branch partner-engine.

## Design
`Player` keeps `cmd`, `cmd_in_zone`, `tax`, `cmd_dmg` for the first commander (every single-commander deck and every
deck-specific module untouched) and gains `cmd2: Option<CardId>`, `cmd2_in_zone`, `tax2`, `cmd2_dmg`. API on `Player`:
`set_partner`, `is_commander(c)`, `commanders()`, `cmd_zone_of(c)`, `any_cmd_in_zone()`, `cmds_in_zone()`,
`set_cmd_zone(c, v)`, `tax_of(c)`, `add_tax(c)`. `values::cmd_dmg_by` / `add_cmd_dmg` count damage per commander.
`Seat` gains `commander2` and `Seat::player(id)`; `Engine::seat` finds commander2 from decks.json by deck key.

## Data
- `data/my_decks.toml`: `thrasios` and `kraum` with `commander2`; decks.json carries `commander2` only for them
  (existing decks' JSON diff is empty; cards.json only gains the new cards).
- cardgen: `MyDeck.commander2`, `structural_problems` / `card_problems` take a commander slice (two: both need plain
  Partner, identity = union), `validate` also checks my decks with two commanders, `update_deck` handles the
  `**Commanders (2).**` line. Both new decks validate OK (BGUW, BRUW).
- Both deck documents listed `Abigale, Poet Laureate` in the import list; the card is a double-faced card named
  `Abigale, Poet Laureate // Heroic Stanza`, so I changed that one import line in both documents (export needed it).
- `tests/fixtures/my_decks_parsed.json` extended with the two decks (deck guard test).

## Engine / AI
Command zone, tax (mana.rs, cast.rs), commander damage (combat.rs, life.rs, brain.rs lethal check), death/exile/bounce/
discard return (zones.rs, stack.rs), `is_cmd` on cast, cast option for each commander in the zone, cheaper first
(brain.rs candidate order, pool.rs `generic_prio` tiebreak by cmc), recast utility with its own tax, opponents'
removal targets see both (ai/decks.rs), determinize (search.rs) keeps both known, plus the generic helpers in
common.rs, mine.rs, combos.rs, legends.rs, zur.rs, bulk_white.rs, topdeck.rs, dsl blink, human/legal.rs, human.rs.
Deck-specific code that names "the commander" (Cloud, Titania, Sauron, Yuriko...) still looks at `cmd` only.
`impls/tymna_partners.rs`: `thrasios_prio` / `kraum_prio` stubs (default priority); dispatch in ai/decks.rs.

## Shared files touched (all small, additive)
state.rs, engine/{turn,cast,combat,life,mana,stack,values,zones}.rs, export.rs, run.rs, testkit.rs, ai/{brain,decks,
plans,pool,search,topdeck}.rs, dsl/mod.rs, impls/{mod,common,mine,combos,legends,zur,bulk_white}.rs, human/*, sim-cli
decks.rs/args.rs (FILES, MINE and DECK_KEYS now 12: `--all-decks` runs 12 decks), practice catalog.rs (hides decks with
commander2 from the setup screen; the practice UI shows one commander per deck), practice setup.rs.
Tests changed: game.rs `played` 60 -> 72.

## Checks
- `cargo test --release --no-fail-fast`: all pass. `cargo build --release --all-targets`: no warnings.
- `tests/partners.rs`: 9 tests (both start in zone, both cast, separate tax, 21 from one, 20+20 split, per-commander
  damage, each back to the command zone, Bastion Protector buffs both, single seat unchanged).
- all_decks adaptive 3 / lookahead 1: finish, no panic.
- Same seed (`--seed 7 --games 48 --ai lookahead --jobs 10 --brief`, before vs after binary): sauron t3, titania t1,
  cloud t5 identical (apart from timing lines).
- Smoke, `--pool t3` lookahead 48 games: thrasios 6.2%, kraum 16.7% (default priorities, cards unmodeled).
  A trace shows both commanders cast.

## Open
- Practice mode (human UI) does not offer the Partner decks.
- Deck AI plans for the two decks are not written (stubs).
