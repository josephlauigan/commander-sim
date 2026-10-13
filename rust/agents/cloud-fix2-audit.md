# Task: cloud-fix2-audit — why the second round of Cloud AI fixes didn't raise the win rate

Read `rust/agents/LASTDECK-RULES.md` for the hard rules (no Python, memory cap, worktree, commit, handoff, short
report), with these changes:
- Your branch `cloud-fix2-audit` starts from `cloud-midgar`. Never edit the Cloud deck list.
- **No batch sims** (JD stopped them): no `--games`, `--analyze`, `all_decks`. Single-game traces and unit/probe tests
  only. Build and test under `nice -n 19`, `-j 3` at most (the machine is shared: 7 threads per session).
- **Diagnose only**: no AI or engine changes. Probe tests are fine (commit them; `#[ignore]` any that fail on purpose).
- Don't add a Co-Authored-By line to commit messages.

## The result to explain
The C4 Cloud list played by the AI before and after branch `cloud-ai-fix2` (7 fixes, one commit each:
0c5d390 engine-aware tutor, f404955 hold stranded Equipment, 3ec1dea protection + Plaza of Heroes, a8f6184
PROTECT_FIRST (protection before Cloud into open removal; recasts wait), 1be633d equip move margin, 63197bc Cloud
mulligan, 0a63090 wipe/edict logging), 1,000 games per tier, same seeds:

| Tier | Before | After |
|---|---|---|
| t1 | 13.0% | 13.1% |
| t2 | 11.1% | 12.8% |
| t3 | 8.8% | 8.4% |

So the fixes are roughly break-even. Read `rust/agents/report-cloud-audit-c4.md` (what was wrong) and
`report-cloud-ai-fix2.md` (what was changed) first.

## How to compare
Two ready binaries play the same seeds:
- **Before:** run `/tmp/claude-1000/-home-joseph-projects-mtg-sims/bdfb9e18-5da6-4d07-ab18-c71ffb5f5d3f/scratchpad/csim-v6`
  with the current directory `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/cloud-c4-snap`.
- **After:** run `.../scratchpad/csim-v8` with the current directory
  `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/cloud-ai2-snap`.
Don't modify those two worktrees. `--deck cloud --pool tN --profile loose --ai lookahead --trace K` gives the same
game seed in both, so you can diff the two traces and find the first decision where they diverge.

## Questions
1. For at least 30 seed pairs across t1-t5, find where the games diverge and which fix caused it. Tally, per fix:
   how often it fired, and in how many of those games the outcome got better / worse / stayed the same.
2. **PROTECT_FIRST**: how often does it delay Cloud, by how many turns, and what does the delay cost (Cloud's first
   equipped attack, damage dealt)? Does the "recast waits for protection" rule leave Cloud in the command zone for long?
3. **Tutor**: how often do free-equip engines change the pick, and does the big piece then get cast and attached
   soon? Any cases where it fetches something it can't use yet?
4. **Held equipment**: does holding a stranded piece ever leave mana unused or lose a turn?
5. **Mulligan**: how many hands change, and do the new keeps play better or worse?
6. **Plaza / Formation / Rootborn / Giver**: how often do they fire, and do they save Cloud?
7. Anything the fixes made worse that the earlier audit didn't see.

## Deliverable
`rust/agents/report-cloud-fix2-audit.md`: the per-fix tally table, the evidence (tier, trace K, round), and a ranked
recommendation: which fixes to keep, which to revert or change and how. Commit it. Final reply: at most 15 lines
(line 1 DONE or HANDOFF, line 2 your model ID, then the tally summary and recommendations).
