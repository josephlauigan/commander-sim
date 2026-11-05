# Task: trigger probe for JD's own cards (practice mode)

Worktree: `/home/joseph/projects/commander-sim/.claude/worktrees/practice-trigger-probe` (branch practice-trigger-probe). Task name for files: `trigger-probe`.

## Goal
Practice mode lets a person play one of JD's decks. Three automatic probes already check it:
`rust/crates/sim-core/tests/practice_ability_probe.rs` (activated abilities), `practice_choice_probe.rs` (choices as a card
is cast or enters) and `practice_counter_probe.rs` (counterspells, protection). **Nothing automatic checks triggers later
in a game** (upkeep, attack, combat damage, deaths, landfall, cast triggers, end step, saga chapters, "whenever ... draw"), or
what an ability does when it resolves. A hand audit found the latest batch: Sauron, the Necromancer's attack trigger picked
the graveyard creature for the person, Triple Triad played every card for them, Gilgamesh took all the Equipment. Build the
permanent check so new cards are covered.

**This round you write the probe and classify what it finds. You don't fix the gaps** (the coordinator decides after seeing
the list).

## What to build
New test file `rust/crates/sim-core/tests/practice_trigger_probe.rs`, modelled on `practice_choice_probe.rs` (read it whole:
card list from `data/decks.json` "mine", Oracle text from `data/scryfall_cache.json`, `choice_lines` filtering, the table
with two of everything, `catch_unwind` per card, `TO_FIX` / `FALSE_ALARMS` lists with a `check` that fails on new gaps and
on stale entries, the twice-asked check).

1. **Cards**: every card in JD's 11 decks (`.mine[].cards`, 683 unique) with a triggered line ("When", "Whenever", "At the
   beginning", saga chapters) **other than** "when ... enters" (the choice probe covers that), whose text implies a choice
   (the choice probe's WORDS: search, choose, target, you may, sacrifice a, discard a, return a, ... ). Also the resolution of
   activated abilities the ability probe only activates, where practical (e.g. Urza's {5}).
2. **Drive the moment** with the person controlling the card (P0, `g.humans.add`), on a table with at least two of
   everything (as the choice probe's `play_it`): P0's upkeep (`engine/turn.rs`), P0 attacks with it (`engine::combat::combat`
   with the card as an attacker; see `engine/combat.rs` attack triggers and Tag-driven ones such as `Tag::Necromancer`),
   combat damage to a player, it or another creature dies, a land enters (landfall), P0 casts a spell (cast triggers), a card
   is drawn, end step. Use the engine's own entry points (`engine/turn.rs`, `engine/combat.rs`, `engine/hooks.rs`, the
   `Registry` hooks in `hooks.rs`: upkeep, end_step, attack, combat_damage, dies, self_dies, landfall, cast, draw, ...), not
   card functions called directly, so Tag-driven engine triggers are covered. Do what's practical; record cards you
   couldn't drive in an "unprobed" list with why rather than spending long on one card.
3. **Detect**: the card's trigger resolved (state changed) and P0 was asked nothing while the text implies a choice with two
   or more options. Also flag a question asked twice in a row.
4. **Lists**: every flagged card goes in `TO_FIX` (real gap) or `FALSE_ALARMS` (with why) so the test passes. For each TO_FIX
   entry, find the code that makes the choice (file:line) and say how to fix it.
   Known not-gaps to keep out of TO_FIX: Sauron the Necromancer, Triple Triad, Urza, Gilgamesh (fixed in #258).

## Done when
- `cargo test --release -q -p sim-core --test practice_trigger_probe -- --nocapture` passes with the lists filled and prints
  probed/flagged/unprobed counts.
- `rust/agents/report-trigger-probe.md`: counts; a table of TO_FIX entries (card, deck, what it decides for the person,
  file:line, suggested fix, size S/M/L); false alarms with reasons; unprobed cards with reasons.

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
- Finish with: everything committed; full details in `rust/agents/report-<task>.md` (committed); a reply of at most 15 lines
  (commit hash, done / not done, test results, anything the coordinator must act on). Then stop.
