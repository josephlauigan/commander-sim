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

## Next (in order)
1. legal.rs full port; mana.rs (pool); human.rs (human_main, apply, respond, stack/step priority); combat.rs.
2. Fill `HUMAN(phase 9)` sites in engine/*, ai/*, cardcode.rs, dsl/mod.rs (also hook the human mulligan: `engine::turn::mulligan` returns `()`, must become `Res`).
3. text client (practice/src/text.rs), tests play_{hand,mana,combat,choices,respond,turn}.rs, checks.

## Gotchas
- Python `Perm`/`Player` mixed lists -> `Target`.
