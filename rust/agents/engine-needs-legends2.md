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

## Ertai Resurrected
Needs: the AI answering an opponent's activated or triggered ability on the stack with a flash creature whose enter trigger counters it. `stack::ai_respond` reaches `cardcode::answer_ability` for abilities and triggers (importance 6 or more); that function only knows Azorius Guildmage and Tishana's Tidebinder. Add a call to `t6_legends2::ertai_answer(g, q, item)` there (it returns true when Ertai was cast). Reuses the existing hook.
Where it would hook: `impls/partials.rs::answer_ability` (called from `cardcode::answer_ability`, `engine/stack.rs::ai_respond`).
Card side done: `ertai_answer` (flashes Ertai in for {2}{U}{B}, counters the item, marks `countered_by`, Ertai's enter hook draws the card for the ability's controller). The counter-a-spell mode already works through the `ctr=any` tag (draw added by the enter hook); the destroy mode is the enter hook. Against Godo the call is what counters his enter trigger.
Test to add once the engine has it: Godo's enter trigger on the stack with Ertai and {2}{U}{B} open is countered by the AI through `ai_respond`, and Godo's controller draws.

## Legolas's Quick Reflexes
Needs: (1) split second (while it is on the stack nobody can cast spells or activate non-mana abilities); (2) the "permanent becomes tapped" event (the same one Magda needs) for the until-end-of-turn trigger "whenever this creature becomes tapped, it deals damage equal to its power to up to one target creature"; (3) the AI casting a response spell to protect a creature (hexproof in answer to removal).
Where it would hook: (1) `engine/stack.rs::stack_window` / `ai_respond` and the human legal-cast check `human/legal.rs`; (2) the `became_tapped` call sites listed under Magda: store the pending trigger on the creature (a `Charge`-like `DataKey`) and fire it there; (3) `engine/stack.rs::ai_respond` for removal on the stack, calling `t6_legends2::legolas_cast`.
Card side done: `legolas_resolve` (untap, reach, hexproof until end of turn; the AI holds the card with a zero `prio`), `legolas_cast` (the {G} response cast, countered like any spell).
Test to add once the engine has it: with Legolas's Quick Reflexes on a tapped creature that then attacks, the attack deals damage equal to its power to a creature; and no spell can be cast in response to it.
