# Handoff: garland

## Done (committed)
- Deck registered: sim-cli decks.rs (FILES/MINE 13), args.rs, my_decks.toml, fixture, game.rs 78, cardgen export; validate OK.
- `impls/garland.rs`: recrown_value, attack_targets (brain.rs choose_defender), garland_prio (ai/decks.rs), unsummon_options,
  gl_* stats; `impls/garland_cards.rs`: Coerced to Kill (zur.rs lock kind "coerce" + zones.rs LOCK_KINDS + common.rs aura_fall),
  Skycoach Conductor, Specimen Freighter, Tragic Lesson, Amphin Pathmage, Lazav (Approximate), Xande, Potioner's Trove.
- Shared changes: legends.rs (garland_pick pub, ETB skips current monarch, entered stat), cardcode.rs become_monarch -> crown_moved,
  blue_cards.rs etb_score/best_own_etb allow Garland, t2.rs + common.rs blink_value Garland, cloud.rs cmd_stays_in_hand (garland too),
  bulk_blue.rs copy_value/token_copy (legend rule for Garland copy), stack.rs Dismiss draws, overrides (Shadewriter per_opponent bug).
- tests/cards_garland.rs: 17 tests pass.
- Baseline same-seed outputs (before any engine change): /tmp/claude-1000/base/before.{sauron,cloud,kraum,thrasios}, binary base/commander-sim, script base/seed.sh
  (usage: seed.sh <binary> <outprefix>; compare ignoring timing lines "(Ns total)").

## Next
1. report.rs "Garland" section was edited (sim-cli/src/report.rs, UNCOMPILED: check it builds); then full cargo test (first run killed unfinished), warnings, all_decks adaptive 3 / lookahead 1.
2. same-seed check vs baseline; 200-game run `--deck garland --pool t3 --analyze --seed 1013`; no "never cast" cards.
3. Tuning if steals last <1 round; Desertion/Coerced priorities; report-garland.md.
