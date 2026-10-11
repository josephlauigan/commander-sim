# Task: zada-cards — the rest of Zada's deck, plus the "prepared" mechanic

Deck: `decklists/JD/zada-mono-red-go-wide.md` (key `zada`). Another agent (zada-core) does Zada himself, the Zada
spells and the spell payoffs in `impls/zada.rs`; don't touch those cards.

Your files: `rust/crates/sim-core/src/impls/zada_cards.rs` (new; register it in `impls/mod.rs`), its tests in
`rust/crates/sim-core/tests/cards_zada_cards.rs`, card data rows for these cards.

## Prepared (a mechanic: build it so any card can use it)
*This creature enters prepared. (While it's prepared, you may cast a copy of its spell. Doing so unprepares it.)*
The card is a creature with a second, spell half; while the creature is prepared you may cast a copy of that spell
(paying its cost). Model it with the smallest engine change, and the AI should use the spell when it's worth it.
Cards: Goblin Glasswright (spell: create a Treasure), Strife Scholar (ward: pay 2 life; spell: create two 2/2 red and
white Spirits), and **Blazing Firesinger** (Kefka's deck; spell: add {R}{R}{R}{R}{R}).

## Cards to model (until `cardgen audit --mine` shows each as Full)
Unmodeled: Burn, Burn, Tree and Fern (Saga), Ral and the Implicit Maze (Saga; check its whole text with jq).
Partial: Bomber Corps (battalion), Bonders' Enclave (the draw ability), Command the Stage (Cadet token, counters on
Wizard tokens, the upkeep clause), Glimpse the Impossible, Glitterfang, Goblin Arsonist, Goblin Rimerunner, Heated
Argument, Irascible Wolverine (**plot**), Krenko's Enforcer (**intimidate**), Lightning, Security Sergeant, Living
History, Orcish Cannonade, Pia, Determined Rebuilder, Reiterating Bolt (replicate with energy: model energy only if
the deck can make it; say so), Relic of Legends, Rundvelt Hordemaster, Scorching Dragonfire, Skilled Battlecarver,
Spawn-Gang Commander, Suplex, Violent Echoes (excess damage to a Jace: empower Jace can be a no-op without a Jace:
say so), Wrath of the Bloodmane. Approximate: Generator Servant.

Plot and intimidate are mechanics: build them for any card. Give each new card a sensible cast priority (pool decks
use `ai/pool.rs` `generic_prio`; Zada's deck AI is the zada-core agent's, so keep your priorities in the cards'
`prio` slots).

Smoke run for this deck: `--deck zada` (Zada's own ability may not be merged yet: that's fine).
