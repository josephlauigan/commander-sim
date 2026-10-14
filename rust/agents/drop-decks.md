# Task: drop-decks — unregister and delete the thrasios, kraum and garland decks, keep their code

Read `rust/agents/TYMNA-RULES.md` and `rust/agents/OVERNIGHT-RULES.md` first. Base branch: `tymna-thrasios` (PR #64; the
commit that added this brief). Your worktree's branch: `drop-decks`. Only agent running: `cargo -j 16`, `--jobs 16`.
Never use `git stash`.

JD decided not to build three of the decks explored on this branch. **Remove them as decks, keep the simulator code:**

## Remove
- Deck documents: `decklists/JD/tymna-thrasios-partners.md`, `decklists/JD/tymna-kraum-partners.md`,
  `decklists/JD/garland-dimir-kidnapper.md`.
- Their registration: `sim-cli/src/decks.rs` (table rows and MINE), `sim-cli/src/args.rs` if it lists them,
  `data/my_decks.toml` entries, the deck fixture (`tests/fixtures/my_decks_parsed.json`), deck counts in
  `sim-core/tests/game.rs` and the practice tests (`http_catalog.rs`, `http_server.rs`, `server_fake.rs`, catalog),
  then `cardgen export --offline` (data/decks.json) and `cardgen validate`.

## Keep (do not delete)
- The engine and card code: partner commanders (`cmd2` and helpers), the monarch fixes, goad-free combat changes,
  every card model (`impls/tymna_cards.rs`, `impls/garland_cards.rs`, `impls/legends.rs` changes, overrides), the
  `cardgen audit --cards` option, the `--analyze` sections.
- The deck AI modules `impls/tymna_partners.rs` and `impls/garland.rs` and their key-gated hooks: they stay in place
  (inactive without a deck using the key), so the decks can be revived from git history.
- Tests of engine and card behaviour (`tests/partners.rs`, `tests/cards_tymna.rs`, `tests/cards_garland.rs`,
  `tests/tymna_ai.rs`, `tests/tymna_fix.rs`, `tests/cards_legends.rs`): keep them passing. If a test needs a
  registered deck that no longer exists, rewrite it to build its seat with `sim_core::testkit` (setting the seat key
  directly) instead of deleting it; delete a test only if it can't work without the registered deck list, and list
  every deleted test in your report.
- `audit/` results and `rust/agents/` briefs and reports (history).

## Checks
Full `cargo test --release` passes (`http_play` is a known timing flake: rerun alone if it fails once); no warnings;
`all_decks adaptive 3` and `lookahead 1` finish; same-seed check (seed 7, 48 games, lookahead) identical before vs
after for `--deck sauron --pool t3`, `--deck cloud --pool t5`, `--deck sephfs --pool t3`, `--deck seph --pool t3`;
`commander-sim --all-decks` lists 11 decks. Report in `rust/agents/report-drop-decks.md`; final reply at most 15
lines.
