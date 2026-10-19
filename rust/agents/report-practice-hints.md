# Report: practice-hints

## Inventory: request kinds a human seat gets (sim-core/src/human/*, session/mod.rs)
| kind | prompt (examples) | hint before | hint now |
|---|---|---|---|
| mulligan | "Your opening hand (N to keep)." | "No hint ... yet" | Keep / Mulligan with the AI's rule, lands, mana help, colours, turns 1-3 plays, commander timing, bottom card (real: `turn::keeps_hand`, `bottom_pick`) |
| choose, "Put a card on the bottom ..." | after a mulligan | none | the AI's bottom card (real) |
| priority (main, empty stack) | "Main 1: you have priority" | AI main-phase pick | unchanged |
| priority (spell on stack) | "... You have priority" | counter pick / generic pass | unchanged |
| attack | "Declare attackers" | look-ahead pick (or tip without look-ahead) | unchanged |
| choose: discard | "Discard a card", "End of turn: discard down to N" | none | the AI's discard rule (real: land if 3+ lands, else dearest spare spell) |
| choose: scry / surveil | "Scry: Card {cost}" | none | the AI's `topdeck::desire` vs `KEEP` (real) |
| choose/target: sacrifice | "... sacrifice which creature?" | none | `brain::spare_creature` (real) when it is among the choices |
| block | "X attacks you ... Block Y with?" | none | tip (life, trade value) |
| target | 68 call sites | none | tip |
| search | tutors, basic lands | none | tip (lands first while under ~5, else the best missing card) |
| choose: yes/no, modes, X, pay/tap, creature type, cast, other | ~57 call sites | none | tips by prompt wording |
| continue | step-by-step pause | none | "Nothing to decide" |
| any other kind | | none | a generic full sentence naming the kind and showing the prompt |

Real AI picks: mulligan, bottom, discard, scry/surveil, sacrifice, main phase, attack, counter. Tips: block, target,
search, other choose variants, continue. The AI has no per-request function for blocks/targets/modes (its choices sit
inside whole-turn code), so they are sentences, not picks.

## Ways a hint came out empty or null
1. Every kind except priority/attack returned "No hint for this kind of decision yet." (mulligan, choose, target,
   search, block, continue): the box with nothing useful.
2. `choice` was null in several answers (generic pass, no look-ahead attack), nothing to show.
3. The page (`hint()` in app.js) printed `r.data.text` unchecked: a missing/null `text` showed "The AI would: null",
   a missing `detail` threw a TypeError (box stuck on "The AI is thinking..."), and an error without `error` printed
   "undefined". Fixed: the page shows "No hint for this decision." and skips non-string detail lines.
4. A panic inside the copy would have killed the request. `advisor::hint` now catches it and answers with a sentence,
   and normalises `text` (non-empty, not "null"), `detail` (array) and `choice` (key present).
I could not make the server itself return a null `text` from the old code; the null came from the page and the paths above.

## Changes
- sim-core `engine/turn.rs`: `keeps_hand` and `bottom_pick` pulled out of `mulligan` as `pub fn` (same logic, the AI's
  mulligan now calls them; no change to decisions).
- practice `advisor/` (mod.rs, mulligan.rs, other.rs); `shadow.rs` records a `mulligan` entry (yours vs the AI's keep,
  differs flag, reason in `ai_note`, saved in game saves); `static/app.js` hint box.
- Tests: `tests/hint_every_decision.rs` (5 seeded games, hint at every request, mulligans twice to reach bottoming; every
  kind built by hand), `session_shadow.rs` (mulligan review).
- Not covered live: attack and block requests (the scripted player can't tap mana to cast creatures); both are covered
  with hand-built requests.
- The AI's mulligan rule does not look at key cards, colours or rocks; the hint says so rather than inventing a rule.
