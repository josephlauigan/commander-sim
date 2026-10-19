# Task: practice-walkers, the person's creatures can attack planeswalkers (and better blocks)

Read `rust/agents/practice-fixes-common.md` first. Worktree:
`/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/practice-walkers`, branch `practice-walkers`.

## The problem
`engine/combat.rs` (around line 559): `to_walker` is empty for a person, so their creatures can never attack a
planeswalker. `human/combat.rs::human_attack` asks only which creatures attack and which **players**. Also (audit
g5): blocks allow one blocker per attacker, and combat damage / trample assignment is fixed for the person.

## Do (in this order; commit after each)
1. **Attack planeswalkers.** In `human_attack`, each attacker can attack a player or a planeswalker an opponent
   controls (planeswalkers alive on the battlefield, not phased out). Keep the current flow for the plain case (all at
   one player). Feed the person's walker attacks into the same structure the AI uses (`walker_attacks` / `to_walker`
   in combat.rs) so damage, blocks and triggers work as they do for the AI's walker attacks. The page renders whatever
   choices it is given; check `rust/crates/practice/static/app.js` / `table.js` show the new options sensibly (labels
   like "Teferi, Hero of Dominaria (Alice's planeswalker, 4 loyalty)"), and the text client (`practice/src/text.rs`).
2. **Several blockers on one attacker** for a person defending (`human_blocks`): let them assign more than one
   blocker to an attacker (menace already needs two). The engine's damage step must handle it (look at how the AI's
   multi-blocks work, if it has them; if the engine can't, stop and say so in the report rather than rewriting combat).
3. **Damage assignment order** when the person's attacker is blocked by several creatures, and trample: only if
   step 2 shows the engine supports it cheaply. Otherwise leave it, and explain in the report.

## Tests
`tests/play_walker_attacks.rs`: a person attacks an opponent's planeswalker and its loyalty drops; attacking a player
still works; a split attack (one at the walker, one at the player); several blockers on one attacker (if done).

## Files owned
`human/combat.rs`, the person-only branches in `engine/combat.rs`, the practice page's attack UI if needed, the text
client's attack prompt, the new test, `rust/agents/{handoff,report}-practice-walkers.md`.

Commit message for the last commit: "Practice mode: attack planeswalkers; several blockers on one attacker".
