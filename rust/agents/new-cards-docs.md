# Task: new-cards-docs — apply JD's five new-card swaps to the lists and deck docs

JD has made (or is making) these changes to his physical decks. Make the repo match. Work directly on branch
`new-cards-1018` in `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/new-cards` (no other worktree).

| Deck (key, doc) | Out (goes to `bulk-cards.txt`) | In |
|---|---|---|
| Kefka (`kefka`, `decklists/JD/kefka-grixis-discard.md`) | Feed the Swarm | Marvin, Murderous Mimic |
| Seph/Atraxa (`seph`, `decklists/JD/sephiroth-phyrexian-reanimator.md`) | Dread Return | Victimize |
| Tymna/Kraum (`kraum`, `decklists/JD/tymna-kraum-partners.md`) | Elemental Mascot | Lightning, Army of One |
| Cloud (`cloud`, `decklists/JD/cloud-midgar-equipment.md`) | Citizen's Arrest | Stroke of Midnight |
| Cloud | Hero's Blade | Andúril, Narsil Reforged |

Do **not** touch Titania, Sauron, Y'shtola or Cecil (not changing).

## How (follow the last approved swap commit, `git show 9eb627e`, for which files move together)
- Change each list with `cardgen deck KEY LIST --log "Why."` (see `rust/crates/cardgen/README.md`); the log line
  goes to `decklists/CHANGELOG.md`. Why-lines (one sentence each, from the coarse sim pass 10/18, loose, 500 games
  per list per tier, all within noise): Marvin +0.2 at T3 (copies Niv-Mizzet's draw, Fangtail, Blightspeaker, Rummaging
  Goblin); Victimize +0.8 over T3-T4 (two creatures back for 3 mana, wins 77-82% when cast); Lightning -2.8 at T5
  (noise; JD's choice); Stroke of Midnight -1.4 over T1-T3 (instant answer to any nonland permanent); Andúril -0.3
  over T1-T3 (a +1/+1 counter on every creature each swing). Numbers: `audit/<deck>/FINDINGS.md`.
- Deck docs describe only the current list (no history, no buy lists; history lives in CHANGELOG.md): update the
  card list, the type counts / section totals, and every strategy paragraph that names a card out; describe the new
  card where its role fits (Oracle text in `data/scryfall_cache.json` or `data/cards.json`). Keep each deck at 100.
- `bulk-cards.txt`: add the five cards out (match the file's format). `binder-cards.txt`: remove an incoming card
  only if it is listed there.
- `data/printings/<key>.txt`: replace the out card's line with the new card. JD's exact printing is unknown: use
  the card's Scryfall default printing and list each one in your report so JD can correct it.
- Re-export (`cargo run --release -p cardgen -- export` from `rust/`) and update any fixture the tests compare
  (e.g. `tests/fixtures/my_decks_parsed.json`) the way 9eb627e did.

## Rules
No Python. Heavy commands under `systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0 nice -n 19`,
cargo `-j 4`. No sims. Commit (no Co-Authored-By line), don't push. Checks: `cargo test --release` passes;
`cardgen validate` (or the export) reports each list at 100 cards. Report ≤15 lines (commit, files changed, printings
used, anything unclear); details in `rust/agents/report-new-cards-docs.md`.
