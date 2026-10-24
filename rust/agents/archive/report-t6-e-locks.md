# Report: t6-e-locks (plan rows 2 + 3)

## Mechanism
- `state.rs`: `Player::cast_locks: Vec<CastLock>` (`LockKind` AllSpells / Noncreature / InstSorcAbilities; `LockEnd` Turn(stamp) / TurnOf(player, turns)), `Game::locks_used` (sticky), `Game::cast_checks()` = hooks present or locks used. `Player::hope_lock` removed.
- `engine/cast.rs`: `add_cast_lock`, `lock_this_turn`, `spell_locked` (lands are never spells), `abilities_locked`, `split_second_up` (a spell with the card keyword "split second" on the stack). `castable` and `cast_card` refuse locked casts and any cast while a split-second spell is up; `cast_card` sets `locks_used` when a split-second card is cast.
- Gates `!g.hooks.is_empty() && !castable(..)` became `g.cast_checks() && ..` (ai/brain.rs x3, engine/stack.rs, engine/altcost.rs, human/legal.rs, impls/mine.rs x3), so the lock works in games with no hooked card. Human cast lists already call `castable`.
- Abilities: `rules::ability_locked` (AI option lists, all hooked options) and `dsl::activate` refuse non-mana abilities under Abeyance or split second.
- `stack.rs::take_priority`: nobody gets priority (AI or human) while a split-second spell is on the stack.
- Stand-ins removed: `hope_lock`, `t6_stax::cast_lock`, `t6_counters::lock_flag` / `abeyance_flag` (+ their flag helpers), `t6_redirects::noncreature_lock`. Hope of Ghirapur now adds a Noncreature lock ending at its controller's next turn (`TurnOf`), same behaviour as before.

## Cards
- Full: Silence, Abeyance, Calamity's Wake.
- Still Partial: Orim's Chant (kicker, creatures can't attack), Mandate of Peace ("only during combat" not enforced), Angel's Grace ("can't lose" + damage floor), Bilbo's Gambit (gift choice never offered, bilbo_bounce not called by the engine), Sudden Substitution (response casting, resolving under the new controller), Legolas's Quick Reflexes (becomes-tapped damage trigger). Split second itself works for the last three.
- Voice of Victory left as its own `can_cast` hook: it is a static permanent ability (opponents, only on its controller's turn), not a per-turn grant, so the player-lock list is not a natural fit. Behaviour unchanged.
- AI: Silence / Chant / Grace / Mandate stay priority 0. `ai::combo_interrupted` is the opponents' interruption of a combo, not the combo player's own pre-emptive lock, so it is not a natural place; when to cast them is a Stage 5 deck-plan call.

## Other decks
No behaviour change: locks only exist after one of these cards resolves; Hope of Ghirapur is equivalent. One real change: the old Silence tests used `turns += 1` to end the lock; locks now end with the turn stamp (tests updated).

## Not done / needs found
- Mana abilities exempt from Abeyance/split second only by being outside `ability_locked`/`dsl::activate` (mana code does not call them). Human ability menus not audited for `ability_locked`.
- `cast_card` is not the only way card code casts a spell (`counter_window` for hand-coded casts): those do not look at locks unless the card code calls `castable`, as before.
- Tests: new `tests/locks.rs`; Silence flash creature + next turn, Angel's Grace split second, Abeyance, Bilbo, Calamity tests added/updated. Full `cargo test --release` passes, no build warnings.
