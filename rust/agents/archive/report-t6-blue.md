# Report: t6 blue (15 cards)

Code: `crates/sim-core/src/impls/t6_blue.rs` (registered in `impls/mod.rs`), tests `tests/cards_t6_blue.rs` (26 tests),
data in `data/card_overrides.toml` and the regenerated `data/cards.json` (Exhibition Tidecaller: `dsl = []`, its compiled
mill-3 trigger replaced by the Opus code). Engine needs: `rust/agents/engine-needs-blue.md`. Nothing else shared touched.

| Card | Status | Note |
|---|---|---|
| Borne Upon a Wind | Partial | draws, sets the one-turn flash flag (`flash_turn_grant`); engine must read it |
| Mnemonic Betrayal | Partial | exiles opponents' graveyards, any-mana cast via `yshtola::take_card`; leftovers should return to graveyards at end step |
| Timetwister | Full | |
| Demonic Counsel | Full | Demon search / delirium any card |
| Rain of Filth | Partial | turn flag only; needs a sacrifice-a-land mana source |
| Sevinne's Reclamation | Full | flashback copy returns a second card |
| Exhibition Tidecaller | Partial | Opus done; mana spent is approximated (mana value + X) |
| Voice of Victory | Full | Mobilize 2 + opponents can't cast on your turn |
| High Fae Trickster | Partial | `static_flash` exposed; engine must read it |
| Drift of Phantasms | Full | Transmute as a sorcery-speed hand option |
| Emrakul, the Promised End | Partial | cost reduction done; turn control, extra turn, protection from instants are engine needs |
| Rograkh, Son of Rohgahh | Full | partner is the engine's |
| Silas Renn, Seeker Adept | Full | combat-damage mark + cast from graveyard in a main phase |
| Hidden Strings | Partial | tap/untap done; cipher is an engine need |
| Valley Floodcaller | Partial | cast trigger done; noncreature flash is `static_flash` |
