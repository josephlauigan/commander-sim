# Report: t6 stax (branch t6-stax, one commit per card)

Files: `crates/sim-core/src/impls/t6_stax.rs` (+2 lines in `impls/mod.rs`), `crates/sim-core/tests/cards_t6_stax.rs` (19 tests),
`data/card_overrides.toml` (10 entries), `rust/agents/engine-needs-stax.md`, `rust/agents/handoff-t6-stax.md`.
No shared engine file touched. Full `cargo test --release` passes; `cargo build --release --all-targets` has no warnings.

| Card | Status | What works / what is missing |
|---|---|---|
| Disruptor Flute | Full | already in the engine (AI names a card via `flute_pick`, +3 on spells with that name, abilities off except mana); test + override only |
| Silence | Partial | opponents locked from noncreature spells (`Player::hope_lock`) + flag `cast_lock`; needs the engine's real cast lock |
| Orim's Chant | Partial | same lock on the target; kicker and "creatures can't attack" (flag `attack_lock`) need the engine |
| Mandate of Peace | Partial | lock on opponents + fog for the rest of the turn instead of ending combat; "only during combat" not enforced |
| Angel's Grace | Partial | turn flag + `grace_floor`; engine reads neither; no split second |
| Blind Obedience | Partial | extort works; opposing artifacts/creatures (and creature tokens) tapped right after entering, not entering tapped |
| Damping Sphere | Partial | spell tax exact; lands tapping for 2+ make 1 mana but keep their colour (no {C} override) |
| Pithing Needle | Partial | name chosen (as Flute would) and stored, `needle_stops` answers the rule; no engine activation check asks it |
| Chalice of the Void | Partial | X fixed at 1 (cost {2}); lock via `can_cast` (like Vexing Bauble), not a real counter |
| Void Mirror | Partial | lock via `can_cast` for free / generic-only spells without a coloured source; not a real counter |

Engine needs (see `engine-needs-stax.md`): player-level cast lock; kicker + attack lock; end the combat phase; split second
and "can't lose" flag + damage floor; enters-tapped replacement; land colour override; name-based ability lock separate from
the Flute; counter-on-cast hook + X choice. Reused hook names/wording from the copies, counters, godo and redirects files.
The AI never casts Silence, Chant, Mandate or Angel's Grace (priority 0): when they win a turn is a deck-plan matter.
