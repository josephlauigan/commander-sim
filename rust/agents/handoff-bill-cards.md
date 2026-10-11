# Handoff: bill-cards

## Findings
- "Full" in the audit = an entry in data/card_overrides.toml with status = "Full" + note (checked first by
  card_status). status+note alone keeps the card's data; `tags`/`dsl`/`cost` rebuild it. After editing run
  `cardgen export`.
- Counters are `Perm::plus`. `cardcode::add_counters` is the central place; `cardcode::scales(g,p)` is Hardened Scales.
- Bill's landfall is the compiled DSL; `dsl::choose_target` calls `bill::counter_target` for key == "bill".
- A `resolve` slot replaces the DSL spell. Morph machinery is `galadriel::enter_face_down/turn_face_up`.
- `g.last_cast_etb` is true inside an Etb hook when the permanent was cast.
- Do not use `perl s|...|` with `|` inside the pattern (it corrupted files twice); use Edit.

## Done (committed)
All 17 cards in the brief are Full in the audit with tests in tests/cards_bill.rs (36 tests): Hardened Scales, Bill's
doubling (options: EOT, or before combat when lethal), Boughside, Cactarantula, Canopy, Freestrider + Tumbleweed (plot),
Ghalta, Greenbelt Radical (disguise), Lifestream (foretell), Sample Collector (collect evidence), Silhana, Spider-Man
(web-slinging), Studious First-Year (prepared), Venomous Brutalizer, Wildgrowth Archaic, Royal Treatment (real Role).

## Status
DONE; see report-bill-cards.md. (Old list below.)

## Next (in order)
1. Check Retreat to Kazandu, Druid Class, Evolution Sage, Harrow, fetch lands for this deck (brief).
2. AI plan leftovers: hold ramp/Harrow/fetch lands until Bill is out (fetch hold needs a land_options crack),
   Harrow at instant speed, bite with biggest creature, finish with Overwhelming Stampede / End-Raze when lethal.
3. Full checks: cargo test, build warnings, all_decks adaptive 3 and lookahead 1, smoke run --deck bill.
4. Report in rust/agents/report-bill-cards.md.
