# Report: audit-grixis (Sauron 6 problems, Kefka 8 problems)

Branch audit-grixis. Tests: rust/crates/sim-core/tests/cards_audit_grixis.rs (15 tests) plus tests/ai.rs updated.
All 12 problems that were bugs are fixed; Sauron 2 traced; Kefka 4 and 7 are not bugs (below). Details in audit/sauron/FINDINGS.md
and audit/kefka/FINDINGS.md (before/after numbers, adaptive vs t3, 600 games per profile).

Sauron: 1 tutors by board state (decks.rs sauron_tutor). 2 cause = removal/wipes plus own Nibelheim Aflame; self-sweep fixed
(wipe_eval Army x3 unless Ozolith), Ozolith tutored. 3 FoW pitch keeps engines (stack.rs pay_counter, decks::sauron_pitch_keep).
4 hold_value holds counter mana for sauron. 5 Cost of Brilliance feeds Bowmasters (sheoldred.rs). 6 archivist_worth (Kaervek/Sauron).
Kefka: 1 tutors (kefka::tutor_win_piece, decks::kefka_wish; Grim Servant via ai/mod.rs). 2+3 discard keep values (kefka.rs).
4 dry run diagnosed (temporary debug print, removed): failures are storm < table life, not a counting bug. 5 Mana Leak
(stack.rs soft_counter_factor). 6 Profane Tutor 4.0 (legends.rs). 7 Rhystic: no evidence of misplay, unchanged. 8 doc fixed.
Shared code touched (all gated on deck key): stack.rs (pay_counter, ai_respond, soft_counter_factor), brain.rs hold_value,
decks.rs (wipe_eval, archivist_worth, assault prio), ai/mod.rs dsl_search_pick, legends.rs profane, sheoldred.rs.
