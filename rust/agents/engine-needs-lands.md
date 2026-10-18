# Engine needs: lands batch

## City of Traitors
Needs: a "you played another land" event that lands (not only permanents) can hook, so the City is sacrificed even when the new land has its own land_etb.
Where it would hook: engine/turn.rs play_land_card (fires Event::LandPlay for permanents only; cardcode::land_etb is one slot per land card).
Card side done: the City is sacrificed through land_etb registered on every land without its own land_etb (impls/t6_lands.rs); lands that have their own land_etb (Bojuka Bog, Urza's Saga, Mortuary Mire, Mystic Sanctuary, Mosswort Bridge, Windbrisk, Alela's hub land) do not sacrifice it.
Test to add once the engine has it: play Bojuka Bog with City of Traitors on the battlefield; the City goes to the graveyard.

## Gemstone Caverns
Needs: a start-of-game step after the opening hands: a player not going first may begin the game with this land from the opening hand on the battlefield (with a luck counter) and exile a card from hand.
Where it would hook: engine/turn.rs game setup after the mulligans (where the opening hand is settled); the keep/mulligan code.
Card side done: taps for {C}; with a luck counter (DataKey::Counters Int on the land) it makes one mana of any colour (t6_lands.rs caverns_cols). Played from hand it is a plain {C} land, as the Oracle says.
Test to add once the engine has it: seat 2 opens with Gemstone Caverns in hand: it starts on the battlefield with a luck counter, one other card exiled, and taps for any colour.
