# gandalf-ai report
Changes (all in `rust/crates/sim-core/src/ai/brain.rs`, gated on `has(g, p, Tag::Gandalf)`):
1. `gandalf_hold`: in `main_options`, sorcery draw / tutor / token / top / targeted removal get -3.0 utility so they wait for the end-of-turn window (not held: ramp, creatures, wipes, protection, removal when danger >= 0.8, draw/tutor while no land in hand on turn <= 5).
2. `end_of_turn_window`: removal sorceries allowed (via `removal_options`), and sorcery wipes allowed in `wipe_options(eot)`.
3. `attack_response` (defend step): removal sorceries count as instant removal; new `gandalf_wipe_response` casts a sorcery wipe when the attack is >= 30% of life and the wipe nets positive value (attackers counted).
Tests: 4 new in `tests/cards_elrond.rs` (hold, EOT removal, attacker removal, wipe after attack), each with a no-Gandalf negative.
Measurement: Sauron, look-ahead, loose, 1000 games per tier, same seeds, `--swap "Night's Whisper=>Gandalf, Friend of the Shire"`, delta in win points (noise about +-3):
T1 -3.2 -> -3.1; T2 -3.4 -> -2.3; T3 +0.3 -> +1.2; T4 +1.4 -> +0.9; T5 -1.0 -> -0.6. Mean -1.2 -> -0.8: no clear effect.
Elrond regression run dropped by the lead (partial baseline only: T1 10.0, T2 11.2, T3 9.2, T5 6.1%).
