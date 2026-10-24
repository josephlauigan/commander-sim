# Report t6 dack (branch t6-dack)
Cards (code impls/t6_dack.rs, tests tests/cards_t6_dack.rs, 24 tests; one commit per card):
- Fiend Hunter: Full.
- Touch the Spirit Realm: Full (ETB banish; Channel returns at next end step via g.eot_returns).
- All-Fates Stalker: Partial (banish done; warp coded but not offered: no cast-from-exile hook).
- Radiant Solar: Partial (venture = Lost Mine only, state in Player::grudge; trigger + hand ability done).
- Preston, the Vanisher: Partial (Illusion copy keeps the copied types; five-Illusion exile done).
- Dack Fayden, Helping Hand: Partial (goad lapses at the goader's next turn instead of the rest of the game).
- Gideon of the Trials: Partial (+1 via Dovin lock = attacker's combat damage only; no damage prevention to him; emblem recorded, can't-lose not read).
- Paladin Class: Partial (tax, level-ups, level 3 trigger done; level 2 anthem missing).
Engine needs: rust/agents/engine-needs-dack.md (exile-zone cast option, dungeon system, copy exceptions replacing types,
loop cap for Preston+Felidar, indefinite goad, first-class give-control, Gideon can't-lose/prevention, level-gated anthem).
Shared files touched: impls/mod.rs (2 lines), data/card_overrides.toml, data/cards.json (regenerated), nothing else.
Design notes: banisher/Solar/Preston ETB hooks read "you" from the enter call's p; Dack wraps the pile in g.resolving so
triggers queue until donation is done; mark_donated/controlled_when keep a donated Preston/Solar valid for the pile.
