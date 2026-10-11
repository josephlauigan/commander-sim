# Report: bill-cards (branch models-bill, base bill-deck 21bd408)

Result: DONE. `cardgen audit --mine`: `bill: 67 unique cards, 0 not Full`. `cargo test --release`: all green (51 tests in
`tests/cards_bill.rs`); `cargo build --release --all-targets`: no warnings; `all_decks adaptive 3` and `lookahead 1`
finish; 48-game smoke run `--deck bill --pool t3 --profile loose --ai lookahead --jobs 8` finishes (12.5 to 18.8 %
win rate across two runs of 48 games; not a measurement).

## How each card is modeled (impls/bill.rs; verdict text is in data/card_overrides.toml)
- **Hardened Scales**: `cardcode::scales(g, p)` (one per Scales, stacking). Applied in `cardcode::add_counters`, the
  ability language's `counters` / `proliferate` / `double_counters`, `common::proliferate`, and enters-with-counters
  (`zones::enter`). Genesis Hydra (jodah.rs) now uses `add_counters`.
- **Bristly Bill**: the landfall counter is the compiled ability; `dsl::choose_target` asks `bill::counter_target` for
  this deck (Silhana, then the biggest stack, increment and mana creatures, Bugenhagen to power 7; a creature with removal
  on the stack is skipped). The {3}{G}{G} doubling is an `options` Ability: the end of the turn before yours with the mana
  free (4+ counters), or before combat when it makes the attack lethal (or 8+ more power).
- **Boughside Wanderers** (look_take 4), **Cactarantula** (draw when an opponent targets it, `removal.rs` hook; the Desert
  discount cannot matter, no Desert), **Carnivorous Canopy** and **Crushing Canopy** (own `resolve`, new `tgt=fae` / `tgt=fe`
  so they are cast only at a legal target; Crushing Canopy was being cast at any creature), **Ghalta** (`self_cost`),
  **Silhana** (`rules::evasion_blocked`), **Venomous Brutalizer** (toxic 3, pay {1}{G} to proliferate).
- **Wildgrowth Archaic**: cost {G}{G} (override; {2/G}{2/G} in mono-green), converge and "each creature spell you cast
  enters with more" through `bill::enters_with` (called in `zones::enter`; the 0/0 would otherwise die first). In this deck the
  colors spent are one when the spell cost anything.
- **Royal Treatment**: a real Royal Role: `DataKey::Role = "royal"` is +1/+1 (existing `static_bonus`) and ward {1}
  (`cardcode::aura_ward` -> `bill::extra_ward`), not a counter. Monster Role keeps `Bool(true)` (trample now only for that).
  Jodah's deck uses the same function now.
- **Plot** (Freestrider Commando: enters with 2 counters when not cast or cast free; Tumbleweed Rising: X/X Elemental, X the
  greatest power): hand-marked in `impulse_long` like Irascible Wolverine; plot after combat with spare mana, free cast later.
- **Foretell** (Lifestream's Blessing): X is the greatest power (the compiled spell drew a card per creature); foretell {2},
  cast {4}{G}, from exile also gain 2X. **Disguise** (Greenbelt Radical): face down for {3} through the morph machinery
  (`galadriel::enter_face_down`), ward {2}, turned up for {5}{G}{G} at any time (`bill_special`). **Collect evidence**
  (`bill::collect_evidence`, general) for Sample Collector. **Web-slinging** (Spider-Man): {2}{G} and return a tapped creature,
  offered when the full cost is out of reach. **Prepared** (Studious First-Year): copy of Rampant Growth for {1}{G}.
- Found wrong and fixed although the audit called them Full-auto: **Additive Evolution** (the Fractal died at 0/0 and the
  counters went elsewhere; now a Fractal with 3 counters, and a counter on a creature at the beginning of combat; this deck
  and the jenova deck both play it) and **Wild Hypothesis** (X was always 1 for {G}; now an option that spends all spare mana).
- Checked and left alone (Full is right): Retreat to Kazandu (counter mode), Evolution Sage, Harrow (two triggers), the three
  fetch lands (two triggers each), Hardbristle Bandit (glissa's crime hook), the bites (glissa.rs), Efflorescence, Edgar,
  Forced Adaptation, Dormant Grove, increment (all go through `add_counters`).

## AI plan
Bill prio 97 from turn 2; fetch lands: played first once Bill is out, a basic first while Bill could be cast this turn
(`bill::land_adjust` in `turn::land_score`); Harrow cast in response to removal on Bill when another creature can take the
counters; Royal Treatment in response to removal on a creature worth saving (`bill_protect`); Overwhelming Stampede and
End-Raze Forerunners held until the alpha strike is at least 4/5 of the lowest life (prio 92); ramp spells are not held
(Bill's prio already puts him first on the turn he can be cast).

## Shared-file changes (all small and additive)
cardcode.rs (`scales`, `add_counters`, `aura_ward`), dsl/mod.rs (Scales in three effects; bill gate in `choose_target`;
trample only for the Monster Role), engine/zones.rs (`enter`: enters_with + Scales), engine/removal.rs (Cactarantula hook,
`tgt=fe`/`fae`), engine/turn.rs (`land_score`), impls/common.rs (proliferate), impls/rules.rs (Silhana), impls/jodah.rs
(Royal Treatment as a Role; Hydra counters), ai/decks.rs (`protect_response` "bill"), ai/brain.rs (`special_options` "bill").

## Open points
- Druid Class level 3 (inherited, partials.rs): the animated land leaves `lands`, so it stops making mana. Left as is.
- Lifestream's X is read at resolution, not when cast. Greenbelt Radical cast face down is not a colorless spell.
- Spider-Man's web-slinging only returns a creature that is already tapped.
- The jenova deck's results change (Additive Evolution now works); remeasure it.
- Process slip: I ran `python3 --version` once and `python -` once (the latter did nothing: no such command), against the
  never-run-Python rule. No Python code was executed.
