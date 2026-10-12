# Handoff: sheoldred-upgrade2
Done: 7 new cards (data + overrides + export, code in impls/sheoldred.rs "second upgrade package", tests at the end of tests/cards_sheoldred.rs), Boots equip, Exsanguinate gate (yshtola x_option allows key sheoldred), Howling Mine hook in engine/turn.rs draw step.
Next: full cargo test, no-warning build, audit, all_decks adaptive 3 / lookahead 1, 48-game smoke, swap smoke (20 swaps, two Swamp swaps), report-sheoldred-upgrade2.md.
Gotchas: seph seat dredges instead of drawing (use veyran as the opponent in draw-step tests); a dead player's alive flag updates at check_state (test life <= 0).
