# Brief: pool rebalance: Tergrid (t3)

You work in `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/tergrid-balance` on branch
`tergrid-balance` (origin/main: rounds 1-2 of the pool rebalance are merged, #218). Read the common rules first:
`/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/common-rules.md` (no Python in
any form; cargo `-j 6`; another agent shares the machine). Then `rust/agents/pool-balance.md` (targets, bracket
rules, Full-only additions, CHANGELOG, export) and the round 2 method (diagnose first): `rust/agents/pool-balance-2.md`.

## Why (JD approved 2026-11-02)

Tergrid, God of Fright (mono-black disruption, t3, high B3) reads 35.9-37.5% within t3 (1500-2000 games; HIGH above
35%) and 41-42% into t2. The AI fixes (PR #221) did not move her (36.6 -> 37.5), so the cause is her list or how her
cards are modelled. Note Tergrid's own Bottomless Pit (random discard each upkeep, which she then steals) took
Korvold's Goblin Bombardment and Birds in trace t3-6. Aim for 20-30% within t3, keeping her archetype (discard and
sacrifice disruption that she steals from), axis, role in the pool and bracket. Lord Windgrace's round 3 (PR #220,
not merged yet) changes t2, so judge her t3-into-t2 number loosely.

## What to do

Diagnose first (up to 20 traced games, `--trace N --pool t3`, `--jobs 4`; up to 4 `--calibrate within --pool t3
--games 300` checks): what decides her games (which discard/sacrifice engines, how often she steals, what she steals,
whether a card or interaction is modelled too strongly: report modelling or AI causes with evidence, don't fix code).
Then swaps sized to the gap, with all of round 1's rules (legal, in colours, **Full** additions only, cuts in the
current list, Game Changers within high B3, CHANGELOG entry "Tergrid, 2026-11-03" with reasons, `cardgen export` with
no leftover diff, the Spellbook guard and full `cargo test --release` passing). Commit; don't push.

## The queue (write it; don't run it)

`/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/tergrid-balance/queue.sh`: like
`scratchpad/pool-balance-2/queue.sh` (resumable, 24G cap, `--jobs 22`, `queue.log` with `ALL DONE`; takes the branch
binary; builds nothing). Outputs to `audit/pool-balance/tergrid/`: `--calibrate within --pool t3 --games 1500 --seed
500000`, `--calibrate ordering --pool t3` and `--pool t4`.

Report: the diagnosis, the swaps and why, test counts, the queue path.
