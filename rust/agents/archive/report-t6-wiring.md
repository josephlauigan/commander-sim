# Report: Tier 6 stage 3a, wiring t6 and Godo

Tests: `cargo test --release` passes (all crates, one earlier http_play event-order flake that passed on rerun);
`cargo build --release --all-targets` has no warnings. The whole-game test (every JD deck incl. Godo vs t0..t6) runs
without a panic or loop. `cardgen validate`: all 37 decks pass.

## Files changed
- Tier lists (t6 added): sim-cli `decks.rs` (TIERS, tier_label, FILES, MINE), `args.rs` (--pool choices, DECK_KEYS),
  practice `catalog.rs` / `session/setup.rs` (TIERS, MY_DECKS), cardgen `data.rs` (TIERS), `sim-core/examples/all_decks.rs`.
  The win-rate zip `TIERS[1..]` x 5 table values still lines up (zip stops at 5; t6 has no win rate).
- Partners: cardgen `decks.rs::load_pool_deck` splits `A / B` into commander + commander2; `export.rs` writes
  `commander2` and the union colour identity for pool decks, and fetches both commanders; sim-cli
  `decks.rs::PoolDeck::parse` keeps the first commander (decks.json carries both; run.rs seats commander2 for any deck).
  Test: `cardgen/tests/workflow.rs::a_pool_deck_with_two_commanders_carries_both`.
- Export fix: `export.rs::uncached` now also fetches hand-tagged cards a pool deck plays (Cryptolith Rite is in
  carddb.txt, so `validate` found no Scryfall record for it).
- Data: `git mv decklists/pool/staging-t6-cedh decklists/pool/t6-cedh`; `TARGETS.md` moved to
  `decklists/pool/TARGETS-t6.md` (both pool loaders read every .md in a tier folder and it has no Commander line);
  README tier row + 6 deck rows; `data/my_decks.toml` [godo]; regenerated `data/cards.json`, `data/decks.json`,
  `data/scryfall_cache.json`; `tests/fixtures/my_decks_parsed.json` gained godo (deck guard test).
- Test counts updated: pool 30->36, t6 has 6 decks, JD decks 12->13, game.rs 91 games (13 x 7), tiers 6->7
  (practice http_catalog/http_server/server_fake, sim-cli decks.rs + poolmode.rs, sim-core export.rs, cardgen workflow.rs).
- Left alone: `cards_yshtola.rs:653` and `cards_t3.rs:445` (specific-deck stress loops over t1-t5, t6 not needed).
- Gotchas: the first `cardgen export` run failed on a double-faced fetch (Shatterskull Smashing); the second run was
  fine. Two t6 decks share the short name "Rograkh" (log name); `cardgen audit --deck` only takes pool keys, Godo is
  audited with `--mine`. `gc_rules` needed no change (t6 falls through to None).

## Audit (unique cards per deck; Full column = Full + Full-auto as the tool reports them)
| Deck | Full+Full-auto | Approximate | Partial | Unmodeled |
| --- | --- | --- | --- | --- |
| dack-fayden-white-stax | 49 (40 + 9) | 1 | 31 | 15 |
| kinnan-bonder-mana-combo | 69 (67 + 2) | 1 | 19 | 11 |
| kraum-tymna-blue-breach-midrange | 75 (71 + 4) | 1 | 17 | 7 |
| rograkh-silas-turbo-breach | 74 (70 + 4) | 1 | 18 | 7 |
| rograkh-thrasios-creature-combo | 58 (56 + 2) | 2 | 30 | 10 |
| sisay-legends-toolbox | 68 (65 + 3) | 1 | 28 | 3 |
| godo (87 unique) | 49 | 1 | 24 | 13 |

## Unmodeled and Partial cards per deck
### dack-fayden-white-stax
Unmodeled: Angel's Grace, Apple of Eden,  Isu Relic, Blind Obedience, Chalice of the Void, Damping Sphere, Defense Grid, Disruptor Flute, Gideon of the Trials, Mandate of Peace, Manifold Key, Orim's Chant, Pithing Needle, Razorgrass Ambush // Razorgrass Field, Silence, Void Mirror
Partial: Abeyance, All-Fates Stalker, Bilbo's Gambit, Calamity's Wake, City of Traitors, Dack Fayden,  Helping Hand, Eiganjo,  Seat of the Empire, Eldrazi Confluence, Emergence Zone, Erode, Fiend Hunter, Fomori Vault, Gemstone Caverns, Get Lost, Gleaming Splendor, Jeweled Amulet, Kozilek's Command, Monumental Henge, Paladin Class, Petrified Hamlet, Preston,  the Vanisher, Radiant Solar, Scout's Warning, Soul Partition, Sunscour, Talon Gates of Madara, Tezzeret,  Cruel Captain, The Mind Stone, The Mycosynth Gardens, Touch the Spirit Realm, Vexing Bauble

