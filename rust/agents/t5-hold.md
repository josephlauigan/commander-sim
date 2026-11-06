# Task: measure Tier 5 counter-mana holding (JD approved, 2026-11-06; measure only, no behaviour change)

Worktree: `/home/joseph/projects/commander-sim/.claude/worktrees/t5-hold` (branch t5-hold, from c3-casual-speed 4feaac25,
PR #274). Task name: `t5-hold`. Read `audit/realism/c3-2026-11-06/REPORT.md` first ("Tier 5 mana": 35-47% of Tier 5
turns in rounds 5-10 end with 2+ mana unused and a castable card, against 10% at Tier 3, mostly Zur and the blue decks
keeping counter mana up via `ai/brain.rs` `hold_of` / `hold_value`).

## The question JD wants answered
When a Tier 5 deck ends its turn holding mana for an answer, how often does that held answer actually get cast before
the deck's next turn, and what does it hit? If most holds are wasted, the hold should shrink; if most are used, it stays.

## Do
1. Add **trace-only** counters (no decision reads them, they draw no random numbers, so every game result is unchanged:
   prove it with a test that a seeded game's result and log are identical with the counters on), per deck and pooled
   per tier, printed in the `--realism` output (find where C3 printed "Tier 5 mana" and add beside it):
   - turns ending with mana held for an answer (`hold_of` returned a card), by what was held (counter / protection /
     removal / Ad Nauseam / other) and how much mana was left unused;
   - of those, how often the held card (or any instant of the same kind) was cast before the holder's next turn, and
     on what: an opponent's combo piece or win attempt, a commander, a creature/engine, a removal spell, or nothing
     useful; and how often the held mana was spent on something else instead (an instant draw, an ability);
   - the same for "2+ mana unused with a castable card in hand and nothing held" (the turns that are plain idle).
2. Write a queue script `rust/agents/t5-hold-queue.sh` that the coordinator runs (don't run it yourself): it builds
   release and runs, under the memory cap, `commander-sim --realism --pool t5 --ai adaptive --games 2000 --seed 900000
   --jobs 23 > audit/realism/t5-hold-2026-11-06/t5-adaptive-2000.txt` and the same for `--pool t3` (the comparison
   tier) into `t3-adaptive-2000.txt`, then prints `ALL DONE`.
3. In your report, propose (don't implement) what the hold rule could become depending on the possible outcomes
   (e.g. hold only against a deck that is close to winning, cap the held mana to the cheapest answer, etc.), with the
   code points it would change.

## Tests
New `rust/crates/sim-core/tests/t5_hold.rs` (the identical-results test plus one counter test). Run it, `c3_casual_speed`,
`ai`, `ai_flow`, `ai_eval`, and JD's decks' card test files (`cards_yshtola`, `cards_thanos`, `cards_sauron*`,
`cards_sheoldred*`).

## Ground rules (these override the older rust/agents/RULES.md, which is from the Python-port era)
- **JD's rule: no shortcoming is accepted quietly.** Anything you can't make right (a card below Full, a case you skip, a
  caveat): commit what you have, list it at the top of your reply under `NEEDS JD`, and carry on.
- Work only in your worktree; `cd` into it for every command. Don't merge other branches. Don't push.
  Toolchain: `export PATH=$HOME/.cargo/bin:$PATH`; build from `rust/`.
- **Never run Python, not even a version check** (a hook refuses it). Use jq, shell tools, cardgen or Rust.
- **Never use `git stash`** in any form (the stash is shared with other sessions). Use a WIP commit.
- **Memory cap on every cargo command and every binary you run, no exceptions**:
  `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 <cmd>` (zsh: write the prefix out in full).
  Exit 137 = cap hit = a bug.
- No sims (no `--realism`, `--pool`, `--calibrate` or multi-game runs) and no full `cargo test` suite: the coordinator
  runs those. Single `--trace N` games are fine. Leave nothing running when you stop.
- Loose profile only. Other agents work in other worktrees at the same time: never touch files outside yours.
- Never let an unsigned counter go below zero. AI and person code stay separate (`if crate::human::is_human(g, p)`).
- `rustfmt --edition 2024 <the files you changed>` only; never `cargo fmt` on the workspace.
- Commit often inside the worktree (`git commit`, **no Co-Authored-By line**). Keep `rust/agents/handoff-<task>.md`
  current. If your context gets long (~80+ tool calls), stop at a clean point, everything committed, reply's first line
  `HANDOFF`.
- Write the full report to `rust/agents/report-<task>.md` and commit it (if the harness refuses the write, put the full
  report in your reply after a `---- FULL REPORT ----` line). Reply in at most 15 lines.
- `cargo build --release --all-targets` with no warnings before you finish; run the test files the brief names with
  `cargo test --release --no-fail-fast --test <name> ...`.
