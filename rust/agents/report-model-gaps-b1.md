# Report: model-gaps B1 (practice mode: probe the unprobed cards, fix what they find)

Branch model-gaps-b1 (from model-gaps). **AI paths unchanged**: every fix is a person-only entry (`cards::abilities`, `abilities::land_abilities`
are asked only for a person's permanents); nothing in `engine/`, `impls/`, `ai/` changed. No sims, no full suite.

## Result
`UNPROBED` in `practice_trigger_probe` went from 23 entries to 0: every entry is now driven and the person is asked.
The probe's counts: 201 pairs, 189 fired and asked, 12 flagged (all the existing `FALSE_ALARMS`, none new), 0 unprobed.
Real gaps found: 3 cards (Loran, Maelstrom: the person had no ability at all; Ramosian Revivalist: chose the target too late).

Tests (all pass): `practice_trigger_probe`, `practice_choice_probe`, `practice_ability_probe`, new `practice_model_gaps` (12), every
`practice_*`, `play_*` and `model_gaps_*` file, plus `generic_paths`, `mana_pool`, `cards_galadriel`, `cards_legends`, `cards_cloud`,
`cards_t5`, `cards_karn_verify`. `cargo build --release --all-targets`: no warnings. Not run: the full suite and sims.

## Per card
Probe hooks: `precondition(g, name, sc, ...)` (per-card setup before the person is seated), new moments `OppDestroy`, `OppCast`,
`UseAbility`, a one-shot `win` action for a priority window of the person's own trigger (the Camera), special placements in `run`
(Walking Ballista cast with X = 6, Jocasta in the graveyard with the commander out, Urza's Saga with `land_etb`). `PROBE_ALL=<card>`
now also prints what the person was told (`TOLD`). Also: the `": which ability?"` filter no longer hides "copy which ability?".

| Card | Probed | What the person chooses | Fix / file / test |
|---|---|---|---|
| Angelic Page | yes (attacker in `g.in_combat`) | the attacking or blocking creature | fine |
| Ballista Squad | yes | the creature, then X | fine |
| Mishra's Foundry / Factory | yes (a second Assembly-Worker, attacking) | which Assembly-Worker (attacking, for the Foundry) | fine; tests `factory_pump_asks_which_assembly_worker`, `foundry_pump_only_offers_attacking_workers_and_pays_one` (A1 abilities) |
| Cid, Freeflier Pilot | yes (Equipment in the graveyard) | which Equipment/Vehicle card | fine |
| Plaza of Heroes | yes (a legendary creature) | which legendary creature | fine |
| Lawbringer / Lightbringer | yes (red / black creature) | which creature to exile | fine |
| Ramosian Revivalist | yes (Rebel in the graveyard) | **the target Rebel card, chosen on activation** | GAP FIXED: the person used to pay, tap and only then pick (with a "find nothing" the Oracle does not give). Now the target is picked first (cancel costs nothing), the card returns when the ability resolves if still there. `human/cards.rs` `revivalist`; tests `revivalist_targets_the_rebel_the_person_names_before_paying`, `revivalist_cancelled_target_costs_nothing` |
| Walking Ballista (karn, thrasios) | yes (cast with X = 6, three counters) | the target, then how many counters | fine |
| Peter Parker's Camera | yes (Godo's enter trigger on the stack) | which ability to copy | fine (the probe's filter hid "copy which ability?"; fixed) |
| Dauthi Voidwalker | yes (an opponent creature dies: void counter; `check_state` sweeps the graveyard) | which exiled card to play free | fine |
| Maelstrom of the Spirit Dragon | yes | **the Dragon card to search for** | GAP FIXED: the person had **no** ability for the {4},{T},sacrifice search ("no ability to activate beyond mana"). New land ability: `human/model_gaps.rs` `maelstrom` (pays {4}, sacrifices as a cost, window, search for a Dragon card/changeling, reveal to hand, Opposition Agent respected, shuffle). Tests `maelstrom_*` (3) |
| Loran of the Third Path | yes | **the target opponent** | GAP FIXED: the person had **no** ability for "{T}: you and target opponent each draw a card" (the AI's option hook only offers it when its own conditions say so). New: `model_gaps.rs` `loran` (`gaps_t2::pick_opponent`; hexproof respected). Tests `loran_*` (2). Its enter trigger was already asked (choice probe) |
| Urza's Cave | yes (already in the Activate pass: it asks "Search for a land card") | the land | fine; `practice_choice_probe` reasons updated |
| Karmic Justice | yes (new moment `OppDestroy`: an opponent destroys the person's Bonesplitter) | window, then which permanent (or "don't destroy") | fine |
| Jocasta, Automaton Avenger | yes (in the graveyard, the commander attacks) | yes/no "return her attacking?" | fine |
| Rings of Brighthearth (karn, thrasios) | yes (new moment `UseAbility`: Grim Monolith's untap) | yes/no "pay {2} to copy it?" | fine for the abilities that carry out copies. **Limitation (not a regression)**: the person is offered the copy only for abilities that go through `stack::ability_run` / `ability_window_ex` / `ability_window_rc` (Monolith, Hellkite +1/+0, Ballista, Thrasios, DSL abilities, loyalty abilities); every other person ability (Cid, Mind Stone ... through plain `ability_window`) gets no Rings offer. A general fix needs each effect to be re-runnable; not attempted |
| Urza's Saga | yes (`land_etb` gives lore I; `step_start` x2) | window, then the artifact card with mana cost {0} or {1} | fine |
| The Ozolith | yes (2 counters on it, combat) | which creature (or "don't move them") | fine |
| Vision, Synthezoid Avenger | yes (new moment `OppCast`: an opponent casts on the person's turn) | the mode | fine |
| Tymna the Weaver | yes (opponent marked as dealt combat damage, second main) | yes/no pay X life | fine |

## model-gaps A1 abilities, used by a person (tests in `practice_model_gaps.rs`)
Mishra's Factory / Foundry pumps (see above), Steel Hellkite {X} (`steel_hellkite_x_asks_x_and_destroys_that_mana_value`: asks X,
destroys that mana value only), Liquimetal Torque (`liquimetal_torque_asks_which_permanent_becomes_an_artifact`), Tezzeret's emblem
(`tezzerets_emblem_asks_which_artifact_gets_the_counters`). All already worked for a person; no change needed.

## AI-side mismatches found (not touched; they need sims)
- **Maelstrom of the Spirit Dragon**: the AI never uses the Dragon search at all (no AI code; `t45_urdragon.rs` only has its mana).
- **Loran of the Third Path**: the AI draws only at the end-of-turn window (`post.is_none()`) and always picks the least threatening opponent; it never uses it in its own turns.
- **Ramosian Revivalist** (AI, `galadriel::revivalist_go`): the card is picked when the ability resolves, not as a target; equivalent in practice.
- **Rings of Brighthearth** for the person: see the limitation above.
- Dauthi Voidwalker (AI) still plays only spells of mana value 4 or more and never a land (known).

## Files
`human/model_gaps.rs` (new), `human/cards.rs` (`revivalist`, dispatch), `human/abilities.rs` (land dispatch), `human/mod.rs`;
`tests/practice_trigger_probe.rs`, `tests/practice_choice_probe.rs` (reasons), `tests/practice_model_gaps.rs` (new, rustfmt'd).
`model_gaps.rs` is rustfmt'd; `cards.rs` and the probe files are not (repo is not fmt-clean).
