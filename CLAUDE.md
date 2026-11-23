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

## Models and routing

This follows JD's global model organization and routing rules (`~/.claude/CLAUDE.md`, `~/.claude/ROUTING.md`; JD,
2026-11-08). The essentials are repeated here so sessions without those files (cloud, other machines) follow them
too. **Routing decides who does the work, never that work may start:** the approval rule above always applies.

**Roles**
- **Sonnet 5.5 (main agent, coordinator)**: `claude-sonnet-5-5`, high effort, set in `.claude/settings.json` (effort in
  `~/.claude/settings.json`). Plans, routes, writes handoffs, implements scoped work, checks every result, talks to JD.
- **Opus 5.5 (advisor, final reviewer)**: an Agent call with `model: "opus"`, `effort: "high"` (xhigh for
  architecture and root-cause), one short consult that ends when it answers. Send it the code, files and evidence,
  not your conclusion.
- **Haiku 5.5 (scout, clerk)**: Agent calls with `model: "haiku"`, `effort: "low"`: searching, grepping,
  `cardgen audit`, file reads, legality and colour-identity lookups, summarizing sim output, one-file mechanical edits
  with an exact spec. No card modelling, rules judgements or deck advice. Returns paths, lines, snippets and counts,
  not conclusions. Sonnet checks every Haiku result and diff (open the cited files and lines, re-run the key search
  or lookup) and never passes it on unchecked.
- **GPT-6.1 Sol (Codex, herdr worker)**: long run → fail → fix loops with a checkable finish condition (tests, builds,
  difftests, engine debugging, bisects), competing attempts on a bug Sonnet failed twice, adversarial tests against
  Sonnet's card and engine code. Effort: medium; high for debugging and reviews; xhigh/max for competing attempts on
  hard bugs.
- **Every other subagent: Sonnet 5.5 at high effort**: pass `model: "sonnet"` and `effort: "high"` on every other Agent
  call, workflow agents included. No subagent default is set: `CLAUDE_CODE_SUBAGENT_MODEL` was removed from both
  settings files (JD, 2026-11-09) because it overrode the `model` passed on Agent calls, so Haiku requests ran as
  Sonnet. The explicit `model` is required.

**Decision procedure** (first match wins): (1) irreversible or high blast radius → Opus approves the plan, JD signs
off; (2) ambiguous requirements, a design trade-off, competing options or agents disagreeing → Opus first, then
re-route the clarified task; (3) read-only "where / what / how many" → Haiku; (4) mechanical, exact spec, one file,
small diff → Haiku; (5) many run/fail/fix iterations with a machine-checkable finish → Sol; (6) everything else →
Sonnet. A task that fits nothing is under-specified: write its finish condition first.

**Consult Opus** for: a plan, before it goes to JD; ambiguous or conflicting requirements; the design of a tricky or
cross-cutting change before building it; a recommendation JD will act on; surprising or conflicting evidence;
accepting any limitation or "out of scope"; Sonnet and Sol disagreeing; a worker failing the same task twice; changes
to rule or config files; declaring a multi-step task done; and the final review of every diff. In this repository
also:
- deck swap, cut and buy proposals, and package adoption (it reads the swap breakdown, not just the `delta` line);
- surprising sim deltas, regressions against the recorded baseline, numbers that contradict `FINDINGS.md`;
- any card left below Full, before it is put to JD;
- modelling a card with tricky Oracle text (replacement effects, layers, stack timing) before building it.
Not for routine edits, status updates, searches or log reading.

**Review chain**: Opus is the final reviewer of every change, not only on disputes. Haiku's work → Sonnet → Opus;
Sonnet's → Sol (adversarial tests) → Opus; Sol's → Sonnet → Opus; high-risk → Opus → JD. Then Sonnet re-runs the
checks itself before merging (next section). No agent is the last check on its own work. Escalation: Haiku fails →
Sonnet; Sonnet fails twice → Sol competing attempt; both fail → Opus root-cause; Opus unsure → JD.

