# Engine needs: batch `green1`

## Badgermole Cub
Needs: earthbend N / lands that are creatures: a land you control becomes a 0/0 creature with haste that is still a land (+N +1/+1 counters), and "when it dies or is exiled, return it to the battlefield tapped". Lands are `LandId`s, not `PermId`s, so `is_creature`, creature mana (Cub's additional {G}, Cradle's count), summoning sickness and combat all ignore them. A general "animate land" (a `Land` flag read by `is_creature`/`pval`, plus the return-tapped replacement) would also serve Druid Class's animated land (`DataKey::DruidClass`).
Where it would hook: `state.rs` `Game::is_creature` and `Land`; `engine/zones.rs` `die`/`leave` for the return; `engine/mana.rs` `spend_unit` (Source::Land) to fire `mana_tapped` for creature lands.
Card side done: the second ability (an additional {G} whenever a creature is tapped for mana), `t6_green1::cub_tapped`, reusing the existing `mana_tapped` slot.
Test to add once the engine has it: Cub enters with a Forest in play: the Forest is a 1/1 haste creature land; tapping it for mana makes {G}{G} (Cub's bonus); destroying it returns it tapped.
