# Handoff t6-mana
Module: rust/crates/sim-core/src/impls/t6_mana.rs (registered in impls/mod.rs), tests: tests/cards_t6_mana.rs.
Data: add override in data/card_overrides.toml, then `cargo run --release -q -p cardgen -- export --offline` (from rust/) and commit data/cards.json too.
Test deck keys: pool keys (long) or mine keys ("godo", ...) in `table(&[...])`.
Order: Chromatic Star, Desperate Ritual, Rite of Flame, Simian Spirit Guide, Manifold Key, Honor-Worn Shaku, Metalworker, Treasonous Ogre, _____ Goblin, Ignoble Hierarch, Jeweled Amulet, The Mind Stone, Grinding Station, Apple of Eden, Monumental Henge.
Done: Chromatic Star, Desperate Ritual, Rite of Flame, Simian Spirit Guide (Partial, engine need), Manifold Key, Honor-Worn Shaku, Metalworker, Treasonous Ogre, _____ Goblin, Ignoble Hierarch, Jeweled Amulet, The Mind Stone.
Next: Grinding Station, Apple of Eden, Monumental Henge, then full tests + report.
Plans: rituals = resolve slot + floating.r + prio zero; SSG = engine need (partials::hand_mana is name-hardcoded);
Shaku/Metalworker via dyn_mana_perm; Ogre via extra_mana unit (life-for-mana, only when untapped) + on_tap_perm pays life;
Amulet via extra_mana + DataKey::Charge; Mind Stone via options + end_step (t2::end_step_flicker); Henge enters_rule already Control(plains) -> only the land_options ability;
Hierarch: exalted + dork already work -> status only. Apple: take_card (yshtola) for opp hand, return at end window via gy_options.
