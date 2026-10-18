# Handoff: audit-grixis (Sauron + Kefka)

Done (all committed, tests in rust/crates/sim-core/tests/cards_audit_grixis.rs):
- Sauron 1 tutors by board state (decks.rs `sauron_tutor`), Assault prio 58 with Cloak out.
- Sauron 2 traced: Army deaths are opponent wipes/spot removal (~30 in 37 traced games) plus own Nibelheim Aflame;
  fixed own-wipe (wipe_eval x3 Army loss unless Ozolith); Ozolith is now tutored.
- Sauron 3 FoW pitch keeps engines (stack.rs pay_counter -> decks::sauron_pitch_keep). 4 counters: hold_value applies
  to sauron like outside decks. 5 Cost of Brilliance feeds Bowmasters (sheoldred.rs). 6 archivist_worth (Kaervek/Sauron).
- Kefka 1 tutors (kefka::tutor_win_piece, decks::kefka_wish, Grim Servant via ai/mod.rs dsl_search_pick). 2+3 discard keep
  values (kefka.rs). 4 dry run: diagnosed with temporary debug print, failures are real (storm < opponents' life), no
  code change. 5 Mana Leak (stack.rs soft_counter_factor). 6 Profane Tutor suspend 4.0 on empty turn (legends.rs).
  7 Rhystic: no misplay evidence, unchanged. 8 doc sentence fixed.
Gotchas: never run `cargo fmt` (reformats 59 files); perl s||| with `||` in the pattern breaks files; use Edit.
