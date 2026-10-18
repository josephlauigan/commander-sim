# Engine needs: batch dack

## All-Fates Stalker
Needs: a cast-from-exile option for a card that was exiled by its own warp (an exile-zone options hook, general: warp, foretell, suspend-like effects).
Where it would hook: `hooks.rs` CardImpl next to `gy_options` / `hand_options` (an `exile_options` slot), called from the AI option list (see `kefka_options` for the hard-coded Voidborn version).
Card side done: ETB banish (non-Assassin, until it leaves), leaves return, warp cast from hand (`stalker_warp`, switch `WARP_RECAST_SUPPORTED`), exile at the next end step (`stalker_end`).
Test to add once the engine has it: warp Stalker, end step it is exiled and its victim returns, next turn it can be cast from exile.
