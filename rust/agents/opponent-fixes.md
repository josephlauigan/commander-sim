# Task: fix the opponent-effects gaps (practice mode)

Worktree: `/home/joseph/projects/commander-sim/.claude/worktrees/practice-opponent-fixes` (branch practice-opponent-fixes, from practice-probes / PR #259). Task name: `opponent-fixes`.

## Goal
`rust/crates/sim-core/tests/practice_opponent_probe.rs` lists 29 places (`TO_FIX`) where an AI opponent's card sacrifices,
discards, picks or pays **for the person** (P0, `crate::human::is_human`). Read `rust/agents/report-opponent-probe.md` first:
it gives each cause, file:line and the suggested helper. Fix them all so the person decides, and take each fixed entry off
`TO_FIX` (the probe fails on stale entries). **AI paths must not change**: every fix goes in an `is_human(g, q)` branch for
the affected player q (often not the card's controller).

Order (commit after each group):
1. The DSL "sacrifice" arm (`dsl/mod.rs` ~1016-1041): a person picks via `human::choices::sacrifice_creature` /
   `sacrifice_permanent` from the same candidates (11 cards). Note Flare of Malice's Oracle ("sacrifices a creature or
   planeswalker with the greatest mana value among creatures and planeswalkers they control"): for the person, offer only
   the greatest-MV candidates if the filter can express it; don't change the AI's behaviour, just report the AI's mismatch.
2. The DSL "discard" arm (~972-987): a non-random discard by a person goes through `engine::cast::discard_worst` (it
   routes to the person). Random discards stay random.
3. The individual cards (report table): Burglar Rat / Elderfang Disciple (`impls/t3.rs` discard_worst_for), Chittering
   Rats, Archfiend of Depravity, Council's Judgment (the person's vote), Painful Quandary, Rakdos Patron of Chaos, Single
   Combat, Sword of Feast and Famine (also `archon_attack`'s identical discard), The Tabernacle at Pendrell Vale, Wandering
   Archaic, Braids Cabal Minion, Liliana of the Veil -6 (Oracle: "Separate all permanents target
   player controls into two piles. That player sacrifices all permanents in the pile of their choice." The *caster* separates,
   the person picks the pile), Liliana Dreadhorde General -9 (the person picks which permanent of each type to keep).
   Read each card's Oracle text in `data/scryfall_cache.json` before coding it.
4. **Attack taxes** (JD asked for this): Propaganda / Ghostly Prison style taxes give the person one yes/no for the strongest
   k creatures (`engine/combat.rs` ~198, `attack_limits`). Let the person choose which creatures attack and pay for each
   (e.g. a pick-any of the creatures it can afford), keeping the AI path.
5. **Chain of Vapor's copy** (JD asked for this): Oracle "Return target nonland permanent to its owner's hand. Then that
   permanent's controller may sacrifice a land of their choice. If the player does, they may copy this spell and may choose
   a new target for the copy." The person, as that permanent's controller, is never offered it (no code). Add the offer for
   a person: sacrifice which land (or none), then the copy's target (`human::legal::describe_target` / spell targets). If
   the AI side has no code either, leave the AI as it is and say so.

Each fix gets a test in a new file `rust/crates/sim-core/tests/practice_opponent_fixes.rs` (one per cause at least; patterns:
`tests/practice_binder.rs`, `tests/play_common`), showing the person is asked and their answer is what happens.

## Done when
- `practice_opponent_probe` passes with `TO_FIX` empty (or only entries you explain in the report), `practice_opponent_fixes`
  passes, and the existing test files for code you touched pass (grep for them).
- `rust/agents/report-opponent-fixes.md`: per cause: done / not, files, tests; anything that would change the AI listed.
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
