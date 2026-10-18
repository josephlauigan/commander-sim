# Rules for Claude Code sessions in this repository

## Models

- The main agent runs on **Opus 5.5** (`claude-opus-5-5`), set in `.claude/settings.json`.
- **Every subagent runs on Sonnet 5.5** (`claude-sonnet-5-5`), set by `CLAUDE_CODE_SUBAGENT_MODEL` in the same file.
  Don't override it: leave the Agent tool's `model` unset (or pass `"sonnet"`).

## Checking subagents' work

A subagent's report is a claim, not a result. Before treating delegated work as done, or reporting it to the user:

1. **Read the diff** the agent made (`git diff` / `git show` in its worktree or branch), not only its summary.
2. **Re-run the checks yourself**: `cargo test --release` in `rust/` (under the memory cap below), plus any test,
   difftest or sim command the task names. Don't rely on the agent's "all tests pass".
3. **Spot-check the claims**: for card code, compare the behaviour with the card's Oracle text; for numbers, open
   the output files the agent cites.
4. **Merge only after the checks pass.** If something is wrong, send it back with the failing output (or fix it),
   and tell the user what was found.

Agents work from brief files (`rust/agents/RULES.md`, `rust/agents/<task>.md`), keep a handoff file, and report in
15 lines or fewer with details in `rust/agents/report-<task>.md`.

## Running heavy commands

Run every sim, Rust example and test binary under a memory cap, so a runaway can't take VS Code down with it:

```
systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 <command>
```

Exit code 137 means the cap was hit: treat it as a memory bug, not a flake.

## No Python

Never run Python here, not even a version check or a one-liner: use `jq`, shell tools, `cardgen`, `commander-sim` or a
Rust test or example. A PreToolUse hook (`.claude/hooks/no-python.sh`, wired in `.claude/settings.json`) refuses any
shell command that starts with `python`, `python3`, `pip`, `pipx` or `uv`, in every session and subagent.

## Opponent tiers

Each of JD's decks is simulated only against the opponent tiers listed for it in `data/my_decks.toml` (`tiers = [...]`,
also exported to `data/decks.json`): yshtola, sephfs and kraum t5 (band 1);
sheoldred, sauron and seph t3-t4 (band 2); kefka t3 (band 3); cloud, veyran and titania t1-t3 (band 4);
alela and galadriel t1 (Avery's). `--pool all`, `--all-decks`, `--analyze`, swap runs and
`searchtest` use only those tiers; `--deck X --pool tN` outside them is refused. Report averages over the deck's own
tiers (label them, e.g. "over t1-t3"). Use `--any-tier` only when JD asks for it. `--calibrate` is unchanged.

## Swap tests explain themselves

`--swap` (and every package test built on it) runs with the paired breakdown by default (`--no-explain` turns it
off): before the run, the model status of each swapped-in card (Full ... Unmodeled, WARNING on Partial/Unmodeled);
after it, the swapped cards' drawn / cast / average round / stuck-in-hand numbers for both lists, and the game
shape (win, survival, how it wins and dies, plan timing, commander removals, mana and hand at turns 4 and 8). It
costs no extra games and the `delta` line is unchanged. After every run, update `audit/<deck>/FINDINGS.md`, the
deck's living list of strengths and weaknesses and what was tested (with why it helped or not).
