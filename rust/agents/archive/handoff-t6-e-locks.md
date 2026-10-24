# Handoff: t6-e-locks (plan rows 2+3: cast locks, split second)

Worktree .claude/worktrees/t6-e-locks, branch t6-e-locks. Brief: rust/agents/t6-engine.md.

## Plan
1. state.rs: `Player::cast_locks: Vec<CastLock>` (kind AllSpells / Noncreature / InstSorcAbilities; end Turn(stamp) / TurnOf(player, turns)), `Game::locks_used` (sticky gate).
2. engine/cast.rs: `add_cast_lock`, `spell_locked`, `abilities_locked`, `split_second_up`; `castable` + `cast_card` read them; gates `!g.hooks.is_empty() && castable` become `g.cast_checks()`.
3. stack.rs `take_priority` skips everyone while a split-second spell is on the stack; `rules::ability_locked` reads the Abeyance lock.
4. Wire cards: Silence/Chant/Mandate/Calamity/Bilbo/Abeyance/Hope of Ghirapur; remove hope_lock + flag stand-ins. Voice of Victory stays a `can_cast` hook (static permanent ability, not a per-turn grant).
5. Overrides, tests, report, full test run.

## Done
(nothing yet)
Done: engine+cards wired, tests written; next: run tests, overrides, report
