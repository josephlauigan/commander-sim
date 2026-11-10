# Handoff: Aragorn Breach rebuild, the C and B1 paired sims (2026-11-10)

For a local agent on JD's test machine. JD approved the C and B1 runs (11/10) and asked for them to run on the test
machine: the cloud session can't, since one background run there is cut off at 2 hours. Read the repository's
`CLAUDE.md` and `AGENTS.md` first. Their rules apply: tiers, loose profile, explain on, and the memory cap.

## Background

- The audit, the four packages (B1-B4) and the simulator changes behind them are in `audit/aragorn/FINDINGS.md`,
  top section "2026-11-10". The code is on branch `claude/wizardly-edison-7dxm55` (draft PR #319), on top of
  origin/main `bfcd176`.
- **C** is the whole rebuild: B1, B2, B3 and B4 swapped in together (29 swaps). **B1** is the core Breach package alone:
  Lion's Eye Diamond, Birgi, Enlightened Tutor, Gamble and Grand Abolisher. The swap lists are in
  `audit/aragorn/2026-11-10-breach-rebuild/run.sh`.
- Both runs are paired swap runs over Aragorn's own tiers (t4, t5: `--pool all`). They use the look-ahead AI and the
  loose profile (the CLI defaults), 1,500 games per list per tier, seed 500000, and the explain breakdown on.
- They run with `--no-preflight`. JD chose this on 11/10: Thassa's Oracle and Lion's Eye Diamond are cast only inside
  the Breach line, so the preflight's "drawn but rarely cast" stop fires on them by design.

## Cost: plan for it

- The CLI's time estimate is wrong for this deck. It prints "about 75 minutes", but the measured cost was about 19
  CPU-seconds a game at look-ahead (the estimate assumes 3). Each run is 6,000 games, about 32 CPU-hours: roughly
  32 / (number of cores) hours of wall time, since the CLI uses every core.
- Sampled stacks showed the time goes to the ordinary look-ahead search, not to the Breach line's dry run.
- Run them where nothing kills a long job: tmux, or a job that survives logout.

## Steps

```
git fetch origin claude/wizardly-edison-7dxm55 && git checkout claude/wizardly-edison-7dxm55   # (or a worktree)
cd rust && systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 cargo build --release -p sim-cli && cd ..
bash audit/aragorn/2026-11-10-breach-rebuild/run.sh C
bash audit/aragorn/2026-11-10-breach-rebuild/run.sh B1
```

- The script applies the memory cap itself: `systemd-run`, or `ulimit -v` where systemd isn't available.
- It writes `C.txt`, `B1.txt`, `status.txt` (one line per run: exit code, minutes, time) and `commit.txt` into
  `audit/aragorn/2026-11-10-breach-rebuild/`.
- Exit 137 means the memory cap was hit: report it as a memory bug, don't retry.
- **Don't run `rest` (B2, B3, B4).** Those need JD's own go-ahead.

## Done when

- `status.txt` shows `C exit=0` and `B1 exit=0`.
- Both outputs end with the delta lines for t4 and t5 and the explain breakdown.

## Then

1. Commit the four output files to the branch and push. Commit only to `claude/wizardly-edison-7dxm55`.
2. Add a dated results section to `audit/aragorn/FINDINGS.md`, above the 11/10 audit section. For each run, include:
   - the command and seed;
   - the win-rate delta per tier with its noise band (label the average "over t4-t5");
   - the swapped-in cards' drawn / cast / average round / stuck-in-hand numbers;
   - the Breach line counters (how often the dry run passed, waited for protection, and was taken) and how games
     were won (Breach line, Aragorn burn, combat) and lost;
   - Aragorn's red damage, commander removals, and mana and hand at turns 4 and 8.
   - **The why.** For each package, say what helped or hurt and which numbers show it, not just the delta.
3. Say whether B1 alone gets most of C's gain (if any). That decides whether the mana and land packages (B2, B3) are
   worth their cost.
4. **Don't change the deck list** (`data/my_decks.toml` or the deck file). JD decides on swaps after reading the
   results.
5. Report to JD in 15 lines or fewer: the numbers, the reasons, anything surprising. Name `audit/aragorn/FINDINGS.md`
   for the details.

## Constraints

- Tiers t4 and t5 only.
- Loose profile only.
- Never `--no-explain`.
- No `--any-tier`.
- No code changes. If the sim panics or a number looks wrong (for example, the Breach line is never taken in C),
  stop and report it with the output, instead of fixing it.
