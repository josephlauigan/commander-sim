# Report: zada-core

Branch models-zada-core. Every card in the brief is Full in `cardgen audit --mine` (the 28 still Partial/Unmodeled in
the zada deck are zada-cards' own).

## Modeled (impls/zada.rs, tests/cards_zada_core.rs, 25 tests)
- Zada: `cast` hook reads the spell's target from `g.cur_cast`; if it targets only Zada, a trigger window, then one
  `resolve(.., "copy")` per other creature, each with its own `ctx.target`. Copies are not cast (no storm count, no
  cast triggers; `magecraft(copy)` only). Return the Favor (held, {1}{R}{R}) copies the trigger: a second round.
- Spells with `resolve` slots: Ancestral Anger, Monstrous Rage, Bulk Up, Act of Treason, Into the Fray, Blazing
  Crescendo (exile-and-play via `impulse_long`, the old compiled version drew), Twinflame (single target; Zada's copies
  make the rest; token creatures are copied by attributes; a legendary target gets no copy: legend rule).
  Sure Strike, Furious Bellow, Smaug's Fury, Crowd's Favor are compiled and take each copy's target (tested).
- Monster Role: `DataKey::Role` = +1/+1 (in `static_bonus`) and trample (in `dsl::has_kw`); a second Role changes nothing.
- Payoffs: Burning Prophet, Speed (pay {1}; eot keyword `haste_blockers`, checked in `can_block`), Expressive Firedancer
  (double strike at 5+ mana spent; the +1/+1 stays compiled), Dreadhorde Arcanist (free cast from the graveyard,
  exiled; `Ctx.paid_otherwise` marks a free cast), Firebird, Empty the Warrens (storm via `self_cast`).
- AI: `zada_prio`, `zada_options` (a Zada spell on Zada once he has 2 other creatures, creatures first; cantrips on the
  best creature when nothing else is castable), Zada's attack hook casts pump spells on him as attackers are declared
  (before blockers: the engine has no after-blockers window), Bulk Up flashback included.

## Shared-file changes (all small, additive)
ai/brain.rs (`do_cast_with`, zada in special_options and GENERIC_PLAYS), ai/mod.rs (MAIN_STYLES 9), ai/decks.rs
(deck_prio), ai/plans.rs (response_only), state.rs (DataKey::Role), engine/values.rs, dsl/mod.rs, engine/combat.rs
(can_block), engine/cast.rs (spell_imp), engine/zones.rs (token colour), settings.rs, impls/rules.rs (CTHRESH),
impls/mine.rs (3 fns made pub(crate)), impls/mod.rs, sim-cli (decks.rs, args.rs, report.rs: zada is a deck key; its
decks test counts 8). data: card_overrides.toml (17 new Full rows, RTF/Twinflame notes), cards.json re-exported.

## Not modeled / simplified (said in the notes)
Into the Fray's splice (no other Arcane spell in the deck). Pump spells go on before blockers. Twinflame's strive extra
targets are never chosen. Arcanist's AI picks only Zada spells and a few untargeted spells.
