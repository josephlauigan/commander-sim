# Repository instructions for coding agents

## Scope

These instructions apply to the entire repository. The current implementation is the Rust workspace in `rust/`.
Read `README.md` for the product and command overview, then use the task-specific documents linked below instead of
guessing at simulator behavior.

`CLAUDE.md` contains Claude-specific harness settings as well as older copies of several repository rules. Do not
carry its model names or Claude-tool instructions into Codex. Use the Codex model policy below. Keep the durable
project rules in this file and the detailed domain documentation aligned when either changes.

## Always use the current rules (origin/main)

At the start of a session, run `git fetch -q origin main`, then read `git show origin/main:AGENTS.md` and
`git show origin/main:CLAUDE.md`. Follow the current remote repository rules where the local copies differ;
Claude-specific model names and tool instructions remain scoped to Claude, and Codex uses the model policy below.
This applies in worktrees and sessions started from a parent directory too; run the commands from the repository
root (or use `git -C <repo>`). If fetching is unavailable, state that the remote rules could not be refreshed and
use the available copies.

Two rules must never be missed: keep swap and package explanations enabled for deck decisions, and simulate each
owned deck only against its matched tiers in `data/my_decks.toml`. Details appear in the simulation rules below.

## Codex model policy

The main chat's model is selected by the Codex session and cannot be changed by this file. When choosing a model for
a new session or for delegated work that the user explicitly requests, route work by role:

| Work | Model | Reasoning | Use for |
| --- | --- | --- | --- |
| Lead agent and complex work | `gpt-6.1-sol` | `high` or `xhigh` | Coordination, architecture, difficult debugging, final reviews, cross-crate changes, and conflicting simulation evidence |
| Normal implementation or subagent work | `gpt-6.1-terra` | `high` | Feature work, focused bug fixes, card implementations, tests, and most delegated coding tasks |
| Visual or UI workflow | `gpt-6.1-terra` | `high` | Practice-mode UI, `ios-shell`, screenshots, computer use, and tightly scoped visual fixes |
| Mechanical or high-volume work | `gpt-6-luna` | `medium` | Repository searches, repetitive data edits, triage, audits, and output summarization |

- Use `gpt-6.1-sol` as the highest-level agent for coordination, complex decisions, and final review, with high
  reasoning by default and xhigh reasoning for especially difficult work. Do not select `gpt-6-astra` unless the user
  explicitly requests it.
- Use `gpt-6.1-terra` with high reasoning as the default implementation subagent, including visual and UI work.
  Escalate ambiguous architecture, subtle engine correctness, and conflicting evidence to the Sol lead.
- Use `gpt-6-luna` only for bounded tasks with clear acceptance criteria; do not use it to judge subtle simulator
  correctness or ambiguous Magic rules.
- Do not select the older `gpt-6-sol` for new work when `gpt-6.1-sol` is available, unless the user asks to preserve
  an existing model configuration.
- Do not delegate work or spawn subagents unless the user explicitly asks for delegation or parallel agent work.
- Keep every delegated assignment narrowly scoped and end the subagent when its assigned work is done or blocked.

## Non-negotiable rules

- Do not run Python, Python package managers, or Python one-liners in this repository. Do not introduce new Python
  tooling. Use Rust programs, Rust tests/examples, `jq`, or shell tools instead.
- Run builds, tests, Rust examples, and simulations through the repository's memory-capped commands when possible.
  The root `Makefile` applies a 24 GiB `systemd-run` cap where systemd is available and runs directly on macOS. Exit
  code 137 is a memory failure, not a flaky test.
- Preserve unrelated user changes and untracked files. Never clean or rewrite broad parts of the tree to make a
  focused change.
- Work from the repository root unless a command below explicitly uses `rust/` as its working directory.
- Prefer deterministic, seed-controlled comparisons. Do not present simulation output as real-world win-rate
  prediction.

## Repository map

- `rust/crates/sim-core/`: game engine, rules, AI, card implementations, and shared test support.
- `rust/crates/sim-cli/`: the `commander-sim` simulation and analysis CLI.
- `rust/crates/practice/`: browser/terminal practice mode and its HTTP server.
- `rust/crates/cardgen/`: card/deck data generation, validation, audits, and deck editing.
- `data/`: hand-edited card/deck sources, plans, realism targets, knobs, caches, and generated JSON.
- `decklists/`: JD's decks, Avery's decks, opponent pools, and changelogs.
- `documents/`: current architecture and workflow documentation. `documents/archive/` is historical.
- `audit/`: simulation evidence and each deck's living findings.
- `ios-shell/`: native iPad shell around the Rust practice server.

Read these references for task-specific work:

- Engine and architecture: `documents/how-it-works.md`
- Cards and generated data: `documents/cards.md` and `rust/crates/cardgen/README.md`
- Deck comparisons: `documents/testing-decks.md`
- Realism work: `documents/realism.md`
- Practice mode: `documents/practice-mode.md`
- Rust workspace: `rust/README.md`

