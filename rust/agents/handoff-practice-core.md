# Handoff: practice-core

**MILESTONE A COMMITTED.** (practice-cards and practice-session may start.)

## API landed (all in `sim-core`)
- `flow::Stop::Cancelled` (engine `run_rounds` treats it as `g.stopped = true` and returns; playouts panic on it).
- `control.rs` (additive): `Answer::Decline` (the page's 'cancel' answer, distinct from `Cancel` = session closed);
  `Human::live() -> bool` (default false; true = a person at browser/terminal: choices then carry `data.refs`, and the
  Human itself must add `data["view"]` when it gets priority/choice requests, since sim-core cannot build views);
  `Human::autopass() -> Option<&'static str>` ('respond'|'stack'|'all').
- `human/mod.rs`: `human_of(g,p) -> Option<Arc<dyn Human>>` (cloned out, so `g` stays usable), `is_human`, `humans(g)`,
  `enum Target { Perm(PermId), Player(PlayerId) }`.
- `human/script.rs`: `ScriptHuman::{new(Vec<Answer>), from_funcs, from_fn(closure)}`, `.asked()`, `.asked_kind(k)`,
  `.told()`, `.told_kind(k)`, `.set_autopass(..)`; empty list / over 20000 requests -> `Answer::Cancel`.
- `human/legal.rs`: `describe_target(g,p,Target)`, `player_targetable(g,q)`, `token_name(g,&Perm)` (+ `set_token_guesser(fn)` for
  the session/images task to register Python's `view.guess_token`), `title()`. `spell_targets` is a STUB (unimplemented!).
- `human/choices.rs`: all helpers with final signatures and real bodies: `choose`, `choose_one`, `pick_cards`, `mulligan`, `discard`,
  `discard_to_hand_size`, `sacrifice_creature`, `search(pred: &dyn Fn(&Game,CardId)->bool)`, `pick_basic`, `scry(.., to_gy: bool)`,
  `pick_modes` (returns mode indexes), `yes_no`, `damage_targets`, `deal_damage`, `ring_bearer`, `etb_removal`, `pay_tax` (-> mana.rs),
  `choose_rean`, `rean_candidates`, `fill`, `sac_artifact_or_creature`, `target_opponent`, `alania`, `copy_targets`, `atraxa_pick`,
  `card_label`, `table_refs`, `shuffle_library`. All return `Res<..>`: use `?`.
- Stubs still `unimplemented!`: `legal::spell_targets`, `mana::pay_tax`. Empty placeholder files: human.rs, mana.rs, combat.rs
  (core), cards.rs, abilities.rs (practice-cards).

## Done since milestone A (all committed, `cargo build --release -p sim-core` clean)
- `human/mana.rs` complete: `ManaPool` (stored as `Player.pool`, new additive field in state.rs), `sources`, `tap`, `cost_problem`
  (read-only: floating mana is merged virtually), `pay_from_pool`, `pool_can_pay`, `empty_pools`, `pay_tax`, `Spending` (set
  `g.player_mut(p).pool.spending = Some(..)` around a cast, like Python's `mana.SPENDING`). `engine/mana.rs` `can_pay`/`pay` use it for a human seat.
- `human/legal.rs` complete: land/cast/equip/counter/gy checks (`Option<String>` = reason), `spell_targets`, `gy_mode`, `equip_cost`,
  `target_extra_cost`, `phyrexian_ways`, `base_cost`. `check_cast` takes `&mut Game` (additional_cost dry run). New field `Game.responding: u32`.
- `human/cards.rs` has two STUBS practice-cards must replace (merge note): `needs(g,p,c) -> Option<String>`, `granted_flashback`.
- Python quirks kept: `FLOATS` omits the floating black unit (it shows up as a tappable source named "FB"); floating white mana is never
  gathered into the pool (no `floatW` in Python).

## Done since (agent 2; all committed; baseline checks re-run clean: tests, all_decks, difftest 0 AI / 0 build diffs)
- `human/human.rs` complete (human_main, stack_view, autopass, stack_priority, step_priority, copy_window, can_act/can_respond,
  respond, cast_counterspell, apply, abilities_of/use, cast/cast_inner/cast_rean, order_triggers). The `data.view` is the Human's job.
- `human/combat.rs` complete (can_attack, attack_candidates, human_attack, human_blocks).
- ALL non-impls `HUMAN(phase 9)`/`PORT(phase 9)` markers filled and deleted (engine/*, ai/*, cardcode.rs, dsl/mod.rs). Extras:
  `turn::setup_game_for(.., human:&[&str]) -> (Game, Vec<(PlayerId, Rng)>)` (humans are shuffled, not mulliganed; session calls
  `choices::mulligan(g,p,&mut rng)` with the returned rng), `Game.trig_orders`, `Game.secret_lines` + `engine::log_secret(g,p,public,msg)`
  (MERGE NOTE: the session task added `Game::log_secret`; replace my engine::log_secret calls (tutors.rs x3, dsl/mod.rs) with it and drop mine),
  `choices::wipe_modes`, `legal::what`. dsl `choose_target`/`select` cannot return Err: a cancelled pick yields None (next tick stops the game).
- Stand-ins in `human/cards.rs` / `human/abilities.rs` for practice-cards (each marked PORT(practice-cards), returning "not handled" so the AI
  path runs): abilities, copy_card, copy_in_response, hand_override/cast_from_hand, gy_override/cast_from_gy, cast_target, mastery_target,
  needs, granted_flashback, pick_blue (real), dredge, jeskas_will, flashback_target, crackle; abilities::{permanent_abilities, land_abilities}.
  The cards agent's real versions replace these (same names/signatures; `cards::abilities` returns Option<Vec<(String, PermFn)>>).
- tithe_unpaid / rhystic_unpaid use choices::pay_tax for a human.

## NOT done (in order)
1. practice/src/text.rs + `--text` flag in main.rs (port play/text.py; `--deck sauron --tier t1 --ai adaptive`, scripted stdin to game end,
   no panic vs three AI seats). Needs a Human impl (live() true, adds data.view? text client can skip views).
2. Tests: port tests/test_play_{hand,mana,combat,choices,respond,turn}.py (71 tests) into sim-core/tests/play_*.rs with ScriptHuman
   (use `setup_game_for` + `choices::mulligan` for human seats; see testkit.rs).
3. Not verified at all yet: nothing exercises human.rs/combat.rs at run time (no human seat in any check). Expect bugs; the tests find them.
4. report-practice-core.md (full report: engine changes list above, Python quirks, stubs), final RULES checks, commit message
   "Rust port: the human seat's rules (priority, mana, combat, choices) and the text client".
5. Not wired: `legal::set_token_guesser` (session), impls/* HUMAN sites (cards agent), Teferi's/Najeela-like Python quirks as ported.

## Gotchas
- Python mixed Perm/Player lists -> `Target`. Python `describe_target` for tokens uses `view.guess_token` (images): hook `legal::set_token_guesser`.
- `human_of` returns a cloned `Arc<dyn Human>`; use `?` on all helpers (Res).
- `gen` is a reserved word in Rust 2024: use `gn`.
