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
