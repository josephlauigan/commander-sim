# Brief: a generic play path for every card whose only one was a deleted hand AI

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/generic-paths` on branch
`generic-paths`: `hand-ai-delete` (the hand AI of 11 decks deleted) + `plan-files-3` (plan schema features `[equip]`,
`plays`, `[[stay_home]]`, `[[prio_add]]`, two knobs; plan files for cloud, yshtola, sheoldred). Run every command
from there. Read `CLAUDE.md`, `rust/agents/RULES.md`, `audit/phase4/plans-design.md` and the "Deleted (2026-11-01)"
section of `audit/phase4/hand-off-design.md` first: memory cap, no Python.

**A sim agent runs at the same time** (22 threads): use **`-j 6`** for cargo, in the background. **Run no
`commander-sim` and no game-playing example yourself**: cargo tests only. Wait on background work by its completion
notice or an output file, never `until ! pgrep -f ...`.

## Why

Deleting the hand AI left some cards with no way to be played by the generic policy (their only play path was behind
a deck-key gate; the generic policy never reached them on main either, with the hand AI off). The deletion's list:
the Underworld Breach line, Ballista Squad, Lawbringer, Lightbringer, Errant Doomsayers, Whipcorder, Mishra's Factory
blocks, the Archivist wheel, Voidborn's recast, Yawgmoth's Will, Mizzix's Mastery's mode choice, Blossoming Defense as
a response. The plan-files work found the same for cloud, yshtola and sheoldred and fixed those with `plays` and
`[equip]`.

## Work

1. **Audit:** find every card in a deck in `data/decks.json` (`mine` and `pool`) whose play path (an option, an
   activation, a response, a mode choice) is reachable only through a deck-key gate (`hand_key(..) == "<deck>"`,
   `is_main(hand_key(..))`, `matches!(hand_key(..), ...)` and the like) that no longer opens for any live deck by
   default. List each with its deck(s) and the gate. Check the list above is complete.
2. For each, give it a generic path: preferably the card's own code with the gate dropped or opened by the plan's
   `plays` (when the play is right for any deck holding the card), else a general feature the generic policy reads.
   Add the card to the plan file of each JD deck that holds it where `plays` is the mechanism. Behaviour must match
   the Oracle text in `data/scryfall_cache.json`; the AI's choice should be sensible for any deck, not tuned to one.
   If a card needs more than about a third of the session, write down what it needs and move on.
3. Unit tests: for each card, with the defaults, the generic policy now plays it in a hand-built position where it
   should, and declines where it shouldn't.
4. Write `audit/phase4/generic-paths-2026-11-01.md`: the audit list, what was done per card, what is left, and which
   decks and tiers each change reaches (this changes default play for those decks; the coordinator will re-run them).

## Checks

- `cargo test --release --no-fail-fast -j 6` passes; `cargo build --release --all-targets -j 6` 0 warnings;
  `cardgen validate` passes; `cardgen export` leaves no diff.

## Rules

- Don't change deck lists, `ai/search.rs`, `ai/ismcts.rs`, `ai/learned.rs`, `ai/rules.rs`, `data/eval/`,
  `audit/recal/`, or the hand AI of yshtola, sheoldred and cloud. No sims.
- Format only new files (`rustfmt --edition 2024 <file>`), lines up to 120. Commit per card or group, no
  Co-Authored-By line, never push. Scratch in
  `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/generic-paths/`.
- Report in 15 lines or fewer, then the full report as text (the harness refuses report files from subagents).
