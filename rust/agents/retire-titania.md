# Task: retire-titania, Omnath replaces the Titania deck (key `titania` goes away)

Read `rust/agents/RULES.md` (skip `git merge rust-port`). Worktree:
`/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/omnath-deck`, branch `omnath-deck` (the Omnath deck
is already added there; see `rust/agents/report-omnath-deck.md` and `audit/omnath/FINDINGS.md`).

JD retires the mono-green **Titania** deck: its cards become the **Omnath** deck (key `omnath`). Titania,
Protector of Argoth stays as a card in Omnath's 99, and Titania's landfall plan (`impls/titania.rs`, now shared via
`is_deck()`) keeps working for `omnath`. Only the **deck** `titania` goes. Look at how Zada was retired for the
shape (commits `ac7f103`, `e4d8039`: key lists, `my_decks.toml`, docs, fixtures, test counts), but unlike Zada,
**keep all card code**: no card or plan is deleted.

## Do
1. **Deck key:** remove `titania` from `data/my_decks.toml`, then `cardgen export` (decks.json) and the
   `my_decks_parsed.json` fixture through cardgen's own commands; remove `titania` from every deck-key list in
   `rust/` (grep `"titania"`: practice catalog, sim-cli, ai, settings, engine). `impls/titania.rs`: `is_deck()` keeps
   working for `omnath`; the `titania`-key-only paths (Titania as the commander) can stay if harmless, but say in the
   report what you kept and why. Rename nothing else.
2. **Tests:** tests that seat the `titania` deck (`table(&["titania", ...])`, `play(...)`) move to `omnath` where the
   test is about Titania's cards or the landfall plan (the cards are in Omnath's list too); tests about Titania as the
   commander (cast from the command zone, commander tax) either use a hand-built seat or are dropped with a note.
   Deck counts go back: 14 -> 13 (practice tests, `game.rs` 98 -> 91 "thirteen decks").
3. **Docs:** `decklists/JD/titania-lands.md` is removed; its history goes into `decklists/CHANGELOG.md` (Titania's
   section: a "Retired 2026-10-19: became the Omnath deck" line, keeping the past change entries). `decklists/JD/omnath-
   landfall.md` gets a one-line "Built from the Titania deck (2026-10-19)" note. Update any README / `documents/` list
   of JD's decks, `audit/titania/FINDINGS.md` gets a final "Retired: see audit/omnath/FINDINGS.md" line (don't delete
   it). The root `CLAUDE.md` "Opponent tiers" sentence names the decks and their tiers: replace titania with omnath
   (t1-t3).
4. **Printings:** remove `data/printings/titania.txt` (omnath.txt has the printings).
5. **Physical cards** (`bulk-cards.txt`, `binder-cards.txt`, `1x Name (set)` lines, sorted): the cards cut from
   Titania's list go **to bulk** (Hard-Hitting Question, Diplomatic Relations, Thirsting Roots, Tenured Concocter,
   Glorious Decay, Crushing Canopy, Studious First-Year, Spider-Man Brooklyn Visionary, Infestation Expert, Pick Your
   Poison, Chelonian Tackle, Knockout Maneuver, and 18 Forests), using the set from `data/printings/titania.txt`; the
   collection cards now in Omnath's list come **out of** bulk/binder (Wrenn and Six, Terror of the Peaks, Decimate,
   Scorching Dragonfire, Disintegrate, Blasphemous Act, Birds of Paradise, Fyndhorn Elder, Hellkite Tyrant, Command
   Tower, Exotic Orchard, Cinder Glade, Karplusan Forest, Rootbound Crag), one copy each, and 12 Mountains if bulk lists
   Mountains. Report every line you changed. The four new buys are not in either file (JD is buying them).

## Checks
`cargo test --release -j 4` (whole workspace) passes; `cargo build --release -j 4` no warnings; `commander-sim --deck
titania ...` is refused (no such deck); `--deck omnath --pool t1 --games 50 --ai adaptive --profile loose --jobs 4`
runs. Practice catalog lists 13 decks with omnath. Limits: `-j 4`, `nice -n 19`, memory cap, no Python, no other sims.
Run long commands in the foreground with a timeout. Handoff `rust/agents/handoff-retire-titania.md` early; commit
after each piece; don't push. Report in 15 lines or fewer; details in `rust/agents/report-retire-titania.md`.
