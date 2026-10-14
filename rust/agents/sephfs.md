# Task: sephfs — register, model and plan the Sephiroth, Fabled SOLDIER deck (`sephfs`)

Read `rust/agents/TYMNA-RULES.md` (where it says thrasios/kraum, read sephfs; it points to OVERNIGHT-RULES) first.
Base branch: `tymna-thrasios` (the commit that added this brief). Your worktree's branch: `sephfs`. You are the only
agent running: `cargo -j 16`, `--jobs 16`. Never use `git stash` (shared with other sessions): use WIP commits.

Deck: `decklists/JD/sephiroth-fs-wipe-steal.md` (mono-black; commander "Sephiroth, Fabled SOLDIER // Sephiroth,
One-Winged Angel", already modeled in `impls/sheoldred.rs`; read Strategy and Key lines). Card list:
`rust/agents/sephfs-cards.list` (68 names). Key `sephfs` (the key `seph` is JD's Atraxa deck: don't touch it).
Earlier deck setups to copy from: `rust/agents/report-garland.md` (registration files, the report section, gating
shared-code changes to the deck's seat).

## 1. Register the deck `sephfs`
Files as for garland; `cardgen export` (online), `cardgen validate` OK; deck module `impls/sephfs.rs`.

## 2. Cards
`cardgen audit --cards rust/agents/sephfs-cards.list`; model everything below Full-auto until all are Full or
Full-auto. Missing from the database: Blood on the Snow, Dread Slaver, Extinction Event, Grave Betrayal, Living
Death, Necromantic Selection, Rabid Attack. **Check Sephiroth as a commander:** the sacrifice-to-draw on entering and
attacking; the drain on every other creature's death; the transform on the fourth resolution in a turn (only if he is
still on the battlefield); the emblem (persists after he leaves); the back face's attack trigger (sacrifice any number,
draw that many). Hand-check the Full-auto cards the plan relies on: Phyrexian Altar, Phyrexian Tower, Viscera Seer,
Yahenni, Woe Strider, Massacre Girl, Pestilence, Butcher of Malakir, Blood Artist, Zulaport Cutthroat, Plaguecrafter,
Fleshbag Marauder, Reanimate / Animate Dead (any graveyard). Tests in `tests/cards_sephfs.rs`.

## 3. AI plan (`impls/sephfs.rs`, gated to key "sephfs")
- Cast Sephiroth early (turn 3), with a sacrifice outlet if possible; recast after removal.
- **Flip first:** while Sephiroth is unflipped and on the battlefield, count how many more deaths this turn flip him;
  use edicts (Plaguecrafter, Fleshbag Marauder, Accursed Marauder) and sacrifices (Altar, Tower, Seer, Yahenni, Woe
  Strider) to reach four when it's reachable, not otherwise.
- **Wipes:** prefer after the emblem exists, when opponents have more on board; choose wipes/modes that spare
  Sephiroth while he is unflipped (Extinction Event: even; Pestilence: activate up to his toughness minus one); hold
  Living Death for when graveyards favour us.
- **Steal:** Reanimate / Animate Dead / Grave Researcher take the best creature from any graveyard; Grave Betrayal
  before a wipe when possible.
- **Sacrifice for value:** stolen and token creatures to Altar / Tower for mana when the mana is used, to Sephiroth for
  cards; never sacrifice for nothing. Exsanguinate with all available mana when it's lethal or the board is safe.
- Stats for `--analyze` (a "Sephiroth" section in `sim-cli/src/report.rs`): flips per game and the turn of the first
  flip, emblem drains per game, wipes cast, creatures stolen (by card), sacrifices per game, cards drawn by Sephiroth.

## Checks
OVERNIGHT-RULES checks 1-3 (`http_play` in practice is a known timing flake: rerun it alone if it fails once);
same-seed check (seed 7, 48 games, lookahead, before vs after) identical for `--deck sauron --pool t3`, `--deck cloud
--pool t5`, `--deck thrasios --pool t3`, `--deck garland --pool t3` and `--deck seph --pool t3`; a 1,000-game `--deck
sephfs --pool t3 --analyze --seed 1013` with no card "never cast by the sim"; numbers in `rust/agents/report-sephfs.md`.
