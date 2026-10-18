# Engine needs: batch dack

## All-Fates Stalker
Needs: a cast-from-exile option for a card that was exiled by its own warp (an exile-zone options hook, general: warp, foretell, suspend-like effects).
Where it would hook: `hooks.rs` CardImpl next to `gy_options` / `hand_options` (an `exile_options` slot), called from the AI option list (see `kefka_options` for the hard-coded Voidborn version).
Card side done: ETB banish (non-Assassin, until it leaves), leaves return, warp cast from hand (`stalker_warp`, switch `WARP_RECAST_SUPPORTED`), exile at the next end step (`stalker_end`).
Test to add once the engine has it: warp Stalker, end step it is exiled and its victim returns, next turn it can be cast from exile.

## Radiant Solar
Needs: a dungeon system: per-player dungeon state (which dungeon, current room, completed count), the choice of dungeon and branch, and `venture` as an engine verb (Tomb of Annihilation, Dungeon of the Mad Mage, rooms with targets).
Where it would hook: a `Player` field (state.rs) and a `venture(g, p)` in engine/ next to `scry`; cards call it.
Card side done: `t6_dack::venture` always enters Lost Mine of Phandelver (Cave Entrance scry 1, Goblin Lair token, Dark Pool drain 1 / gain 1, Temple draw), room kept in `Player::grudge["dungeon_room"]`; Solar's enter trigger (itself or another nontoken creature) and `{W}, discard: venture, gain 3 life` are done.
Test to add once the engine has it: ventures count toward "completed a dungeon"; the dungeon choice is real.

## Dack Fayden / all donated creatures (see Dack's section)
Needs: indefinite goad ("for the rest of the game"), see below.

## Preston, the Vanisher
Needs: copy exceptions that replace characteristics (a token copy that is a 0/1 white Illusion and has ONLY the Illusion creature type) -- same mechanism as in engine-needs-copies.md.
Where it would hook: `engine/zones.rs::enter_token_copy` (a spec of exceptions), `engine/values.rs::subtypes` (ttypes are added to the card's subtypes, never replace them).
Card side done: `t6_dack::preston_etb` makes the token copy (enters trigger fires) and patches it to 0/1, white, plus the Illusion type afterwards (so the copy's enters trigger sees the printed stats for one moment and the copied creature types stay); the sacrifice-five exile ability.
Test to add once the engine has it: a Preston copy of a Human Cleric is not a Human.

## Felidar Guardian + Preston (unbounded loop)
Needs: a recursion/loop cap for self-feeding enter triggers. Preston's Illusion copy of Felidar Guardian flickers the original, which re-enters uncast, which copies again: the game recurses until the stack overflows (seen in a test; tests use Serra Angel instead). The sheet treats this as the infinite combo, so the engine needs "unbounded loop = win/stop" handling.
Where it would hook: `engine/zones.rs::enter` depth counter (as `blink_depth` in t2) or a loop detector in `stack::resolve_batch`.
Card side done: Preston's token copy guard (`preston_copy_born`) so a copy never triggers Preston itself.
Test to add once the engine has it: Preston + Felidar Guardian uncast ends the resolution with a cap, no overflow.

## Dack Fayden, Helping Hand
Needs: goad with a duration "for the rest of the game" (and, generally, goad that lasts until a given event).
Where it would hook: `impls/t0_zurgo.rs::goaded_by` (compares the goader's turn count with `DataKey::Goaded`'s stamp); a stamp such as -1 meaning "forever" would do. `engine/combat.rs` already calls `goad_attackers`.
Card side done: the whole flip (reveal until X creatures, enter under the controller, shuffle, donate with `marchesa::steal`, `mark_donated` for the queued triggers); `DataKey::Goaded` is set with the current turn count, so it lapses at the goader's next turn.
Test to add once the engine has it: after two of Dack's controller's turns the donated creatures are still goaded.

## Giving a permanent to an opponent (Dack)
Needs: a first-class "gain control" with a duration and controller-change events (the engine has only `marchesa::steal`, which moves the permanent between `perms` lists and sets sickness).
Where it would hook: `engine/zones.rs` (a control-change fn that also fires leaves/enters-control hooks and clears attachments / hooks bookkeeping).
Card side done: `marchesa::steal(g, opp, m, false)` is used; ETB triggers of the donated creatures use the entering player as "you" (hook argument `p`), `mark_donated` / `controlled_when` for Preston and Radiant Solar.
Test to add once the engine has it: the donated creature's queued enters trigger is still the giver's.

## Gideon of the Trials
Needs: (1) "can't lose the game / opponents can't win" granted by an emblem or a condition (emblem + "as long as you control a Gideon planeswalker"); (2) prevention of all damage dealt by a permanent for a duration (any damage, not just an attacker's combat damage) and of all damage dealt to a permanent this turn.
Where it would hook: (1) `engine/life.rs::check_state` uses `t0_giada::cant_lose` (Herald of Eternal Dawn by name): make it read a player-level "can't lose" source list; (2) `engine/combat.rs` ~l.593 (`dovin_blocked` only zeroes an attacking creature's damage) and the damage-to-permanent paths.
Card side done: walker abilities in `t6_dack.rs`; +1 uses `DataKey::Dovin`; the animation sets `Anim` + `Indestr` and `gideon_end` undoes it; the emblem is pushed on `Player::emblems` as "gideon_trials".
Test to add once the engine has it: with the emblem and a Gideon, a player at 0 life stays in the game; damage dealt by the +1 target (blocking or burn) is prevented.

## Paladin Class
Needs: a level-gated static anthem (a "+1/+1 to creatures you control" that turns on at Class level 2). Static anthems are the `Tag::Anth` card tag read in `values::static_bonus`, which is not per-permanent.
Where it would hook: `engine/values.rs::static_bonus` (a check of `DataKey::Level` for Class cards, or a `static_pt` slot on `CardImpl`).
Card side done: the {1} opponents' spell tax on your turn (`cost` slot), level-up abilities (data `Level`), the level 3 attack trigger (`attack` slot, once per combat, +N/+N and double strike until end of turn on the highest-power attacker).
Test to add once the engine has it: at level 2 your other creatures are +1/+1.

NOTE (stopgap): `t6_dack::PRESTON_CAP` (30 Preston copies per turn, then "Preston loop capped (infinite combo, engine pass will resolve it)") only keeps the loop from overflowing the stack. The real fix is a combo shortcut in the engine pass that recognizes the infinite loop and resolves it (win via Solar's ventures).

## Lead note (merge, 2026-10-17): the Preston + Felidar loop stops after 2 copies
Under the rules it is unbounded (each Felidar token's ETB flickers the real Felidar, which re-enters uncast, so Preston
copies it again; Radiant Solar ventures each time). In the engine it yields 2 copies and stops, so Dack's main win line
does not happen yet. The engine pass must add it as a combo in impls/combos.rs (pieces: Preston + Felidar under your
control + Radiant Solar or a Solar copy, all under your control; with answer windows), and the Dack 35-pile engine test
(audit/t6/dack-fayden-white-stax/flips-*.md) checks it end to end.

## Engine pass t6-e-copies: Preston's Illusion exceptions done
Preston's token copy is created with `CopyExc` (0/1, white, only Illusion) through `enter_token_copy_with`. The Felidar loop combo is still open.
