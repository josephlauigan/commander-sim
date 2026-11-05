# Task: the person's planeswalker abilities (practice mode)

Worktree: `/home/joseph/projects/commander-sim/.claude/worktrees/practice-walker-fixes` (branch practice-walker-fixes, from practice-trigger-fixes). Task name: `walker-fixes`.

## Goal
`rust/crates/sim-core/tests/practice_trigger_probe.rs` (activated pass on by default) lists 8 real gaps in `TO_FIX`: when the
person activates these abilities, the engine makes the ability's choice for them. Read `rust/agents/report-trigger-fixes.md`
and `rust/agents/handoff-trigger-fixes.md` first (the previous agent's notes on each and the fix pattern).

Cards (check each ability against the Oracle text in `data/scryfall_cache.json` before coding it):
Karn Liberated (+4, -3), Ugin the Ineffable (-3), Karn, Scion of Urza (+1: the AI opponent rightly chooses which revealed
card the person gets; check what the person should choose, e.g. -1 and -2, and what is silent), Karn, the Great Creator,
Teferi, Time Raveler (-3), Wrenn and Six (+1, -1), Venser, the Sojourner (+2), Blightspeaker.

Fix pattern (from the report): a person's walker abilities come from a `walker(...)` entry in `human/cards.rs` (see Vraska and
Ral). Such an entry replaces the walker's whole generic list, so it must list every loyalty ability, ultimates included,
each asking the person what the Oracle text lets them choose (target, card searched, card returned, ...), and reuse the
AI's resolution code where it's right. Blightspeaker is a creature ability (Rebel search): follow the pattern for a person's
creature abilities in `human/gaps_a.rs`. Take each fixed entry off `TO_FIX`.

Tests: add to `rust/crates/sim-core/tests/practice_trigger_fixes.rs` (a test per card: the person is asked and their answer
is what happens; for walkers also that every loyalty ability is still listed).

## Done when
- `practice_trigger_probe` passes with `TO_FIX` empty (or entries explained), `practice_trigger_fixes` passes, and the existing
  test files for code you touched pass (walker tests, practice_gaps_*, practice_ability_probe, cards_karn*).
- `rust/agents/report-walker-fixes.md`: per card: done / not, what the person now chooses, files, tests.
## Ground rules (these override the older rust/agents/RULES.md, which is from the Python-port era)
- Work only in your worktree (path above); `cd` into it for every command. Don't merge other branches. Don't push.
- **Never run Python** (a hook refuses it). Use jq, shell tools, or Rust.
- **Memory cap on every cargo command**: `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 cargo ...`
  (run from `rust/`). The shell is zsh: write the prefix out in full, don't store it in a variable. Exit 137 = cap hit = a bug.
- **Don't run the full suite** (`cargo test --release` with no filter) and don't run sims: the coordinator does that. Build and
  run only your own test file(s) and the existing test files you touched or that test the code you changed
  (`cargo test --release -q -p sim-core --test <name>`). Run test commands in the foreground with a long timeout; don't start
  background jobs and don't leave anything running when you finish.
- The person's seat: `crate::human::is_human(g, p)`; questions go through `human::choices::{choose, choose_one, pick_cards,
  pick_any, yes_no, search, ...}` and `human::cards::pick_perm`; an engine decision made for a person is announced with
  `human::auto_note`. In a trigger, ask the person's question AFTER the trigger's `trigger_window(...)` call (the engine runs each
  trigger once as a probe up to its first window; a question before it is asked twice). AI paths must not change unless this
  brief says so: put person code in an `if is_human` branch.
- Check card behaviour against the Oracle text in `data/scryfall_cache.json` (keys are lowercased names:
  `jq '.["sol ring"].oracle_text' data/scryfall_cache.json`).
- Edit `rustfmt --edition 2024 <your new files>` only; never `cargo fmt` on the workspace (the repo isn't fmt-clean).
- Commit often (`git add -A rust/ && git commit -q -m "..."`); **no Co-Authored-By line**. Keep `rust/agents/handoff-<task>.md`
  current (done, next, gotchas). If your context gets long (~60+ tool calls), stop at a clean point with everything committed and
  end your reply with a first line `HANDOFF`.
- Finish with: everything committed; full details in `rust/agents/report-<task>.md` (committed; if a file write is refused, put the full report in your reply instead); a reply of at most 15 lines
  (commit hash, done / not done, test results, anything the coordinator must act on). Then stop.
