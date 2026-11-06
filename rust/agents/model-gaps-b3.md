# Task: four follow-ups to model-gaps-b2 (JD's decisions, 2026-11-05)

Worktree: `/home/joseph/projects/commander-sim/.claude/worktrees/model-gaps-b2` (branch model-gaps-b2, at 9d4b5c3f).
Task name: `model-gaps-b3`. Read `rust/agents/report-model-gaps-b2.md` first (its NEEDS JD items 2, 4, 7, 8 are this task)
and the doc comments of `ability_run`, `ability_run_w`, `trigger_window`, `with_rings_auto` in
`rust/crates/sim-core/src/engine/stack.rs`.

**JD's rule: no gap may be left quietly.** Do not leave anything below the rules, add a "left out" note, or a caveat on
your own judgement. Anything you truly can't do: stop that part, commit, and put it at the top of your reply under
`NEEDS JD` (exact gap, effect in games, what it would take). JD decides.

## Items (JD chose each one)
1. **Soul-Guide Lantern's enter effect is a trigger** ("When this artifact enters, exile target card from a graveyard."):
   today the person path (`human/karn.rs` ~1175) puts it on the stack with `ability_window_card`, an activated-ability
   window. Make it a real triggered ability (`trigger_window` or however the engine's other enter triggers do it for a
   person, with a `Trigger` the engine can run again) so Peter Parker's Camera can copy it (the copy picks a new target),
   and so Rings does not offer a copy (rule: Rings copies activated abilities only). Check the AI path for the same trigger
   and keep the AI's behaviour identical (if it uses a different path, leave it; list any AI change in the report).
2. **Staff of Domination's {1} untap is covered by the once-per-run Rings answer**: `human/abilities.rs` ~292 runs the
   untap under `with_rings_auto(Some(false))`, so it is never offered a copy. JD keeps one Rings answer for the whole run
   but wants it to cover the untap too: the run's answer (`ans`) applies to both the untap and the draw activations. Keep
   the run's other questions identical. Test with Rings: "yes" copies both the untap and the draw; "no" copies neither.
3. **A person can activate Clue, Food and Blood tokens in practice mode** (today only the AI can: `Action::Clue` in
   `ai/act.rs`, `ai/brain.rs`; there is no person menu entry). Oracle text:
   - Clue: "{2}, Sacrifice this artifact: Draw a card."
   - Food: "{2}, {T}, Sacrifice this artifact: You gain 3 life."
   - Blood: "{1}, {T}, Discard a card, Sacrifice this artifact: Draw a card."
   Find how the engine stores these tokens (perms with a name/subtype, or counters on the player like `treasures`) and add
   a person ability for each through the person's ability menu (`human/abilities.rs` / `human/cards.rs`), paying the cost
   from the pool, sacrificing as a cost (Sacrifice triggers fire), using `ability_run` (so Rings and Camera can copy it).
   Blood's discard is the person's choice of card. Reasons when it can't be used (no mana, tapped, no card to discard).
   The AI's handling stays identical.
4. **Lathliss, Dragon Queen's pump** ("{1}{R}: Dragons you control get +1/+0 until end of turn."): `impls/t45_urdragon.rs`
   `lathliss_go` / its option computes the number of activations k from all mana but each one needs {R}; offered to a
   person it can be refused ("that can't be done right now"). Make k (and any person path) count what can actually be
   paid ({1}{R} each: k <= red sources, k <= total / 2). This changes the AI's offers (it only removes ones it couldn't
   pay): say so in the report.

## Tests
New file `rust/crates/sim-core/tests/model_gaps_b3.rs`: Lantern's trigger (person picks the target; a Camera copy picks a
new one; no Rings offer); Staff with Rings yes/no; each token ability for a person (and a refusal reason each), with a
Rings copy of one; Lathliss with not enough red mana (k right, no refusal) for the AI option and a person.
Run: your file, `practice_rings_probe`, `play_rings`, `play_abilities`, `play_cards`, `play_karn_*`, every `practice_*`
file, `model_gaps_*`, the test files of the code you touched (`cards_t45_urdragon`, `cards_karn*`, the sheoldred and
alela card tests if tokens are theirs, `ai`, `ai_flow`).

## Done when
- Items 1-4 done (or under `NEEDS JD`); tests pass; `cargo build --release --all-targets` has no warnings.
- Report: the harness may refuse an agent's write of a report file. Return the full report as text in your reply, after
  the 15-line summary, under a line `---- FULL REPORT ----` (per item: what changed, rules and AI, files, tests; every
  shared-file change). The coordinator saves it to `rust/agents/report-model-gaps-b3.md`.

## Ground rules (these override the older rust/agents/RULES.md, which is from the Python-port era)
- Work only in your worktree (path above); `cd` into it for every command. Don't merge other branches. Don't push.
  Toolchain: `export PATH=$HOME/.cargo/bin:$PATH`; build from `rust/`.
- **Never run Python, not even a version check** (a hook refuses it). Use jq, shell tools, or Rust.
- **Memory cap on every cargo command, no exceptions**: `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 cargo ...`
  (run from `rust/`). The shell is zsh: write the prefix out in full, don't store it in a variable. Exit 137 = cap hit = a bug.
- **Don't run the full suite** (`cargo test --release` with no filter) and don't run sims: the coordinator does that. Run
  only the test files named above (`cargo test --release -q -p sim-core --test <name>`), in the foreground with a long
  timeout; don't start background jobs and don't leave anything running when you finish.
- Practice mode: the person's seat is `crate::human::is_human(g, p)`; questions go through `human::choices::{choose,
  choose_one, pick_cards, pick_any, yes_no, search, ...}`. AI and person code stay separate (`if is_human`).
- Never let an unsigned counter go below zero (release builds have overflow checks).
- `rustfmt --edition 2024 <your new files>` only; never `cargo fmt` on the workspace (the repo isn't fmt-clean).
- Oracle text: `jq '.["<lowercased name>"].oracle_text' data/scryfall_cache.json`.
- Commit often (`git add -A rust/ data/ && git commit -q -m "..."`); **no Co-Authored-By line**. Keep
  `rust/agents/handoff-model-gaps-b3.md` current. If your context gets long (~60+ tool calls), stop at a clean point with
  everything committed and end your reply with a first line `HANDOFF`.
- Finish with everything committed and a reply: at most 15 lines (commit hash, done / not done per item, test results,
  anything the coordinator must act on), then the full report as above. Then stop.
