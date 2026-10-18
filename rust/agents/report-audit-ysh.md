# Report audit-ysh (Y'shtola, Tymna/Kraum)
See audit/yshtola/FINDINGS.md and audit/kraum/FINDINGS.md (2026-10-18 sections) for the numbers and reasons.

Commits: kraum doc tier text; Kraum fixes; Y'shtola fixes; tests/handoff/report.
Shared code touched (all gated on the key): ai/brain.rs (removal_options, attack_response, cast_removal sample filter,
wipe_options, cast_wipe: yshtola only), ai/decks.rs (breach_candidates: kraum wipes; new behind_on_board),
impls/tymna_partners.rs (partner_prio: kraum branch), impls/tymna_cards.rs (protect: Dark Endurance for kraum;
Firebird and Lightning card code; Diabolic Tutor pick), impls/yshtola.rs.
Test change: cards_tymna.rs the_kraum_seat_tutors_for_... updated to the new finisher rule.
Pool check: --calibrate within --pool t3 --games 40 --ai adaptive --jobs 2 identical to ai-audit's binary.
