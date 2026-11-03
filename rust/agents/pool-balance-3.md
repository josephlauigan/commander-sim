# Brief: pool rebalance round 3: Lord Windgrace (t2)

You work in `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/pool-balance-2` on branch
`pool-balance-2` (branch `pool-balance` = PR #218, rounds 1-2, origin/main merged). Run every command from there. Read
the common rules first: `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/common-rules.md`
(no Python in any form; cargo `-j 6`; another agent shares the machine). Then `rust/agents/pool-balance.md` and
`rust/agents/pool-balance-2.md` (the targets, bracket rules, Full-only additions, CHANGELOG, export, and how round 2
diagnosed before swapping).

## Why (JD approved 2026-11-02)

After round 2, Lord Windgrace (Jund lands, t2) reads 38.5% within t2 (1500 games, `audit/pool-balance/round2/within-t2.txt`;
HIGH above 35%), likely the share Sythis gave up (41.4 -> 21.2%). Aim for 20-30%, keeping its archetype, axis, role
in the pool and the bracket (mid B3). Tergrid (t3, 35.9%) is **not** in this task: an AI fix that changes how Korvold
feeds her runs at the same time, and she is judged after it.

## What to do

Same method as round 2: diagnose first (up to 20 traced games, `--trace N --pool t2`, `--jobs 4`; up to 4
`--calibrate within --pool t2 --games 300` checks), say what decides Windgrace's games (an engine? a card? an AI or
modelling cause, reported, not fixed), then swaps sized to the gap: legal, in colours, **Full** additions only, cuts in
the current list, CHANGELOG entry (round 3, 2026-11-02, reasons), `cardgen export` with no leftover diff, the
Spellbook guard and the full `cargo test --release` passing. Commit; don't push.

## The queue (write it; don't run it)

`/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/pool-balance-2/queue.sh`:
like round 2's `queue2.sh` (resumable, 24G cap, `--jobs 22`, `queue.log` with `ALL DONE`; takes the branch binary;
builds nothing). Outputs to `audit/pool-balance/round3/`: `--calibrate within --pool t2 --games 1500 --seed 500000`
and `--calibrate ordering --pool t2` and `--pool t3` (Windgrace into t1; t3 decks into t2).

Report: the diagnosis, the swaps and why, test counts, the queue path.
