# Report: practice-cards

Done: human/abilities.rs (abilities.py), human/cards.rs (cards.py: ability registry, hand casts, copies, flashback, dredge,
Jeska's Will, Crackle, convoke, Galadriel/Zur/Y'shtola abilities), all HUMAN(phase 9) sites in impls/*.rs, yshtola log_secret sites.
Tests: sim-core/tests/play_{cards 16, abilities 12, marchesa 6, sauron 17, seph 17, veyran 15} = 83 (Python 79 + split), play_hand's
8 ignored tests un-ignored and passing. cargo test --release all green; no warnings; all_decks adaptive 3 / lookahead 1 fine;
difftest --gen 1000: 0 AI, 0 build, 0 phase 6 differences (card code 36 positions, random 46: not examined for being mine).

Shared edits: human.rs (ability struct call sites; mastery_target returns Option<CardId>; use_land pub), state.rs (Ctx.mastery_pick,
Player.fb_grant, Player.necro_wait), engine/cast.rs (Mastery uses mastery_pick), impls/mine.rs (cast_copy_with; pw_once, pw_use,
loop_card, insight_cost, kefka_wheel pub), impls/rules.rs (remora_age pub), galadriel.rs (rebel_cards, put_rebel, shapeshifter pub).

Notes: Necropotence's end-step delivery (cardcode::necro_deliver, PORT(M5) placeholder, not mine) must move Player.necro_wait to hand.
Prepared-face copies under Thousand-Year Storm copy only Seething Song / tutor / self-targeted Ancestral. Python bugs: none found.
legal::set_token_guesser is not wired (not mine).
