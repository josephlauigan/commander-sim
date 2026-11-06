# Task: Tier 5 holds: shrink the wasted protection hold, fix the idle turns (JD approved, 2026-11-06)

Worktree: `/home/joseph/projects/commander-sim/.claude/worktrees/t5-hold` (branch t5-hold, main merged in). Task name:
`t5-hold-b`. Read `rust/agents/report-t5-hold.md` (the previous agent's measurement work and its proposals) and the runs
in `audit/realism/t5-hold-2026-11-06/` (`t5-adaptive-2000.txt`, `t5-lookahead-2000.txt`, `t3-adaptive-2000.txt`: the
"Held mana at the end of the turn" sections, pooled and per deck).

## What the measurement showed (Tier 5, own turns 5-10)
- **Counter held** (16% of turns): cast by the holder's next turn 54% (adaptive) / 44% (look-ahead), and 25-28% of the hits
  stop a combo piece. That is realistic: **don't change the counter hold.**
- **Protection held** (2% of turns, mostly Miirym, about 4.8 mana held): used 5%, unused 65-67%. Wasted.
- **Plain idle turns** (15.8%: 2+ mana unused, a castable card in hand, nothing held; Tier 3 4.9%): Zur 19%, Miirym 12%.

## Do
1. **Protection hold (fix).** In `ai/brain.rs` `hold_of` (the outside-deck floor for `!is_main(...)` decks, and the
   look-ahead's `search::hold_bonus` / `hold_credit` that mirror it), give protection its own term: hold mana for a
   protection spell only when it protects something worth it (a commander or combo piece on the battlefield, or the
   deck's engine) and an opponent can plausibly remove it at instant speed (open mana and cards; `prob_holding` of
   removal), not as the counter floor does. Counters keep their current term exactly. Outside decks only: JD's decks and
   Tier 6 keep their own rules (say which decks the change touches).
2. **Idle turns (diagnose, then fix the mistakes).** Trace Tier 5 games (`commander-sim --calibrate within --pool t5 --ai
   adaptive --games 2000 --seed 900000 --trace N`, single games only) and find, per idle turn, why the castable card was
   not cast: prio 0 from a plan rule, a value below "stop", a main-2 stop that keeps mana for nothing, a card that is
   castable but useless now (a removal spell with no target, a counter with nothing to counter: those are fine), a
   sorcery-speed tax (Zur's own enchantments), etc. Count the causes over enough traced games (at least 40 idle turns),
   and fix those that are mistakes compared with real play (prefer a general rule over a per-deck tweak; plan-file rows
   where a plan holds a card wrongly). Leave legitimate ones and say why. Check Tier 3's 4.9% idle turns don't rise.
3. Append a section with the causes (game and round numbers), the fixes and the expected effect to
   `rust/agents/report-t5-hold.md`.

## Tests
New `rust/crates/sim-core/tests/t5_hold_b.rs`: a scripted test per fix (old code fails, new passes), and a control that a
held counter is unchanged. Run your file, `t5_hold`, `c3_casual_speed`, `ai`, `ai_flow`, `ai_eval`, `ai_search*` if
present, `plans`, `generic_paths`, and JD's decks' card test files (`cards_yshtola`, `cards_thanos`, `cards_sauron*`,
`cards_sheoldred*`, `cards_kefka*`, `cards_cloud*`, `cards_alela`, `cards_galadriel`).

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
