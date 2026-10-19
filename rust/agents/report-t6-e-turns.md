# Report t6-e-turns (plan row 9, extra turns)
Mechanism: `Game::extra_turns: Vec<(PlayerId, bool)>` (who, loses at end) and `Game::doomed_turn` (state.rs).
`turn::add_extra_turn(g,p,lose)` queues; `turn::run_extra_turns(g)` plays the queue (last queued first, a normal full
turn each, dead players skipped, may queue more); `turn::play_turn` = `take_turn` + `run_extra_turns`, used by
`run_rounds` and `ai/search.rs::play_on` (which also drains the queue after resuming a copied game mid-turn).
`turn::end_step` (after `delayed_end_step`) eliminates the doomed player ("extra turn" death kind), honouring `cant_lose`.
Cards: Final Fortune, Last Chance (impls/t6_godo.rs) resolve by queueing a losing extra turn; both Full (data).
AI (`extra_turn_prio`, 85): never on a doomed turn or with one already queued; casts when (a) Godo is out with Helm,
summoning sick and hasty-less, or (b) Godo is in hand/command zone, this turn's mana after RR can't cast him + equip
({2}), but the whole mana base (tapped included) can, with Helm/Hammer in play, hand or library.
Other cards: no existing card grants extra turns (grep extra_turn: only skip_turns for Ral Zarek, untouched).
Other decks: no behaviour change (queue empty unless these cards resolve; only Godo's AI casts them).
Tests: cards_t6_godo.rs (queue, loss at end step, ordering, round loop, AI priorities).
Needs found: none. Emrakul's extra turn can use add_extra_turn(g, opp, false).