**Handoffs** state GOAL, SCOPE (files allowed to change), DONE WHEN (a command and its expected result), CONSTRAINTS
(minimal diff, no new deps without asking, no destructive git), and end with "Write your results to
/tmp/handoff/<name>.md and reply with only that path." Never give two workers overlapping files.

- **An agent is open no longer than its work needs** (JD, 2026-11-01: "My usage is completely getting eaten up"). A
  Sol worker's loop is its work; it ends when its finish condition is met or it is blocked. **Sims stay with the
  coordinator** (JD, 2026-11-08): no agent, Sol included, runs deck sims or waits on them. An agent writes code or a
  queue script and stops; the coordinator runs the queue as a background script with a watcher on its "all done"
  line, and a new short agent reads the outputs and writes the results. An agent whose work is done or blocked
  reports and ends; resume it only for a short follow-up. Never keep an agent open as a monitor.
- **When Sol is out of usage** (JD, 2026-11-23): the run ends with `SOL EXIT 1` and its log has a "…hit your usage
  limit… try again at <date>" line (the log writes "You’ve" with a typographic apostrophe: search for `hit your usage
  limit`; it can happen at any point in a run). Do not wait for the reset and do not skip the step: give the same brief
  to a fresh Sonnet agent (`model: "sonnet"`, `effort: "high"`), a new instance and never the author of the code under
  test, with the same scope, finish condition and handoff path; a review or adversarial-test pass opens with "You are
  an independent adversarial reviewer, not the author". The Opus final review stays and its brief says which pass was
  Sonnet's. Tell JD in the next report that Sol was out of usage and who did the work. A competing attempt on a bug
  Sonnet failed twice is not handed back to Sonnet: it goes to Opus root-cause (JD, 2026-11-23). Full rule: ROUTING.md §3.

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

## Opponent tiers

Each of JD's decks is simulated only against the opponent tiers listed for it in `data/my_decks.toml` (`tiers = [...]`,
also exported to `data/decks.json`): yshtola t5 (band 1); kyodai t5 (Kyodai artifact combo, a Cali Commander list since 11/13; this key was `thanos` until 11/22 and `--deck thanos` is now refused);
yoshimaru t4-t5 (Yoshimaru and Tymna blink deck, registered 11/21; it replaces thrasios); sheoldred t3-t4 (band 2;
ghave retired 11/02); cloud retired 11/21 (JD; its Equipment package moved into the yoshimaru deck, its list stays only as a
test seat, its card code stays in the engine; veyran retired 11/01); urdragon t5 (new Dragons list 11/13; the old Ur-Dragon
list retired 11/05); thrasios retired 11/21 (the Tymna/Thrasios Doomsday list stays only as a test seat, its card code
stays in the engine); aragorn retired 11/15
(Kefka and Sauron had been merged into it, both keys retired 11/06; its list stays only as a test seat, its card code
stays in the engine); karn retired 11/05; Avery's alela and galadriel no longer tracked 11/06.
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
2. **Colour identity**: the card's `color_identity` must fit the deck's `ident` in `data/decks.json` (Yoshimaru/Tymna is W/B; Sheoldred is mono-B).
3. **The cut is in the current list**: fetch origin/main first and check the card out is in that deck's `cards` in
   `data/decks.json`; FINDINGS files can name cards that have since left.

## Recommend packages, not loose cards

(JD, 2026-11-22.) Decks run their strategy through packages: cards that do one job together (an engine with its
payoffs, enablers and tutors; a combo with its protection; an answer suite; a mana-base change). Recommend at that level.

1. Start from the deck's plan (its guide in `decklists/`, its `data/plans` file, FINDINGS, `decklists/CHANGELOG.md`) and
   name its packages before naming any card.
2. A proposal is a package: its goal; each card in with its job and where it comes from (another deck, spare cards,
   buy); each card out with the job lost and why it was chosen (evidence, proxy or not owned, a weaker sub-plan); the
   `Out=>In` pairs; the change in roles, curve, lands and colour sources, Game Changers and any count or invariant the
   plan relies on.
3. A single swap stands alone only slot-for-slot: same job (role family), mana value at most one higher, land for
   land, and no count the plan relies on changes. Say so for that pair. Any other pairing belongs to a package or plan
   change, proposed as one, with what the deck loses.
4. Choose cuts by job first, then by evidence or ownership: cut proxy or not-owned cards first, an owned card only for a
   true upgrade (JD, 11-06; standing rule since 11-22). Read `data/ownership.toml` when it exists and the deck is
   marked confirmed there; otherwise ask JD which cards are proxies or not owned.
5. Every card gets the swap checks; all below-Full cards go to JD in one list; the package runs as one paired test (all
   pairs together, the deck's own tiers, explain on) in the plan JD approves. Details: `documents/card-knowledge.md`.

## Card recommendations: use the card knowledge base

When JD asks what to swap, cut, buy or adopt, or what a deck should be doing, work from the catalog tools, not from
memory (approach, data and limits: `documents/card-knowledge.md`). This adds to the rules above and overrides none
of them (approval, swap checks, packages, tiers, explain, FINDINGS). Steps 1-4 are read-only investigation: `map` and `digest`
also rewrite their own gitignored view file (even with `--stdout`), and nothing else is written.

1. **Map**: `cardgen map --stdout` (about 3k tokens): what exists, role and colour counts, JD's decks.
2. **Digest**: `cardgen digest --deck KEY --stdout` (about 40k tokens). Read its header first: data date, role table
   (`rules-only` or `merged`), flag legends, and the `d:` letters (per digest). `digest --deck KEY --role R` pages
   through every card with role R; `--view roleless` and `--view new` reach the rest. Absent from the digest does
   not mean it does not exist.
3. **Query**: `cardgen find ...` for what the digest trimmed (`--deck KEY` applies the colour ceiling and leaves out
   the deck's cards; also `--role`, `--text`, `--status`, `--in-deck`, `--not-in-deck`; status, decks and FINDINGS
   hits show only with `--json` or `--full`).
4. **Read before proposing**: `cardgen find --name NAME --full` for every card you intend to propose: Oracle text,
   roles with their evidence line, engine status, which of JD's decks play it, FINDINGS hits.
5. **Check, then test**: do the swap checks above yourself (flags are a pre-filter: no `BAN` is not the legality
   check, `*` and `d:` are not the cut check), then put the `--swap` runs (explain on, the deck's own tiers) in the
   plan JD approves. The `--swap` explain output is the model-status authority, not `m:`.
6. **Say what you used**: the digest or queries, the data date and role table from the digest header, and which
   role tags are unreviewed (`?`).

How the flags mislead if read carelessly:
- `m:N` = NO engine record: status unknown, not fine and not Unmodeled. An incoming card with `m:N` or below Full
  (`m:f`, `m:A`, `m:P`, `m:U`; Full-auto counts as below Full) goes to JD before it is accepted. Giving it a status
  means editing data and running `cardgen export`, which JD approves first; `m:` lags unexported data edits.
- `?` = role tags not reviewed yet (about the tags, not the card); a `rules-only` table has many. No role = no rule
  matched.
- `k:N` counts FINDINGS lines (commander shorthand included): read `find --full` before citing one, and check
  `data/decks.json`, because FINDINGS can name cards that have since left a list.
- `BAN` or `NL` (section A can show them): not Commander-legal; never propose the card; tell JD.
- An error naming `bulk-sync`, `roles` or `export` (no catalog, no or stale role table, engine status unavailable):
  stop and report it. Do not run them in a recommendation: in a worktree they can write the main checkout's data,
  and `bulk-sync` replaces the shared catalog the weekly job maintains (`oracle.json`, `oracle.prev.json`, `sync.lock`).
