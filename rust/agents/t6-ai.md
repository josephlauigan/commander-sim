# Task: Tier 6 AI fixes (JD approved, 2026-11-06)

Worktree: `/home/joseph/projects/commander-sim/.claude/worktrees/t6-ai` (branch t6-ai, from c1b-engines bf3f1899, PR #276,
which sits on c1-t6-lines PR #271). Task name: `t6-ai`. Read `audit/realism/c1-2026-11-06/REPORT.md` ("Looked at, not
changed") and `audit/realism/c1b-2026-11-06/REPORT.md` ("What is left") first.

## Items
1. **Finale for Witness (fix).** Rograkh/Thrasios (`data/plans/rograkh-thrasios-creature-combo.toml`) holds Finale of
   Devastation + Snap with 8-11 mana when Eternal Witness is the missing combo piece: the planner (`ai/planner.rs`)
   builds no Finale-for-Witness plan (seed 900000, `commander-sim --calibrate within --pool t6 --ai adaptive --games 4000
   --seed 900000 --trace 139`, rounds 6-7). Find why (Finale's X tutor = a creature with mana value X or less from library
   or graveyard onto the battlefield; check how the planner sees it as a tutor) and fix it so the planner fetches the
   missing creature piece with Finale when the mana is there. Check Green Sun's Zenith / Chord of Calling / other
   X-creature tutors the same way.
2. **Whir of Invention (fix).** Whir (an artifact with mana value X or less from library onto the battlefield; improvise)
   is not a planner tutor: Kinnan (`data/plans/kinnan-bonder-mana-combo.toml`) can't fetch its second Basalt Monolith
   with it. Make it a planner tutor for artifact pieces (and check its improvise payment is counted). Check the other
   artifact tutors in the Tier 6 lists the same way (Transmute Artifact, Fabricate, Urza's Saga, Reshape, ...).
3. **Lion's Eye Diamond and Moxen held on turns 1-3 (investigate only, no fix).** Count and trace why Tier 6 decks end
   turns 1-3 with LED / Mox Diamond / Chrome Mox / Mox Opal / Mox Amber etc. in hand (single `--trace N` games and the
   flow tables in `audit/realism/c1b-2026-11-06/t6-adaptive-8200.txt` if they show it). Real play: Moxen are cast turn 1
   almost always; LED is held for a line (Breach, Wheel, Doomsday) but often cast early as a land-sac for mana. Write the
   causes with game numbers and a proposed fix in the report under `NEEDS JD`; don't change code for this item.
4. **"An opponent is close to winning" for every Tier 6 cast (fix).** `ai/brain.rs` ~line 167: `cmdr_combo` is true
   whenever any opposing commander on the battlefield is a modelled combo piece, and it feeds `near` (reserve penalty 8.0,
   +2.5 on the hold value). C1b switched only the engine rule to the archenemy reading (`combo_near`). For Tier 6 decks
   (`impls::t6_plans::is_t6`), make every use of `near` read `combo_near` alone (the archenemy's real progress); leave
   non-Tier-6 decks as they are. JD's `thrasios` is a Tier 6 key: say how it is affected.
5. **Ad Nauseam hold for Sheoldred (fix).** `ai/brain.rs` ~line 227: mana is held for Ad Nauseam only with room for four
   cards (`plans::adnaus_room(g, p) >= 4`), but only for Tier 6. Apply it to JD's `sheoldred` too. List any other non-Tier-6
   deck that plays Ad Nauseam (jq on `data/decks.json`) under `NEEDS JD` without changing it.

## Tests
New `rust/crates/sim-core/tests/t6_ai.rs`: one scripted test per fix (1, 2, 4, 5) where the old AI failed and the new one
does it. Run your file, `c1_t6_lines`, `c1b_engines`, `ai_t6_*`, `engine_t6_*`, `cards_t6_*`, `ai_planner`, `ai`,
`ai_flow`, `ai_eval`, `plans`, `plan_files_3`, `generic_paths`, `thrasios_ai`, and JD's decks' card test files
(`cards_sauron*`, `cards_sheoldred*`, `cards_cloud*`, `cards_kefka*`, `cards_yshtola`, `cards_alela`, `cards_galadriel`,
`cards_thanos`). Say which of JD's decks each change can affect.
Write `audit/realism/t6-ai-2026-11-06/REPORT.md` (causes with trace game numbers, fixes, item 3's findings).

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
