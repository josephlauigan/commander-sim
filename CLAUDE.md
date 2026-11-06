# Rules for Claude Code sessions in this repository

## Always use the current rules (origin/main)

At the start of a session, run `git fetch -q origin main`, then read `git show origin/main:CLAUDE.md` and
`git show origin/main:AGENTS.md`. Follow the current remote rules where the local copies differ, keeping
Claude-specific model and tool instructions scoped to Claude. This applies in worktrees and sessions started
from a parent directory too; run the commands from the repository root (or use `git -C <repo>`).
If fetching is unavailable, state that the remote rules could not be refreshed and use the available copies.

Three rules must never be missed: ask JD for approval before starting work, keep swap and package explanations
enabled for deck decisions, and simulate each owned deck only against its matched tiers in `data/my_decks.toml`.
Details follow below.

## Ask for approval before starting work

Before doing any work, present the plan and **explicitly ask JD to approve it**, then wait for a clear go-ahead
(for example "approved", "go ahead", "start"). The plan says what will change, which branches or worktrees, which
agents and which sims. The work that waits includes editing or creating files, creating branches or worktrees,
committing, pushing, opening PRs, spawning agents and running builds, tests or sims. Answering the plan's open
questions, or approving one step, is not approval to start the rest. If the plan changes, ask again. Read-only
investigation needs no approval: reading files, `git log`/`diff`/`fetch`, `grep`, `cardgen audit`. (JD, 2026-11-04.)

## Models

- The main agent runs on **Opus 5.5** (`claude-opus-5-5`), set in `.claude/settings.json`.
- **Every subagent runs on Sonnet 5.5** (`claude-sonnet-5-5`), set by `CLAUDE_CODE_SUBAGENT_MODEL` in the same file.
  Don't override it: leave the Agent tool's `model` unset (or pass `"sonnet"`).
- **Every subagent runs at high effort** (JD, 2026-11-01): pass `effort: "high"` on every Agent call, workflow
  agents included. No setting does this for subagents alone (`CLAUDE_CODE_EFFORT_LEVEL` would also change the main
  session), so it goes on each call.
- **Pass `model: "sonnet"` on every Agent call too** (JD, 2026-11-01): `CLAUDE_CODE_SUBAGENT_MODEL` loads only in
  sessions started inside `commander-sim/`; a session started in a parent folder otherwise gives its agents the main
  model (Opus).
- **An agent is open no longer than its work needs** (JD, 2026-11-01: "My usage is completely getting eaten up"). No
  agent waits on sims or long builds: an agent writes code or a queue script and stops; the coordinator runs the
  queue as a background script with a watcher on its "all done" line, and a new short agent reads the outputs and
  writes the results. An agent whose work is done or blocked reports and ends; resume it only for a short follow-up.
  Never keep an agent open as a monitor.

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
also exported to `data/decks.json`): yshtola t5 (band 1); thanos t5 (new artifacts and Dragons list 11/05; no longer Scion's t6 seat);
sheoldred and sauron t3-t4 (band 2; ghave retired 11/02); kefka t3 (band 3); cloud t1-t3 (band 4;
veyran retired 11/01); thrasios t6 (cEDH; new Doomsday list 11/05); karn and urdragon retired 11/05; Avery's alela and
galadriel no longer tracked 11/06.
`--pool all`, `--all-decks`, `--analyze`,
swap runs and `searchtest` use only those tiers; `--deck X --pool tN` outside them is refused. Report averages over
the deck's own tiers (label them, e.g. "over t1-t3"). Use `--any-tier` only when JD asks for it. `--calibrate` is unchanged.

## Loose profile only

Sims run the **loose** interaction profile only (JD, 2026-10-19): loose opponents counter and remove more freely, the
more interactive table and the worst case for JD's decks. It is the CLI default (`--profiles` defaults to `loose`, and
`swaptest` uses it too); don't pass `--profiles conservative,loose` or `--profile conservative` unless JD asks for it.
Report one number per tier.

## Swap tests explain themselves

`--swap` (and every package test built on it) runs with the paired breakdown by default (`--no-explain` turns it
off): before the run, the model status of each swapped-in card (Full ... Unmodeled, WARNING on Partial/Unmodeled);
after it, the swapped cards' drawn / cast / average round / stuck-in-hand numbers for both lists, and the game
shape (win, survival, how it wins and dies, plan timing, commander removals, mana and hand at turns 4 and 8). It
costs no extra games and the `delta` line is unchanged. Never pass `--no-explain` for a deck decision. After every
run, update `audit/<deck>/FINDINGS.md`, the deck's living list of strengths and weaknesses and what was tested
(with why it helped or not), and tell JD the why along with the number.

## Check every swap before proposing it

Before you propose a card for one of JD's decks (or test, model or buy-advise it), check all three, and say so:

1. **Commander legality**: `legalities.commander` must be `legal` (Scryfall: `data/scryfall_cache.json`, or
   `https://api.scryfall.com/cards/named?exact=<name>`). Banned or not legal: don't propose it (Primeval Titan,
   10/18).
2. **Colour identity**: the card's `color_identity` must fit the deck's `ident` in `data/decks.json` (Cloud is mono-W).
3. **The cut is in the current list**: fetch origin/main first and check the card out is in that deck's `cards` in
   `data/decks.json`; FINDINGS files can name cards that have since left.
