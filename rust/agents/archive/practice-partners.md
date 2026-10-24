# Task: practice-partners — make two-commander (Partner) decks playable in practice mode

Read `rust/agents/LASTDECK-RULES.md` first: its hard rules (never run Python, memory cap, no wait loops, commit often
with no Co-Authored-By line, never push) and checks apply, with these changes:
- Your worktree is `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/practice-partners`, branch
  `practice-partners` from origin/main (2026-10-17). Don't merge anything in; don't push.
- Builds and tests: `nice -n 19 systemd-run --user --scope -q -p MemoryMax=18G -p MemorySwapMax=0 cargo ... -j 8`.
  Skip the `all_decks` checks and sim runs (none are needed).
- Don't touch `ios/` or `ios-shell/`.

## Why
JD wants every deck playable by a person in practice mode (`rust/crates/practice`, the `commander-practice` server).
All of them are, except the partner deck `kraum` (Tymna the Weaver + Kraum, Ludevic's Opus): `catalog.rs`
`my_deck_files` drops any deck with a `commander2` line ("the practice setup screen shows one commander per deck").
The simulation engine (`sim-core`) already plays two commanders per seat: `Player.cmd` / `cmd2`, separate command
zones (`cmd_zone_of`, `cmds_in_zone`), separate tax (`tax_of`, `add_tax`), commander damage per commander
(`cmd_dmg` / `cmd2_dmg`, `cmd_dmg_from`). The AI and bots already play `kraum`. What's missing is the human side.

## What to do
1. **Catalog / setup:** list partner decks (keep the `my_decks.toml` order). Show both commanders wherever the setup
   screen and the deck list show a commander (name, art, colour identity). `display_name` reads
   `**Commander (1).**`; partner deck files use `**Commanders (2).** Tymna the Weaver, Kraum, Ludevic's Opus`.
2. **Playing as the partner deck (human seat):** both commanders start in the command zone; the person can cast
   either from the command zone when they can pay (each with its own tax); after a commander dies or is exiled the
   person gets the usual choice to move it to the command zone, for each commander; the view shows both commanders'
   zone, tax and commander damage dealt. Find where the human seat's commander actions, prompts and view are built
   (`session/human.rs`, `session/api_impl.rs`, `view.rs`, the web client under `practice/` assets) and extend them to
   `cmd2`; reuse the engine's two-commander helpers rather than duplicating them.
3. **Opponents / bots:** a partner deck as an AI opponent or in the advisor/shadow features must also work (it may
   already; check).
4. **Tests:** update the deck-count / deck-list assertions (`tests/http_catalog.rs`, `http_server.rs`,
   `server_fake.rs` ...) and add tests: the catalog lists `kraum` with both commanders; a session with the human on
   `kraum` can cast Tymna and Kraum from the command zone, pays separate tax on a recast, and returns each to the
   command zone. Use the existing session and HTTP test helpers (`tests/session_common`, `tests/common`).
5. **A real check:** start the practice server the way its README / `main.rs` says, set up a game with `kraum` as the
   human deck, and drive a few turns through the HTTP API (casting both commanders). Record what you did in the report.

## Done when
`cargo test --release -j 8` passes (note any test that also fails on origin/main, e.g. the known `http_play` undo flake,
instead of fixing it), `cargo build --release -j 8 --all-targets` has no warnings, and the step-5 check worked.
Report in at most 15 lines (line 1 DONE or HANDOFF, line 2 your model ID); full report in
`rust/agents/report-practice-partners.md`, committed.
