
## (t6-e-altcost) Snapback's mana cost
Card data: `data/cards.json` gives Snapback {1}{U}; the printed card is {3}{U}. Not checked against Scryfall (offline); the alternative cost works either way. Where: data/card_overrides.toml / the cardgen source of the cost. **Lead, 2026-10-17: checked on Scryfall: Snapback is {1}{U}; the data is right, nothing to fix.**

## (t6-e-altcost) Human (practice-mode) alternative costs
`human/legal.rs` and `human/human.rs` offer only the Force of Will pitch; Daze, Flare of Denial, Disrupting Shoal, Mindbreak Trap, Snapback, Pyrokinesis and Sunscour alternative costs are not offered to a person yet (they go through `engine::altcost::pay_alt`, which picks the cards for the AI).

## (lead, 2026-10-17) The legend rule (DONE: t6-e-legend, engine/life.rs sba_legend)
The engine has no legend rule (found by t6-e-copies): two legendary permanents with the same name under one controller
both stay. It is a state-based action (CR 704.5j): the controller chooses one and the rest go to the graveyard. Affects
every deck; for Godo, a Twinflame / Heat Shimmer / Molten Duplication copy of Godo (legendary) must die at once (its
enter trigger still fetches an Equipment, which the Hammer-first line relies on), while Helm tokens (not legendary)
stay. Priority 1: add to the SBA check (engine/life.rs check_state), with the AI keeping the better one.
