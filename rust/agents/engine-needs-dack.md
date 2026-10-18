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
