# Report: model-gaps-b2 (Rings of Brighthearth and Peter Parker's Camera copy every ability a person can activate)

Branch `model-gaps-b2` (from `model-gaps-b1`). Brief: `rust/agents/model-gaps-b2.md`. The agent's report was returned as
text (the harness refused the agent's write of this file); the coordinator saved it here.

## Result
- Before: a person was offered Rings' copy only for abilities that went through `ability_run`, `ability_run_w`,
  `ability_window_ex`, `ability_window_rc` and their card forms. 46 of the 100 abilities in the karn, thrasios and sauron
  decks (and many in the other eight) used plain `ability_window` / `ability_window_card`: no offer, and the Camera could
  not copy them either.
- After: 65 of the 67 call sites in `human/` are converted (the other two are not activated abilities), and so are the
  `impls/` functions the person's menus reach.
- A sweep activates every ability of the 11 owned decks (battlefield, hand and graveyard) on two boards with Rings and {2}
  in play: 257 ask the Rings question and carry out a "yes" copy; 11 are listed exceptions (`EXCEPTIONS` in the probe).
- The conversion is `ability_run` / `ability_run_w`: the cost is paid before, as before; the effect is a closure run once
  per resolved copy, then for the original if it was not countered. A copy pays no cost. Where the ability has a target,
  the copy asks for a new one ("The copy of X's ability: ...?", "keep the target" keeps it). X and modes are the original's.
- Planeswalker abilities (`cards::walker`, 37 effects): `Ready::Go` holds a re-runnable `Fn`; a copy runs the effect's
  own choice function again (asks for its target again; cancelling keeps the original's choice). A copy pays no loyalty.

## Classification of the 67 call sites in `human/` (base 75058661 line numbers)
C = converted; C+T = converted, the copy chooses its target anew.

| site | card | ability | copyable? | result |
|---|---|---|---|---|
| gaps_a.rs:120 | Garth One-Eye | {T}: name, copy, cast | yes | C (copy names another card) |
| gaps_a.rs:291 | Scion of the Ur-Dragon | {2}: search a Dragon | yes | C |
| gaps_a.rs:337 | Faerie Mastermind | each player draws | yes | C |
| gaps_a.rs:363 | Ranger-Captain of Eos | no noncreature spells | yes | C |
| karn.rs:322 | Mishra's / Urza's Bauble | look, draw next upkeep | yes | C+T (player) |
| karn.rs:401, 420 | Soul-Guide Lantern | exile graveyards / draw | yes | C |
| karn.rs:498 | Unstable Obelisk | destroy target permanent | yes | C+T |
| karn.rs:528 | Kuldotha Forgemaster | search an artifact | yes | C |
| karn.rs:692-693 | Mishra's Factory / Foundry | pump an Assembly-Worker | yes | C+T |
| karn.rs:768 | Bonders' Enclave | draw | yes | C |
| karn.rs:798 | Buried Ruin | return an artifact card | yes | C+T (card) |
| karn.rs:849, 862 | Treasure Vault / Urza's Factory | Treasures / Assembly-Worker | yes | C |
| karn.rs:882 | Factory / Foundry animate | becomes a creature | yes | C (a copy finds the land already a creature: no second token) |
| karn.rs:1021, 1037 | Urza's Saga / Urza's Tome | Construct / draw-discard | yes | C |
| karn.rs:1119 | Soul-Guide Lantern | enters: exile a card | no, a trigger | NOT converted (NEEDS JD 2) |
| cards.rs:421-422 | every walker loyalty ability | loyalty abilities | yes (606) | C+T, one change for all 37 effects |
| cards.rs:663, 686 | Triskelion / Aetherflux | damage | yes | C+T |
| cards.rs:704, 731, 781 | Sakura-Tribe Elder / creature outlets (Marvin's too) / Coalition Relic | various | yes | C |
| cards.rs:980 | Strip Mine | destroy target land | yes | C+T |
| cards.rs:1002, 1018 | Desolate Lighthouse / Spectacle Summit | loot / surveil | yes | C |
| cards.rs:1659, 1691 | Azorius Guildmage | tap / counter ability | yes | C+T |
| cards.rs:1717 | Niv-Mizzet, the Firemind | draw | yes | C |
| cards.rs:2014, 2060 | Rebel searchers / Lin Sivvi X | search | yes | C |
| cards.rs:2040, 2096, 2137 | Blightspeaker / Lin Sivvi bottom / Ramosian Revivalist | player or card target | yes | C+T |
| cards.rs:2212, 2246 | Ballista Squad / tappers | damage / tap | yes | C+T |
| cards.rs:2262, 2274 | Maskwood Nexus / Mirror Entity | token / X/X | yes | C |
| cards.rs:2337 | Necropotence | pay N life, exile N | yes | C (with Rings and N>1: N windows, Rings asked once for the run) |
| abilities.rs:100 | Aggravated Assault | untap, extra combat | yes | C |
| abilities.rs:148 | Tortured Existence | return a creature card | yes | C+T |
| abilities.rs:227 | Basalt Monolith + Rings | "start the loop" | no, a table shortcut | NOT converted (NEEDS JD 1) |
| urdragon.rs:210, 252, 304 | Brass Squire / Peter Parker's Camera / Jaxis | attach / copy ability / copy creature | yes | C+T |
| gaps_c.rs:143, 235 | Angelic Page / Cabal Patriarch | pump / shrink | yes | C+T |
| gaps_c.rs:268 | Credit Voucher | shuffle, draw | yes | C (the copy picks its own cards) |
| gaps_c.rs:338, 434, 515, 590, 604 | Emry / Mirage Mirror / Gix / Dauthi (land and spell) | targeted | yes | C+T |
| gaps_c.rs:376, 457, 701 | Harnfel / Wispdrinker / Resplendent Angel | no target | yes | C |
| cloud.rs:86, 191 | Balan / Stoneforge Mystic | no target | yes | C |
| cloud.rs:121, 170, 265 | Cid / Giver of Runes / Plaza of Heroes | targeted | yes | C+T |
| fetch.rs:112 | every person fetch land | search for a land | yes | C |
| combos.rs:128 | Yawgmoth | -1/-1 counter, draw | yes | C+T |
| model_gaps.rs:45 | Maelstrom of the Spirit Dragon | search a Dragon | yes | C |

## `impls/` code reached from the person's paths (all converted)
Found with the sweep, a scan of every `impls::x::f` that `human/` calls, and a scan of owned permanents with an `options`
hook and no person path (`scan_option_hooks`).
- karn_cards: `disk_go`, `stone_mark` (+T), `stone_wipe`, `bomb_tick`, `bomb_blow`, `overseer_go`, `hellkite_go`, `leveler_unearth`.
- karn_engines: `mazemind_go` (the page counter only if it resolved), `forge_exile`, `fair_go`, `atlas_go`, `skull_go`, `torque_go` (+T), `staff_go`.
- lands: `war_room_go`, `ld_land_go` (+T).
- alela: `chimes_go`, `transmute_go` (a copy searches for its own card), `thopter_go`, `greaves_go` (+T).
- cloud: `zack_sacrifice` (+T).
- partials: `ring_go` (the burden counter by the first run only; every run draws a card per counter).
- mine: `kefka_ruin_do`.
- sheoldred: `seeker_flip`, `reveler_drain`, `tome_go`, `diary_go`, `bell_go`, `atlas_go`, `draw_land_go`.
- others: `t6_thanos::thanos_go`, `card_models::bolas_flip`, `t5::urza_top`, `yshtola::{inheritance_sac (+T), syphon}`,
  `legends::{sundering_go (+T), jar_go}`, `t45_urdragon::{hoard_go, lathliss_go}`.

AI behaviour unchanged: for a seat without a person `ability_run` is `ability_window` plus the effect, in the same place;
copies are always 0 for the AI (only a person's `rings_of(p)` and Camera put copies there); `ability_run` sets copy worth
0.0, so the AI never Cameras it (it needs `CAMERA_WORTH`); person-only choices in shared functions are guarded by
`is_human`. Two shared functions differ for a person on purpose: `sheoldred::atlas_go` (the counter ability has a stack
window for a person) and `partials::ring_go` (counter placement).

## Abilities that had no stack window, now with one
Mistrise Village; Bolas's Citadel's "each opponent loses 10" (sacrificing ten is the cost); Otherworld Atlas' charge
counter (person only).

## Files changed
- `engine/stack.rs` (shared): `ability_copies` logs "the copy of X resolves" per copy carried out (four callers); doc
  notes on `ability_window` / `ability_window_card` say to use `ability_run` for a person.
- `human/`: abilities, cards (walker framework, `new_damage_target`), cloud, combos, fetch, gaps_a, gaps_c, karn
  (`targets` is `pub(crate)`), model_gaps, urdragon, walkers (`tell_info` is `pub(super)`).
- `impls/` (shared, AI behaviour identical): alela, card_models, cloud, karn_cards, karn_engines, lands, legends, mine,
  partials, sheoldred, t45_urdragon, t5, t6_thanos, yshtola.
- Tests: `tests/practice_rings_probe.rs` (new), `tests/play_rings.rs` (extended). No `data/` change.

## Tests
- `practice_rings_probe.rs` `every_ability_a_person_can_use_offers_the_rings_copy`: every ability of every card in the
  11 decks, from battlefield, hand and graveyard, on a rich and a lean board; each must ask Rings, and a "yes" must log
  the copy and leave no copy waiting. A new failing card fails the test; an exception needs a reason. Two `#[ignore]`d
  scans (`scan_hand_and_graveyard_ways`, `scan_option_hooks`) print the data behind this report.
- `play_rings.rs`: 9 new spot tests (17 in all): Cid, Mind Stone, a fetch land, Urza's Saga, a copy choosing a new target,
  a loyalty ability, original countered / copy resolves, copy countered / original resolves, a Camera copy.
- The agent ran the brief's files (32 binaries) and 53 more covering the touched `impls/`; all pass; build has no
  warnings. The coordinator ran the full suite.

## NEEDS JD
1. `human/abilities.rs` ~237, Basalt Monolith + Rings "start the loop": not converted; a table shortcut standing for the
   whole infinite loop. Each real Basalt untap does ask Rings. Converting it means playing the loop out as repeated
   activations. Listed exception for karn and thrasios.
2. `human/karn.rs` ~1175, Soul-Guide Lantern's enter effect: uses `ability_window_card` but is a triggered ability (Rings
   never copies triggers). Making it a `trigger_window` would let the Camera copy it, but changes its window type.
3. Behaviour changes for a person: Mistrise Village, Bolas's Citadel's drain and Otherworld Atlas' counter now go on the
   stack (opponents can respond); Necropotence with Rings and N>1 makes N windows.
4. Run-level Rings questions (existing design, now also Necropotence): Ballista pings, Thrasios x N, Staff draws and
   Necropotence ask Rings once per run; Staff of Domination's {1} untap is never offered Rings (`human/abilities.rs:292`,
   `with_rings_auto(Some(false))`); the last odd draw of a run is never copied.
5. Probe exceptions: Mystic Forge's two "cast from the top" ways and Bolas's Citadel's (spell casts); Blazing Firesinger's
   prepared cast (a spell); Niv-Mizzet's "where pings go" (a setting); Phyrexian Tower and Lion's Eye Diamond (mana
   abilities, rule 605); Peter Parker's Camera's own ability (needs another ability on the stack; tested in `play_rings`);
   the two Basalt shortcuts (item 1).
6. Copy rules chosen: "keep the target"/cancel keeps the original's target; a walker's copy asks the same question with an
   info line above; a copy of Mishra's Factory's animate does nothing (already a creature); a copy of Memory Jar wheels
   again, with two delayed returns.
7. Not about Rings: a person has no way to activate Clue, Food or Blood tokens in practice mode (no menu entry;
   `Action::Clue` is the AI's).
8. Lathliss, Dragon Queen's AI option, offered to a person, can fail with "that can't be done right now": it computes k
   from all mana but needs k red mana. Pre-existing.
