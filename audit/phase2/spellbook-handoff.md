# Handoff: Phase 2 PR 4, combos from Commander Spellbook

Branch `phase2-spellbook` (from origin/main 2bdfead0). Status: done, not pushed.

- Data: `data/spellbook/<deck>.json`, 48 files, fetched 2026-10-29 with `scripts/spellbook-fetch.sh` (curl + jq,
  one POST per deck, skipped when the deck's request hash is unchanged; `--force` refetches). Credit: Commander
  Spellbook, https://commanderspellbook.com (in every file and the audit).
- Guard and report: `rust/crates/sim-core/tests/spellbook.rs`. `spellbook_files_match_the_decks` (names line up),
  `included_combos_of_catalogue_pieces_are_modelled` (fails on a new uncovered included combo whose cards are all
  line pieces; the 15 current ones are in `KNOWN_GAPS`), `spellbook_report` (ignored; `SPELLBOOK_REPORT=<file>`).
  `CARD_CODE` lists 8 combos the sim plays through card code (Kefka loops, Sheoldred + Peer, Revel in Riches).
- Audit: `audit/phase2/spellbook-2026-10-29.md` (narrative on top, generated per-deck tables below).
- After a deck change: rerun the script for that deck, regenerate the report, and update `KNOWN_GAPS` if the guard
  names a new pair. After modelling a gap: drop it from `KNOWN_GAPS`.
- Not done (by the brief): modelling any new combo. No engine change (`rust/crates/sim-core/src` untouched).
