# Task: karn-engines — model the card draw and engines of JD's Karn deck, and verify its already-modeled cards

Read `rust/agents/RULES.md` (skip its `git merge rust-port` step and its difftest/Python lines: there is no Python
any more; your worktree is based on branch `karn-deck`). Worktree:
`/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/karn-engines`, branch `karn-engines`.

Two other agents work at the same time: `karn-core` (commander, lands, restricted mana, wiring, AI plan) and
`karn-threats` (threats, removal, wipes). Only touch the cards below. Keep shared-file edits small and additive.

The deck (key `karn`) is already in `data/my_decks.toml` and every card is in `data/cards.json`. Read
`decklists/JD/karn-colorless-artifacts.md` for how the deck uses them. Oracle text: `data/scryfall_cache.json` or
`curl -s -m 30 "https://api.scryfall.com/cards/named?exact=<name>"`. Conventions: `rust/PORTING_CARDS.md`; reuse
`impls/common.rs` and existing helpers. New card code goes in a new `impls/karn_engines.rs` (registered in
`impls/mod.rs`) unless a card belongs with an existing family. Record `status`/`note` in `data/card_overrides.toml`
(`Full` only when every Oracle line is modeled; otherwise say what is missing), then `cardgen export`.

## Part 1: new cards (6)
Karn, Scion of Urza (+1: an opponent chooses which of two cards you get, the other is exiled with a silver counter;
−1: return a silver-counter card; −2: Construct token that grows with your artifacts; loyalty from Scryfall);
Mazemind Tome; Mirrorworks; Mystic Forge (cast artifact and colourless spells from the top of the library; the AI
plays the top card when it would cast it from hand); Urza's Blueprints (echo {6}; {T}: draw); Urza's Tome (discard
unless you exile an artifact/legendary card from your graveyard).

## Part 2: verify the deck's cards that are in the engine but not marked Full (20)
Ancient Tomb, Batterskull, Burnished Hart, Endless Atlas (needs three lands with the same name: Wastes), Expedition
Map, Hangarback Walker, Hedron Archive, Inventors' Fair, Liquimetal Torque, Mana Vault, Meteor Golem, Pithing Needle
(marked Partial: finish it if you can), Rogue's Passage, Sol Ring, Solemn Simulacrum, Staff of Domination, Thought
Vessel, Urza's Cave, Wayfarer's Bauble, Wurmcoil Engine. For each: find how it is played today (grep its name;
`data/carddb.txt` tags; DSL in `data/cards.json`), compare with its Oracle text, fix what is wrong or missing with a
test, and record `status`/`note`. In a colourless deck, the land-fetchers find Wastes. Staff of Domination with
infinite mana (Basalt Monolith + Rings of Brighthearth) should draw the AI's library down to a safe size, not deck it.

Tests in `sim-core/tests/` (use `play_common` helpers, see `tests/play_zero_cost.rs`).

## Checks
`cargo test --release -j 7` (whole workspace, `--test-threads=7`) passes; `cargo build --release -j 7
--all-targets` has no warnings. Don't run sims beyond a 50-game smoke run if you need one.

Limits: at most 7 threads, `nice -n 19`, memory cap `MemoryMax=8G` per RULES.md, no Python. Handoff
`rust/agents/handoff-karn-engines.md` after your first card; commit after each few cards (standalone, no
Co-Authored-By); don't push. Report in at most 15 lines; details in `rust/agents/report-karn-engines.md` (per card:
status, what was wrong, what is not modeled).
