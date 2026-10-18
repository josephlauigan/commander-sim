## Derevi, Empyrial Tactician
Needs: activated ability from the command zone (put this card onto the battlefield from the command zone for {1}{G}{W}{U}, any time it is in the zone)
Where it would hook: a command-zone options slot like hand_options/gy_options (hooks.rs CardOptionsFn), in the brain's option list
Card side done: enters / combat-damage "tap or untap target permanent" (untap mana source, else tap best blocker)
Test to add once the engine has it: Derevi as commander in the zone, {1}{G}{W}{U} available, ability puts her onto the battlefield and the enter trigger untaps a tapped dork

## Agatha's Soul Cauldron
Needs: (1) a "spend mana as though any colour" player flag for activated abilities of creatures (mana payment context: ability vs spell); (2) granting activated abilities of other (exiled) cards to a permanent, applied while it has +1/+1 counters
Where it would hook: engine/mana.rs plan_pay (pay_for / ability context), and the options collection in ai/brain.rs for borrowed abilities
Card side done: {T}: exile creature card from an opponent's graveyard (best by card_worth), +1/+1 counter on best own creature (options hook, t6_legends1.rs); data DSL (wrong "exile all graveyards") replaced by dsl = []
Test to add once the engine has it: Cauldron with Sisay's exiled ability: a creature with a counter offers Sisay's activation, paid with any colours

## Deadpool, Trading Card
Needs: replace a permanent's text box (abilities, compiled triggers, hook slots) with another permanent's, as it enters (a per-permanent "ability source card" indirection for registry lookups)
Where it would hook: hooks.rs Registry lookup by card (hooked(), imp_of) and the enters path (as_enters slot, hooks.rs AsEntersFn)
Card side done: 5/3, upkeep lose 3 life, {3} sacrifice: other players draw (options hook, t6_legends1.rs)
Test to add once the engine has it: Deadpool entering swaps with an opposing 1/1 Spirit token that has the drain upkeep trigger; the Spirit's controller loses 3 life at their upkeep and Deadpool has a Spirit's (blank) text
