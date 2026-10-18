# Engine needs: batch `green1`

## Badgermole Cub
Needs: earthbend N / lands that are creatures: a land you control becomes a 0/0 creature with haste that is still a land (+N +1/+1 counters), and "when it dies or is exiled, return it to the battlefield tapped". Lands are `LandId`s, not `PermId`s, so `is_creature`, creature mana (Cub's additional {G}, Cradle's count), summoning sickness and combat all ignore them. A general "animate land" (a `Land` flag read by `is_creature`/`pval`, plus the return-tapped replacement) would also serve Druid Class's animated land (`DataKey::DruidClass`).
Where it would hook: `state.rs` `Game::is_creature` and `Land`; `engine/zones.rs` `die`/`leave` for the return; `engine/mana.rs` `spend_unit` (Source::Land) to fire `mana_tapped` for creature lands.
Card side done: the second ability (an additional {G} whenever a creature is tapped for mana), `t6_green1::cub_tapped`, reusing the existing `mana_tapped` slot.
Test to add once the engine has it: Cub enters with a Forest in play: the Forest is a 1/1 haste creature land; tapping it for mana makes {G}{G} (Cub's bonus); destroying it returns it tapped.

## Biomancer's Familiar (and Training Grounds)
Needs: a general "creature ability cost reduction": every activated ability of a creature (generic part reduced by {2} per Familiar / Training Grounds, never below {1}) should go through one function before it is paid. Today each ability hard-codes its cost in its own `can_pay`/`pay`. The helper `t6_green1::creature_ability_cost(g, p, printed_generic)` (and `ability_discount`) already counts the two cards; the engine pass only has to call it from the creature abilities that matter: Thrasios, Triton Hero ({4}, `tymna_cards::thrasios_options`/`thrasios_go`, both the affordability check and the payment) first of all, and Rograkh's partner deck's other creature abilities.
Where it would hook: `impls/tymna_cards.rs` `thrasios_options` / `thrasios_go` (replace the literal 4); a shared `ability_cost` wrapper if more abilities want it.
Card side done: the two cards are recognised by name; the discount is applied to Oboro Breezecaller, Magus of the Candelabra and Formidable Speaker.
Test to add once the engine has it: with Training Grounds in play, Thrasios activates with only {2} available (and with both Grounds and Familiar, with {1}).
Familiar's second ability (the next adapt this turn ignores +1/+1 counters) needs adapt, which the pool does not have.
