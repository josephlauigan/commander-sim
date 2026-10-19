# Handoff practice-oppchoice
- before sims: /tmp/oppchoice-before-{sheoldred,cloud,kefka}.txt (script /tmp/oppchoice-sims.sh <tag>; builds sim-cli first). Caveat: build ran while first edits landed; edits are human-only.
- DONE (code, compiles): Marauder/Executioner-style each_player_sacrifice (t3), Plaguecrafter, Obliterator, Invoke Despair, Kefka {8}, End of the Hunt, Intellect Devourer, Esper Sentinel, Norn's Annex, Sauron ward, Eldest Reborn I, Torment, Braids III, Fact or Fiction (rules.rs both ways). New helper choices::sacrifice_permanent.
- TODO: tests/play_opponent_choices.rs; after sims `/tmp/oppchoice-sims.sh after` + diff; full cargo test; report; final commit message per brief.
