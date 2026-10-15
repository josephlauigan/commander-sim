# Report: t0-giada (DONE)

All 30 cards of `cardgen audit --deck giada-calling-all-angels` are Full. Card code: `impls/t0_giada.rs`; tests: `tests/cards_t0_giada.rs`
(35 tests, one per card plus a full game); verdicts and tag/DSL overrides appended to `data/card_overrides.toml`.

## How the cards are modeled
- Giada: etb and tokens_enter hooks add the counters (replacement, at once); Angel-only {W} is a check in `engine/mana.rs` (like Hydro-Channeler, via `pay_for`).
- Clues are `Player::clues`; `investigate` = `make_artifact_tokens(.., "Clue", n)`. Search the Premises, Wojek, Merchant, Angelic Sleuth, Fateful Absence use it.
- Triggers use `trigger_window` after their conditions; "enters with", statics and exalted run at once.
- Compiled DSL was replaced (`dsl = []`) where it was partial: Angel of the Ruins, Court of Grace, Defy Death, Emeria Shepherd, Exemplar, Fateful Absence, Firemane, Herald of War, Heraldic Banner. Cut a Deal's DSL is exact, only its verdict changed.
- Exemplar of Light draws (once a turn) for counters from lifegain, Defy Death and Norn's Choirmaster's proliferate (not other sources).
- Heraldic Banner: the chosen colour is fixed to white (an all-white deck). Righteous Valkyrie's `anth` tag was removed and replaced by a conditional (47+ life) `creature_pt`.

## Approximations (documented in the overrides)
- Search the Premises / Firemane Commando: the attack hook sees the whole attack and its main defender only (split attacks count by the main defender); planeswalkers aren't defended.
- Angel of the Ruins' plainscycling is offered only at the end-of-turn window, under AI heuristics. Secret Rendezvous targets the opponent with the weakest board.
- Sephara's alternative cost is offered by the AI only when the mana cost is out of reach.

## Shared-file changes (all small and additive)
- `engine/life.rs`: `gain` adds `t0_giada::gain_bonus` (Angel of Vitality); `check_state` skips elimination if `t0_giada::cant_lose` (Herald of Eternal Dawn).
- `engine/cast.rs` `castable`: `cast_locked` (Serra Avenger). `engine/mana.rs`: Giada's mana only for Angel spells.
- `engine/combat.rs`: `combat_proof` (Seraph of the Sword takes no combat damage) and `damaged` (Metropolis Reformer). `engine/removal.rs`: `damaged` for damage spells.
- `engine/zones.rs` `leave`: queues Angelic Sleuth's Leaves hook for another permanent of its controller that had counters (the Leaves event only existed for the leaver itself).
- `data/cards.json` regenerated (`cardgen export`).

## Checks
cargo test --release -j 3: all pass; build with --all-targets: no warnings; all_decks adaptive 3 and lookahead 1: no panic;
smoke (cloud vs t0, 48 games, lookahead): finishes; Giada wins 71% of the games it is seated in (small sample).
Traces (games 1, 2, 5): Giada is cast on turn 2-4, Fateful Absence (the opponent investigates), Secret Rendezvous, Court of Grace (monarch), Metropolis Reformer lifegain all fire.
