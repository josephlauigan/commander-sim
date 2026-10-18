# Engine needs: batch `godo`

## Godo, Bandit Warlord
Needs: no cap on additional combat phases (the combat loop stops at four combats a turn); Godo + Helm of the Host makes one more combat per hasty token, so the loop needs to run until the table is dead.
Where it would hook: `engine/combat.rs` `combat` (`g.player(p).extra_combats > 0 && ncomb < 4`, line ~1127); a bound that stops a true infinite loop (all opponents dead, or a few dozen combats) should replace the 4.
Card side done: ETB fetches an Equipment onto the battlefield (Helm, or Hammer first with a copier), first attack each turn untaps Godo and Samurai and adds one combat (`extra_combats += 1`), `impls/t6_godo.rs`. Helm's token copy (t3::helm) triggers Godo's ETB and attack as the card does.
Test to add once the engine has it: Godo + Helm with six opposing creatures-free players at 40 life: all opponents die in the one turn.

## Sting, the Glinting Dagger
Needs: (1) a beginning-of-combat event for every player's combat, fired before attackers are declared and also when nobody attacks (`cardcode::combat_start` runs only for the attacking player, after attackers are chosen, and only if something attacks); (2) a conditional keyword depending on the blocker/attacker pairing ("first strike while blocking or blocked by a Goblin or Orc"): `GrantKwFn` has no combat context.
Where it would hook: `engine/combat.rs` ~line 1004 (the `combat_start` call) and the damage step near line 628 (`first strike` checks, `kw(g, m, "first strike")`).
Card side done: +1/+1, haste, equip {2} (compiled); untap of the equipped creature through the existing `combat_start` slot for its controller's combat (`impls/t6_godo.rs`).
Test to add once the engine has it: an opponent's combat untaps the equipped tapped creature (it can block); a Sting-equipped creature blocked by a Goblin token deals first-strike damage.

## Tezzeret, Cruel Captain
Needs: a new emblem kind for the walker framework: "at the beginning of combat on your turn, put three +1/+1 counters on target artifact you control; if it is not a creature it becomes a 0/0 Robot artifact creature" (the -7). `rules::give_emblem(g, p, key)` only knows its fixed keys, and the beginning-of-combat event (`combat_start`) is a permanent-slot hook, with no player-level one.
Where it would hook: `impls/rules.rs` `give_emblem` and the code that reads the emblem keys (grep `give_emblem`), plus a player-level `CombatStart` run in `engine/combat.rs` ~line 1004.
Card side done: starts at 4 loyalty; +1 loyalty for each artifact entering under its controller; 0 (untap, +1/+1 counter on an artifact creature) and -3 (artifact with mana value 1 or less to hand) as `common::walker` abilities (`impls/t6_godo.rs`); the -7 is left out.
Test to add once the engine has it: at 7+ loyalty -7 gives the emblem, and at the next beginning of combat an artifact gets three +1/+1 counters (a non-creature becomes a 0/0 Robot creature with three counters).
