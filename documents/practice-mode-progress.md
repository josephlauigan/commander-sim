# Practice mode: progress

Working log for the build in [practice-mode.md](practice-mode.md). Updated at every commit and checkpoint.

## Working rules

- Branch `practice-mode`. Build the steps of the Build plan in order (1a → 3f); do not reorder.
- Commit each finished piece with the whole test suite passing (`python3 -m unittest discover -s tests -t .`). Never commit failing tests; never change decklists.
- The simulations must not change: every new code path is inert unless a human seat is present. The sim-guard test (added in 1a) must stay green.
- Push `practice-mode` only at checkpoints. Never push `main`.
- Checkpoints: 7:30am, 11am, 3pm, 6pm and 10pm Mountain time. At each: commit, push, update this file, send one phone notification (under 200 characters) saying what's new and what to try.

## Status

- Current step: 1e (combat: attackers, defenders, blockers): next
- Last checkpoint: none (the user skipped the 2026-09-29 10pm checkpoint; next is 7:30am)

## Done

- **1a** (2026-09-29): `commander_sim/play/`: `session.py` (a game on a worker thread; events: log, turn, request, over, error), `controller.py` (HumanController: requests and answers across threads; `controller_of`/`is_human`), `view.py` (the table from one seat; hidden hands; token names), `text.py` and `__main__.py` (`python3 -m commander_sim.play --text --deck sauron --tier t4 [--step]`: watch a game from your seat). Engine change: `search.clone` drops `g.controllers`, so look-ahead copies play the human seat as the AI. Tests: `test_sim_guard.py` (24 seeded games fingerprinted) and `test_play.py`.
- **1b** (2026-09-29): `play/mana.py`: `ManaPool` (W U B R G C plus engine 'any' mana; pays pips from their colour, generic from colourless first; says what's missing), `sources` / `tap` (tap or sacrifice a source into the pool; painlands and Talismans may make colourless without pain), `pay_from_pool` (ritual floating mana joins the pool; the error text for the "Can't cast" message), `empty_pools`. Engine: `engine.pay` split into `spend_unit` (unchanged behaviour); `ais.continue_turn` empties pools at each step when a human is seated. The seat's view and the text client show the pool and untapped sources. Tests: `test_play_mana.py`.
- **1c** (2026-09-29): your main phases are yours. `play/legal.py` (the rules check: land drops, sorcery timing, instants, locks, mana with the reason), `play/human.py` (`human_main`: priority loop; actions `tap`, `land`, `cast` from hand or command zone, `pass`; illegal moves are explained and you keep priority). Engine: the turn start skips the AI's land drop and the main phases hand over to `human_main` when the seat is human. Text client commands: `show`, `tap N [C]`, `land N`, `cast N`, `cast cmd`, `pass`, `quit`. Still automatic (reported as "(automatic)" where the code knows): targets, X, the creature Diabolic Intent sacrifices, combat, responses on other turns, discards at end of turn. Tests: `test_play_turn.py` (including a bot that plays whole games of every deck through the human seat).
- **1d** (2026-09-29): targets and abilities. `legal.spell_targets` (rules-legal targets: any player's permanents, players for burn; hexproof, shroud, protection, nonblack, mana-value limits, Fatal Push's revolt; not the AI's "only what it kills" filter), `describe_target`, `target_extra_cost` (ward; Bloodchief's Thirst's kicker), `base_cost` (the printed cost where the engine stores the AI's), `equip_cost` / `check_equip`. `human.py`: targets chosen before paying and cancellable (`choose`), `use` action: Equip plus the permanent's card-code abilities (planeswalker loyalty abilities and hooked abilities). Engine: `can_pay` / `pay` for a seat with a mana pool draw from the pool (so abilities, ward and taxes use the mana you floated); look-ahead copies drop the pool. Text client: numbered permanents, `use N`, numbered choices. Tests: `test_play_abilities.py`; the bot now answers choices and equips.

## Notes and decisions

- 2026-09-29: design agreed; build plan and checkpoint times set by the user.
- Heuristic-AI games are identical across processes and memory layouts (checked with three perturbed runs), so the sim guard uses them. Look-ahead games are not (see step 3a).
- Found while building 1d, to fix with Sauron's card audit in 1h (fixing changes sims, so re-record the sim guard then): Go for the Throat is tagged "nonblack" (it should target any nonartifact creature, black ones included). Bloodchief's Thirst is stored at its kicked cost (the AI always kicks).
- Abilities that live in deck AI code rather than card hooks (Jace's Archivist's wheel, Rogue's Passage, Vandalblast's overload, Scavenger Grounds) aren't offered yet: 1h (Sauron) and 1i (other decks).
- `practice-mode` is branched from main, which does not yet have `sauron-breach` (Breach line, mana-payment tie-break, override fix). When that merges, re-record the sim guard.
