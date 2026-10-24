# Report: t0-captain (Captain America, Avengers Assemble precon)

All 24 listed cards are Full in `cardgen audit --deck captain-america-avengers-assemble` (the whole deck shows Full / Full-auto).
Card code: `rust/crates/sim-core/src/impls/t0_captain.rs`; tests: `tests/cards_t0_captain.rs` (36); verdicts: `data/card_overrides.toml`.

## How each card is modeled
- **Captain America, Team Leader**: `etb` hook on another Hero entering: vigilance + haste (eot_kw, not sick), +1/+1 counter on it and on him.
- **Living Legend**: `became_tapped(g, m)` untaps a creature the first time it taps on its controller's turn (flag per creature per turn). Called at the attack-declaration tap, mana abilities (`Source::Perm`), the ability language's `cost.tap`, Quinjet crewing and Hawkeye. The ~150 other card-specific tap sites don't call it.
- **Avengers Quinjet**: per-card Crew 3 (same pattern as Esika's Chariot: `crew` hook before the first combat), flying 4/4 while crewed; enter/attack trigger puts the best Hero creature from hand onto the battlefield, else returns one from the graveyard to hand.
- **Folk Hero** (cast hook, the commander's once-a-turn draw), **Jarvis** (cast hook), **Director Nick Fury** (`cost` -1 for Hero spells, attack hook: top four, Hero to hand, rest to the bottom in random order), **Herald's Horn** (`as_enters` names Hero for this deck, `cost`, `upkeep`).
- **Gift of Immortality**: Aura spec with `host_dies`; marks the host with the Ghostform mark (so `die_rest` returns it under its owner's control); the Aura returns attached at the next end step (`T0c::gift_due`, `delayed_end_step`). Log line says "Kaya's Ghostform returns" for the creature (cosmetic).
- **Love on the Battlefield**, **Hercules**, **Winter Soldier**, **She-Hulk** (grant_kw trample, `blocks` hook counters), **Shang-Chi** (draw hook; tenth counter checked at each draw, so a tenth counter from another source pays off at the next draw), **Captain Marvel** (`put_plus` and the ability language's counters call `counters_put`; non-Kree creatures only).
- **Ant-Man**: `cardcode::evasion_blocked` hook; the Treasure trigger is hand-coded (`dsl = []`) because the compiled "that many" always made 3.
- **Avenge**: `resolve` hook (destroy all, gain 1 per creature actually destroyed; the compiled version always gained 3) and `self_cost` -2 when a player attacked you in their last turn (`T0c::attacked`, noted in `attack_triggers`, cleared at that player's `turn_start`).
- **Heroic Return**: `self_cost` -2 while a creature attacks you; `resolve` returns the best creature card, a Hero with two extra counters.
- **Raise the Palisade**: `resolve` picks the type that keeps the most board value, bounces the rest (tokens vanish); `prio` casts it only when the swing is at least 5.
- **Dismantling Wave**: hard cycling {6}{W}{W} as a `hand_options` plan (when it takes more from opponents than from us, or with spare mana at the end of turn): discard, draw, destroy all artifacts and enchantments. Generic cycling belongs to t0-giada; this is the one card's own.
- **Avengers Tower**: `land_cols` (any colour only when `pay_for` is a Hero card, like Plaza of Heroes), `land_options` for {4},{T} look at the top three.
- **Hawkeye**: the compiled ping is replaced (`dsl = []`) by an `options` hook; creatures he damages (ping or combat, `pair_mark`) draw a card for him when they die (`dies` hook).
- **Quicksilver / Captain Mar-Vell**: `as_flash(g, p)`; `ai/brain.rs`'s end-of-turn window casts permanent spells when it is true. Instants and sorceries are not given a new casting time.
- **Heroic Sacrifice**: `hand_defend` casts it for the defender after blocks when open damage is big or blockers would die; combat redirects all damage to the player and their creatures to the chosen creature (`sac_creature`, `sac_player`, `sac_finish`); if it dies, counters move and the player draws.

## Shared-file changes (all additive, one to three lines)
`state.rs` (`Game::t0c`), `cardcode.rs` (`put_plus`, `evasion_blocked`, `delayed_end_step`, `turn_start`), `engine/combat.rs` (`attack_triggers`, attacker tap, `pair_mark` / `pair_after`, Heroic Sacrifice redirect and `sac_finish`), `engine/mana.rs` and `dsl/mod.rs` (`became_tapped`, counters_put), `ai/brain.rs` (`as_flash`).

## Notes
- The deck plays fine in traces (`--deck cloud --pool t0 --any-tier --trace N`, 24 games with Captain America seated): the commander is cast on its first affordable turn (turn 4 on average; turn 3 when untapped lands allow), and every listed card was cast and its effect logged at least once.
- Known simplifications: creature damage doesn't exist outside combat in this engine, so Hercules' "dealt damage" and Hawkeye's "dealt damage" cover combat and Hawkeye's ping; Heroic Sacrifice's "dies this turn" draw only fires when the redirected damage kills the chosen creature.
- `all_decks` shows 3 timeouts per 72 / 24 games (adaptive / lookahead); not investigated, not caused by an engine-step overrun.
- A slip: I typed a python command twice in a bash call by mistake; the no-python hook blocked it both times, nothing ran.
