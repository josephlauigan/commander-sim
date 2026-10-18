# Handoff t6-green2
Cards: Shifting Woodland; Sowing Mycospawn; Chatterstorm; Cloud of Faeries; Enduring Vitality; Noxious Revival; Invasion of Ikoria // Zilortha; The Cabbage Merchant; Endurance
Files: impls/t6_green2.rs (+ mod.rs lines), tests/cards_t6_green2.rs, data/card_overrides.toml. Oracle in scratchpad o.json (re-fetch if lost).
Test deck key: rograkh-thrasios-creature-combo; test cmd: cargo test --release -p sim-core --test cards_t6_green2 (from rust/, capped).
After override edit: rust/target/release/cardgen audit --deck rograkh-thrasios-creature-combo | grep <card>
## Done
- Chatterstorm: Full (self_cast -> t0_stella::storm_copies; token from compiled spell)
- Cloud of Faeries: Full (etb hook untaps 2 lands; cycling = hand_options Plan)
- Noxious Revival: Full (resolve: best own gy nonland card to library top; Phyrexian mana is engine-supported; note cast_card does not pay mana; hand() takes cards out of the library)
- Sowing Mycospawn: Partial (kicker missing; fetch via self_cast; engine-needs-green2.md written). Fetch picks fetchlands fine (engine cracks)
- Endurance: Full (etb bottoms a graveyard; evoke via hand_options like rules2 Solitude; flash tag already handled by AI)
- Enduring Vitality: Partial (extra_mana hook gives any-colour mana to all creatures; returns once, marked DataKey::Final; non-creature override is an engine need)
- The Cabbage Merchant: Full (cast+combat_damage hooks; Food mana via extra_mana + on_tap_perm, tapped foods in DataKey::Burden, reset at upkeep)
- Shifting Woodland: Partial (delirium copy = tapped land + temp token copy via enter_token_copy; ETB wrongly fires)
## Next (order)
Invasion of Ikoria // Zilortha, then report + full cargo test
## Gotchas
- "cast trigger" = self_cast hook; storm_copies reusable. Engine needs go to rust/agents/engine-needs-green2.md
- cast_card in tests doesn't pay mana; hand() removes the card from the library; glog needs g.log=Some(vec![]) to inspect
- Shifting Woodland data has tag ck (enters tapped if <2 lands) but Oracle is "unless you control a Forest": fix in card code
