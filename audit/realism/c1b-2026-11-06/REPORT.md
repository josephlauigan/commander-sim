# C1b: Tier 6 decks hold their draw engines and wheels (branch c1b-engines, 2026-11-06)

Brief: `rust/agents/c1b-engines.md`. Branch c1b-engines, from c1-t6-lines 8a7650e8 (PR #271). The agent returned this report
as text (its write of this file was refused); the coordinator saved it. No sim by the agent: evidence is single-game
traces (`commander-sim --calibrate within --pool t6 --ai adaptive --games 4000 --seed 900000 --trace N`, N = 1..400,
before and after; each deck sat in about 270). The before-files are in this folder; the coordinator's scorecards are at
the end.

## The trace line
`brain::stop_diag` (trace only: reads the game, draws no random numbers) runs when a Tier 6 deck passes its main phase. For
every payable nonland card it prints its `card_utility` (or "no utility (prio P)" / "not castable"), the utility offered
after the reserve penalty, then the stop's utility, the card held for (`hold_value`), the plan's `hold_up` and `near`.
Example (g20, round 5): `[Rograkh/Silas stops (u 7.0; hold Ad Nauseam 7.0, plan 0.0, near false), mana 5, hand 11;
payable: Ad Nauseam: no utility (prio 80); Wheel of Fortune: no utility (prio 0); ... Silas Renn: u 6.1 (offered -1.9, prio 76)]`

## Why the AI holds each card
`brain::main_options` offers every payable play at a utility and "stop" at `hold_v` + the plan's `hold_up`; `hold_v` is the
value of the mana kept up for the cheapest held answer (a counter, a protection spell, or Ad Nauseam). The AI samples an
order from the softmax, so a play 0.5 below "stop" is passed about half the time, and "stop" ends the phase.
1. **Reserve penalty.** A cast that leaves the held card unpayable loses 0.7 x `hold_v`, or 8.0 when `near`. `near` =
   `combo_near` (archenemy score >= 5) or `cmdr_combo` (any opposing commander on the battlefield that is a modelled combo
   piece). In a cEDH pod an opposing Kinnan, Thrasios, Tymna or Rograkh is nearly always out, so "an opponent is close to
   winning" was true on turn 2: Rograkh/Thrasios's Rhystic Study offered at 6.4 became -1.6 on round 2 (g31); Thrasios in
   Kinnan u 4.4 offered at -5.1 (g373). `combo_near` alone is true in 2% of Tier 6 stops in rounds 1-3, 20% in 4-6, 35%
   later; the rest was `cmdr_combo`.
2. **Mana held for an Ad Nauseam that could not be cast.** `hold_value` keeps up to 7.0 of "stop" for any Ad Nauseam in
   hand, but Ad Nauseam is cast only with room for four cards (`adnaus_cast_now`). Rograkh/Silas held 5 mana for it with 11
   cards in hand (g20; also g250, g17 with 14): "stop" at 7.0 beat everything and the hand never shrank. 340 of 12361 Tier 6
   stops held for Ad Nauseam, about 200 with 6+ cards in hand.
3. **Coin flips.** An engine offered at 6.0 against a stop at 5.5-8.0 is passed about as often as cast (g113).

| card (deck) | what the traces showed | cause |
| --- | --- | --- |
| Rhystic Study, Remora, One Ring, Sylvan Library, Esper Sentinel (all) | offered at -2 to +0.5, or below stop | reserve penalty with `cmdr_combo`; Ad Nauseam hold; coin flip |
| Necropotence (Rograkh/Silas, Kraum) | 6 offered negative, 5 below stop; prio 0 only with 7+ other cards | same; the 0 is `necro_prio` (room < 2) and is right |
| Thrasios (Kinnan), Faerie Mastermind (Kinnan, Kraum, Rograkh/Thrasios) | Thrasios u 4.4 offered -5.1 (g373) | same, plus the generic creature priority (44) |
| Wheel of Fortune, Windfall (Rograkh/Silas) | prio 0 in 55 stops, rounds 1-6 | `wheel_prio`/`windfall_prio` need hand <= 2; or a combo piece in hand (deliberate) |
| Timetwister (Rograkh/Silas) | offered below stop 10 times | the generic 46 for any draw spell |
| Ad Nauseam (Rograkh/Silas, Kraum) | "no utility" in 98 + 63 stops | `adnaus_cast_now`: deliberate |
| Remora, Esper Sentinel "not castable" (Kraum, Sisay, Rograkh/*) | e.g. g125 | Dack's Chalice of the Void at 1 / Void Mirror: a lock |
| Faerie Mastermind (Rograkh/Thrasios) "no utility" | g117, g34 | the plan holds it while Green Sun's Zenith is in hand (its target): deliberate |

## Fixes (Tier 6 only)
1. **`brain::engine_rule`** (`ENGINE_MARGIN` 1.0, `ENGINE_EARLY_TURN` 4; `T6_ENGINES`: Rhystic Study, Mystic Remora, Esper
   Sentinel, Sylvan Library, The One Ring, Necropotence, Faerie Mastermind, Thrasios when not the commander, Consecrated
   Sphinx). Applies when no opponent is `combo_near` (archenemy reading only) and none of the deck's own lines is ready.
   Through own turn 4 the reserve penalty is waived; whenever the penalty is 0 the cast is offered at least stop + 1 (a
   flash creature only on curve). A plan that sets the card to 0 still holds it.
2. **Ad Nauseam hold** (`brain::hold_value`, `plans::adnaus_room`): mana is held for it only with room for four cards.
3. **Wheels** (`partials::wheel_prio`, `partials::t6_refill`, `t5::windfall_prio`, `pool::generic_prio_base` for
   Timetwister): Tier 6 also wheels from a stuck hand (at most one other payable nonland card, 2+ mana left after, no
   opponent with more than 5 cards, no combo piece in hand). The old "never wheel a combo piece" and "hand <= 2" rules stay.
4. Necropotence's pay (`decks::necro_pay`) is unchanged and right.

## Kept (deliberate)
Ad Nauseam as a finisher (opposing end step, with room); no wheel over Tainted Pact / Demonic Consultation / any combo
piece; Faerie Mastermind waits while GSZ is in hand (Rograkh/Thrasios plan); engines held while an opponent is
`combo_near` (all 25 negative-offered engine sightings in rounds 1-4 after the fix are `near=true`); Remora and Esper
Sentinel under Chalice at 1 and Void Mirror.

## Tests (`tests/c1b_engines.rs`, 11)
9 fail with the new rules off and pass now (Rhystic and Necropotence on turn 3 and Rhystic on turn 7 with Ad Nauseam held;
Thrasios and Mastermind on curve with counters held; an opposing combo commander alone; the engine rule's gates; mana not
held for an uncastable Ad Nauseam; a stuck hand wheels with Wheel of Fortune, Windfall, Timetwister); 2 controls. The agent
ran 74 test binaries (1211 tests) including JD's decks' card files: 0 failed.

**JD's decks affected:** `thrasios` (a Tier 6 key) gets the engine rule and the Ad Nauseam hold. No JD deck plays a wheel.
The others are not Tier 6 keys: unchanged.

## Effect on the same 400 traced games (not a measurement: games diverge after the first changed choice)
Stops that held for Ad Nauseam 340 -> 144; the nine engines sighted payable at a stop in rounds 1-4: 88 -> 48, offered
negative 45 -> 25. Casts before -> after: Rograkh/Silas Rhystic 25 -> 33, Timetwister 21 -> 39, Necropotence 39 -> 38;
Kraum/Tymna Rhystic 48 -> 46, Necropotence 19 -> 21; Kinnan Rhystic 39 -> 39, Remora 40 -> 41, Mastermind 38 -> 39;
Rograkh/Thrasios Remora 49 -> 52; Sisay Rhystic 26 -> 24. Each card is stuck in only 2-6% of games: expect a small effect.

## What is left (for JD)
1. **Sheoldred (JD's deck)** has the same Ad Nauseam hold with a full hand; the fix is gated to Tier 6 so her numbers don't
   move. Apply it to her too?
2. **`cmdr_combo`** still counts as "an opponent is close to winning" for every other Tier 6 cast (reserve penalty 8.0, +2.5
   on the hold value); only the engine rule now reads the archenemy alone. Switch all Tier 6 casts?
3. "Cantrips cast 0.02 a turn" is the lists: the six Tier 6 lists have no Brainstorm, Ponder, Preordain or Opt.
4. The median win round 8 against 5 is not an engine-hold problem: missed land drops with no land in hand 36% by turn 3
   and 52% by turn 4; Dack, Sisay and Rograkh/Silas draw 1.0-1.3 cards a turn through turn 4. The lists, the mulligan rule
   and the mana set the speed.
5. An engine offered at +1 over stop is still passed about one time in five at the Tier 6 temperature.

Files: `ai/brain.rs` (stop_diag, engine_rule, hold_value, main_options), `ai/plans.rs` (adnaus_room), `ai/pool.rs`
(Timetwister), `impls/partials.rs` (wheel_prio, t6_refill), `impls/t5.rs` (windfall_prio), `tests/c1b_engines.rs`,
`rust/agents/handoff-c1b-engines.md`.
