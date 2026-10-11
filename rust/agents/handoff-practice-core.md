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

## Next (in order)
1. `human/human.rs` (port human.py: human_main with `Request{kind:"priority"}` + Answer::Action(json), apply, respond, stack_priority,
   step_priority, can_act/can_respond, cast/_cast/cast_rean/cast_counterspell, use/use_land, autopass via `Human::autopass()`; the
   request `data.view` is added by the Human impl, not here). `stack_view` too.
2. `human/combat.rs` (combat.py: can_attack, attack_candidates, human_attack, human_blocks).
3. Fill `HUMAN(phase 9)`/`PORT(phase 9)` sites in engine/*, ai/*, cardcode.rs, dsl/mod.rs (`grep -rn 'HUMAN(phase 9)' ... | grep -v impls`);
   delete markers. Also the human mulligan: `engine::turn::mulligan` returns `()` and must become `Res` + call `choices::mulligan` first
   (see Python `engine.mulligan`). `ais.py:2727` empty_pools between steps = `engine/turn.rs:749` PORT marker.
4. practice/src/text.rs + `--text` flag in main.rs; tests play_{hand,mana,combat,choices,respond,turn}.rs (port tests/test_play_*.py).
5. Checks in RULES.md + difftest 0 differences; report-practice-core.md.

## Gotchas
- Python mixed Perm/Player lists -> `Target`. Python `describe_target` for tokens uses `view.guess_token` (images): hook `legal::set_token_guesser`.
- `human_of` returns a cloned `Arc<dyn Human>`; use `?` on all helpers (Res).
- `gen` is a reserved word in Rust 2024: use `gn`.
