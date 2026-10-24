# Task: stack-overflow-t4 — find and fix a stack overflow in a Sauron game

Read `rust/agents/LASTDECK-RULES.md` first: its hard rules and checks apply, with these changes: base branch
`cloud-midgar` (your worktree's branch `stack-overflow-t4` starts there); the machine is shared with other sessions and
two other agents: every build, test and run under `nice -n 19`, cargo with `-j 4`, sims with `--jobs 4`.

## The bug
This run aborts with `thread '<unknown>' has overflowed its stack / fatal runtime error: stack overflow` (it does on
the commit before the Elrond removal too, so that is not the cause):

```
commander-sim --deck sauron --pool t4 --profile loose --games 500 --ai lookahead --jobs 4 --quiet \
  --swap "Sword of Feast and Famine=>Chaos Warp" --swap "Sword of Hearth and Home=>Spell Pierce"
```

The same tests at t1, t2, t3 and t5 finish. Likely an unbounded recursion (a trigger or replacement loop, a search
rollout that never bottoms out, or a copy/cast chain) in one game of the swapped list.

## Steps
1. Find the game: narrow it with `--games`, `--seed` and `--jobs 1` (see `sim-cli/src/args.rs` for `--seed`,
   `--trace GAME`, and how game seeds are derived), then trace it. A debug build or `RUST_BACKTRACE=1` with a
   bigger stack (`RUST_MIN_STACK`) can help show the repeating frames.
2. Fix the root cause in the engine or card code (not by raising the stack size), with a regression test that builds
   the position (testkit) and fails before the fix.
3. Confirm the run above finishes, and that `--deck sauron --pool t3 --profile loose --games 240 --seed 500000 --ai
   lookahead --jobs 4` (no swaps) gives the same result before and after your fix, unless the bug can occur there too
   (then explain the difference).

## Done when
The LASTDECK-RULES checks pass (use a sauron smoke run instead of elrond/cloud), the run above completes, and the
regression test is in. Report in at most 15 lines (line 1 DONE or HANDOFF, line 2 your model ID, then the cause and
the fix); full report in `rust/agents/report-stack-overflow-t4.md`, committed.
