# Task: remove practice-mode simplifications (part 2)

Worktree: `/home/joseph/projects/commander-sim/.claude/worktrees/practice-simplifications` (branch practice-simplifications). Task name for files: `simplifications`.

## Goal
Six simplifications in how JD's decks play in practice mode (one seat played by a person). Fix each, with a test per fix
(new file `rust/crates/sim-core/tests/practice_simplifications.rs`, using `tests/play_common` and the patterns in
`tests/practice_binder.rs` / `tests/practice_followups.rs`). Do them in this order; commit after each.

1. **Urza, Lord High Artificer's {5} and Triple Triad: the free card waits until end of turn** (person only). Today
   `human::triggers_t1::play_free_now` (shared by `impls::t5::urza_top` and `impls::mine::triad_person`) plays it
   immediately, so a counterspell or an Aura (Kefka's Animate Dead) is lost and an instant can't be held. Oracle: "you may
   play that card [those cards] this turn without paying its mana cost". Make the card playable free until end of turn:
   the engine already holds "playable this turn" exiled cards in the hand marked `Player::impulse` (returned to exile at end
   of turn by `engine/turn.rs` ~line 788, with `stolen` for cards owned by an opponent), and the person's alternative costs
   are labelled in `human/legal.rs` (~line 450) and paid via `engine/altcost.rs`. Add a "free this turn" permission for those
   cards (a land: playable as a land drop). Respect Triple Triad's rule (your card, plus others with lesser mana value).
   Keep the AI paths as they are. Update the doc comments that call this a simplification, and `tests/practice_binder.rs`
   if its expectations change.
2. **Mirage Mirror (Alela)**: `human/gaps_c.rs` (Mirage Mirror section) can't copy lands with code (fetch lands, bounce
   lands, Chasm-style or ability lands) or the person's own legendary lands. Allow them, with the copied land's abilities
   working (e.g. a copied fetch land can be cracked). Oracle: "{2}: Mirage Mirror becomes a copy of target artifact,
   creature, enchantment, or land until end of turn." Person only.
3. **Sylvan Library (Thanos, Thrasios, Ur-Dragon)**: `human/triggers_t1.rs` `sylvan_library` guesses "cards drawn this turn"
   as the hand's last `draw_n` cards. Track the real ones (e.g. a per-player list of cards drawn this turn, filled in the
   draw code, cleared at the start of each turn) and offer exactly those. If the AI's Sylvan code (`impls/topdeck.rs` /
   `impls/common.rs`) uses the same guess, leave it unless it's a one-line switch; say which in the report.
4. **Loop shortcuts (Ur-Dragon)**, `human/urdragon.rs`: (a) when a loop or a batch of tokens triggers another card once per
   token (Terror of the Peaks; also check `impls/t2.rs`, `impls/t45_miirym.rs`), the person is asked per token: ask once
   "same target for every token?" with the option to choose each one. (b) The Garth loop's Black Lotus and Braingeyser rounds
   aren't played out: play them (see the module doc and `impls/t45_urdragon_combos.rs`, `impls/garth.rs`). If (b) turns out
   larger than ~1 hour, do (a), stop, and describe (b)'s design in the report.
5. **Ragavan, Nimble Pilferer (Thanos) — AI-affecting, approved**: its exiled card is held in the hand (`impulse`,
   `stolen`; `impls/t4.rs` ~1084), so effects that count cards in hand see it. Make hand-size counts that mean "cards in your
   hand" use the real hand (`impls/t45_prosper.rs` `hand_proper` does this: move it somewhere shared if needed). Be
   conservative: change counting effects (maximum hand size / discard to hand size, "for each card in your hand",
   "if you have no cards in hand", draw-to-N), not code that moves or casts cards. List every site changed in the report.
6. **Goldspan Dragon (Ur-Dragon) — AI-affecting, approved**: Oracle "Whenever Goldspan Dragon attacks or becomes the target
   of a spell, create a Treasure token." Today only spells' cast-time targets trigger it (`impls/rules.rs` ~706,
   `impls/card_models3.rs` ~468). **Check the Oracle first**: it says "a spell", not abilities. If abilities don't trigger
   it, the remaining real gap is spells whose target is chosen at resolution; fix that if practical, else report it. Don't
   add ability triggers that the Oracle text doesn't have.

## Done when
- Each fix has a test in `practice_simplifications.rs` (6 or more tests) that passes; the existing test files covering the
  code you touched pass (find them with grep: e.g. practice_binder, practice_followups, practice_gaps_c, play_urdragon_human,
  cards tests for Sylvan Library, Ragavan, Goldspan).
- `rust/agents/report-simplifications.md`: per item: done / not done, the design, files changed, AI-affecting changes
  listed (items 5 and 6, plus anything else that changes what the AI does), tests.

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
