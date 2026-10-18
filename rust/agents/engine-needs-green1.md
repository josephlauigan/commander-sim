# Engine needs: batch `green1`

## Badgermole Cub
Needs: earthbend N / lands that are creatures: a land you control becomes a 0/0 creature with haste that is still a land (+N +1/+1 counters), and "when it dies or is exiled, return it to the battlefield tapped". Lands are `LandId`s, not `PermId`s, so `is_creature`, creature mana (Cub's additional {G}, Cradle's count), summoning sickness and combat all ignore them. A general "animate land" (a `Land` flag read by `is_creature`/`pval`, plus the return-tapped replacement) would also serve Druid Class's animated land (`DataKey::DruidClass`).
Where it would hook: `state.rs` `Game::is_creature` and `Land`; `engine/zones.rs` `die`/`leave` for the return; `engine/mana.rs` `spend_unit` (Source::Land) to fire `mana_tapped` for creature lands.
Card side done: the second ability (an additional {G} whenever a creature is tapped for mana), `t6_green1::cub_tapped`, reusing the existing `mana_tapped` slot.
Test to add once the engine has it: Cub enters with a Forest in play: the Forest is a 1/1 haste creature land; tapping it for mana makes {G}{G} (Cub's bonus); destroying it returns it tapped.

## Vibrance
Needs: hybrid mana costs ({R/G}) kept in card data and paid with a chosen colour, and "mana of colour X was spent to cast it" tracked on the spell for enter triggers.
Where it would hook: `cost_of` / `pay` in engine/mana.rs and `Enter { was_cast }` in engine/zones.rs.
Card side done: evoke options from hand for {R}{R} (3 damage) and {G}{G} (land + 2 life), sacrificed on entry, via a thread-local mode; a normal cast is priced {3}{R}{R} and deals 3 damage.
Test to add once the engine has it: cast Vibrance for {3}{G}{G}: land to hand, 2 life, no damage; cast for {3}{R}{R}{R}... both modes when {R}{R} and {G}{G} were both spent.

## Springheart Nantuko
Needs: bestow (cast as an Aura attached to a creature, +1/+1, becomes a creature when the host leaves) for a general card, and a landfall "pay {1}{G}: token copy of the enchanted creature". `partials::bestow_*` exists only for Eidolon of Countless Battles.
Where it would hook: `DataKey::Bestow` / `cardcode::bestow_fall` in engine/zones.rs, `partials.rs` bestow section.
Card side done: 1/1 body; landfall makes a 1/1 green Insect (the "if you didn't" branch).
Test to add once the engine has it: Nantuko bestowed on Eternal Witness; landfall pays {1}{G} for a Witness token copy (its ETB returns a card).

## Biomancer's Familiar / Training Grounds (remaining)
Done 10/17 (t6-e-mana): `creature_ability_cost` is called by Thrasios, Oboro, Magus, Speaker. Every other creature ability still hard-codes its cost in its own `can_pay`/`pay`; route each through `t6_green1::creature_ability_cost` as it is touched. Familiar's adapt clause needs adapt.
