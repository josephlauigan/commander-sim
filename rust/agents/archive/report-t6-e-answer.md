# Report t6-e-answer (plan rows 6 and 7)

Mechanism
- Answering abilities: `cardcode::answer_ability` runs `partials::answer_ability` (Guildmage, Tidebinder), then
  `t6_legends2::ertai_answer`, then new `t6_counters::louisoix_answer` while the item is still uncountered.
  `stack::ABILITY_ANSWERS_HAND` (Ertai, Louisoix) makes `abilities_answered` open a priority round when an opponent
  holds one. Louisoix: {U} + {2}, or sacrifices the least valuable legendary creature when it cannot pay {2} and that is
  worth less than the ability. `stack::counter_cost` adds the {2} for Louisoix against spells (it was {U} only).
- Importance: Godo enter 6, Dack enter 6 (already); Helm of the Host combat trigger 5 -> 6 (t3.rs); Thassa's Oracle's
  win now goes through a `trigger_window` (importance 9, combos.rs `oracle_etb`) so it can be countered.
- Copying: `Game.trig_copies`; `stack::copy_trigger` (copy item on the stack above the original, own priority round,
  recorded when it resolves), `take_copies` / `run_copy` (the hook runs again with `trig_current = 0`, before the
  original, in `trigger_window` and the known-trigger path of `resolve_batch`); `ai_respond` calls
  `cardcode::copy_ability` -> `t6_copies::camera_copy` for the AI's own trigger of importance 6+ ({2}, tap, -1 film
  counter, the Camera ability itself goes through `ability_window`). `camera_ready` also opens windows for the Camera's
  controller (`abilities_answered`, `resolve_batch`).
- Tests: rust/crates/sim-core/tests/engine_t6_answer.rs (12): Ertai/Louisoix vs Godo through ai_respond, no-mana cases,
  Louisoix sacrifice, Camera line (copy fetches Hammer, original Helm, Hammer attaches Helm, Camera tapped, 2 counters),
  Ertai countering the copy, Helm trigger, Oracle trigger.

Cards: Ertai Resurrected -> Full; Louisoix's Sacrifice -> Full (new override; spells: AI always pays {2}, the
sacrifice option is only used for abilities); Peter Parker's Camera stays Partial (activated abilities and new targets
not copied; only hook-run triggers have a copyable item).
Other decks: behaviour changes only when an opponent holds Ertai/Louisoix or a Camera; Louisoix as a spell counter now
costs {2}{U}; Helm trigger importance 6 means Tidebinder/Guildmage now answer it; Thassa's Oracle wins can be countered
(decks: Oracle in hand). Combo shortcuts (row 18) are unchanged (Oracle `combo_spell` path has no window).
Needs found: copying activated abilities (ability_window effects are inline, no re-runnable hook); Louisoix sacrifice
option for the spell mode.
