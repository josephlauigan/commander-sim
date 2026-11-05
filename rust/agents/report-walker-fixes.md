# Report: walker fixes (practice mode)

Branch practice-walker-fixes (from practice-trigger-fixes). All eight `TO_FIX` entries of `practice_trigger_probe` are fixed;
`TO_FIX` is now empty. **AI paths unchanged**: every new ability is a person-only entry (`cards::abilities` is only asked for a
person's permanent); the engine files got visibility changes only (`pub(crate)` on `ugin_plus`, `karn_plus`, `venser_emblem`,
`w6_ult`), plus one no-op-for-AI call in `cardcode::turn_start` (see Karn, the Great Creator).

Tests: `practice_trigger_fixes` 29 (19 new), `practice_trigger_probe` passes with `TO_FIX` empty, and 43 existing test files that
touch the code or the cards (walker tests `play_cards`, `cards_sauron`, `play_walker_attacks`; `practice_gaps_a/c`,
`practice_ability_probe`, `practice_followups`, `practice_simplifications`, `practice_limits`, `cards_karn_*`, `cards_alela`,
`cards_t3`, `cards_t5`, `cards_legends`, `cards_kefka`, `play_*` for the human seats ...) all pass. No sims, no full suite.

## Per card
Every walker entry lists ALL of the card's loyalty abilities (ultimates included), asks before the loyalty is paid (a cancelled
choice costs nothing), and carries the effect out after the ability resolves. File: `human/walkers.rs` (new; hooked in at the top
of `cards::abilities`). Tests are in `practice_trigger_fixes.rs`.

| Card | Done | What the person now chooses |
|---|---|---|
| Karn Liberated | done | +4: which player (any, "(you)" included) exiles a card; an AI target gives up its least-wanted card (now really exiled: the AI's own +4 removed it without exiling); a person target (yourself) picks the card. -3: which permanent to exile (any permanent, lands included; the AI's code never took lands). -14: the AI's restart (read as a win), listed. Tests: lists all 3; +4 names the opponent; +4 on yourself asks which card; -3 exiles a picked land |
| Ugin, the Ineffable | done | +1 (the AI's code) plus the person is now told the card exiled face down ("you look at X"). -3: which permanent that is one or more colours (colourless offered not). Tests: lists both, +1 told the card, -3 picks the Elves over the Dragon |
| Karn, Scion of Urza | done (+1 stays the AI's by design) | -1: which silver-counter card comes back (the AI took the best); -2 Construct (no choice). +1: "an opponent chooses" is an AI seat's choice, so it stays; the person is told both revealed cards. Probe: moved to `FALSE_ALARMS` with this reason. Tests: lists all 3; -1 with either card; +1 and -2 still work |
| Karn, the Great Creator | done | +1: which noncreature artifact (or none) becomes an artifact creature with P/T = its mana value until your next turn. The AI's +1 did nothing at all; this is a new, person-only effect (see below). -2: which artifact card in exile to take (or none); "from outside the game" is not modeled. Tests: lists both; +1 animates and reverts only at the person's own next turn; -2 takes the picked card |
| Teferi, Time Raveler | done | -3: which artifact, creature or enchantment to bounce (or "no target"), then draw. +1 listed; casting sorceries at instant speed is still not modeled (for either seat), and the person is told so. Tests: bounce picked + draw; no target + draw |
| Wrenn and Six | done | +1: which land card from the graveyard (or none); -1: the target of 1 damage (`damage_targets`, any target; `apply_damage`); -7: the emblem (the AI's). Tests: +1 picked land; -1 picked player vs the AI's creature |
| Venser, the Sojourner | done | +2: which permanent you own (lands and tokens included) to exile and return at the beginning of the next end step (before: the AI blinked at once). -1: creatures can't be blocked this turn: **the person never had it** (the AI's -1 is a crew hook that skips people), now listed (`unbl_all`). -8: emblem. Tests: lists all 3; +2 exiles then returns at `delayed_end_step`; -1 sets the flag |
| Blightspeaker | done | {T}: which player loses 1 life (you included); {4},{T}: the Rebel search asks which Rebel (Ramosian Lieutenant's `searcher(4, 3)`; the compiled DSL version searched any permanent card). Tests: lists both; player question; Rebel question |

## Notes for the coordinator
- **New hook**: `human::cards::replaces_dsl(g, m)` (list: Blightspeaker). `abilities_of` skips the compiled DSL abilities for such a card;
  without it Blightspeaker's DSL abilities were listed next to the new ones. Add a card name there when a card has both.
- **New state**: `Game::karn_anim` (artifacts animated by a person's Karn the Great Creator: owner, perm, previous P/T), reverted by
  `human::walkers::karn_revert` from `cardcode::turn_start` (empty list = no-op, so AI games and sims are unchanged). The animation sets
  `DataKey::Anim` and pow/tgh (the Starfield of Nyx way). Not tested in combat; a 0-mana-value artifact (Treasure) becomes 0/0 and dies,
  as the rules have it.
- Venser's return logs "(Eerie Interlude)" because it reuses `g.eot_returns`; the label is wrong, the behaviour right (galadriel.rs
  is AI code: left alone).
- Karn Liberated -14 is still the AI's simplification (opponents lose, "exiled with Karn" is not tracked); +4 does not mark its cards.
- Karn, Scion of Urza +1: if a second person ever sits at an opponent seat, "an opponent chooses" is still the AI's rule.
- Probe: `FALSE_ALARMS` gained `karn: Karn, Scion of Urza (Activate(0))` and `kefka: Blightspeaker (Activate(1))` (the probe's library holds no
  Rebel of mana value 3 or less, so the search has nothing to ask; practice_trigger_fixes covers it with a Rebel in the library).
- Not rustfmt'd: `practice_trigger_probe.rs`, `cards.rs`, `human.rs` (not mine / repo isn't fmt-clean). rustfmt'd: `walkers.rs`,
  `practice_trigger_fixes.rs`.
- Process slip: I once ran `python3 --version` by reflex; the hook refused it, nothing ran.
