# Report: Phase 2, PR 2, the hard doubles (branch `phase2-doubles`)

Brief: `rust/agents/phase2-doubles.md`. Handoff: `audit/phase2/doubles-handoff.md`. Measured 2026-10-29, after merging `origin/main` (f6cb3ebc: AI foundations, goldfish, `scion`, Sephiroth FS retired, list swaps, #153).

## Merge and checks
- `git merge origin/main` (merge commit d9430727): no conflicts. The Treasure and floating-mana memory fix is on both sides as the same change. `git diff origin/main` shows nothing in the mana code, so one copy remains.
- `cardgen export --offline`: no change to `data/`. `cardgen validate`: all decks pass.
- `cargo test --release --no-fail-fast -j 22`: 3585 passed, 0 failed, 2 ignored.
- `cargo build --release --all-targets -j 22`: 0 warnings.
- `cardgen audit --paths`: 0 hard doubles, both over all cards and over the cards the decks play. `audit/phase2/paths-after-doubles.md` regenerated (371d09db). The merge added no new double, scion's cards included.

## Cards handled (34a92a6a)
| Card | Double class | Side kept | Deleted / overridden | Test (`sim-core/tests/paths_doubles.rs` unless noted) |
|---|---|---|---|---|
| Ruby, Pearl, Jet Medallion | hard (cost hook + static:cost) | compiled | cost hooks | `the_medallions_cut_exactly_one_from_their_colour` |
| Foundry Inspector, Etherium Sculptor | hard (cost) | compiled | cost hooks | `foundry_inspector_and_etherium_sculptor_cut_one_each_from_artifacts` |
| Starfield Mystic | hard (cost), now split | compiled discount; the card code keeps the +1/+1 counter | cost hook; `paths_ok` | `starfield_mystic_cuts_one_from_enchantments` |
| Dune Mover, Paladin of Predation | hard (toxic given twice) | compiled toxic (`toxic_hit`) | combat_damage hooks; `HOOK_TOXIC` | `toxic_creatures_give_their_poison_exactly_once` |
| Ribskiff, Venomous Brutalizer | hard (toxic), now split | compiled toxic; the card code keeps the rest | toxic hooks; `paths_ok` | `ribskiff_crewed_gives_two_poison_once` |
| Snap | hard (resolve + spell) | compiled spell | hand-written resolve | `snap_bounces_its_target_and_untaps_exactly_two_lands` |
| Laelia, the Blade Reforged | hard (compiled draw + hand-written impulse on attack) | card code | `dsl = []` | `laelia_exiles_one_card_and_grows_once_per_attack` |
| Liliana, Waker of the Dead | competing (+1) | card code (the walker) | `dsl = []` | `liliana_waker_of_the_dead_offers_only_the_walker_s_abilities` |
| Oath of Teferi | hard (enter exile) | card code (blink) | `dsl = []` | `oath_of_teferi_blinks_one_permanent_and_exiles_nothing_for_good` |
| Heliod's Pilgrim | load-time `data_fixes` | card code (Aura search) | `data_fixes` replaced by a `dsl = []` override | cards_t1 Pilgrim test |
| 18 shadowed spells (Liliana's Triumph, Kaya's Wrath, Vona's Hunger, Calamity's Wake, Erode, Open the Armory, Timetwister and others) | dsl-covers / shadowed | the hand-written resolve; the compiled spell stays as the AI's value | `paths_ok` on each | `the_shadowed_spells_...`, plus the Kaya's Wrath and Timetwister tests |

- `HOOK_TOXIC` is gone: every compiled toxic is now given in `tymna_cards::toxic_hit` (brief item 8).
- Heliod's Pilgrim's fix is now an override (brief item 7).
- `ai/pool.rs`: a noncreature permanent with a compiled cost static now ranks 68 on turns 1-5 and 52 after, as the cost hook did. This also lifts Emerald and Sapphire Medallion, Oketra's Monument and The Wind Crystal.

## False positives (`paths_ok`)
Each of these is two paths doing different parts of the card:
- Allosaurus Shepherd and Hexing Squelcher: the unc tag covers "this spell"; the card code covers "your spells".
- Contaminant Grafter: its combat_damage code is the proliferate; toxic is compiled.
- Tataru Taru: you draw (tag); the opponent may draw (card code).
- Twilight Diviner: surveil vs the token copy.
- Daxos: the life triggers vs the devotion toughness.
- Elemental Mascot: the opus trigger.
- Static Net: the compiled exile vs the life gain and Powerstone.

## Follow-ups for the ability language
- Laelia: "exile the top card, you may play it this turn" is read as a plain draw; the counter trigger is missing.
- Oath of Teferi: exile until the next end step.
- Static Net: exile until it leaves.
- Liliana, Waker of the Dead: the +1's "each opponent who can't loses 3 life".
- The 18 shadowed spells are dsl-partial (second modes, riders); each `paths_ok` note names the missing part. Until the AI phase their compiled spell is the AI's value of the card, so `dsl = []` would stop the AI casting six of them.

## Paths summary
| | all cards | cards the decks play |
|---|---|---|
| double: hard | 21 -> 0 | 0 |
| shadowed | 25 | 20 |
| competing | 21 | 14 |
| split | 161 | 122 |
| split (paths_ok) | 29 | 20 |
| dsl-covers | 28 -> 20 (8 deletions) | 14 |
| dsl-partial / dsl-none | 186 / 1474 | 138 / 1081 |

## Which tiers and decks
Pool decks that play a card whose rules changed:
- t0: Anikthea (Starfield Mystic)
- t1: Light-Paws (Heliod's Pilgrim, Pearl Medallion, Starfield Mystic)
- t2: Sythis (Starfield Mystic)
- t3: Atraxa (Oath of Teferi), Aurelia (Pearl), Marwyn (Emerald), Tergrid (Jet, Liliana)
- t4: Krenko (Ruby), Prosper (Laelia)
- t5: Urza (Etherium Sculptor, Foundry Inspector, Sapphire), Yawgmoth (Jet)
- t6: Dack Fayden (Pearl), both Rograkh decks (Snap)

So all of t0-t6. JD's decks play no card whose rules changed. Four play cards that only got `paths_ok`, which changes no behaviour: cloud and seph (Open the Armory), yshtola (Static Net), scion (Borne Upon a Wind, Hexing Squelcher, Rite of Flame, Tataru Taru). They were run because their opponent tiers change.

## Realism: `--realism --pool tN --games 4000 --jobs 22` (before = origin/main f6cb3ebc, after = branch head)
Decks marked * play a changed card.

| Tier | Targets pass | Deck shares, before -> after | Games skipped (before / after) |
|---|---|---|---|
| t0 | 4 -> 4 of 9 | Anikthea* 32.1 -> 31.7, Captain America 24.9 -> 25.0, Giada 29.6 -> 29.4, Stella Lee 18.6 -> 18.6, Zurgo 19.7 -> 20.2 | 0 / 0 |
| t1 | 5 -> 5 of 9 | Isshin 27.2 -> 26.9, Lathril 18.2 -> 18.9, Light-Paws* 44.9 -> 44.3, Tatyova 17.6 -> 17.5, Teysa 17.1 -> 17.4 | 0 / 0 |
| t2 | 1 -> 2 of 9 | Brago 13.5 -> 13.4, Kaalia 25.8 -> 25.9, Windgrace 26.0 -> 25.3, Meren 17.0 -> 17.3, Sythis* 43.1 -> 43.3 | 0 / 0 |
| t3 | 1 -> 2 of 9 | Atraxa* 24.1 -> 24.5, Aurelia* 27.4 -> 27.0, Korvold 17.0 -> 18.0, Marwyn* 19.6 -> 19.3, Tergrid* 37.0 -> 36.2 | 0 / 0 |
| t4 | 1 -> 2 of 5 | Chulane 13.9 -> 14.1, Edgar 49.4 -> 49.1, Krenko* 28.5 -> 28.3, Prosper* 15.4 -> 14.6, Yuriko 18.5 -> 19.6 | 0 / 1 |
| t5 | 1 -> 2 of 5 | Kinnan 30.8 -> 31.1, Miirym 30.0 -> 31.2, Urza* 22.0 -> 22.1, Yawgmoth* 19.8 -> 18.4, Zur 22.5 -> 22.2 | 12 / 10 |
| t6 | 3 -> 3 of 6 | Dack* 24.7 -> 24.7, Kinnan 22.8 -> 22.7, Kraum/Tymna 29.7 -> 29.4, Rograkh/Silas* 20.8 -> 19.5, Rograkh/Thrasios* 21.5 -> 22.4, Sisay 30.6 -> 31.5 | 0 / 0 |

- Every flip is a seat-share row on a band edge, not a mechanism:
  - t2 seat 3: FAIL -> PASS (24.1 -> 23.1)
  - t3 seat 3: FAIL -> PASS (23.6 -> 23.0)
  - t4 seat 3: PASS -> FAIL; seat 4: FAIL -> PASS
  - t5 seat 2: FAIL -> PASS (25.4 -> 26.3)
- t6's deck win spread failed in both runs (9.8 -> 12.1; band 0-8).
- Win-type rows move by 0.5 pts or less. t3 poison wins went 17.5 -> 17.0: with the toxic hooks gone, each toxic creature still poisons exactly once, so the double never ran in play.
- Every deck's move is inside its 95% CI (about ±1.5 pts at about 3200 games). Nothing was recalibrated.

## JD's decks: `--deck KEY --pool all --games 2000 --jobs 22`
| Deck | Before | After |
|---|---|---|
| cloud | t1 17.0, t2 13.4, t3 16.2: 15.5% over t1-t3 | t1 16.8, t2 12.9, t3 17.2: 15.6% over t1-t3 |
| seph | t3 23.2, t4 16.9: 20.1% over t3, t4 | t3 23.7, t4 17.2: 20.5% over t3, t4 |
| yshtola | 24.0% over t5 (1998 games) | 24.0% over t5 (1995 games) |
| scion | 27.4% over t6 | 27.5% over t6 |

All within noise. Every run in both progress logs ended rc=0, apart from the first four deck attempts, which had no `--pool all`, exited rc=1 at once and were rerun.

## Risks
- **The t4 unwrap panic is on main too.** The after-t4 run skipped seed 502910 to `called Option::unwrap() on a None value`. Replayed alone (`--calibrate within --pool t4 --trace 2910`), that game panics the same way on main's binary (f6cb3ebc) and on the branch's, so this branch did not cause it; main's 4000-game run just did not hit it. It joins the overflow panics for the bug hunt.
- **Overflow panics did hit.** t5 realism skipped 12 games on main and 10 on the branch to "attempt to add/subtract with overflow". Yshtola vs t5 skipped 2 and 5. This is the same family as main's 13 all-decks skips; not fixed here.
- Snap's compiled bounce retargets if its target is gone, where the hand-written resolve fizzled.
- The pool AI's cost rule (68/52) also moves Emerald and Sapphire Medallion, Oketra's Monument and The Wind Crystal.
- The 18 shadowed spells still run the hand-written resolve. Their compiled spell is only the AI's value until the AI phase.

## Replays for the bug hunt
1. `commander-sim --calibrate within --pool t4 --trace 2910`: the unwrap panic (main and branch).
2. The t5 overflow seeds (`--trace 210`, `--trace 862`), for the overflow fix.

## Commits
8d16e95a, 34a92a6a, d6e8b034, 2afe37db, be11c4f0, b5429034, d9430727 (merge), 371d09db (paths report after the merge), c69503c7 (handoff).
