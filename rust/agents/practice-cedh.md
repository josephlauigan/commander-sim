# Task: practice-cedh, partner names and commander pictures on the setup screen (+ `make ios-clean`)

Read `rust/agents/RULES.md` (skip its `git merge rust-port` step). Worktree:
`/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/practice-cedh`, branch `practice-cedh` (from main).

JD plays practice mode on the iPad (`ios-shell/`, the same Rust server and page as `commander-practice`). On the setup
screen's cEDH tier (t6) he sees two decks both called "Rograkh", and not every deck has its commander picture.

## 1. Partner pool decks are named after their first commander only
`catalog::catalog` (`rust/crates/practice/src/catalog.rs`, the `tiers` part, about line 215) names each tier deck
`short_name(commander)`, and gives the page only `commander`. Three Tier 6 decks have a partner (`commander2` in
`data/decks.json` `pool`): Rograkh + Silas Renn (`rograkh-silas-turbo-breach`), Rograkh + Thrasios
(`rograkh-thrasios-creature-combo`), Kraum + Tymna (`kraum-tymna-blue-breach-midrange`). Name them like JD's own partner
deck ("Tymna the Weaver and Thrasios", see `display_name`): "Rograkh and Silas Renn", "Rograkh and Thrasios",
"Kraum and Tymna the Weaver" (`short_name` of each; keep one style). Give each tier deck `commander2` and
`commanders` like JD's decks have. The page (`rust/crates/practice/static/app.js`, `renderSetup` and `opponentCard`)
shows both commanders' pictures for a partner deck, the way `deckImages(d)` does for JD's decks. Anywhere else the
tier deck name is shown (opponent cards, the table, the review) should read right too: grep for where the pool deck's
display name comes from.

## 2. Commander pictures for decks not played yet
`Site::options` (`server/routes.rs`) only returns pictures already in the image cache (`Images::cached`,
`cached_printed`); pictures are downloaded when a game starts (`Images::prepare_online`). So decks never played in
this install (often all of Tier 6) have no picture. `catalog::commanders` also leaves out the tier decks'
`commander2`. Fix both: include every commander (both partners) in `commanders`, and when the setup screen asks for
its options and some commander pictures are missing, start ONE background download of the missing ones
(`prepare_online` with just those names, plus the commander printings of JD's decks) without blocking the reply; the
page shows them on its next options fetch (a re-render after a short delay, or when the download finishes, whatever
fits the page's existing polling). Offline (no network, as on the iPad without Wi-Fi) it must fail quietly and the
page keeps drawing the card from its name. Never start a second download while one runs. Tests: the naming
(catalog unit test), `commanders` including partners, and the background fetch with a fake lookup/download (see how
the existing image tests fake Scryfall: `PRACTICE_SCRYFALL` or the `lookup`/`download` parameters).

## 3. `make ios-clean`
The root `Makefile` has `make clean` (`cargo clean` only). Add `ios-clean`: removes `ios-shell/build`,
`ios-shell/AppData`, `ios-shell/Resources` (old name) and the generated `ios-shell/CommanderPractice.xcodeproj`, and
says that `ios-shell/run.sh` rebuilds everything. Add it to the Makefile's help text and to `ios-shell/README.md`
("Build it on your Mac": a line on starting clean: `make ios-clean`, then `ios-shell/run.sh`; and in Xcode,
Product > Clean Build Folder). Also fix the README checklist line that says the setup screen lists "your 12 decks":
it is 13.

## Checks
`cargo test --release -j 6` (whole workspace) passes; `cargo build --release -j 6` no warnings. Run
`commander-practice --port 8792 --no-browser` (under the memory cap, in the background, stop it after) and
`curl -s http://127.0.0.1:8792/api/options`: the t6 decks have distinct names and `commander2`; with network, a second
call a little later lists more cached commander images than the first. Don't run sims.

Limits: `-j 6`, `nice -n 19`, memory cap per RULES.md, no Python. Handoff `rust/agents/handoff-practice-cedh.md`;
commit after each piece (standalone, no Co-Authored-By); don't push. Report in 15 lines or fewer; details in
`rust/agents/report-practice-cedh.md`.
