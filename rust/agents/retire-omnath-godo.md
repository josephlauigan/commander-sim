# Task: retire-omnath-godo, JD's decks `omnath` and `godo` go away (14 -> 12)

Read `rust/agents/RULES.md` (skip `git merge rust-port`, Python and difftest: they no longer apply). Worktree:
`/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/retire-omnath-godo`, branch `retire-omnath-godo-rust`
(from `retire-omnath-godo`).

JD (10/25) built The Ur-Dragon (`decklists/JD/ur-dragon-equipment.md`, not in the sim) from Omnath's cards and the
never-built Godo list, and retires both decks. **Already done on this branch** (commit `79d92e6f`, don't redo): the
two deck files and `data/printings/omnath.txt` removed, `data/my_decks.toml`, `data/decks.json` (re-exported, 12 decks),
the `tests/fixtures/my_decks_parsed.json` fixture, CHANGELOG, CLAUDE.md, README, `documents/testing-decks.md`, FINDINGS
notes, bulk. What's left is `rust/`: the tests now fail because they expect 14 decks or seat `omnath` / `godo`.

Shape to follow: the Kraum retirement, commit `4bcea2ff` ("key lists 14 -> 13, tests seat thrasios") and its brief
`git show 909e10a5:rust/agents/retire-kraum.md`. **Keep all card code** (Omnath, Titania, the landfall cards, Godo,
Helm of the Host, `impls/t6_godo.rs`, the Tier 6 plan entries): only the deck-key wiring goes. Say what you kept.

## Do
1. **Deck-key lists:** remove `omnath` and `godo` from every list of JD's decks in `rust/` (grep `"omnath"`, `"godo"`:
   `practice/src/catalog.rs`, `practice/src/session/setup.rs`, `sim-cli/src/decks.rs` (`MINE`, the file table),
   `sim-cli/src/args.rs`, `sim-cli/src/poolmode.rs`, `sim-cli/src/report.rs`, `sim-core/src/settings.rs`,
   `sim-core/src/ai/mod.rs` styles, `ai/decks.rs`, `ai/brain.rs`, `engine/cast.rs`, `impls/rules.rs`, `t6stats.rs`,
   `practice/src/advisor/mulligan.rs`). Array lengths 14 -> 12. Gates like `"omnath" | ...` on card behaviour may
   stay if harmless; remove dead deck-only branches where that's simple.
2. **Tests that count or list decks:** update to 12 (practice `app_entry`, `http_catalog`, `http_server`,
   `server_fake`, `session_partners`, sim-cli `poolmode` tier test, sim-core `game.rs` (played = 12 x 7 = 84)).
3. **Tests that seat `omnath` or `godo`** (about 40 files, many Tier 6 tests seat `godo`): first check whether the
   testkit can still load a retired list by key. The pool keeps retired test decks in `decklists/pool/retired/`
   (e.g. Glarb, Heliod, Winota: see commits `4987e79c`, `3b380efd` and how `decks()` / `tier_of` find them). If that
   works for JD-style decks, restore the two lists there as retired test decks (from `git show
   79d92e6f^:decklists/JD/omnath-landfall.md` and `godo-cedh-equipment.md`) so the tests keep seating them without
   them being playable decks; otherwise move each test to a hand-built seat or another key whose list has the cards,
   and drop only tests that can't be kept, listing each in the report. Tests must still test the same card
   behaviour.
4. **Docs in `rust/` and `documents/how-it-works.md`** that list JD's decks or say "seven Tier 6 decks (... and
   `godo`)": update.

## Checks
`cargo test --release -j 6 --no-fail-fast` (whole workspace) passes; `cargo build --release -j 6 --all-targets` has no
warnings; `commander-sim --deck omnath ...` and `--deck godo ...` are refused; `--deck cloud --pool t1 --games 50
--jobs 6` runs; `cardgen export` leaves `data/` unchanged (`git status`). Practice lists 12 decks. Limits: `-j 6`,
`nice -n 19`, memory cap (`systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0`), no Python, no other
sims. Handoff `rust/agents/handoff-retire-omnath-godo.md` early; commit after each piece (standalone, no
Co-Authored-By); don't push. Report in 15 lines or fewer; details in `rust/agents/report-retire-omnath-godo.md`.
