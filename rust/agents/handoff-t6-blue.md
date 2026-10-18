# Handoff: t6 blue
Branch t6-blue. Code: impls/t6_blue.rs, tests/cards_t6_blue.rs, rust/agents/engine-needs-blue.md, overrides in data/card_overrides.toml (then `rust/target/release/cardgen export --offline`).
Test deck seat 0 = rograkh-silas-turbo-breach, seat 1 = seph. Oracle text: scratchpad oracle.json (scryfall).
## Done
- Borne Upon a Wind (Partial: flash grant is an engine need)
## Next (order)
Mnemonic Betrayal (use yshtola::take_card + impulse), Timetwister, Demonic Counsel, Rain of Filth, Sevinne's Reclamation, Exhibition Tidecaller, Voice of Victory, High Fae Trickster, Drift of Phantasms, Emrakul, Rograkh, Silas Renn, Hidden Strings, Valley Floodcaller
## Gotchas
- `resolve` slot overrides the DSL; return "gy" / "exile" / "handled".
- Cards taken from opponents: yshtola::take_card (agent_ids = any mana type, stolen = owner for gy).
