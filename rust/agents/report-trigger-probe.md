# Report: trigger probe (practice mode)

Test: `rust/crates/sim-core/tests/practice_trigger_probe.rs`
Run: `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 cargo test --release -q -p sim-core --test practice_trigger_probe -- --nocapture` (passes, 0.05 s).
Env switches: `PROBE_DUMP=1` (every card x moment: fired / asked), `PROBE_ALL="<card>"` (every request for one card), `PROBE_CHOICES=1`
(choices in the prompts), `PROBE_SIG=1` (state-diff detections), `PROBE_ACTIVATE=1` (the unreviewed activated-ability pass, below).

## Counts (default run)
- 91 (deck, card) pairs / **76 cards** (of 683 unique) are permanents with a triggered line other than "when ... enters" that
  implies a choice (WORDS as the choice probe, matched on the effect).
- 74 pairs: the trigger fired and the person was asked (96 moments asked in all).
- **9 flagged** (trigger resolved, person asked nothing): **3 TO_FIX**, 6 FALSE_ALARMS.
- **8 unprobed** (listed in `UNPROBED` with why; a new one or one that now fires fails the test).
- Twice-asked check: clean (`TWICE_OK` empty). It only looks at moments with one triggering event (death, life, discard, draw, token).

## How it drives a card
Table: the card's deck vs `seph` and `thanos` (three seats, so "target opponent" has two options), two of everything, graveyards
as in the choice probe, equipment on top of P0's library, a hand of land / creature / Lightning Bolt / Brainstorm / Mind Stone.
The card enters *before* the person sits (its own enter trigger is the choice probe's), except `SelfEnter`, where the person is
seated first (the "enters" half of "whenever ~ enters or attacks", which the choice probe skips).
Moments, each through the engine's own entry point: `step_start` (upkeep, draw step, main 1 beginning, Saga chapters twice),
the person's main phase (land, creature, instant, artifact cast: landfall and cast triggers), `combat::combat` (opponent's
creatures tapped), `continue_turn` from main 2 (postcombat and end step), the AI opponent's whole turn, `zones::die` (the card,
P0's creature, P1's creature, a sacrifice), `life::gain` / `lose_life`, `discard_index`, `zones::draw` (P0, P1), `make_tokens`,
creatures/artifacts entering (P0, P1), `cardcode::ring_tempt`.
Moments are picked from the trigger's *condition* (the words before the first comma, quotes removed).
A trigger "fired" if its "Your trigger: <card>" priority window showed (the probe person has auto-pass `all`), or the person was
asked a question headed by the card's name, or the table's state changed compared with the same moment run with a blank
permanent (Shivan Dragon / Forest) in the card's place ("state only" in the key).
Flagged = fired and no question other than the probe's own plays / prompts headed by another card.
Limits: a card with two triggers in one moment, one asking and one silent, is not flagged; every answer is the first choice.

## TO_FIX (real gaps: the AI decides for the person)
| Card (deck) | What it decides | Where | Suggested fix | Size |
|---|---|---|---|---|
| Felidar Retreat (alela) | Landfall "choose one": always the 2/2 Cat Beast token, never the +1/+1 counters + vigilance mode; no trigger window either | `engine/zones.rs:1719` (`landfall_once`, the `Tag::Landfall2` loop) | In an `is_human` branch: `trigger_window(...)` then `choices::choose` between the two modes; make the Cat or put the counters on each creature (+vigilance until end of turn) | S |
| Sun Titan (thrasios) | Which permanent card with mana value 3 or less returns (enters and attacks): the AI takes the best | `engine/zones.rs:1823` (`sun_titan`; called from `engine/combat.rs:399` and `engine/zones.rs:1516`) | In an `is_human` branch after a `trigger_window`: `choices::choose_one`/`pick_cards` over `cs` ("you may": allow none); a land enters tapped via `add_land_entering` | S |
| Mirrorworks (karn) | Whether to pay {2} for the token copy: after the window it pays and copies with no yes/no | `impls/karn_engines.rs:307` (`mirror_etb`) | Replace the `trigger_window` + `pay` with `may_window_ask` (a yes/no headed "Mirrorworks: ...") for a person, then `pay` | S |

