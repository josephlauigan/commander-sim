# Task: omnath-deck, add JD's new Omnath, Locus of Rage deck (key `omnath`)

Read `rust/agents/RULES.md` (skip its `git merge rust-port` step: the worktree is on main). Worktree:
`/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/omnath-deck`, branch `omnath-deck`.

JD is turning his mono-green Titania landfall deck into a red-green deck led by **Omnath, Locus of Rage**, with
Titania in the 99. The list is final: `rust/agents/omnath-list.txt` (100 lines, one card per line, basics repeated).
**Add it as a new JD deck alongside `titania`** (Titania stays for now; it is retired later, once JD has seen the
comparison). Model the change on how the Titania deck was added: commit `3c1830e` ("Titania deck: list and engine
wiring") and `git show --stat 3c1830e` (my_decks.toml, decks.json via cardgen export, the deck doc, practice
`catalog.rs` and its tests, `sim-cli/src/decks.rs` and `args.rs`, `ai/decks.rs` / `ai/mod.rs` / `ai/brain.rs`,
`settings.rs`, `impls/mod.rs`, the `my_decks_parsed.json` fixture). Grep for `"titania"` across `rust/` to find
every place a deck key is listed (a key list missing `omnath` refuses the deck or gives it no AI plan).

## Do
1. **The deck:** `data/my_decks.toml` entry `[omnath]` with `tiers = ["t1", "t2", "t3"]` (Titania's; re-set after the
   comparison), commander Omnath, Locus of Rage, colour identity RG. The doc `decklists/JD/omnath-landfall.md`,
   written like `decklists/JD/titania-lands.md` (current list and strategy only; history goes in
   `decklists/CHANGELOG.md`): what the deck does (Omnath makes a 5/5 Elemental on each landfall; each Elemental you
   control that dies deals 3 damage; Titania's 5/3s are Elementals; Goblin Bombardment, Terror of the Peaks, Wrenn and
   Six, Harrow, Oracle of Mul Daya), key lines, the bracket (3; 1 Game Changer: Worldly Tutor), the import list and
   the by-type section. Add a short **"Cards to confirm"** note: Terror of the Peaks, Blasphemous Act, Birds of
   Paradise, Command Tower, Exotic Orchard come from JD's spares and each also appears in another deck: JD is checking
   he owns a second copy. Use `cardgen deck` where it helps (it writes the import list and by-type section and checks
   legality, singleton, identity, bans and Game Changers), then `cardgen export`.
2. **Wiring:** every deck-key list gets `omnath` where it has `titania`. **The AI plan:** Titania's deck plan
   (`impls/titania.rs`, and whatever is keyed on `"titania"` in `ai/`, `engine/`, `cardcode.rs`) should also apply
   to `omnath` (same landfall / sacrifice-land / fetch-land plan; Titania herself is in the 99). Make it shared by
   key list, not copied. Omnath, Locus of Rage, Goblin Bombardment, Oracle of Mul Daya and Harrow are in
   `data/cards.json`; check their model status (`data/card_overrides.toml`); if Omnath's "Elemental dies: 3 damage" or
   landfall token is missing or partial, model it (Oracle text from `data/scryfall_cache.json` or Scryfall,
   `curl -s -m 30`), with a test. Add a cast-priority rule for Omnath's deck if the shared plan needs one (Goblin
   Bombardment as a sac outlet for the Elementals only when it gains: Omnath's 3 damage).
3. **Printings:** `data/printings/omnath.txt`, like the other printings files: the cards from Titania's list keep
   their printings from `data/printings/titania.txt`; the collection cards take the set JD's lists give
   (`bulk-cards.txt` / `binder-cards.txt`, `1x Name (set)`; use Scryfall for a collector number); the four new buys
   (Omnath, Locus of Rage; Goblin Bombardment; Oracle of Mul Daya; Harrow) get a common printing with a comment
   `# placeholder until JD's copy is known`.
4. **Bulk and binder:** do NOT move cards between the bulk/binder files and the deck yet (that happens when Titania
   is retired). Note in your report which cards would move.
5. **Practice mode:** the deck shows in the practice catalog and can be started (practice's catalog tests).

## Checks
`cargo test --release -j 4` (whole workspace) passes; `cargo build --release -j 4` has no warnings;
`./target/release/commander-sim --deck omnath --pool t1 --games 100 --ai adaptive --profile loose --jobs 4` runs and
the AI casts Omnath (check `--cards` for unmodeled cards, and the run's card report). Don't run longer sims: I will.

Limits: at most 4 build threads (`-j 4`, `--test-threads=4`), `nice -n 19`, memory cap per RULES.md, no Python.
Handoff `rust/agents/handoff-omnath-deck.md` after your first step; commit after each piece (standalone, no
Co-Authored-By); don't push. Report in at most 15 lines; details in `rust/agents/report-omnath-deck.md`.
