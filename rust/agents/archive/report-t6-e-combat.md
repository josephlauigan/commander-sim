# Report t6-e-combat

Mechanism
- engine/combat.rs `combat`: cap `ncomb < 4` replaced by `MAX_COMBATS = 200` (logs when hit, then clears `extra_combats`); extra-combat check no longer has its own cap (Sauron's `ncomb < 12` untouched).
- New event `Event::EachCombat` ("each_combat", slot `CardImpl::each_combat`, `EachCombatFn`; hooks.rs, engine/hooks.rs): fired at the start of every combat phase of every player, before attackers are declared, also when nobody attacks. The old `combat_start` slot (Helm of the Host, Rionya, Beatrix, Ardenn, Forger) is unchanged: still attacker-only, after declaration.
- `Game::end_combat` (state.rs): set by "end the combat phase"; `resolve_combat_inner` returns after the attackers/blockers priority windows, the defender loop stops, attackers and blockers leave combat; flag cleared per combat phase, so later additional combats still happen (real rules; the needs note said otherwise).

Cards
- Godo, Bandit Warlord: Partial -> Full.
- Sting, the Glinting Dagger: now untaps at every combat via `each_combat` (its controller chooses); still Partial (first strike vs Goblin/Orc not done).
- Mandate of Peace: `Game::end_combat` replaces the fog stand-in; still Partial (cast lock stand-in, "only during combat" not enforced).
- data/audit.toml extra_combat note updated; card_overrides.toml updated; export re-run.

Tests: rust/crates/sim-core/tests/engine_t6_combat.rs (Godo+Helm past four combats kills three goldfish; each_combat at an opponent's combat with no attackers; end_combat), cards_t6_godo.rs Sting test, cards_t6_stax.rs Mandate test. Full `cargo test --release` passes, build --all-targets has no warnings.

Other decks: any extra-combat card (t1/t2/t3 lists, Combat Celebrant etc.) can now go beyond 4 combats a turn; before it was silently truncated. Sting is the only card on each_combat.
