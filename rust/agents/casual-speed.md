# Brief: why casual games run long (Tiers 0-2)

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/casual-speed` on branch `casual-speed`, which is based on
`casual-combos` (PR #144, stacked on #142). Run every command from there. Read `CLAUDE.md` first: memory cap, no
Python, loose profile. Do not use `rust/agents/RULES.md`.

## Goal

Playgroup's median win round is 9 in Tiers 0 and 2 and 8 in Tier 1. The sim's is 12 in Tier 0, 9 in Tier 1 and 12 in
Tier 2, and 11% (Tier 0) and 17% (Tier 2) of games run past round 15. The current scorecards are in
`/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/cc-rescore/t0.txt`, `t1.txt` and `t2.txt`. Tier 3 has the same shape (`audit/realism/t3-trace-2026-10-24.md`:
"attacks are small, attackers are often held back, and big wipes are common"). Find out **why the games are slow**,
fix what is a bug or a clearly wrong AI choice, and report the rest.

1. **Trace.** Run 100 games each of `commander-sim --calibrate within --pool t0 --ai lookahead --jobs 16 --trace N`,
   and the same for t2. For every round, tally:
   - total damage dealt to players, and what dealt it;
   - attacks declared against attackers held back, and why they were held (the log's reasons);
   - wipes cast, and life gained;
   - lands and mana available by turn;
   - the turn each deck's commander is first cast.
   
   Compare the shape with what a real casual pod looks like: commanders out by turn 3-5, steady attacks, a win by
   round 8-10.
2. **Find the causes.** Candidates:
   - the AI holds attackers back too much (the block and crack-back estimate, the threat model);
   - it attacks the wrong player or spreads its damage;
   - it doesn't cast its commander, or casts it late;
   - it mulligans or misses land drops more than it should;
   - there are too many wipes or life gain;
   - a rules bug: combat damage, evasion, trample, or tokens not attacking.
   
   Check each suspected bug against the rules and Oracle text.
3. **Fix** each bug or clearly wrong choice with a test. Keep fixes narrow. A fix to the attack AI in shared code must
   not change JD's decks' play (`ai::is_main`) unless it's a plain rules bug: say which.
4. **Measure, paired, on the same seeds:** `commander-sim --realism --pool tN --games 4000 --jobs 16 --quiet` for
   t0, t1 and t2, before and after. Report the median win round, games over 15 rounds, seats and deck shares. Also
   run `--realism --pool t3 --games 4000 --jobs 16` after, because the same AI plays Tier 3.

## Rules

- **Commands.** Run every build, test and sim from `rust/`, under
  `systemd-run --user --scope -q -p MemoryMax=16G -p MemorySwapMax=0 <command>`.
- **Thread limits.** Pass `--jobs 16` to every `commander-sim` run and `-j 16` to cargo. Run one command at a
  time, in the background.
- **No Python**, anywhere in a command line.
- **Deck seats.** Don't seat the decks `omnath` or `godo` in new tests. Don't change any deck list.
- **Shared files** (`engine/*`, `ai/*`, `state.rs`, `hooks.rs`, `combos.rs`): make the smallest change, mark it
  `// --- casual speed`, and list each one in your report.
- **Formatting.** Format only your files (`rustfmt --edition 2024 <file>`), never `cargo fmt`. Lines go up to 120
  characters.
- **Commits.** Commit often, with no Co-Authored-By line, and never push.
- **Before you finish:** `cargo test --release --no-fail-fast` passes, and `cargo build --release --all-targets` has
  no warnings.
- **Scratch files.** Put them in `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/casual-speed/` only.
- **Report** in your final reply: a summary first, then the causes found (with numbers from the traces), what you
  fixed (files), what you didn't fix and why, before and after numbers, and the risks.
- **Waiting.** Don't end your turn while your own background command is still running: wait for it.
