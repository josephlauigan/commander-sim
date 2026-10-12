# Handoff: elrond-core

Done (commit f152948 and after): impls/elrond.rs has all 22 cards of the brief plus the AI plan (elrond_prio,
elrond_special = warped creatures from exile, elrond_protect = counter the removal spell / Unsummon / Whoosh!).
Overrides (status Full + notes) for every card in data/card_overrides.toml; Elrond toughness fixed to 3/2 in
data/carddb.txt. tests/cards_elrond.rs: 30 tests passing.

Shared-file changes: state.rs (DataKey Stun, Weaken, Resolved, ResolvedAt), engine/turn.rs (stun at untap),
engine/values.rs (epow: Mouth of the Storm), engine/mana.rs (cost_of: Defiler's {U} discount), impls/mine.rs
(ring_tempt: elrond bearer pick + Gandalf draw), dsl/mod.rs (search: a land going to the top/graveyard no longer goes to
the hand; fixes Dune Mover and Silver Deputy).

Next: run the LASTDECK-RULES checks (full cargo test, all-targets build warnings, all_decks adaptive 3 / lookahead 1,
smoke run of --deck elrond), write rust/agents/report-elrond-core.md, final report.
