# Report: model-gaps-b3 (follow-ups to model-gaps-b2 NEEDS JD items 2, 4, 7, 8)

Branch model-gaps-b2, commits ef6040b9, 2a40eca8, 1da76a8d, d323d676 (on top of 9d4b5c3f). Brief: `rust/agents/model-gaps-b3.md`.
The agent returned this report as text; the coordinator saved it here.

## 1. Soul-Guide Lantern's enter effect is a trigger
- Before: the person path (`human/karn.rs` `lantern_enters`) chose the card, then put it on the stack with
  `ability_window_card`, an activated-ability window. Rings could offer a copy; a Camera could not copy it.
- After: `lantern_enters` follows `spine_enters`. With no graveyard card it does not trigger. Otherwise
  `trigger_window(g, o, Some(src), "exile a card from a graveyard", Some(5.0))`; after the window it re-reads the
  graveyards, the person picks the card ("Soul-Guide Lantern: exile which card from a graveyard?"), and it is exiled.
- A Camera copy re-runs the hook through `run_copy`, so it asks for a new target (the copy resolves and asks first).
  Rings offers nothing (not an activated ability).
- Importance 5 on purpose: `urdragon::camera_window` gives the person priority on their own trigger only at imp >= 5 or
  copy_worth set; at the default 3 the Camera never got a window. Side effect: AI opponents see this person-only trigger
  as importance 5 instead of 3 (slightly more willing to respond to it).
- AI path: `impls/rules2.rs lantern_etb` untouched. Files: `human/karn.rs`.

## 2. Staff of Domination's untap is covered by the run's Rings answer
- `human/abilities.rs staff_draw`: `ans` (the run's answer, or Some(false) for the last odd draw of a longer run) is
  computed before the untap; both the untap and the draw run under `with_rings_auto(ans, ...)`. The run's other questions
  are identical (one "pay {2} to copy each draw ability" question).
- n = 1: no preset answer, so Rings' ordinary per-activation question is asked for the untap (if tapped) and for the draw.
- The untap before the last odd draw is not copied (Some(false), the same as that draw).
- Tests: Rings "yes", n = 4: four cards, 999 - 7 - 3 - 7 mana spent (the untap's {1}+{2} paid), one Rings prompt. Rings
  "no": 999 - 5 - 3*6, one Rings prompt.

## 3. A person activates Clue, Food and Blood tokens
- Storage: counts on the player (`clues`, `foods`, `bloods`), not permanents, so no permanent ability menu entry.
- New action `{"do":"use","token":"Clue"|"Food"|"Blood"}` in `human::human::apply`; new module `human/tokens.rs`
  (`KINDS`, `count`, `problem`, `use_token`).
  - Clue: {2}, sacrifice: draw. Food: {2}, {T}, sacrifice: gain 3 life. Blood: {1}, {T}, discard a card (person's choice),
    sacrifice: draw.
  - Paid from the pool (`mana::pay_from_pool`); Forensic Gadgeteer's discount via `artifact_cost` (Blood never below {1}).
    The AI's Clue path already applies it; the AI's Food and Blood paths do not.
  - Discard through `zones::discard_cards`. Sacrifice is a cost: the count drops and `Event::Sacrifice` fires with
    `Sacrificed::Token(kind)` before the ability goes on the stack (tested with Trail of Crumbs). Then `ability_run`
    (the table can respond; Rings and a Camera can copy it; a copy pays nothing). Shown as "A token: Clue: draw a card".
- Refusals (nothing changes): "You have no Food token."; "All your Food tokens are tapped." (The Cabbage Merchant's tapped
  Foods; not covered by a test); "You have no card in hand to discard to the Blood token."; "Can't activate a Clue
  token. <the pool's reason>".
- `practice` crate: `src/view.rs` (`me.tokens` = {kind: count}, non-zero only), `static/app.js` (a "Tokens" row of
  buttons), `src/text.rs` (`use clue|food|blood`, a "Your tokens" line, help line, unit test).
- AI untouched (`ai::act::clue`, `common::eat_food`, `sheoldred::blood_rummage`).
- Tests: Clue (cost, draw, 2 refusals), Food (+3, refusal), Food's sacrifice trigger (Trail of Crumbs), Blood (discard
  choice, empty-hand refusal, no-mana refusal), Rings copy of a Clue (draws 2, one prompt), `count`.

## 4. Lathliss, Dragon Queen's pump
- AI option (`impls/t45_urdragon.rs lathliss_options`): k was `(total_mana - 2) / 2`, but each activation needs {R}.
  k is now the largest value <= that which `can_pay(k, "R"*k)` accepts (binary search).
- AI behaviour change: the AI no longer gets an offer it could not pay (1 red source and 6 mana: +1 instead of a refused
  +2; no red: nothing). With enough red it is identical (6 Mountains: +2).
- Person: before, a person saw the AI hook (k from untapped sources while the person pays from the pool; gated by AI
  heuristics: main 1 only, an open attack, nothing castable). New person path `human/urdragon.rs lathliss_go`
  (registered in `urdragon::abilities` and `cards::abilities`, hiding the hook for a person): listed at any time; with no
  {1}{R} in the pool it refuses ("Can't activate Lathliss, Dragon Queen. <reason>"); otherwise the person chooses 1..k
  activations (k = what the pool pays), each its own {1}{R} and its own `ability_run`; with Rings and n > 1 one answer
  for the run. `impls::t45_urdragon::lathliss_pump` is the shared effect.
- Tests: AI with 1 red and 5 Islands (+1), no red (no offer), plenty (+2); person (1 time with {R}{C}{C}{C}, 3 with 3
  red, refusal with no red, Rings with one answer for the run).

## Shared-file changes
- `engine/stack.rs`: none.
- `impls/t45_urdragon.rs`: the k clamp (AI offers) and `pub fn lathliss_pump`.
- `human/` (person only): abilities (Staff), karn (Lantern), tokens (new), human (`use token` action), mod, cards
  (dispatch), urdragon (Lathliss).
- `practice` crate: view.rs, text.rs, static/app.js.
- Tests: `tests/model_gaps_b3.rs` (new, 18 tests).

## Verification
Agent: `model_gaps_b3` and the brief's list with `--no-fail-fast`, `practice` crate tests, build with no warnings.
Coordinator: full suite 4226 passed, 0 failed (5 ignored); build no warnings; audit unchanged.

## Notes
- A single-draw Staff run (n = 1) asks Rings twice (untap, draw) if the Staff was tapped; n > 1 asks once.
- Rings is also offered a copy of the Camera's own ability when the Camera copies a Lantern trigger (as the rules say).
- Found, not fixed (pool AI): the AI's Food and Blood activations don't apply Forensic Gadgeteer's discount (its Clue does).
