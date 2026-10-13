# Task: retire-jodah — remove the Jodah deck, keep the card code other decks use

Read `rust/agents/OVERNIGHT-RULES.md` first: its hard rules (never run Python, memory cap, edit only what this brief
names, commit often, handoff, 15-line report) apply. The machine is shared: every cargo/sim command under
`nice -n 19`, cargo `-j 4`, sims `--jobs 4`. Base branch: `cloud-midgar` (your worktree's branch `retire-jodah`).

JD retired the Jodah deck (key `jodah`, `decklists/JD/jodah-wubrg-legends.md`). Remove it completely, the way
`retire-elrond` removed Elrond (read `rust/agents/report-retire-elrond.md` and `git show 1ceab09 --stat` for the
pattern). Every other deck must play exactly as before.

## Remove
- The deck doc `decklists/JD/jodah-wubrg-legends.md` and `decklists/JD/jodah-illustrators.md` (Jodah's art theme
  reference), the `[jodah]` entry in `data/my_decks.toml`, its entry in `tests/fixtures/my_decks_parsed.json`, then
  rerun `cardgen export`. Don't edit `bulk-cards.txt` or `binder-cards.txt` (the lead does the inventory).
- The key everywhere it is registered (`grep -rn '"jodah"' rust/crates`): sim-cli `decks.rs` (FILES, MINE, counts),
  `args.rs`, `report.rs`, `ai/brain.rs` (the key checks, GENERIC_PLAYS, the special_options arm), `ai/decks.rs`
  (prio, tutor, protect, the other key checks), `ai/mod.rs` (MAIN_STYLES), `ai/plans.rs`, `engine/cast.rs`,
  `engine/zones.rs`, `impls/rules.rs` (CTHRESH), `settings.rs`, practice `catalog.rs` and `session/setup.rs`
  (MY_DECKS), and the deck lists/counts in the practice and sim-core tests.
- Jodah's deck-only AI in `impls/jodah.rs`: everything that only runs for `key == "jodah"` (cast priorities, the
  free-legend plan, tutor picks, protection, the milestone, response-only lists).

## Keep (card code keyed by card name)
Everything in `impls/jodah.rs` that models a card's own rules and can run for any deck or pool deck: Jodah, the
Unifier (the card), Profane Tutor and suspend (`suspend_upkeep`, called by `cardcode::suspend_upkeep`: **Kefka plays
Profane Tutor now**), Giver of Runes / Mother of Runes (`_rune_save`: the Cloud deck plays Giver of Runes; check
whether Cloud's protection reaches it through `ai::pool::protect` and keep whatever it needs), the legends and other
cards pool decks play. Rename the module `jodah` to `legends` (file `impls/legends.rs`, update every `jodah::` path)
so nothing is named after the deck, and fix its doc comments.

## Tests
`tests/cards_jodah.rs` seats the `jodah` deck. Rename it `tests/cards_legends.rs`; keep every test of a card's
behaviour, rebuilt on a deck that exists (or build the board with testkit); delete the tests of the deck-only AI.
Other tests that seat `jodah` as a table filler (`ownership.rs`, `play_marchesa.rs`, `land_entry.rs`, `cards_rules.rs`,
`cards_seph.rs`, practice `session_shadow.rs`, `play_bot_games.rs`, `cardgen/tests/workflow.rs` which uses the jodah
list as its sample deck): switch them to another deck and keep what they check.

## Done when
The OVERNIGHT-RULES checks pass (full `cargo test --release`, no warnings, `all_decks adaptive 3` and `lookahead 1`,
`cardgen audit --mine` with no `jodah` and no new non-Full cards elsewhere), `grep -rn '"jodah"' rust/crates` finds
nothing, and 240-game checks `--pool t3 --profile loose --games 240 --seed 500000 --ai lookahead --jobs 4` for
`--deck kefka`, `--deck cloud` and `--deck sauron` give **exactly the same results** before and after (run them on
the base commit first and save the output). Report: line 1 DONE or HANDOFF, line 2 your model ID, at most 15 lines;
full report in `rust/agents/report-retire-jodah.md`, committed.
