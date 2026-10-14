# Task: garland — register, model and give an AI plan to the Garland deck (`garland`)

Read `rust/agents/TYMNA-RULES.md` (where it says thrasios/kraum, read garland; it points to OVERNIGHT-RULES) first.
Base branch: `tymna-thrasios` (the commit that added this brief). Your worktree's branch: `garland`. You are the only
agent running and the user allowed the whole machine: `cargo -j 16`, `--jobs 16`. Overnight run: the user is asleep, so
do not stop to ask; if something is truly blocked, write it in the report and finish what you can.

Deck: `decklists/JD/garland-dimir-kidnapper.md` (Dimir, commander Garland, Royal Kidnapper; single commander). Card list:
`rust/agents/garland-cards.list` (76 non-basic names). Read the deck document's Strategy and Key lines first.
Garland is already modeled (`impls/legends.rs`, "Garland, Royal Kidnapper (10-09)": enter trigger crowns the opponent
whose best creature is most worth stealing, `garland_pick` chooses the steal). The monarch is in the engine (`g.monarch`,
`engine/turn.rs`, `engine/combat.rs`). Earlier finding (Jodah deck): the steal lasted only ~0.6 rounds because Jodah's
AI attacked the monarch and took the crown back.

## 1. Register the deck `garland`
As earlier decks were (see `rust/agents/report-partner-engine.md` for the files: `sim-cli/src/decks.rs` table + MINE,
`data/my_decks.toml`, the deck fixture json, tests/game.rs count, practice catalog if needed), `cardgen export`
(online), `cardgen validate` OK. A deck module `impls/garland.rs` for its AI plan, dispatched from `ai/decks.rs`.

## 2. Cards
`cardgen audit --cards rust/agents/garland-cards.list`; model everything below Full-auto until all are Full or
Full-auto (hand-check the Full-auto ones the plan relies on). Missing from the card database now: Chase Inspiration,
Coerced to Kill, Skycoach Conductor // All Aboard (prepare machinery exists), Specimen Freighter (station:
`impls/tymna_cards.rs` / other spacecraft code may help), Tragic Lesson, Xande, Dark Mage. Check Garland's three
abilities against the Oracle text (the steal lasts as long as that player is the monarch, also after Garland leaves;
+2/+2 and can't be sacrificed for creatures you control but don't own), and Relm's Sketching / Echocasting Symposium
copying Garland (a token copy entering triggers the crown; legend rule). Tests in `tests/cards_garland.rs`.

## 3. AI plan (`impls/garland.rs`, gated to key "garland" in shared code)
- **Never deal combat damage to the monarch** while it's an opponent (that would make us the monarch and end the
  steals): attack the other opponents with stolen creatures and evasive threats. Only take the crown ourselves if no
  opponent holds it and nothing is stolen, or the hit is lethal.
- Crown choice on each Garland entry: the opponent whose best stealable creature is worth most (commanders count);
  prefer an opponent the others are likely to attack.
- Re-crown: return Garland to hand (Rescue at instant speed, Matterbending Mage, Peerless Ropemaster after he attacked,
  Specimen Freighter, Whoosh!, Unsummon) or copy him (Relm's Sketching, Echocasting Symposium) when the crown has
  stayed with one player for a full round, or a clearly better target exists elsewhere; recast from hand.
- Desertion on a strong creature spell; Coerced to Kill on the best opposing creature.
- Stats for `--analyze` (a "Garland" section in `sim-cli/src/report.rs`, like the Tymna one): steals per game, average
  rounds a steal lasts, crown changes per game, Garland entries per game (cast, recast, copies), damage dealt by
  stolen creatures, how often we became the monarch.

## Checks
OVERNIGHT-RULES checks 1-3; same-seed check (seed 7, 48 games, lookahead, before vs after) identical for `--deck sauron
--pool t3`, `--deck cloud --pool t5`, `--deck kraum --pool t3` and `--deck thrasios --pool t3`; no card listed "never
cast by the sim" in a 200-game `--deck garland --pool t3 --analyze --seed 1013`; that run's numbers in the report.
Full report in `rust/agents/report-garland.md`.
