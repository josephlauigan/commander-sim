# Task: kraum-phyrexian — add and model four Phyrexian-mana cards for the Tymna/Kraum deck

Read `rust/agents/LASTDECK-RULES.md` first: its hard rules, modeling method and checks apply, with these changes:
- Your worktree is `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/kraum-phyrexian`, branch
  `kraum-phyrexian` (from `tymna-kraum`). Don't merge anything in; don't push. Don't change any deck list.
- **The machine is busy with sims:** every build and test runs as
  `nice -n 19 systemd-run --user --scope -q -p MemoryMax=12G -p MemorySwapMax=0 cargo ... -j 4`. No sim runs except
  the smoke run below (`--jobs 4`). Skip the `all_decks` checks.

## Why
JD wants to test a "Phyrexian mana" package in the `kraum` deck (Tymna the Weaver + Kraum, W/U/B/R): cheap cards that
can be cast by paying 2 life instead of a coloured mana. These four are not in the card database
(`commander-sim --swap` refuses them: "Not in data/cards.json"). Add them and make each **Full** in
`cardgen audit --cards <file>`, with at least one test per card, from its Oracle text (fetch from Scryfall's API if the
local cache lacks it; see how earlier agents added missing cards: `git log --all --oneline -S'not in the card database'`,
the tymna-cards and cloud agents' reports in `rust/agents/report-*.md`).

## Cards
- **Dismember** ({1}{B/P}{B/P} instant): target creature gets -5/-5 until end of turn. AI: kill the biggest threat;
  pay life for the {B/P} when black mana is short and life allows (look at how Norn's Annex / Gitaxian Probe pay
  Phyrexian mana: grep `W/P`, `U/P`, `phyrexian` in `rust/crates/`).
- **Porcelain Legionnaire** ({2}{W/P} artifact creature, 3/1 first strike).
- **Vault Skirge** ({1}{B/P} artifact creature, 1/1 flying, lifelink).
- **Apostle's Blessing** ({1}{W/P} instant): target artifact or creature you control gains protection from artifacts or
  from the colour of your choice until end of turn. AI: cast it in response to removal aimed at a commander (Tymna or
  Kraum) or the best creature, choosing the removal's colour; reuse the existing response-protection hooks (grep
  `protect_main`, Gods Willing, Loran's Escape, Take Up the Shield).

Put the card code in `impls/tymna_cards.rs`, tests in `tests/cards_tymna.rs`. Run `cardgen export` after adding data.

## Done when
`cargo test --release -j 4` passes, `cargo build --release -j 4 --all-targets` has no warnings, all four cards are Full
in `cardgen audit --cards`, and a smoke run with them swapped in finishes:
`nice -n 19 systemd-run --user --scope -q -p MemoryMax=12G -p MemorySwapMax=0 rust/target/release/commander-sim --deck kraum
--pool t3 --profile loose --games 40 --ai lookahead --jobs 4 --swap "Memory Trap=>Dismember" --swap "Elemental
Mascot=>Porcelain Legionnaire" --swap "Tocasia's Welcome=>Vault Skirge" --swap "Disenchant=>Apostle's Blessing"`.
Report in at most 15 lines (line 1 DONE or HANDOFF, line 2 your model ID); full report in
`rust/agents/report-kraum-phyrexian.md`, committed.
