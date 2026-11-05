# Task: fix the trigger-probe gaps and review the activated-ability leads (practice mode)

Worktree: `/home/joseph/projects/commander-sim/.claude/worktrees/practice-trigger-fixes` (branch practice-trigger-fixes, from practice-probes / PR #259). Task name: `trigger-fixes`.

## Goal
`rust/crates/sim-core/tests/practice_trigger_probe.rs` checks triggers on JD's own cards with a person (P0) in control. Read
`rust/agents/report-trigger-probe.md` first. **AI paths must not change**: every fix goes in an `is_human(g, p)` branch.
Ask a trigger's question after its `trigger_window(...)` (see ground rules).

1. Fix the 3 TO_FIX gaps (report table has file:line and suggested fixes), checking each Oracle text first:
   - **Felidar Retreat** (`engine/zones.rs` landfall_once, `Tag::Landfall2`): a person chooses the mode (2/2 Cat, or a +1/+1
     counter on each creature you control and they gain vigilance until end of turn); add a trigger window for the person.
   - **Sun Titan** (`engine/zones.rs` sun_titan; enters and attacks): "you may return target permanent card with mana value 3 or
     less from your graveyard to the battlefield": the person picks one or none.
   - **Mirrorworks** (`impls/karn_engines.rs` mirror_etb): "you may pay {2}": ask yes/no before paying.
   Take each fixed entry off `TO_FIX`.
2. **Make the activated-ability pass the default** (today behind `PROBE_ACTIVATE=1`) and review its 27 leads by reading the
   code that resolves each ability. Likely real: Karn Liberated (+4, -3), Ugin the Ineffable (-3), Teferi Time Raveler (-3),
   Wrenn and Six (+1, -1), Venser the Sojourner (+2), Karn the Great Creator, Karn Scion of Urza (+1), Kuldotha Forgemaster,
   Inventors' Fair, Blightspeaker, Lin Sivvi, Dauthi Voidwalker, Peter Parker's Camera, Sakura-Tribe Elder (urdragon). Likely
   the probe's table (report list). For each: a real gap -> fix it (person chooses, per the Oracle text) with a test; a
   false alarm -> `FALSE_ALARMS` with the reason (or improve the probe's table so it has a legal choice, if cheap). If the
   real ones turn out more than ~15 or any is large (> ~45 min), fix the small ones, list the rest in `TO_FIX` with notes, and
   stop.
3. Cosmetic, if quick: Vito, Thorn of the Dusk Rose's question is headed "Sanguine Bond:" (`impls/t4.rs` ~1335): head it with
   the card's own name, and move Vito from FALSE_ALARMS if the probe then sees it as asked.

Tests: a new file `rust/crates/sim-core/tests/practice_trigger_fixes.rs` with a test per fix (patterns:
`tests/practice_binder.rs`, `tests/play_common`), showing the person is asked and their answer is what happens.

## Done when
- `practice_trigger_probe` (activated pass on by default) passes, `practice_trigger_fixes` passes, and existing test files for
  code you touched pass (grep for them; e.g. walker tests for the planeswalkers you change).
- `rust/agents/report-trigger-fixes.md`: per card: real / false alarm, fix, files, tests; anything that would change the AI.
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
