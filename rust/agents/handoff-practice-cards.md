# Handoff: practice-cards

Branch base: rust-port + practice-core milestone A (3563cb4) + practice-core 885cf05 (mana pool, legal.rs; merged because
`pool.add` / `legal::*` were needed; conflict in human/cards.rs resolved: core's two stubs `needs`, `granted_flashback` kept
at the end of cards.rs for me to replace).

## Done (all committed, `cargo build --release -p sim-core` clean)
- ALL `HUMAN(phase 9)` sites in `impls/*.rs` filled (none left: `grep -rn 'HUMAN(phase 9)' rust/crates/sim-core/src/impls`).
  Pattern: `if human_of(g, p).is_some() { ... choices::xxx(..)? }` before the AI code, same order as Python.
- `human/cards.rs` so far: helpers `pick_perm`, `pick_player`, `pick_creature`, `creatures`, `blink`, `ephemerate_rebound`,
  `pick_exile`, `pick_blue`, `expressive_iteration`, `look_and_take`, `damage`, `prismari_charm`.
- Signature changes inside impls (private/pub): `galadriel::choose_type -> Res<Sym>`, `yshtola::steal_pick(.., why) -> Res<CardId>`,
  `t4::bond_target -> Res<PlayerId>`.

## Left on purpose (not mine / for the report)
- `PORT(phase 9): log_secret` at impls/yshtola.rs (2), engine/tutors.rs, dsl/mod.rs: needs a secret-log facility (session task's
  EventLog); Game.log is a plain Vec<String>.
- `cardcode.rs` `tithe_unpaid` / `rhystic_unpaid` still have the HUMAN marker (core's file): they must call
  `choices::pay_tax(g,p,2,"Smothering Tithe (or its owner gets a Treasure)")` / `(g,p,1,"Rhystic Study (or its owner draws a card)")`
  and return `!paid`.
- engine sites that call cards.py functions (core's): `pick_blue` (stack.rs 644), `flashback_target`, `crackle`, `jeskas_will`,
  `dredge`, `mastery_target` (see Python engine.py grep 'play.cards').

## Next (in order)
1. Rest of `play/cards.py` into `human/cards.rs` (functions the engine/legal call): needs/NEEDS, granted_flashback+flashback_target,
   mastery_target, crackle, jeskas_will, dredge, copy_in_response, convoke_pay, cast_from_hand, the per-card spell casts
   (_twinflame, _ephemerate, _insight, _concealment, _rootborn, _triplicate, _morph_cast ...).
2. Abilities: cards.py `abilities(g,p,m)` / `land_abilities` registries and `play/abilities.py` into `human/abilities.rs`.
   Representation chosen: `Ability { label: String, act: Box<dyn Fn(&mut Game, PlayerId, PermId) -> Res<Option<String>>> }`
   (Option<String> = why not). human.rs (core) must call `abilities::permanent_abilities` / `land_abilities`.
3. Tests `sim-core/tests/play_{cards,abilities,marchesa,sauron,seph,veyran}.rs` (79 Python tests).
4. Checks (RULES.md), difftest, report-practice-cards.md.

## Gotchas
- Python `hc.search` default allow_none=True; `choices::search(.., true)`.
- The shell tool refuses compound commands with some heredocs: write patch scripts to the scratchpad with Write and run them.
- Python's ctx['targets'] (Eerie Interlude) is never set by anything: the human path just asks.
