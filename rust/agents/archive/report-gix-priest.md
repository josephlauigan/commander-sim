# Report gix-priest
- Priest of Gix: override (add_mana 3 on ETB, status Full), name added to `extra` in cardgen export.rs so it exports without a deck, cards.json + scryfall cache updated.
- mine.rs: new Loop "Nim Deathmantle + Ashnod's Altar + Priest of Gix" (LOOPS now 5); run_loop floats 20 B and no Zombies without a payoff; kills with Blood Artist/Zulaport/Altar of Dementia like the Titan loop. The human shortcut (loop_abilities), tutors (loop_need), reanimation, cast priority (loop_prio) all read LOOPS, so no change needed there.
- loop_need now ignores pieces the deck does not play (when deck_names is non-empty), so Seph without Priest is unchanged.
- etb_value: Priest of Gix >= 3.5, so flicker_worth/blink_worth >= 3 (flicker package targets it).
- Tests: 6 new in tests/cards_seph.rs. Workspace tests pass, release build warning-free.
- Before/after (cloud t1 200 games; seph t3 200 games, no swap): outputs identical (both have Win rate lines: 20.0% / 44.0%).
- Swap check: Viscera Seer=>Priest of Gix, 50 games: runs; Priest drawn 32%, cast 81%.
