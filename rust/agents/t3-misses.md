# Brief: Tier 3's combo and alternate-win lines

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/t3-misses` on branch `t3-misses`, which is based on main. Run every command from
there. Read `CLAUDE.md` first: memory cap, no Python, loose profile. Do not use `rust/agents/RULES.md`.

## Goal

In the Tier 3 scorecard, 8.6-8.9% of wins come by combo or alternate win; the target is 16.5%. The median win round
is 10 against 8, and poison wins are 19% against 9%. Background is in `audit/realism/t3-trace-2026-10-24.md` and
`rust/agents/handoff-realism.md` (Next, item 3). Every Tier 3 card is modelled. The question is whether each deck's
real lines are in the combo catalogue and whether the AI goes for them. The same audit on Tiers 0-2 found catalogued
lines that never fired: an over-strict ready check, and pieces sacrificed as fodder.

For each Tier 3 deck (Atraxa, Aurelia, Korvold, Marwyn, Tergrid; lists in `decklists/pool/t3-*/`):

1. **List the lines.** From the import list, list every non-combat win: combos of any size, alternate-win cards,
   mill, and big drains. Name the cards each needs and check them against Oracle text (`data/scryfall_cache.json`).
2. **Check each against the sim.**
   - Is it in the combo catalogue (`impls/combos.rs` and the files chained into `all()`)?
   - Does the AI go for it? Trace games with `commander-sim --calibrate within --pool t3 --ai lookahead --jobs 10
     --trace N` (N = 0..199). Look for the "goes for" and "wins by combo" lines, and count how often the pieces are
     drawn, assembled, attempted and stopped.
3. **Fix** what's missing or broken, most frequent first:
   - Add missing lines in a new file `impls/t3_lines.rs` with a `COMBOS` list chained into `combos::all()`, with
     tests.
   - Fix ready checks, tutor wishes or priorities that are wrong, with tests.
   - Don't change any deck list. Tier 3 allows two-card combos only if the list has them already.
4. **Poison.** Look at where the 19% comes from: Atraxa, and Tergrid winning with stolen infect creatures. Report
   whether it's a rules bug. Fix it only if it is.
5. **Measure**, paired, on the same seeds: `commander-sim --realism --pool t3 --games 4000 --jobs 10 --quiet`
   before and after, plus each deck's wins by type in 200 traces.

## Rules

- **Commands.** Run every build, test and sim from `rust/`, under
  `systemd-run --user --scope -q -p MemoryMax=12G -p MemorySwapMax=0 <command>`. Another agent shares the machine.
- **Thread limits.** Pass `--jobs 10` to every `commander-sim` run and `-j 10` to cargo. Run one command at a time,
  in the background.
- **No Python**, anywhere in a command line.
- **Deck seats.** Don't seat the decks `omnath` or `godo` in new tests: they're being retired.
- **Shared files** (`engine/*`, `ai/*`, `state.rs`, `hooks.rs`, `combos.rs`, `common.rs`): make the smallest
  additive change, mark it `// --- t3 lines`, and list each one in your report.
- **Formatting.** Format only your files (`rustfmt --edition 2024 <file>`), never `cargo fmt`. Lines go up to 120
  characters.
- **Commits.** Commit often, with no Co-Authored-By line, and never push.
- **Before you finish:** `cargo test --release --no-fail-fast` passes, and `cargo build --release --all-targets` has
  no warnings.
- **Scratch files.** Put them in `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/t3-misses/` only.
- **Report** in your final reply: a summary first, then each deck's lines with their status before and after, the
  files you changed, and the before and after numbers.
- **Waiting.** Don't end your turn while your own background command is still running: wait for it.
