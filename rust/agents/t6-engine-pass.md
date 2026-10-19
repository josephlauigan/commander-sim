# Tier 6 Stage 3: the engine pass (draft plan, for JD's review)

All 13 card batches are merged (159 cards, none Unmodeled). The card agents logged 90 engine needs in
`rust/agents/engine-needs-*.md` (the `## Card` sections; each names the card side already written and the test to add).
This file groups them into engine features, in the order proposed. Each feature: implement once in the engine, wire
the card code that is already waiting for it, flip those cards to Full, and add the listed tests. After each group:
full `cargo test --release` plus the regression guard (t0-t5 `--calibrate` unchanged beyond noise; needs JD's go
because it runs sims).

## Priority 1: what Tier 6 games and Godo's lines depend on

| # | Feature | Cards waiting |
| --- | --- | --- |
| 1 | **Alternative / pitch costs for hand spells** (one per-card hook: pay_alt_cost), and cost taxes applied to them | Daze, Flare of Denial, Snapback, Mindbreak Trap (free mode), Misdirection, Commandeer, Pyrokinesis, Sunscour, Disrupting Shoal (+ mana-value-X counter), Defense Grid |
| 2 | **Player "can't cast" locks** (per turn; noncreature-only and all-spells variants; also abilities for Abeyance) | Silence, Orim's Chant, Mandate of Peace, Calamity's Wake, Bilbo's Gambit, Abeyance |
| 3 | **Split second** | Sudden Substitution, Angel's Grace, Legolas's Quick Reflexes |
| 4 | **Counter-on-cast hook** (with X choice for Chalice) | Vexing Bauble, Chalice of the Void, Void Mirror |
| 5 | **Non-counter stack responses**: redirect a spell/ability, steal a spell (resolves under the new controller) | Redirect Lightning, Ricochet Trap, Untimely Malfunction, Misdirection, Commandeer, Sudden Substitution |
| 6 | **Answering abilities and triggers** (the AI counters an ability on the stack) | Ertai Resurrected, Louisoix's Sacrifice (both stop Godo's enter trigger) |
| 7 | **Copying an ability on the stack** | Peter Parker's Camera (Godo's Camera line) |
| 8 | **Combat**: no cap on extra combats; beginning-of-combat for every player; "end the combat phase" | Godo (Helm loop), Sting, Mandate of Peace |
| 9 | **Extra turns** (+ "lose at that turn's end step") | Final Fortune, Last Chance |
| 10 | **State-based actions**: toughness 0 or less dies; drawing from an empty library loses (from the specs: Thassa's Oracle lines) | Dismember vs Godo, Eldrazi Confluence; Oracle / Consultation lines |
| 11 | **Enters tapped as a replacement** (opponents' artifacts and creatures), and tapped artifact tokens | Blind Obedience, Spider-Woman, Tataru Taru (both stop Helm tokens attacking) |
| 12 | **Control change and goad** as first-class engine verbs (durations: until end of turn, rest of game) | Dack Fayden (donation, indefinite goad), Loki's Scepter, Volatile Stormdrake |
| 13 | **Copies**: "enters as a copy" replacement with exceptions and added/replaced types; restore the clones' printed 0/0 | Clever Impersonator, Phantasmal Image, Flesh Duplicate, Mockingbird, Copy Artifact, Mirrormade, Sculpting Steel, Molten Duplication, Preston (0/1 Illusion only) |
| 14 | **Flash grants**: a one-turn player grant and a permanent static | Borne Upon a Wind, Emergence Zone, Scout's Warning, High Fae Trickster, Valley Floodcaller |
| 15 | **Blood Moon as a real layer** (nonbasics lose abilities), Damping Sphere colour override | Blood Moon, Magus of the Moon, Damping Sphere |
| 16 | **Mana**: exile-from-hand mana for any card, abilities "as though haste", creature-ability cost reduction, untap any permanent | Simian Spirit Guide, Shang-Chi, Tyvar, Biomancer's Familiar, Training Grounds, Formidable Speaker |
| 17 | **Dungeon / venture** | Radiant Solar (Dack's drain) |
| 18 | **Combo shortcuts** in `impls/combos.rs`, each with answer windows at its real steps (JD's decision) | Godo + Helm (+ Hammer / Squire / Theft / equip; Camera; Mirror / Jaxis copies), Dualcaster + copy spell, Preston + Felidar + Solar, Ioreth + Marvin, Oboro + Cradle + Talon Gates, Kinnan + Basalt, Hullbreaker loops, Oracle / Consultation / Pact, Breach + LED + Brain Freeze |
| 19 | **Scripted combo tests from the specs**, including the Dack 35-pile engine test | all seven specs |

## Priority 2: correct but less decisive

Kicker and gift choices (Orim's Chant, Sowing Mycospawn, Bilbo's Gambit), hybrid costs and "mana spent" tracking
(Vibrance, Exhibition Tidecaller), "can't lose" and full prevention (Angel's Grace, Gideon), cast from exile (warp,
Soul Partition, Apple of Eden, Party Thrasher's convoke), Derevi's command-zone ability, per-name activation locks
(Pithing Needle, Petrified Hamlet), events (becomes tapped: Magda, Legolas; becomes the target: Phantasmal Image;
searches: Wan Shi Tong; played a land: City of Traitors), delayed end-step effects (Mnemonic Betrayal, Apple of Eden),
restricted / any-colour mana (Shang-Chi, Agatha's Soul Cauldron), sacrifice-a-land mana (Rain of Filth), ward paid in
life (Hexing Squelcher), hexproof from abilities, emblems (Tezzeret, Teferi), keyword removal (Emerald Charm), type
override (Enduring Vitality), level-gated anthem (Paladin Class), bestow and landfall copies (Springheart Nantuko),
earthbend land creatures (Badgermole Cub), MDFC land play.

## Priority 3: rare or out of scope for now

Battles (Invasion of Ikoria), vehicles and crew (Imposter Mech), cipher (Hidden Strings), vanishing (Flesh Duplicate),
controlling another player's turn (Emrakul), text-box exchange (Deadpool), land-becomes-a-copy (Shifting Woodland,
The Mycosynth Gardens), Marvin copying every ability, Gemstone Caverns' pre-game step, Map tokens (Get Lost).

## How it would run

The engine is shared code, so unlike the card batches these changes touch the same files (`engine/stack.rs`,
`engine/cast.rs`, `engine/combat.rs`, `engine/turn.rs`, `state.rs`). Parallel agents would conflict constantly. Plan:
priority-1 features in sequence, two agents at most at a time on features that touch different engine files (e.g.
casting rules in `cast.rs`/`stack.rs` vs combat/turns in `combat.rs`/`turn.rs`), each feature a small branch merged
before the next starts. Then combos (18) and the combo tests (19), which only touch `impls/combos.rs` and tests.
