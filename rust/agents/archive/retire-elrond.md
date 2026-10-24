# Task: retire-elrond — remove the Elrond deck, keep the card code

Read `rust/agents/OVERNIGHT-RULES.md` first: its hard rules (never run Python, memory cap, edit only what this brief
names, commit often, handoff, 15-line report) apply. A PreToolUse hook refuses any command starting with python,
python3, pip, pipx or uv. The machine is shared with other sessions: run every cargo/sim command under
`nice -n 19`, build with `-j 4`, sims with `--jobs 8`.

Base branch: `cloud-midgar` (your worktree's branch `retire-elrond` starts there). JD dropped the Elrond deck (key
`elrond`, `decklists/JD/elrond-ring-tempts.md`) for good. Remove it completely; every other deck must play exactly
as before. The earlier `retire-bill` brief (`rust/agents/retire-bill.md`) did the same for another deck: follow
its pattern.

## Remove
- The deck doc `decklists/JD/elrond-ring-tempts.md`, its `[elrond]` entry in `data/my_decks.toml`, its entry in
  `tests/fixtures/my_decks_parsed.json`, then rerun `cardgen export` (data/cards.json, data/decks.json). Cards only
  Elrond used may stay in data/cards.json if export keeps them (bulk/binder cards are fine).
- The key everywhere it is registered (`grep -rn '"elrond"' rust/crates`): sim-cli `decks.rs` (FILES, MINE, the test
  count) and `args.rs` (DECK_KEYS if listed), `report.rs` (plan text and milestone arms), `poolmode.rs` (the
  `me == "elrond"` block), `ai/brain.rs` (GENERIC_PLAYS, the `special_options` arm), `ai/decks.rs` (deck_prio and
  protect arms), `ai/mod.rs` (MAIN_STYLES), `engine/cast.rs` (the 7.0 list), `impls/rules.rs` (CTHRESH),
  `settings.rs` (cthresh), `cardcode.rs` (the `dk == "elrond"` hold block), `impls/mine.rs` (the elrond ring_pick
  branch: other decks keep their Ring-bearer choice), practice `catalog.rs`, and the deck lists/counts in
  `practice/tests/http_catalog.rs`, `http_server.rs`, `server_fake.rs` and `sim-core/tests/game.rs` if they count
  decks or games played.
- Elrond's deck-only AI in `impls/elrond.rs`: elrond_prio, elrond_special (warp planning), elrond_protect,
  ring_pick, entry_set, the hold logic, the `elrond` milestone, and anything else that only runs for
  `key == "elrond"`. Also drop the search gate/bonus code if any is gated to the elrond key (`ai/search.rs`).

## Keep (card code, keyed by card name)
Every card model in `impls/elrond.rs` and `impls/bulk_blue.rs` stays, with simple generic choices where the AI part
was deck-only: Elrond, Lord of Rivendell (the card: scry + Ring tempt), Gandalf, Friend of the Shire (`gandalf_draw`,
called from mine.rs), the Defiler discount (`defiler_discount` in engine/mana.rs), Mouth of the Storm (`weakened`
in engine/values.rs), warp as a mechanic, and the rest of the creature and spell models. Rename the module
`elrond` to `blue_cards` (file `impls/blue_cards.rs`, update every `elrond::` path) so nothing is named after the
deck, and fix the doc comments in it, `bulk_blue.rs` and `bulk_white.rs` that call these "the elrond deck's".
`state.rs`'s per-card counter for Elrond's ability stays (comment it as the card's).

## Tests
`tests/cards_elrond.rs` and `tests/cards_bulk_blue.rs` load the `elrond` deck. Rename the first to
`tests/cards_blue.rs`; keep every test of a card's behaviour, rebuilt on a deck that exists (build the board with
testkit so the deck key doesn't matter, or use another deck key); delete the tests of the deck-only AI (cast
priorities, the planner pairs, protection by bounce, turn-three Elrond).

## Done when
The OVERNIGHT-RULES checks pass (full `cargo test --release`, no warnings, `all_decks adaptive 3` and `lookahead 1`,
`cardgen audit --mine` with no `elrond` and every other deck unchanged), `grep -rn '"elrond"' rust/crates` finds
nothing, and two 240-game checks `--deck cloud --pool t3 --profile loose --games 240 --seed 500000 --ai lookahead
--jobs 8` and `--deck sauron --pool t3` (same flags) give **exactly the same results** before and after your change
(run them first on the base commit and save the output). Report: line 1 DONE or HANDOFF, line 2 your model ID.
