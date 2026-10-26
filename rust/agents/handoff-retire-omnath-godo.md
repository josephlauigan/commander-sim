# Handoff: retire-omnath-godo

## Done
- Brief committed. Deck-key lists 14 -> 12: sim-cli decks.rs (FILES, MINE), args.rs, poolmode test, report.rs (omnath
  plan lines), practice catalog.rs, session/setup.rs; practice advisor/mulligan.rs Godo branch removed (dead: practice
  can't seat godo); practice test hint_mulligan_godo.rs dropped.
- testkit retired_decks: `omnath`, `godo` (commander + basics) so the seat tests keep working; lists restored to
  decklists/pool/retired/.
- Kept (retired seats still run them): sim-core gates ai/mod.rs MAIN_STYLES + jd_deck, settings cthresh, cast.rs
  commander value, rules.rs CTHRESH, t6_plans/t6stats T6_KEYS, brain/decks/titania omnath AI, all card code.
- Count tests -> 12 (app_entry, http_catalog, http_server, server_fake, game.rs 84).

## Next
Nothing: done (see report-retire-omnath-godo.md).

## Was next
1. Build + full test run; fix seat tests that relied on the real libraries.
2. Docs in rust/ and documents/how-it-works.md.
3. CLI checks, cardgen export, report.

## Gotchas
- PR #142 touches ai/decks.rs, engine/cast.rs, cardcode.rs, impls/{combos,common,fixes}.rs, sim-cli/tests/realism.rs:
  only omnath/godo key lines there, no reformatting.
