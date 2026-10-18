# Engine needs: batch legends2

Card side for all of these is in `crates/sim-core/src/impls/t6_legends2.rs` (tests in `tests/cards_t6_legends2.rs`).

## Magda, Brazen Outlaw
Needs: a general "permanent becomes tapped" event (any tap: a tap cost, a "tap target" effect), not only the attack declaration and mana abilities. `impls/t0_captain.rs::became_tapped` is already called at the attack declaration, mana abilities and the ability language's tap cost; a hook slot fired from the same places would serve Magda.
Where it would hook: `engine/mana.rs` (Source::Perm), `engine/combat.rs` ~1068, `dsl/mod.rs` ~1719 (the three `became_tapped` call sites).
Card side done: Treasure on a Dwarf tapping for mana (`mana_tapped` slot) and attacking (`attack` slot); the +1/+0 and the five-Treasure search.
Test to add once the engine has it: a Dwarf tapped by an opposing "tap target creature" effect still gives Magda's controller a Treasure.

## Teferi, Who Slows the Sunset
Needs: emblem hooks on the turn structure: "untap all permanents you control during each opponent's untap step" and "you draw a card during each opponent's draw step" for a player holding the emblem.
Where it would hook: `engine/turn.rs` where each player's untap step and draw step run (the emblem is the string `"teferi_sunset"` in `Player::emblems`; `cardcode::emblem_draw` is the existing per-draw emblem hook).
Card side done: +1 and -2 are complete; the -7 ability pushes the emblem (`teferi_ult`) but its value is `None`, so the AI never uses it until the effects exist (then give `teferi_ult` a value in `TEFERI`).
Test to add once the engine has it: with the emblem, an opponent's untap step untaps our lands and their draw step draws us a card.

## Spider-Woman, Stunning Savior
Needs: tapped artifact tokens: Treasure / Food / Clue are counters on the player (`treasures`, `foods`, `clues`), so "create a tapped Treasure" and "artifacts your opponents control enter tapped" cannot be told for them. A `tapped_treasures` count (untapped at the owner's untap step) would serve this card and Tataru Taru's Scions' Secretary (which now keeps its Treasure on the card until the next upkeep).
Where it would hook: `state.rs` Player, `engine/zones.rs::make_artifact_tokens`, the untap step in `engine/turn.rs`, and the mana code that spends Treasures.
Card side done: opposing artifact and creature permanents and creature tokens enter tapped (`spider_etb` on the enter event, `spider_tokens` on the tokens-enter event; both run at once).
Test to add once the engine has it: with Spider-Woman out, an opponent's Treasure token cannot pay for a spell the turn it is made.
