# Handoff: practice-dsl-may
Findings: cardgen `dsl_parse.rs::split_clauses` strips a leading "you may " from a clause (flag lost); `parse_effects`
runs rules per clause. `Effect.optional` (dsl/model.rs) is set nowhere today (0 in data/cards.json); dsl/mod.rs `draw`
reads it: person -> yes/no, AI -> skip when library <= 12 (dead today, must stay dead: gate AI branch off).
Plan: split_clauses keeps may flag (persists over ", then", resets at ". "/"; "), parse_effects sets optional on those
effects; dsl/mod.rs `run` asks person yes/no for any optional effect; AI ignores flag; regen cards.json; tests/play_dsl_may.rs.
Status: baseline sims started (/tmp/dslmay-*-before.txt).
Status: DONE; committed. See report.
