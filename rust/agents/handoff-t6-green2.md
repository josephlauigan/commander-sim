# Handoff t6-green2
Cards: Shifting Woodland; Sowing Mycospawn; Chatterstorm; Cloud of Faeries; Enduring Vitality; Noxious Revival; Invasion of Ikoria // Zilortha; The Cabbage Merchant; Endurance
Files: impls/t6_green2.rs (+ mod.rs lines), tests/cards_t6_green2.rs, data/card_overrides.toml. Oracle in scratchpad o.json (re-fetch if lost).
Test deck key: rograkh-thrasios-creature-combo; test cmd: cargo test --release -p sim-core --test cards_t6_green2 (from rust/, capped).
After override edit: rust/target/release/cardgen audit --deck rograkh-thrasios-creature-combo | grep <card>
## Done
- Chatterstorm: Full (self_cast -> t0_stella::storm_copies; token from compiled spell)
- Cloud of Faeries: Full (etb hook untaps 2 lands; cycling = hand_options Plan)
## Next (order)
Noxious Revival, Sowing Mycospawn, Endurance, Enduring Vitality, Cabbage Merchant, Shifting Woodland, Invasion of Ikoria
## Gotchas
- "cast trigger" = self_cast hook; storm_copies reusable. Engine needs go to rust/agents/engine-needs-green2.md
