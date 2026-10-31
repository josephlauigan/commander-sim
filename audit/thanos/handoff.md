# Handoff: Thanos replaces Scion; Ur-Dragon joins (brief: rust/agents/thanos-urdragon.md)

## Done
- Rename: deck key `thanos` (my_decks.toml, sim-cli decks/args/poolmode/report/goldfish tests, practice catalog/setup
  and tests, T6_KEYS in t6_plans.rs and t6stats.rs). Plan file data/plans/thanos.toml (Scion's, with its style moved
  from ai::T6_JD_STYLES into `[style]`). data/spellbook/thanos.json refetched (scripts/spellbook-fetch.sh).
- Scion retired: list in decklists/pool/retired/, testkit seats `scion` (its card tests: cards_scion.rs, goldfish
  card code), "scion" stays in T6_KEYS and T6_JD_STYLES as a test seat (like godo).
- Ur-Dragon: key `urdragon` (not `ur-dragon`: cardgen `--deck` matches by prefix, and the retired pool deck is
  `ur-dragon-five-color-dragons`), tiers t5, plan file data/plans/urdragon.toml. Not in ai::MAIN_STYLES/jd_deck:
  it plays through the generic policy + plan file, and `combo_options` offers the catalogue Godo/Celebrant/Dualcaster
  lines.
- Cards: Thanos power-up (impls/t6_thanos.rs; {C} pip support in mana::plan_pay), Kindred Dominance (sweepers.rs
  kind "dominance"), Kaldra Compleat (living weapon in karn_engines.rs + compiled +5/+5 and keywords).
- Tests: tests/cards_thanos.rs, tests/cards_urdragon_jd.rs.

## Next
1. Done on the rebased binary (rc=0 each, seeds from 500000, --jobs 12): thanos t6 29.6% (CI 27.6-31.6; Scion
   30.1%), urdragon t5 19.1% (CI 17.4-20.9), goldfish thanos median kill 11 (Oracle 76%), urdragon median 9
   (damage 62%, combo 31%). FINDINGS.md files: the harness refuses report files from subagents; their text is in
   the final report for the caller to commit.
2. Open: urdragon mulligan rule (pieces = 1 sends 16.6% of goldfish hands to 5 or fewer cards).

## Gotchas
- Another agent runs sims: --jobs 12 / -j 12, one sim at a time.
- Phase 4 files untouched (ai/search.rs, learned.rs, values.rs, data/eval, audit/phase4, values_overrides.toml).

## Rebase (2026-10-30, coordinator's request)
- Rebased onto origin/main after PR #176 (JD's swaps for Sauron, Veyran, Tymna/Thrasios, Kefka). Conflict only in
  data/scryfall_cache.json: took main's data/scryfall_cache.json, cards.json, decks.json and the fixture, then reran
  `cardgen export` (refetched Thanos and Kindred Dominance) and rebuilt the fixture's thanos/urdragon entries (checked
  equal to decks.json for every key; the cardgen equivalence test checks it too). #176's three test edits kept as is.
- Pre-rebase sims (thanos t6 29.6%, urdragon t5 20.1% with 3 panics) are superseded: rerun on the rebased binary.
- Overflow fix: Thrakkus the Butcher doubling each Dragon per Thrakkus copy (Miirym's tokens) passed i32 in three
  urdragon t5 games (seeds 501632, 501721, 501904): saturating adds in common::eot, chars.rs power sum, life.rs.
