# Report: Sephiroth and Veyran (phase 6)

Done: mine.rs (Sephiroth/Veyran/non-Sauron parts), ai/seph.rs, ai/veyran.rs (from the earlier WIP commit 080f43e), dispatch arms in ai/decks.rs and ai/brain.rs, `cardcode::seph_flicker_worth` -> `mine::flicker_worth`, marchesa.rs now uses `mine::ETB_VALUE`, Breach-line bug fix with test (`a_finished_breach_line_is_not_offered`).
Merge of rust-port: conflicts in state.rs, hooks.rs, cardcode.rs, ai/decks.rs, ai/brain.rs resolved keeping both sides (duplicate blink_depth/resto_target fields removed).
Tests: 110 Rust tests (cards_seph 76, cards_veyran 34) vs 53 in test_my_cards.py; test_play_seph/veyran.py are practice-mode, skipped.
Difftest (--gen 1000 --seed 7 --decks seph,veyran,t1,t3): AI 0, build 0, phase 6 0; card code 377 positions, all traced to unported cards (Atraxa Praetors' Voice -1.0 utility, Teferi Time Raveler, Propaganda, Korvold, Marwyn, Nissa, Natural Order, Awakening Zone), none to Seph/Veyran cards.
Shared-file changes: state.rs (loop_turn, te_used, clamp_t, clamp_n, muld_used), hooks.rs, engine/hooks.rs, cardcode.rs, cards.rs, ai/mod.rs, ai/plans.rs, tools/difftest.py (small).
