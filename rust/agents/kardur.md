# Task: kardur — goad in the engine, and register, model and plan the Kardur deck (`kardur`)

Read `rust/agents/TYMNA-RULES.md` (where it says thrasios/kraum, read kardur; it points to OVERNIGHT-RULES) first.
Base branch: `tymna-thrasios` (the commit that added this brief). Your worktree's branch: `kardur`. You are the only
agent running: `cargo -j 16`, `--jobs 16`. Never use `git stash` (shared with other sessions): use WIP commits.

Deck: `decklists/JD/kardur-rakdos-forced-war.md` (Rakdos, commander Kardur, Doomscourge; single commander; read
Strategy and Key lines). Card list: `rust/agents/kardur-cards.list` (74 non-basic names). Sephiroth is already modeled
as "Sephiroth, Fabled SOLDIER // Sephiroth, One-Winged Angel" (`impls/sheoldred.rs`). Previous deck setups to copy
from: `rust/agents/report-garland.md` (registration files, report section, gating to a deck's seat).

## 1. Goad (new engine rule; nothing exists yet)
Rule 701.15: a goaded creature attacks each combat if able, and attacks a player other than the goading player if
able; the goad lasts until the goading player's next turn. A creature goaded by several players attacks a player who
didn't goad it if able. Implement it for every seat's combat (the AI's attacker choice and `engine/combat.rs`
declaration): goaded creatures must attack, and their defender choice excludes the goaders when another player is
attackable. Kardur's enter trigger is the same rule applied to every creature opponents control ("until your next
turn, attack each combat if able and attack a player other than you"). Only two existing decks run goad cards
(anikthea-enduring-enchantments: Ghoulish Impetus; zurgo-mardu-surge: Grenzo, Havoc Raiser): their games may change
(correctly); every other deck's games must stay identical.

## 2. Register the deck `kardur`
Files as for garland; `cardgen export` (online), `cardgen validate` OK; deck module `impls/kardur.rs`.

## 3. Cards
`cardgen audit --cards rust/agents/kardur-cards.list`; model everything below Full-auto until all are Full or
Full-auto (hand-check the Full-auto ones the plan relies on: drain-on-death cards, sacrifice outlets, Phyrexian
Altar / Tower, token makers). Missing from the database: Kardur, Doomscourge; Agitator Ant; Bloodthirsty Blade; Death
Kiss; Disrupt Decorum; Geode Rager; Maximum Carnage; Parasitic Impetus; Shiny Impetus; Vengeful Ancestor; Systems
Override; Rabid Attack. Check Kardur's second ability (any attacking creature that dies, any player's). Tests in
`tests/cards_kardur.rs`, including goad rules tests.

## 4. AI plan (`impls/kardur.rs`, gated to key "kardur")
- Cast Kardur when opponents have creatures that can attack each other (not into an empty table); recast after
  removal the same way.
- Goad targets: an opponent's biggest attacker (Impetus Auras, Bloodthirsty Blade); Disrupt Decorum / Maximum Carnage
  when opponents' boards are large.
- Sacrifice plan: Altar / Tower / Viscera Seer / Carrion Feeder sacrifice our tokens and stolen creatures (Act of
  Treason, Systems Override: steal, attack, then sacrifice before end of turn) for mana and death triggers; aim for
  four deaths in one turn to flip Sephiroth when it's out; never sacrifice for nothing.
- Wipes (Blasphemous Act, Toxic Deluge) preferably after Sephiroth's emblem exists and when the drain or the board
  gain is worth it.
- Stats for `--analyze` (a "Kardur" section in `sim-cli/src/report.rs`): goaded attacks per game, creatures that died
  attacking per game, drain from Kardur and from other death triggers per game, Sephiroth flips, our sacrifices per
  game.

## Checks
OVERNIGHT-RULES checks 1-3 (`http_play` in practice is a known timing flake: rerun it alone if it fails once);
same-seed check (seed 7, 48 games, lookahead, before vs after) identical for `--deck sauron --pool t3`, `--deck cloud
--pool t5`, `--deck thrasios --pool t3`, `--deck garland --pool t3`; no card "never cast by the sim" in a 200-game
`--deck kardur --pool t3 --analyze --seed 1013`; numbers in `rust/agents/report-kardur.md`.