### kinnan-bonder-mana-combo
Unmodeled: Commandeer, Copy Artifact, Dizzy Spell, Hidden Strings, Imposter Mech, Invasion of Ikoria // Zilortha,  Apex of, Mindbreak Trap, Mirrormade, Misdirection, Sudden Substitution, Twincast
Partial: Borne Upon a Wind, City of Brass, Clever Impersonator, Colossal Skyturtle, Disrupting Shoal, Drift of Phantasms, Emrakul,  the Promised End, Endurance, Enduring Vitality, Flash Photography, Flesh Duplicate, Gemstone Caverns, High Fae Trickster, Mockingbird, Snapback, The Cabbage Merchant, Valley Floodcaller, Volatile Stormdrake, Wan Shi Tong,  Librarian

### kraum-tymna-blue-breach-midrange
Unmodeled: Commandeer, Final Fortune, Mindbreak Trap, Misdirection, Orim's Chant, Rain of Filth, Silence
Partial: Borne Upon a Wind, City of Brass, City of Traitors, Exhibition Tidecaller, Flash Photography, Gemstone Caverns, Gleaming Splendor, Into the Flood Maw, Mnemonic Betrayal, Mockingbird, Rite of Flame, Sevinne's Reclamation, Simian Spirit Guide, Snapback, Tataru Taru, Valley Floodcaller, Voice of Victory

### rograkh-silas-turbo-breach
Unmodeled: Defense Grid, Final Fortune, Flare of Duplication, Last Chance, Mindbreak Trap, Misdirection, Rain of Filth
Partial: Borne Upon a Wind, Chromatic Star, City of Brass, City of Traitors, Daze, Demonic Counsel, Gemstone Caverns, Grinding Station, Hexing Squelcher, Into the Flood Maw, Mnemonic Betrayal, Rite of Flame, Rograkh,  Son of Rohgahh, Silas Renn,  Seeker Adept, Simian Spirit Guide, Timetwister, Valley Floodcaller, Vexing Bauble

### rograkh-thrasios-creature-combo
Unmodeled: Candelabra of Tawnos, Commandeer, Earthcraft, Flare of Duplication, Hidden Strings, Invasion of Ikoria // Zilortha,  Apex of, Mindbreak Trap, Misdirection, Noxious Revival, Training Grounds
Partial: Badgermole Cub, Biomancer's Familiar, Borne Upon a Wind, Chatterstorm, Clever Impersonator, Cloud of Faeries, Deserted Temple, Emerald Charm, Emergence Zone, Enduring Vitality, Flare of Denial, Flesh Duplicate, Formidable Speaker, Gemstone Caverns, Hexing Squelcher, Magus of the Candelabra, Minamo,  School at Water's Edge, Mockingbird, Oboro Breezecaller, Phantasmal Image, Rograkh,  Son of Rohgahh, Shang-Chi,  Master of Kung Fu, Shifting Woodland, Sowing Mycospawn, Springheart Nantuko, Talon Gates of Madara, The Cabbage Merchant, Valley Floodcaller, Vibrance, Wan Shi Tong,  Librarian

### sisay-legends-toolbox
Unmodeled: Mindbreak Trap, Redirect Lightning, Silence
Partial: Agatha's Soul Cauldron, City of Brass, Colossal Skyturtle, Deadpool,  Trading Card, Derevi,  Empyrial Tactician, Emiel the Blessed, Enduring Vitality, Ertai Resurrected, Forbidden Orchard, Gemstone Caverns, Gleaming Splendor, Ignoble Hierarch, Ioreth of the Healing House, Legolas's Quick Reflexes, Loki's Scepter, Magda,  Brazen Outlaw, Marvin,  Murderous Mimic, Mockingbird, Mount Doom, Selvala,  Heart of the Wilds, Shang-Chi,  Master of Kung Fu, Spider-Woman,  Stunning Savior, Tataru Taru, Teferi,  Who Slows the Sunset, The Cabbage Merchant, The Queen of Dale, Touch the Spirit Realm, Tyvar,  Jubilant Brawler

### godo
Unmodeled: Blood Moon, Defense Grid, Final Fortune, Heat Shimmer, Last Chance, Magnetic Theft, Manifold Key, Peter Parker's Camera, Pyrokinesis, Redirect Lightning, Ricochet Trap, Sculpting Steel, Shatterskull Smashing
Partial: Abstergo Entertainment, City of Traitors, Desperate Ritual, Electroduplicate, Gemstone Caverns, Goblin Engineer, Goblin Welder, Godo,  Bandit Warlord, Hammer of Nazahn, Hexing Squelcher, Honor-Worn Shaku, Metalworker, Molten Duplication, Party Thrasher, Rite of Flame, Simian Spirit Guide, Sting,  the Glinting Dagger, Talon Gates of Madara, Tezzeret,  Cruel Captain, Tibalt's Trickery, Treasonous Ogre, Untimely Malfunction, Vexing Bauble, _____ Goblin