## Build, format, and test

The workspace uses Rust edition 2024 and the toolchain pinned by `rust/rust-toolchain.toml`. `rust/Cargo.lock` is
committed and must remain authoritative.

From the repository root:

```sh
make build
make test
eval "$(make -s env)"
```

For a focused check, run Cargo from `rust/` and use release mode, for example:

```sh
cargo test --release -p sim-core
cargo test --release -p practice
cargo test --release -p sim-cli realism
```

- Use targeted tests while iterating, then run `make test` for changes that affect shared engine behavior, generated
  data, cross-crate APIs, or release readiness.
- CI builds all release targets and runs the full release test suite.
- Format only Rust files changed by the task: `rustfmt --edition 2024 path/to/file.rs`. Do not format the entire
  workspace. The configured maximum width is 120.
- Add or update focused tests for behavior changes. Card implementations should be tested against the card's Oracle
  text, not merely against existing simulator behavior.

## Data and generated artifacts

- `data/cards.json` and `data/decks.json` are generated by `cardgen export`. Edit their sources, not the generated
  JSON directly, then regenerate and include the resulting JSON changes.
- Card sources include `data/carddb.txt`, `data/card_overrides.toml`, `data/audit.toml`, the Scryfall cache, Rust card
  implementations, and decklists. See `documents/cards.md` before changing the card pipeline.
- The test `export_matches_committed_files` detects stale generated JSON.
- Prefer `cardgen deck <key> <list> --dry-run` before applying a full deck list. When a deck change is accepted, use
  `--log "Why."` so the appropriate changelog is updated, then run `cardgen export`.
- Do not hand-edit audit output or generated evidence to manufacture a passing result. Record the command, seed,
  game count, AI, tiers, and raw-output location for substantive simulation claims.

## Simulation and deck-analysis rules

- Use the loose interaction profile, which is the CLI default. Do not use conservative or multi-profile comparisons
  unless the user explicitly asks.
- Simulate each owned deck only against the tiers currently listed for it in `data/my_decks.toml` (also exported to
  `data/decks.json`). Use `--pool all` for all of that deck's tiers. Use `--any-tier` only when the user explicitly
  requests an off-tier experiment.
- Label aggregate results with the tiers included. Report one result per tier when that distinction matters.
- Use adaptive AI for a quick directional check and the default look-ahead AI for final claims unless the task says
  otherwise. Always state which AI produced a reported result.
- Keep the paired explain breakdown enabled for `--swap` and package tests. Do not pass `--no-explain` for a deck
  decision. This includes incoming cards' model status and the paired drawn / cast / stuck-in-hand and game-shape
  breakdown. Keep preflight enabled unless diagnosing the preflight itself.
- After every swap or package run, and every other substantive deck run, update `audit/<deck>/FINDINGS.md` with the
  date, exact command, result and noise band, raw-output location, strengths and weaknesses, and what was tried and
  why it helped or not. Tell JD the why along with the number.
- Interpret paired differences against their reported noise band. Treat an effect inside the band as no clear
  effect.

Before proposing, modeling, testing, or recommending a card swap, verify and report:

1. The incoming card is Commander-legal.
2. Its color identity fits the deck.
3. The outgoing card is in the current deck list.
4. The incoming card's model status and any known approximation or missing behavior.

Use `data/scryfall_cache.json` for legality and color identity when available, `data/decks.json` for current exported
lists, and `cardgen audit` or `commander-sim --deck <key> --cards` for model status.

## Implementation expectations

- Fix root causes and follow existing module boundaries and naming patterns. Avoid broad refactors during focused
  card, deck, or simulator fixes.
- Keep game behavior deterministic for a fixed seed. Do not introduce iteration-order or shared-RNG changes without
  tests that make the intended effect explicit.
- Preserve overflow checks and bounded resource use. A change that sharply increases memory or runtime needs
  investigation before acceptance.
- When adding card behavior, use the simplest correct layer: data/DSL first, Rust hooks only when the data model
  cannot express the Oracle behavior, and deck-specific AI rules only when a generic rule is insufficient.
- Update current documentation when commands, data ownership, user-visible behavior, or workflow changes. Do not
  use archived port-era documents as current authority.

## Completion checklist

Before reporting work complete:

1. Review the full diff and confirm unrelated files were not changed.
2. Format only changed Rust files.
3. Run the narrowest relevant tests, plus the full release suite when the change has shared or cross-crate impact.
4. Regenerate and verify committed JSON when card or deck sources changed.
5. Record simulation evidence and update the deck's findings when the task includes a substantive deck run.
6. State exactly which checks were run and any checks not run.

If work is delegated, treat the delegate's report as a claim: inspect its diff, rerun the relevant checks, and
spot-check card behavior or numerical output before accepting or reporting it.
