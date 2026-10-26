# Brief: the lookahead AI should see Tier 6's stepped (quiet) combo lines

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/t6-quiet-progress` on branch `t6-quiet-progress`, which is based on
`t6-rograkh-thrasios` (PR #145). Run every command from there. Read `CLAUDE.md` first: memory cap, no Python,
loose profile. Do not use `rust/agents/RULES.md`.

## Goal

The Tier 6 gate is seat 1's share of decisive games. Real is 31.8%; the sim is about 4 points short
(`audit/realism/t6-seat-topdeck-2026-10-24.md`). The lever is decks reaching ready lines earlier. The previous agent
found that `combos::deck_combos` drops every quiet line (the lines a T6 plan runs in steps). So `combo_progress`,
which the lookahead AI uses to judge how close a combo is, ignores all stepped Tier 6 lines. Its own fix only touched
the stats (`t6_measure` now uses `lines_of`). See `audit/realism/t6-rograkh-thrasios-2026-10-26.md`, "Leads left alone".

1. **Read** how `deck_combos`, `lines_of`, `real_groups`, the `QUIET` marker and `combo_progress` fit together
   (`rust/crates/sim-core/src/impls/combos.rs`, and the lookahead's use of `combo_progress` in `ai/`). Write down
   every caller of `deck_combos` and what each one needs. Some callers may rely on quiet lines being left out, for
   example so the generic combo executor doesn't run a line that a T6 plan runs itself.
2. **Fix** `combo_progress` and the lookahead's evaluation so they count quiet lines' real groups, without making the
   generic executor go for quiet lines. Add a test that a T6 deck holding most of a quiet line scores progress.
3. **Measure, paired, on the same seeds:**
   - `commander-sim --realism --pool t6 --games 4000 --jobs 10 --quiet` before and after;
   - `--calibrate within --pool t6 --ai lookahead --jobs 10` before and after, for the per-deck ready and go rounds.
   
   Report seat 1-4, each deck's share, the spread, the median win round and the ready rounds. If one deck's share
   jumps past 32%, say so and find out why; don't hide it.
4. **Optional, only if step 3 is clean:** tag the untagged tutors (Crop Rotation, Gamble, Expedition Map, Sylvan
   Scrying, Spellseeker) so one-tutor-away sees them, if that is a tag question and not new card code. Measure
   separately.

## Rules

- **Commands.** Run every build, test and sim from `rust/`, under
  `systemd-run --user --scope -q -p MemoryMax=12G -p MemorySwapMax=0 <command>`. Another agent shares the machine.
- **Thread limits.** Pass `--jobs 10` to every `commander-sim` run and `-j 10` to cargo. Run one command at a time,
  in the background.
- **No Python**, anywhere in a command line.
- **Deck seats.** Don't seat the decks `omnath` or `godo` in new tests: they're being retired.
- **Shared files** (`engine/*`, `ai/*`, `state.rs`, `hooks.rs`, `combos.rs`): make the smallest change, mark it
  `// --- t6 quiet progress`, and list each one in your report.
- **Formatting.** Format only your files (`rustfmt --edition 2024 <file>`), never `cargo fmt`. Lines go up to 120
  characters.
- **Commits.** Commit often, with no Co-Authored-By line, and never push.
- **Before you finish:** `cargo test --release --no-fail-fast` passes, and `cargo build --release --all-targets` has
  no warnings.
- **Scratch files.** Put them in `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/t6-quiet/` only.
- **Report** in your final reply: a summary first, then what you changed (files), the before and after numbers, and
  the risks.
- **Waiting.** Don't end your turn while your own background command is still running: wait for it.
