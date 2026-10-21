# Task: karn-threats — model the threats, removal and wipes of JD's Karn, Legacy Reforged deck

Read `rust/agents/RULES.md` (skip its `git merge rust-port` step and its difftest/Python lines: there is no Python
any more; your worktree is based on branch `karn-deck`). Worktree:
`/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/karn-threats`, branch `karn-threats`.

Two other agents work at the same time: `karn-core` (commander, lands, restricted mana, wiring, AI plan) and
`karn-engines` (card draw and engines). Only model the cards below. Keep shared-file edits small and additive.

The deck (key `karn`) is already in `data/my_decks.toml` and every card is in `data/cards.json`. Read
`decklists/JD/karn-colorless-artifacts.md` for how the deck uses them. Oracle text: `data/scryfall_cache.json` or
`curl -s -m 30 "https://api.scryfall.com/cards/named?exact=<name>"`. Conventions: `rust/PORTING_CARDS.md`; reuse
`impls/common.rs` and existing helpers (grep for a similar card first: many effects already exist for other decks).
Put new card code in a new `impls/karn_cards.rs` (registered in `impls/mod.rs`) unless a card belongs with an
existing family. Record `status`/`note` in `data/card_overrides.toml` for each card (`Full` only when every Oracle
line is modeled; otherwise say what is missing), then `cardgen export`.

## Cards (19)
All Is Dust; Blightsteel Colossus (infect, indestructible, shuffle-back); Cityscape Leveler (cast trigger, attack
trigger, Powerstone token for the victim, unearth); Darksteel Forge; Duplicant; Kuldotha Forgemaster; Liberator,
Urza's Battlethopter (flash for artifact/colourless spells: the AI may cast them at instant speed); Mishra's
Self-Replicator (copies on historic spells); Nevinyrral's Disk; Oblivion Stone (fate counters); Platinum Angel;
Portal to Phyrexia; Ratchet Bomb; Spine of Ish Sah; Steel Hellkite; Steel Overseer; Triplicate Titan; Ugin, the
Ineffable (cost reduction for colourless spells, +1 and −3; loyalty from Scryfall); Unstable Obelisk.

For each card: code (or DSL/override if the ability language covers it), an AI decision where the card needs one
(when to activate Disk / O-Stone / Ratchet Bomb, Steel Hellkite's X, Forgemaster's target, Ugin's choices), and a
test in `sim-core/tests/` (use `play_common` helpers, see `tests/play_zero_cost.rs`). Also check the Basalt Monolith
+ Rings of Brighthearth infinite-mana combo is played by the AI with Walking Ballista as the outlet (Rings and
Basalt exist for Sauron; grep), and that infect damage (Blightsteel) kills at 10 poison.

## Checks
`cargo test --release -j 7` (whole workspace, `--test-threads=7`) passes; `cargo build --release -j 7
--all-targets` has no warnings. Don't run sims beyond a 50-game smoke run if you need one.

Limits: at most 7 threads, `nice -n 19`, memory cap `MemoryMax=8G` per RULES.md, no Python. Handoff
`rust/agents/handoff-karn-threats.md` after your first card; commit after each few cards (standalone, no
Co-Authored-By); don't push. Report in at most 15 lines; details in `rust/agents/report-karn-threats.md` (per card:
status and what is not modeled).