## FALSE_ALARMS
- alela Stirring Hopesinger: puts a counter on each creature, no choice; "target" is in the condition.
- sheoldred Vito, Thorn of the Dusk Rose: the person *is* asked which opponent loses the life, but the prompt is headed
  "Sanguine Bond:" (`impls/t4.rs:1335`, shared with Sanguine Bond), so the probe filters it as another card's. Cosmetic: make
  `bond_target` take the card name (S).
- thanos, yshtola Necropotence: the discarded card is exiled, no choice (matched "discard a").
- urdragon Goldspan Dragon: makes a Treasure, no choice; "target" is in the condition.
- urdragon Korvold, Fae-Cursed King (sacrifice moment): counter + draw are automatic; its "sacrifice another permanent" is asked on entering and attacking.

## Unprobed (8)
- alela Karmic Justice: needs an opponent's spell/ability to destroy the person's noncreature permanent.
- cloud Jocasta, Automaton Avenger: triggers from the graveyard when the commander attacks.
- karn, thrasios Rings of Brighthearth: triggers on activating an ability (the probe never activates one).
- karn Urza's Saga: a land Saga, placed without lore counters so no chapter comes (chapter III's search: practice_binder.rs).
- sauron The Ozolith: needs counters on it.
- sauron Vision, Synthezoid Avenger: needs an opponent casting a spell on their own turn (AI opponent has nothing to cast).
- thrasios Tymna the Weaver: needs combat damage to an opponent before the second main phase (phases are driven apart).

## Not done: activated abilities' resolution (`PROBE_ACTIVATE=1`)
Built but kept off by default because its 27 hits are not yet reviewed (no code reading, so no TO_FIX/FALSE_ALARM verdicts):
the person uses each activated ability whose effect implies a choice (the ability is picked in the "which ability?" prompt by
its loyalty cost or effect words) and is flagged if it resolves ("You activate <card>" window) with no question. With it on,
201 pairs / 158 cards are probed and the test fails on the unlisted hits. Leads, by likelihood:
- Probably real (planeswalker / search / target picked by the AI): karn Karn Liberated (+4, -3), karn Ugin the Ineffable (-3),
  thanos Teferi Time Raveler (-3), urdragon Wrenn and Six (+1, -1), alela Venser the Sojourner (+2), karn Karn the Great Creator,
  karn Karn Scion of Urza (+1 opponent chooses), karn Kuldotha Forgemaster, karn/urdragon Inventors' Fair, kefka Blightspeaker,
  galadriel Lin Sivvi, thrasios Dauthi Voidwalker, sauron Peter Parker's Camera, urdragon Sakura-Tribe Elder (the other
  decks' Elder asks "Search for a basic land": a key-gated path in urdragon).
- Probably the probe's table (no legal choice): alela Angelic Page, galadriel Ballista Squad, karn Mishra's Foundry (need an
  attacking creature), alela Judge's Familiar (needs a spell to counter), galadriel Lawbringer / Lightbringer (no red/black
  creature), alela Tragic Poet and Crystal Chimes, cloud Cid Freeflier Pilot, galadriel Ramosian Revivalist (no matching card in
  the graveyard), karn Mishra's Factory (only itself is an Assembly-Worker), cloud Plaza of Heroes.
- Also unprobed with it on: karn/thrasios Walking Ballista (leaves the battlefield on entering with no counters),
  urdragon Maelstrom of the Spirit Dragon and yshtola Loran of the Third Path (the ability never went on the stack).
The coordinator can turn it on, hand-check the "probably real" list, and fill the lists the same way.

## Notes for the coordinator
- The 3 TO_FIX entries are small and independent; fixing one means taking its key off `TO_FIX` (the test fails on stale entries).
- The key ends with the moments that were silent, e.g. `thrasios: Sun Titan (Combat, SelfEnter)`.
- No Python, no sims, nothing left running. rustfmt run on the new file only.
