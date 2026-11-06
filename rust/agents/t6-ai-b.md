# Task: Tier 6 AI follow-ups (JD approved, 2026-11-06)

Worktree: `/home/joseph/projects/commander-sim/.claude/worktrees/t6-ai-b` (branch t6-ai-b, from t6-ai bfe3cc3c). Task name:
`t6-ai-b`. Read `audit/realism/t6-ai-2026-11-06/REPORT.md` first (sections 3, 4 and NEEDS JD): it is the previous agent's
work on this code, and these items are its open points, now decided by JD.

## Items
1. **Lion's Eye Diamond early, only with a use for it (fix).** Section 3: LED has no cast priority, so it is cast only by
   the Underworld Breach line. Give it a cast priority in the plan files of the three Tier 6 decks that play it (Dack,
   Kraum/Tymna, Rograkh/Silas: `data/plans/*.toml`) **only when the hand holds a card LED fuels**: Wheel of Fortune,
   Windfall, Timetwister or Underworld Breach (check each deck's list for which it plays; a set in the plan file). Make
   sure that once LED is on the battlefield the engine offers its crack (`mine::led_options`) in response to / to pay for
   that wheel (LED's mana is added as the cost of sacrificing it and discarding the hand: crack it with the wheel on the
   stack, or to cast the wheel from three mana, as real play does), and never cracks it into an empty plan. Also make
   `t6stats::makes_mana_now` return false for LED when no use is open, so the held statistic stops counting it.
2. **Instant removal held for an opposing combo commander (fix).** Section 4 / NEEDS JD 4: `ai/brain.rs` `hold_value`
   still holds instant removal for Tier 6 when an opposing commander is a combo piece (`cmdr_combo`). For Tier 6 decks
   (`impls::t6_plans::is_t6`) read `combo_near` alone there too, as the rest of the Tier 6 `near` now does. Non-Tier-6
   decks unchanged.
3. **Urza's Saga and Tezzeret as tutors, and Mana Vault in g32 (investigate, then fix what is wrong).** Dack's Urza's
   Saga (chapter III: an artifact with mana cost {0} or {1} from the library) and any Tezzeret in the Tier 6 lists: does
   the planner (`ai/planner.rs`, `tutor_rule` / `xtutor_rule`) see them as tutors for the deck's pieces? If a piece they
   can legally fetch is missing, make the planner use them (check against the Oracle text: Saga fetches only mana cost
   0 or 1). Mana Vault "not castable" with two lands untapped in g32 Rograkh/Thrasios R2 (`commander-sim --calibrate
   within --pool t6 --ai adaptive --games 4000 --seed 900000 --trace 32`): find why and fix it if it is a bug (if it is a
   legitimate lock, e.g. Chalice/Trinisphere/Null Rod, say so).

## Tests
New `rust/crates/sim-core/tests/t6_ai_b.rs`: a scripted test per fix (old code fails, new passes). Run your file,
`t6_ai`, `c1_t6_lines`, `c1b_engines`, `ai_t6_*`, `engine_t6_*`, `cards_t6_*`, `ai_planner`, `ai`, `ai_flow`, `plans`,
`plan_files_3`, `generic_paths`, `thrasios_ai`, and the card test files of JD's decks (`cards_sauron*`,
`cards_sheoldred*`, `cards_cloud*`, `cards_kefka*`, `cards_yshtola`, `cards_alela`, `cards_galadriel`, `cards_thanos`).
Say which of JD's decks each change can affect. Append a section to `audit/realism/t6-ai-2026-11-06/REPORT.md`.

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
