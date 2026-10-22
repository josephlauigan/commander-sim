# Task: karn-practice-abilities — every activated ability in the Karn deck works in practice mode

Read `rust/agents/RULES.md` (skip its `git merge rust-port` step and its difftest/Python lines), then
`rust/crates/sim-core/src/human/human.rs` (`abilities_of`, `use_perm`, `use_land`), `human/cards.rs` (hand-written
abilities for the person: `abilities`, `more_abilities`), `human/abilities.rs` (`land_abilities`), `human/alt.rs`.
Worktree: `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/practice-compiled-abilities`, branch
`practice-compiled-abilities` (PR #130, already open: compiled abilities are now offered to the person).

## Background
JD (10/21) plays the Karn deck (`karn`) in practice mode on an iPad. The audit `tests/play_karn_audit.rs` (run:
`cargo test --release -p sim-core --test play_karn_audit -- --ignored --nocapture`) lists every card whose Oracle text
has an activated ability (beyond plain mana) and what practice mode offers the person, with 20 {C} in the pool on
their own turn. 23 cards offer fewer abilities than their Oracle text:
Bonders' Enclave, Buried Ruin, Endless Atlas, Grim Monolith, Inventors' Fair, Kuldotha Forgemaster, Liquimetal Torque,
Mazemind Tome (its draw), Mishra's Bauble, Mishra's Factory, Mishra's Foundry, Nevinyrral's Disk, Oblivion Stone,
Ratchet Bomb, Soul-Guide Lantern, Steel Hellkite, Steel Overseer, Treasure Vault, Unstable Obelisk, Urza's Bauble,
Urza's Factory, Walking Ballista, War Room. Also Batterskull's "{3}: return to hand" (only Equip is offered).
Most are AI card code whose options appear only when the AI wants them; the person never sees them.

## Do
1. For each card, give the person every activated ability its Oracle text has (`data/scryfall_cache.json`, keyed by
   lower-case name), whenever the rules allow it (timing, "activate only if", costs payable from the pool), with a
   clear label, paid from the person's pool (`human::mana`), with the choices the person makes asked of them (targets,
   which card, X, which permanent to sacrifice), as the existing hand-written ones in `human/cards.rs` do. Reuse the
   card's existing engine effect code; don't duplicate rules. Lands go through `use_land` (`abilities::land_abilities`
   or `alt::land_alts`). Manlands (Mishra's Factory, Foundry) become creatures until end of turn and can attack.
2. Make the audit a real test: set up each card's condition (a power-4 creature for Bonders' Enclave, three same-named
   lands for Endless Atlas, three artifacts for Inventors' Fair and Kuldotha), assert each Oracle ability is offered,
   and keep the table output.
3. For each card, a test that the person uses the ability (scripted answers as in `tests/play_abilities.rs`) and it
   does what the card says.
4. Check the page's JavaScript is untouched (if you change `practice/static/*.js`, syntax-check with `gjs -m`).

`cargo test --release --no-fail-fast` passes (memory cap), no build warnings. Commit (no Co-Authored-By) and push to
the branch (PR #130 picks it up). Report in `rust/agents/report-karn-practice-abilities.md` and a reply of at most
12 lines. Keep `rust/agents/handoff-karn-practice-abilities.md` current.

## Limits
No Python. Builds and tests under `systemd-run --user --scope -q -p MemoryMax=16G -p MemorySwapMax=0` and
`nice -n 19`; `cargo -j 16`. Kill exact PIDs only.
