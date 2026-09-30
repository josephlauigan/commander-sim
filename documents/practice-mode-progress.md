# Practice mode: progress

Working log for the build in [practice-mode.md](practice-mode.md). Updated at every commit and checkpoint.

## Working rules

- Branch `practice-mode`. Build the steps of the Build plan in order (1a → 3f); do not reorder.
- Commit each finished piece with the whole test suite passing (`python3 -m unittest discover -s tests -t .`). Never commit failing tests; never change decklists.
- The simulations must not change: every new code path is inert unless a human seat is present. The sim-guard test (added in 1a) must stay green.
- Push `practice-mode` only at checkpoints. Never push `main`.
- Checkpoints: 7:30am, 11am, 3pm, 6pm and 10pm Mountain time. At each: commit, push, update this file, send one phone notification (under 200 characters) saying what's new and what to try.

## Status

- Current step: 1c (your main phase by hand): next
- Last checkpoint: none (the user skipped the 2026-09-29 10pm checkpoint; next is 7:30am)

## Done

- **1a** (2026-09-29): `commander_sim/play/`: `session.py` (a game on a worker thread; events: log, turn, request, over, error), `controller.py` (HumanController: requests and answers across threads; `controller_of`/`is_human`), `view.py` (the table from one seat; hidden hands; token names), `text.py` and `__main__.py` (`python3 -m commander_sim.play --text --deck sauron --tier t4 [--step]`: watch a game from your seat). Engine change: `search.clone` drops `g.controllers`, so look-ahead copies play the human seat as the AI. Tests: `test_sim_guard.py` (24 seeded games fingerprinted) and `test_play.py`.
- **1b** (2026-09-29): `play/mana.py`: `ManaPool` (W U B R G C plus engine 'any' mana; pays pips from their colour, generic from colourless first; says what's missing), `sources` / `tap` (tap or sacrifice a source into the pool; painlands and Talismans may make colourless without pain), `pay_from_pool` (ritual floating mana joins the pool; the error text for the "Can't cast" message), `empty_pools`. Engine: `engine.pay` split into `spend_unit` (unchanged behaviour); `ais.continue_turn` empties pools at each step when a human is seated. The seat's view and the text client show the pool and untapped sources. Tests: `test_play_mana.py`.

## Notes and decisions

- 2026-09-29: design agreed; build plan and checkpoint times set by the user.
- Heuristic-AI games are identical across processes and memory layouts (checked with three perturbed runs), so the sim guard uses them. Look-ahead games are not (see step 3a).
- `practice-mode` is branched from main, which does not yet have `sauron-breach` (Breach line, mana-payment tie-break, override fix). When that merges, re-record the sim guard.
