# Practice mode: progress

Working log for the build in [practice-mode.md](practice-mode.md). Updated at every commit and checkpoint.

## Working rules

- Branch `practice-mode`. Build the steps of the Build plan in order (1a → 3f); do not reorder.
- Commit each finished piece with the whole test suite passing (`python3 -m unittest discover -s tests -t .`). Never commit failing tests; never change decklists.
- The simulations must not change: every new code path is inert unless a human seat is present. The sim-guard test (added in 1a) must stay green.
- Push `practice-mode` only at checkpoints. Never push `main`.
- Checkpoints: 7:30am, 11am, 3pm, 6pm and 10pm Mountain time. At each: commit, push, update this file, send one phone notification (under 200 characters) saying what's new and what to try.

## Status

- Current step: 1a (not started)
- Last checkpoint: none

## Done

(nothing yet)

## Notes and decisions

- 2026-09-29: design agreed; build plan and checkpoint times set by the user.
