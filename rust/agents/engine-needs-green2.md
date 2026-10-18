# Engine needs: batch green2

## Sowing Mycospawn
Needs: kicker (an additional cost {1}{C} chosen as the spell is cast, with a "was kicked" flag the cast trigger can read); also devoid (colourless card) is not modelled anywhere.
Where it would hook: engine/cast.rs `cast_card` (cost payment) and `cardcode::self_cast` (would need the flag, e.g. on Ctx).
Card side done: the cast trigger fetches a land (even if countered); `t6_green2::mycospawn_kicked` exiles the best opposing land (not yet called).
Test to add once the engine has it: cast kicked with {1}{C} spare, an opposing land is exiled and the unkicked cast leaves it.
