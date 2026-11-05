# Task: opponent-effects probe (practice mode)

Worktree: `/home/joseph/projects/commander-sim/.claude/worktrees/practice-opponent-probe` (branch practice-opponent-probe). Task name for files: `opponent-probe`.

## Goal
Practice mode lets a person play one seat. The coordinator wants to find every place where an **AI opponent's card makes a
choice that belongs to the person**: e.g. Portal to Phyrexia picks the person's three sacrificed creatures for them; an
edict, "each opponent discards", "unless that player pays {X}" (Rhystic Study, Mystic Remora, Esper Sentinel, Smothering
Tithe), "of their choice", annihilator, Liliana of the Veil's -2, Grave Pact, Torment of Hailfire, Sheoldred, Whispering
One's upkeep, The Eldest Reborn's chapters, Living Death etc. The shared helpers `engine::zones::edict` and
`engine::cast::discard_worst` already route to the person; the gaps are in individual cards' code.

**This round you write the probe and classify what it finds. You don't fix the gaps** (the coordinator decides after seeing
the list). Exception: if a fix is a one-line routing to an existing person helper, still don't do it; list it.

## What to build
New test file `rust/crates/sim-core/tests/practice_opponent_probe.rs`, modelled on
`rust/crates/sim-core/tests/practice_choice_probe.rs` (read it whole: card list from data files, Oracle filtering,
`catch_unwind` per card, `TO_FIX` / `FALSE_ALARMS` lists with a `check` that fails on new gaps and on stale entries) and
on `tests/play_common` (ScriptHuman seats).

1. **Cards**: every card in the opponent pool decks JD's decks face: `data/decks.json` `.pool[]` whose `.tier` is in the union
   of `.mine[].tiers` (31 decks, 1,422 unique cards). Keep the cards whose Oracle text can make *another player* choose or
   lose something of their choice (sacrifice, discard, exile/return "of their choice", "chooses", "unless ... pays",
   annihilator, "each player" effects, "target player/opponent" + a choice). Be inclusive; record how many were probed.
2. **Table**: the person is P0 (`g.humans.add(P0, ScriptHuman)`) and holds at least two of everything an effect could touch:
   creatures (different powers), artifacts, enchantments, a planeswalker, lands, 3+ cards in hand, cards in graveyard, mana
   available (so "unless pays" can be paid). The AI opponent P1 owns the card. The person answers every question with
   the first choice (or yes) and every question to P0 is recorded.
3. **Drive each card** the way it would hit the person: an instant/sorcery cast by P1 (`engine::cast::cast_card` with
   targets from `engine::cast::spell_targets`, P0 the target player where it targets one); a permanent entering under P1
   (its enter trigger); and for permanents whose text triggers on the person (P0 casts a spell: Rhystic Study, Mystic Remora,
   Esper Sentinel; P1's upkeep / end step; P1 attacks P0 with it (annihilator, Sword of Feast and Famine); a P1 creature dies
   (Grave Pact); a planeswalker's loyalty abilities, e.g. via `impls::common::walker` machinery), drive that moment with the
   engine's own step/hook functions (`engine/turn.rs`, `engine/combat.rs`, `engine/hooks.rs`). Do what's practical; record
   cards you couldn't drive in an "unprobed" list with why, rather than spending long on one card.
4. **Detect**: snapshot P0's state before/after (battlefield permanents, hand, graveyard, exile, life, mana pool/tapped
   lands). Flag a card when something of P0's was sacrificed / discarded / exiled / returned / chosen or a tax was paid or
   refused, **with two or more candidates, and P0 was asked nothing**. Also flag the twice-asked pattern (same question twice
   in a row) like the choice probe.
5. **Lists**: put every flagged card in `TO_FIX` (real gap: the engine chose for the person) or `FALSE_ALARMS` (with why) so
   the test passes. For each TO_FIX entry, find the code that makes the choice (file:line) and say which existing person
   helper would fix it, in the report.

## Done when
- The probe runs (`cargo test --release -q -p sim-core --test practice_opponent_probe -- --nocapture`) and passes with the lists
  filled; it prints probed/flagged/unprobed counts.
- `rust/agents/report-opponent-probe.md`: counts; a table of TO_FIX entries (card, deck(s) it's in, what it decides for the
  person, file:line, suggested fix, size S/M/L); false alarms with reasons; unprobed cards with reasons.

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
