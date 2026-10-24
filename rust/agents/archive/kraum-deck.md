# Task: kraum-deck — bring back the Tymna/Kraum deck (key `kraum`) and model its new cards

Read `rust/agents/LASTDECK-RULES.md` first: its hard rules, modeling method and checks apply, with these changes:
- Your worktree is `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/tymna-kraum`, branch `tymna-kraum`
  (from main). The deck document is already written: `decklists/JD/tymna-kraum-partners.md` (don't change its
  strategy text; `cardgen deck` may rewrite its list sections). Don't merge anything in; don't push.
- **The machine is shared:** every build and test runs as
  `nice -n 19 systemd-run --user --scope -q -p MemoryMax=18G -p MemorySwapMax=0 cargo ... -j 6`; smoke runs use
  `--jobs 6`. Skip the `all_decks` checks. No sim runs beyond the smoke run below.

## Why
JD wants the Tymna the Weaver + Kraum, Ludevic's Opus partner deck back in the simulator, now with Gilgamesh,
Master-at-Arms, Diabolic Tutor and an Equipment package, to find its weaknesses. The deck was registered on
2026-10-13 and unregistered in commit cad27f7 (2026-10-14), which kept the engine, card code and AI modules
(`impls/tymna_partners.rs`, `impls/tymna_cards.rs`, `ai/decks.rs` arms for `kraum`) and turned `kraum` into a
"retired" test deck in `sim-core/src/testkit.rs`.

## 1. Register the deck again
Undo cad27f7's unregistering for `kraum` only (not thrasios or garland): `git show cad27f7` shows every place.
- `data/my_decks.toml`: a `[kraum]` entry (file `JD/tymna-kraum-partners.md`, name `Tymna/Kraum`, commander
  `Tymna the Weaver`, commander2 `Kraum, Ludevic's Opus`, ident `WUBR`), with **provisional** `tiers = ["t1", "t2"]`
  (JD will set the real tiers after the analysis run).
- `sim-cli/src/decks.rs` FILES and MINE (put `kraum` after `cloud`), `sim-cli/src/args.rs` DECK_KEYS, and every
  hard-coded deck count or key list in tests (`practice/tests/http_catalog.rs`, `http_server.rs`, `server_fake.rs`,
  `sim-core/tests/game.rs`, the `mine.len()` test in decks.rs). Check whether practice mode should list a partner
  deck (the 10/13 notes say practice mode hides partner decks: keep that behaviour).
- `testkit.rs`: take `kraum` out of `retired_decks()` so the real deck is used. Tests that used the retired kraum
  seat (`tests/tymna_ai.rs`, `tymna_fix.rs`, `cards_tymna.rs` ...) must still pass: if one depended on the basic-land
  library, fix the test setup, not the behaviour, and say so in the report.
- Then `rust/target/release/cardgen deck kraum <list>` with the deck's own import list (it validates the list, writes the
  parsed fixture `tests/fixtures/my_decks_parsed.json`) and `cardgen export`. Don't add a `--log` (the changelog entry
  exists).

## 2. Model the new cards
Run `rust/target/release/cardgen audit --mine | sed -n '/== kraum/,/^== /p'` and make every card of the deck **Full**
(Razia is no longer in the deck). The new cards are Gilgamesh, Master-at-Arms; Diabolic Tutor; Conqueror's Flail;
Trailblazer's Boots; Medic's Kitesail; Nim Deathmantle; Dragoon's Lance; Samurai's Katana; Hero's Heirloom; Hero's Blade;
Irrigated Farmland; Spectacle Summit; Boiling Rock Prison. Several are already modeled for the Cloud deck (equipment,
job select, landwalk; grep `impls/cloud.rs`, `impls/bulk_white.rs`, `dsl/`): reuse, don't copy. Oracle text:
`jq '.["name lowercased"]' data/scryfall_cache.json`. Put new card code in `impls/tymna_cards.rs`, tests in
`tests/cards_tymna.rs`.

Points to get right:
- **Gilgamesh:** when he enters or attacks, look at the top six; put any number of Equipment cards among them onto the
  battlefield; the rest on the bottom in a random order; then you may attach one of them to a **Samurai** you control
  (Gilgamesh himself, or a creature equipped with Samurai's Katana, which makes it a Samurai).
- **Conqueror's Flail:** +1/+1 per colour among permanents you control; while attached, opponents can't cast spells
  during your turn (check the engine's casting/response code honours it for opponents' instants on your turn).
- **Hero's Blade:** attaches for free whenever a legendary creature you control enters (most of the deck's creatures
  are legendary).
- **Diabolic Tutor:** give the kraum AI a sensible pick (look at the existing tutor-pick code in `ai/decks.rs`): the
  card the game needs (a sweeper when behind on board, a finisher or Conqueror's Flail when ahead, a land when short).

## 3. Check the deck AI (fix in the kraum code if small; otherwise report)
1. **Equipment goes where it creates a Tymna hit:** evasion Equipment (Trailblazer's Boots, Medic's Kitesail,
   Dragoon's Lance, Nim Deathmantle) onto creatures that can't get through on their own, not onto fliers that already
   connect. Equipping costs mana twice; check it doesn't starve casting creatures in the early turns.
2. **Tymna's attack spread still works** with the new cards (each opponent hit by at least one creature).
3. **Conqueror's Flail before the big attack**, and the AI doesn't hold Equipment in hand forever (stuck rate).

## Done when
`cargo test --release -j 6` passes, `cargo build --release -j 6 --all-targets` has no warnings, every card of the deck
is Full in the audit, and a smoke run finishes:
`nice -n 19 systemd-run --user --scope -q -p MemoryMax=18G -p MemorySwapMax=0 rust/target/release/commander-sim --deck
kraum --pool t1 --profile loose --games 40 --ai lookahead --jobs 6 --brief`.
Report in at most 15 lines (line 1 DONE or HANDOFF, line 2 your model ID); full report, including the answers to the
three AI checks and every test you had to change, in `rust/agents/report-kraum-deck.md`, committed.
