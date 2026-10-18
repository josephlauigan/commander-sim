# Engine needs: lands batch

## City of Traitors
Needs: a "you played another land" event that lands (not only permanents) can hook, so the City is sacrificed even when the new land has its own land_etb.
Where it would hook: engine/turn.rs play_land_card (fires Event::LandPlay for permanents only; cardcode::land_etb is one slot per land card).
Card side done: the City is sacrificed through land_etb registered on every land without its own land_etb (impls/t6_lands.rs); lands that have their own land_etb (Bojuka Bog, Urza's Saga, Mortuary Mire, Mystic Sanctuary, Mosswort Bridge, Windbrisk, Alela's hub land) do not sacrifice it.
Test to add once the engine has it: play Bojuka Bog with City of Traitors on the battlefield; the City goes to the graveyard.
