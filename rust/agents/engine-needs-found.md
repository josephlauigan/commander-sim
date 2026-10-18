
## (t6-e-altcost) Snapback's mana cost
Card data: `data/cards.json` gives Snapback {1}{U}; the printed card is {3}{U}. Not checked against Scryfall (offline); the alternative cost works either way. Where: data/card_overrides.toml / the cardgen source of the cost.

## (t6-e-altcost) Human (practice-mode) alternative costs
`human/legal.rs` and `human/human.rs` offer only the Force of Will pitch; Daze, Flare of Denial, Disrupting Shoal, Mindbreak Trap, Snapback, Pyrokinesis and Sunscour alternative costs are not offered to a person yet (they go through `engine::altcost::pay_alt`, which picks the cards for the AI).
