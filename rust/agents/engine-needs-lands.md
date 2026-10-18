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

## Emergence Zone
Needs: the AI's "cast as though it had flash" check reads a one-turn grant from a land ability, not only Quicksilver / Captain Mar-Vell.
Where it would hook: impls/t0_captain.rs `as_flash` (read by the AI's end-of-turn window for creature spells): also return true when `player.flag_turn[intern("as_flash_turn")] == g.turn_stamp()` (t6_lands::FLASH_FLAG).
Card side done: {1},{T}, sacrifice cost and the flag (t6_lands.rs emergence_go, tested); the option is not offered until the engine reads the flag (const EMERGENCE_LIVE = false in t6_lands.rs; set it true when as_flash reads the flag).
Test to add once the engine has it: Emergence Zone used on an opponent's end step lets the AI cast a creature from hand that turn.
