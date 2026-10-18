# Report: audit-cv (cloud + veyran), commit 4a9c7c6

Cloud: 1 fixed (F&F first only at 7+ cards, `CLOGGED_HAND`); 2 fixed (Open the Armory `resolve` hook: Aura/Equipment
only, Cloud's own pick order); 3 fixed (cloud-gated Austere modes via `ai/mod.rs::wipe_modes` -> `cloud::wipe_modes`:
own enchantments weigh 6 each); 4 cause was problem 1 (pick order already prefers pieces castable+equippable within a
turn); 5 no evidence of a defect, left. Cloud t1 adaptive 300: F&F cast 87% -> 73%, Fire and Ice 74% -> 85%; win 23.0 -> 21.0 (noise, n=300).

Veyran: 1 Triple Triad prio 58 with engine + 6 mana (else 44); 2 Recruiter wish list by board (`recruit_wish`);
3 Firesinger 74 -> 48 (56 with payoff + 2 spells); 4 Greaves 75 while Veyran waits/out, Fabricate fetches Greaves first;
5 Mystical/Solve wish list by state (`is_wish`), Solve prio 0 unless a real target (wipe/Crackle); 6 Docent, Alania,
Prodigy, Niv, Eris, Flux Channeler engine prios (46-66, scaled by spells in hand + payoff); 7 dead Kitten combo code
removed (combo option, veyran_try_combo, Inventors' Fair option/veyran_fair, Remora/Chromox/Moxd/Narset/Breach arms,
Kitten/Firesinger wish arms, threat/combo_near/neutered bits). 8, 9 no defect, left.
Veyran t2 adaptive 1500: 28.5/27.5 -> 30.1/28.9 (cons/loose). (300-game run read 33.3 -> 30.0: noise.)

Kept: Displacer Kitten card, Tag::Vkitten/Vfire and their pval/cast.rs valuations (Seph/Atraxa plays Kitten).
Shared code touched (gated on key): ai/mod.rs wipe_modes (key cloud), ai/mod.rs combo_interrupted (dead veyran branch),
ai/brain.rs combo_near (veyran Kitten clause), engine/values.rs threat (veyran Kitten clause).
Tests: new in cards_veyran.rs and cards_cloud_aifix3.rs; two old tests updated/removed (Kitten combo; 4-card "clogged").
Pool check `--calibrate within --pool t3 --games 40`: identical to ai-audit binary.
