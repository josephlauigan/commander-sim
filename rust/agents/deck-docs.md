# Task: bring JD's deck documents up to date with the current lists and strategies

Worktree: `/home/joseph/projects/commander-sim/.claude/worktrees/deck-docs`, branch `deck-docs`, based on origin/main
(51d4d31a). `cd` into it for every command. Commit there; do not push, do not open a PR.

## Which rules apply
Read the repo's `CLAUDE.md` and `AGENTS.md`. Never run Python, not even a version check: a hook blocks it. This task is
already approved by JD; work through it without asking.
- **Memory cap** on any cargo or binary command: `systemd-run --user --scope -q -p MemoryMax=12G -p MemorySwapMax=0
  <cmd>`. The shell is zsh, so write the cap out in full.
- **Run no simulations.** No `Co-Authored-By:` line in commits.
- **Stay in your area.** Another agent is editing `data/plans/urdragon.toml` and Rust combo code on another branch,
  so don't touch either.

## The documents
They're the files that `data/my_decks.toml` lists under `file`: JD's nine in `decklists/JD/` and Avery's two in
`decklists/Avery/`. Each has strategy prose, then the list sections (`## Decklist by type`, `## Import list` or
similar). **`cardgen export` builds `data/decks.json` from these files, so the lists must not change**: no card in or
out, no count change. Treat the list sections as read-only unless a list section is malformed; if it is, report it
rather than fixing it.

The current lists are the `cards` arrays in `data/decks.json` (`.mine[]`, by key). Card text is in
`data/scryfall_cache.json`. Recent changes per deck are in `decklists/CHANGELOG.md` and `git log -- decklists/JD`.

## Work
1. **Card names in the prose.** For every document, check each card named in the strategy prose against that deck's
   current list.
   - Fix stale mentions: a card described as if it were in the deck when it isn't.
   - Keep mentions that are clearly about something else: opponents' cards, cards a Garth copy creates, a package that
     moved to another deck, cards to watch out for.
   
   A first scan found these candidates:
   - Sauron: Hardened Scales.
   - Y'shtola: Counterspell, Massacre Wurm.
   - Kefka: Blazing Firesinger, Seething Song.
   - Karn: Austere Command, Back to Basics, Blood Moon, Collector Ouphe, Vandalblast.
   - Thanos: about 22 names, likely mostly opponents' hate pieces; check each.
   - Ur-Dragon: Disenchant and Helm of the Host; probably fine, because they're Garth's copy and the package that
     moved to Sauron.
2. **Strategies reflect recent swaps.**
   - **Kefka:** never mentions Palantír of Orthanc (added 11/01).
   - **Check the rest:**
     - Sauron's Godo package (from 11/03): Godo, Helm of the Host, Combat Celebrant, Hammer of Nazahn, Peter Parker's
       Camera; the infinite-combat finish.
     - Cloud: Stoneforge Mystic, Steelshaper's Gift, Judgment of Alexander, Forge Anew.
     - Sheoldred: Raise Dead.
     - Tymna/Thrasios: Natural Order.
     - Kefka: Urborg, Tomb of Yawgmoth.
     - Y'shtola: Closet.
   
   Fix anything that describes a removed card or an older plan. Keep each document's voice and structure. Make
   targeted edits, not rewrites. Keep the bracket and Game Changer statements correct for the current list.
3. **The Ur-Dragon document.**
   - **Add a `## Combos` section** after `## Key lines`, covering the three infinite combos that Commander Spellbook
     finds in the deck (`data/spellbook/urdragon.json`, `included`):
     - Astral Dragon + Cursed Mirror;
     - Miirym, Sentinel Wyrm + Astral Dragon + Doubling Season;
     - Garth One-Eye + Twinflame + Spark Double + Doubling Season.
     
     For each one, give:
     - the pieces;
     - the exact sequence step by step, worked out from the cards' Oracle text and checked against the Spellbook
       variant's `cards`, `results` and `prerequisites`;
     - the mana needed and what it produces;
     - how to turn it into a win;
     - which tutors find the missing piece;
     - how to protect the turn with the deck's own protection.
     
     Don't invent rulings. If a step depends on a choice (a Mirror copying Astral Dragon "except it has haste"), say
     so.
   - **Update `## Key lines`** so they point at the combos. The current "Astral Dragon target" advice ignores the
     Cursed Mirror loop.
   - **Rename** `decklists/JD/ur-dragon-equipment.md` → `decklists/JD/ur-dragon-dragon-copies.md` with `git mv`.
     Update the live references: `data/my_decks.toml` (`file`), `rust/crates/sim-cli/src/decks.rs`, and anything else
     that reads the path at runtime or in tests (grep for it). Leave history records as they are: `CHANGELOG.md`,
     `audit/*/FINDINGS.md`, `rust/agents/*.md`. **Don't edit `data/plans/urdragon.toml`**; the other branch updates
     its comment.
4. **Avery's two documents.** Their lists haven't changed recently, so only check for stale names, and report rather
   than restyle.

## Guard
After editing, rebuild cardgen (`cargo build --release -p cardgen` from `rust/`, under the cap) and run
`./target/release/cardgen export --offline` from `rust/`. **`git diff --stat data/` must show no change to
`data/decks.json` or `data/cards.json`.** If decks.json changed only because the file path moved, report exactly what
changed. Then run `cargo test --release --no-fail-fast` (under the cap): it must pass with no failures.

## Finish
- **Commits:** everything committed, with `rust/agents/handoff-deck-docs.md` kept current while you work.
- **Report:** if you can't write `rust/agents/report-deck-docs.md`, put the full report in your final message. List,
  per document, what changed and why, and every name you kept on purpose with the reason.
- **Final message:** at most 15 lines plus the full report if needed: commit hash, the export guard result, test
  result, anything the coordinator must act on. Then stop.
