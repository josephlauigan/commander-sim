# Handoff: karn-threats

Done (commit "Karn threats: ..."): all 19 cards coded in `impls/karn_cards.rs` (registered in mod.rs), overrides in
`data/card_overrides.toml` (dsl = [] for all, hand tags), `cardgen export --offline` run. Shared edits (all small):
t0_giada::cant_lose (Platinum Angel), engine/cast.rs grants_flash (Liberator), engine/combat.rs infect,
engine/zones.rs to_zone_card (Blightsteel shuffle-back), impls/sweepers.rs ("dust" kind), state.rs DataKey::Fate.

Next: tests in `tests/cards_karn_threats.rs` (run, fix failures), then Basalt+Rings+Ballista check, workspace
`cargo test --release -j 7`, no-warnings build, report `rust/agents/report-karn-threats.md`.
