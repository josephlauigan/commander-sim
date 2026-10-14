# Task: jeska — model and register the Tymna + Jeska deck (`jeska`)

Read `rust/agents/TYMNA-RULES.md` first (it points to OVERNIGHT-RULES). Base branch: `tymna-thrasios` (the commit that
added this brief). Your worktree's branch: `jeska`. You are the only agent running: `cargo -j 16`, `--jobs 16`.
Earlier work on the two other Tymna decks: `rust/agents/report-partner-engine.md`, `report-tymna-cards.md`,
`report-tymna-ai.md`, `report-tymna-fix.md` (read the parts you need). Deck: `decklists/JD/tymna-jeska-partners.md`
(Mardu: Tymna the Weaver + Jeska, Thrice Reborn). Card list: `rust/agents/jeska-cards.list` (83 non-basic names).

## 1. Register the deck `jeska`
As partner-engine did for `thrasios` / `kraum`: `sim-cli/src/decks.rs` (table + MINE + anything else it touched),
`data/my_decks.toml`, test fixtures, `cardgen export` (online), `cardgen validate` OK. Deck AI: reuse the shared plan in
`impls/tymna_partners.rs` (`partner_prio`, the attack `spread`, `commander_first`, protect / wipe wiring): add `jeska`
to every `"thrasios" | "kraum"` gate there and in `ai/decks.rs` (grep for both keys across `rust/crates`).

## 2. Jeska, Thrice Reborn (a planeswalker commander with Partner)
Oracle: {2}{R}, loyalty 0. *Jeska enters with a loyalty counter on her for each time you've cast a commander from the
command zone this game. 0: Choose target creature. Until your next turn, if that creature would deal combat damage to
one of your opponents, it deals triple that damage to that player instead. −X: Jeska deals X damage to each of up to
three targets.* Check first that the partner engine handles a **noncreature** commander (cast from the command zone,
tax, back to the command zone when she would die or be exiled, opponents attacking and damaging her as a
planeswalker, look-ahead). Count commander casts from the command zone for both commanders, this cast included (the
taxes already record them). Tripled combat damage counts as commander damage when the creature is a commander
(Tymna: 2 becomes 6) and as lifelink etc. as usual. AI: cast her precombat; 0 on the creature whose tripled hit is
worth most (one that will connect: evasive, double strike, the biggest flier; lethal checks), before attacks; −X when
it kills creatures worth more than her (mana creatures, blockers the attack needs gone, X up to her loyalty), or to
finish a player or planeswalker. Tests for each part.

## 3. The other cards
Run `cardgen audit --cards rust/agents/jeska-cards.list`; model every card below Full-auto until all are Full or
Full-auto (hand-check the Full-auto ones that matter: evasion, double strike, combat-damage triggers). Missing from
the card database now: Char-Rumbler, Exultant Skymarcher, Pursue the Past, Quill-Blade Laureate // Twofold Intent
(prepare machinery exists: Abigale), Scolding Administrator (repartee), Slayers' Stronghold, Thriving Skyclaw
(energy), Vincent, Vengeful Atoner. Card code in `impls/tymna_cards.rs`, tests in `tests/cards_jeska.rs`.

## Checks
OVERNIGHT-RULES checks 1-3; same-seed check identical for `--deck sauron --pool t3`, `--deck cloud --pool t5`
**and `--deck kraum --pool t3`** (seed 7, 48 games, lookahead) before vs after; no card in `jeska` listed "never cast
by the sim" in a 200-game `--analyze --seed 1013` run vs t3; that run's win rate, Jeska's landing turn and how often
her 0 / −X are used in the report (`rust/agents/report-jeska.md`).
