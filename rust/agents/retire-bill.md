# Task: retire-bill — remove the Bristly Bill deck, keep the card code Titania uses

Read `rust/agents/OVERNIGHT-RULES.md` first: its hard rules (never run Python, memory cap, edit only what this brief
names, commit often, handoff, 15-line report) apply. A PreToolUse hook now refuses any command starting with python,
python3, pip, pipx or uv.

Base branch: `bill-deck` (eee6900). JD chose the Titania deck (key `titania`, `decklists/JD/titania-lands.md`) over the
Bill deck (key `bill`, `decklists/JD/bristly-bill-landfall.md`). The two share most cards. Remove the Bill deck
completely; every other deck, Titania above all, must play exactly as before.

## Remove
- The deck doc `decklists/JD/bristly-bill-landfall.md`, its `[bill]` entry in `data/my_decks.toml`, its entry in
  `tests/fixtures/my_decks_parsed.json`, then rerun `cardgen export` (data/cards.json, data/decks.json).
- The key everywhere it was registered (the commits that added it: f7c1ab9, c784a48): sim-cli `decks.rs` (FILES, MINE,
  the test count) and `args.rs` (DECK_KEYS), `ai/brain.rs` (GENERIC_PLAYS, the `special_options` arm), `ai/decks.rs`
  (deck_prio and protect_response arms), `ai/mod.rs` (MAIN_STYLES, the `choose_attack` forced-plan block),
  `engine/cast.rs` (the 7.0 list), `impls/rules.rs` (CTHRESH), `settings.rs` (cthresh), practice `catalog.rs` (the
  "bristly bill" mapping), and the counts in `practice/tests/http_catalog.rs`, `http_server.rs`, `server_fake.rs`
  (decks 18 -> 17) and `sim-core/tests/game.rs` (played 90 -> 85).
- Bill's deck-only AI: everything in `impls/bill.rs` that only runs for `key == "bill"` (bill_prio, the doubling
  timing and lethal/forced-attack plan, counter_target and its gate in `dsl/mod.rs` `choose_target`, land_adjust and
  its call in `engine/turn.rs`, bill_protect, bill_special's Bill-only parts). Bristly Bill the *card* stays modeled
  (landfall counter, the doubling ability) with simple generic choices.

## Keep (card code, keyed by card name)
Hardened Scales (`cardcode::scales` and its call sites), Silhana Ledgewalker, Cactarantula, Carnivorous/Crushing
Canopy, Ghalta, Boughside Wanderers, Venomous Brutalizer, Wildgrowth Archaic, Royal Treatment's Role
(`royal_apply`, `extra_ward`; titania.rs calls `bill::royal_apply`), plot, foretell, disguise, collect evidence,
web-slinging, prepared, Additive Evolution, Wild Hypothesis, Freestrider Commando, Tumbleweed Rising, Lifestream's
Blessing, Spider-Man, Studious First-Year, Greenbelt Radical, Sample Collector. Rename the module from `bill` to
`green_cards` (file `impls/green_cards.rs`, update every `bill::` path) so nothing named after the deck is left,
and fix its doc comment.

## Tests
`tests/cards_bill.rs` loads the `bill` deck (`table(&["bill", "seph"])`). Rename it `tests/cards_green.rs`; keep every
test of a card's behaviour, rebuilt on a deck that exists (`titania`, or build the board with testkit so the deck key
doesn't matter); delete the tests of Bill's deck-only AI (doubling timing, landfall target choice, fetch sequencing
while Bill is out, Harrow in response).

## Done when
The OVERNIGHT-RULES checks pass (full `cargo test --release`, no warnings, `all_decks adaptive 3` and `lookahead 1`,
`cardgen audit --mine` with no `bill` and `titania: 66 unique cards, 0 not Full`), `grep -rn '"bill"' rust/crates`
finds nothing, and a 240-game check `--deck titania --pool t2 --profile loose --games 240 --seed 500000 --ai
lookahead --jobs 22` gives **exactly the same result** before and after your change (run it first on the base
branch and save the output; the Titania AI must not change). Report: line 1 DONE or HANDOFF, line 2 your model ID.
