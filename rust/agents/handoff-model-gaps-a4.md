# Handoff: model-gaps-a4
Done (all committed, tests/model_gaps_a4.rs 17 pass, 36 related test files pass, `cargo build --release --all-targets` no
warnings, cardgen export + audit --mine: only Marvin and Hexing Squelcher below Full):
- Item 1: the five abilities (impls/model_gaps_a4.rs AI + effects, human/model_gaps_a4.rs person, Marvin borrows both).
- Item 3: StepKind::Equip (impls/combos.rs run_steps) for the two Godo Helm {5} equip steps.
- Item 2 part: person path for Marvin's borrowed Kiki-Jiki and Chatterfang (new person abilities, also for the creatures
  themselves), and the whole `human::model_gaps_a4::BORROWED` list (marvin_abilities); a person now chooses what a compiled
  sacrifice cost (dsl::pay_ability_cost) sacrifices.
LEFT (item 2 and so item 4, Marvin stays Approximate until done): the other hand-written (hook) abilities Marvin borrows still
give a person the AI's target. To finish: for each creature below add an entry in human/model_gaps_a4.rs `abilities()` (the
person's own ability for the creature, taking the source `m`: ask the target/choices with human::cards::pick_perm /
choices::choose, then call the AI hook's go function (make it pub(crate)) with the chosen arg, as `kiki`/`fang` do) and add its
card name to `BORROWED`; then set Marvin to Full in data/card_overrides.toml + data/audit.toml (card_notes) and re-run
`cardgen export --offline` / `audit --mine`.
Hook-only creatures with a choice and no person path (regenerate the list with a throwaway test: every creature card whose
registry entry has `options`, entered with testkit::perm, then check human::cards::abilities / dsl::human_abilities /
abilities::permanent_abilities / karn::perm_abilities / cloud::perm_abilities are all empty): Amphin Pathmage, Bloodline Keeper,
Cankerbloom, Captivating Vampire, Chittering Witch, Deathrite Shaman, Destiny Spinner, Drivnod, Eladamri, Eldrazi Displacer,
Elvish Reclaimer, Goblin Cratermaker, Goblin Engineer, Goblin Trashmaster, Goblin Welder, Grim Hireling, Hawkeye, High Perfect
Morcant, Hope of Ghirapur, Ioreth, Jagged-Scar Archers, Kinnan, Loot, Loran, Magda, Magus of the Candelabra, Mina and Denn, Mogg
Fanatic, Muldrotha, Nullmage Shepherd, Nyx Weaver, Oboro Breezecaller, Pashalik Mons, Patriot, Pia Nalaar, Preston, Priest of
Forgotten Gods, Razia, Recruitment Officer, Ruthless Technomancer, Sai, Savvy Hunter, Sisay, Skrelv, Tavern Scoundrel, Tekuthal,
Temur Sabertooth, Teysa, Thunderclap Drake, Timberwatch Elf, Walking Ballista, Vilis, Weathered Wayfarer, Formidable Speaker,
Deadeye Navigator, Abigale, Silas Renn, Conspicuous Snoop, Valgavoth. (No choice, fine as hooks: Krenko, Allosaurus Shepherd,
Battle Cry Goblin, Bristly Bill, Tolsimir, Speaker of the Heavens, Elvish Warmaster, Lathliss, Scourge of Valkas, Shalai, King
Darien, Incubation Druid, Pteramander, Twilight Drover, Urza Powerstone Prodigy, Deadpool, Joraga, Elvish Clancaller, Gilded
Goose, Weathered ... check each.)
Creatures with a person path of their own that are not in BORROWED yet (their closures need a check that they use `m` only as the
source): Balan, Blazing Firesinger, Brass Squire, Cabal Patriarch, Cid, Dauthi Voidwalker, Demon of Fate's Design, Emeritus of
Conflict / Ideation, Emry, Garth One-Eye, Gingerbrute, Gix, Jaxis, Kuldotha Forgemaster, Nicol Bolas the Ravager, Resplendent
Angel, Sanar, Scion of the Ur-Dragon, Steel Hellkite, Steel Overseer, Stoneforge Mystic, Thanos, Thrasios, Triskelion, Urza Lord
High Artificer, Venat, Whipcorder, Yawgmoth. Kefka and Tekuthal are Marvin's own hooks (kefka_borrowed, tekuthal_borrowed).
Gotchas: no Python (the hook blocks even a version check); cargo only under the systemd-run cap from rust/; zsh does not split an
unquoted `$args` string, use an array for several `--test` flags.
