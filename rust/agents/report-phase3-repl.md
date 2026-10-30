# Report: Phase 3, PR 3: one replacement step (branch `phase3-repl`)

Brief: `rust/agents/phase3-repl.md`. Design: `audit/phase3/repl-design.md`. Handoff: `audit/phase3/repl-handoff.md`.
Code: `rust/crates/sim-core/src/engine/repl.rs`, `tests/repl.rs`, `examples/repl_cost.rs`. Commits: b612998c (design),
e50a8466 (Stage A), d15e2de9 (Stage B), 80b54be6 (Stage C), 523bdd5c (doc). No sims (JD's rule).

## Stage A: the step and the table (results identical)
Event kinds (`Ev`): Enter, Counters, Tokens, Damage, LifeGain, LifeLoss, Graveyard, Draw, Trigger. One table
(`NAMED` + `TAGGED`): Hardened Scales, Vorinclex, Doubling Season, Solemnity, Melira, Mauhúr, Cho-Manno, Tajic,
Academy Manufactor, Hushbringer, Laboratory Maniac / Jace, Spelunking; each card's entry looked up once at load into
`CardDef::repl`. The counters step takes a `Path` recording which effects each old site applied, so every site kept
its coverage. The old helpers became readers of the step. 9 tests compare the step with copies of the old helpers on:
Doubling Season + Hardened Scales + Vorinclex; Melira with infect and wither; Rest in Peace and a dying creature with
a dies trigger; Solemnity with an undying creature; Cho-Manno and Tajic; token doublers.

## Stage B: call sites (results identical)
Every site calls the step for its event kind. About 20 old helpers deleted (`cardcode::scales`, `vorinclex`,
`put_plus`, `add_counters`, `damage_prevented`, `values::melira`, `values::no_damage`, `heliod_zur::solemnity`,
`dsl::token_mult`, `counter_mult`, `lifegain_mod`, `zones::hushed`, `hooks::enters_tapped` and others).

## Stage C: behaviour fixes (each tested; 15 tests in `tests/repl.rs`)
1. One counters path: every placement by an effect goes through `repl::put_counters`. Before, the main
   `put_plus` / `add_counters` path (about 140 sites), enters-with-counters, undying and card code skipped Doubling
   Season, and `put_plus` skipped Hardened Scales and Mauhúr. Breena, Hydaelyn and Spark Double's direct writes go
   through the step too.
2. Doubling Season doubles by the permanent's controller, only through its compiled doubler; planeswalkers still
   enter with doubled loyalty exactly once.
3. Order (rule 616.1): the affected controller orders the effects (`repl::ordered`); the AI tries every order and
   keeps its best; a practice-mode player is asked (`replacement_order`) when orders give different results.
   Hardened Scales + Doubling Season on 1 counter gives 4.
4. A permanent that lost its abilities no longer applies Hardened Scales, Vorinclex, Melira, Mauhúr, Manufactor,
   Lab Man, Tajic, or the compiled counter and token doublers.
5. Poison: infect damage to players and Phyrexian Unlife go through the poison step (Melira stops them, Vorinclex
   applies); Melira stops -1/-1 counters from wither, infect and `put_plus`.
6. Solemnity also stops proliferate's kept counters and counters on players; Tajic respects "can't be prevented".

## Checks
- `cargo test --release --no-fail-fast`: 3655 passed, 0 failed, 3 ignored (rerun by the coordinator). Build 0
  warnings. Practice 104. Export / validate / audit clean (0 doubles, `paths_known` empty, no data change).
- Microbenchmarks (ns per call, empty board / with Doubling Season, Scales and Vorinclex): +1/+1 counters 328-334 /
  342-346 → 74-107 / 76-81; -1/-1 112-116 → 47-74 / 68-70; tokens 186-196 / 228-245 → 173-181 / 225-233; life gain
  unchanged; `clone_cost` 2.95-3.13 → 2.89-3.03 µs. Nothing slower.

## Decks touched (for the recalibration)
- atraxa-infect (t3): Doubling Season and Vorinclex. Reaches JD's decks that play t3: sheoldred, sauron, seph, kefka,
  cloud, veyran.
- zur (t5): Solemnity and Phyrexian Unlife. Reaches karn, thrasios, yshtola.
- **JD's sauron, directly:** Mauhúr adds a counter on every placement on Armies, Goblins and Orcs.
- Small: Spark Double (miirym t5), Breena (isshin t1); neutered doublers only with an "abilities lost" card out
  (edgar t4, miirym t5, korvold t3, prosper t4, chulane t4). No deck plays Hardened Scales, Melira or Topiary Lecturer.

## Not fitted yet
- No damage event: damage amounts (Fiery Emancipation, Gisela, Stagger) stay at their sites; Gisela's halving
  still comes after the doublers.
- Destroy replacements (regeneration, indestructible, Umbra armor) stay in `zones::die`.
- Panharmonicon / Teysa share the trigger step though they are not replacements (603.2d).
- Moved counters and Saga lore counters are not counters events. Draw replacements keep a fixed order.
- Erebos and Iroas read their own tag / card id. Two Academy Manufactors count once; Mimic's Army and Vraska's nine
  poison counters stay direct writes.
