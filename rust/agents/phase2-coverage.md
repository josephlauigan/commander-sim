# Brief: Phase 2, PR 3: export refuses doubles; model coverage printed per deck

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/phase2-coverage` on
branch `phase2-coverage` (from origin/main 2bdfead0, which has PRs #153 and #161). Run every command from there. Read
`CLAUDE.md` first: memory cap, no Python, loose profile. Another agent works in parallel on `phase2-spellbook`; use
`-j 12` for cargo and `--jobs 12` for any sim.

## Background

Phase 2 of `documents/realism.md` ("One card model") has the gate **"No double-modeled cards; coverage printed per
deck"**. PR #153 added `cardgen audit --paths` (per card: its DSL abilities, executing tags, registry rule and AI
slots, and a double class: hard / competing / shadowed / split / dsl-covers ...). PR #161 fixed every hard double
(0 left) and marked the deliberate splits with `paths_ok = "why"` in `data/card_overrides.toml`. Reports:
`audit/phase2/paths-after-doubles.md`, `rust/agents/report-phase2-doubles.md`.

## Work

1. **Guard: `cardgen export` refuses a new double.** After it builds the cards, it runs the paths classification
   and fails (non-zero exit, nothing written) if any card is a hard or competing double, or an undeclared split,
   that has no `paths_ok`. The message names each card, its class and its two paths, and says how to fix it (delete
   one path, a `dsl` override, or `paths_ok = "why"`). Decide whether shadowed cards also fail: they are allowed
   today only through `paths_ok`, so check that each of the current ones has one; if all do, refuse new ones too.
   Note: the registry slots live in sim-core, so check how `audit --paths` gets them and reuse that, not a copy.
   Also add a test (sim-core or cardgen, whichever owns the logic) that runs the same check on the real data, so
   `cargo test` fails on a double even if nobody re-exports. Current data must pass with no new `paths_ok`.
2. **Coverage per deck, printed on every run.** Every `commander-sim` run that plays decks (`--deck`, `--pool`,
   `--all-decks`, `--realism`, `--goldfish`, `--calibrate`) prints, for each deck it plays, one line of model
   coverage of its 99 + commander: how many cards are Full / Partial / Unmodeled (use the same status that
   `explain.rs`'s swap model status and `cardgen audit` use; don't invent a second scale), plus the names of the
   Unmodeled and Partial cards that matter most (cap the list, e.g. the first 5, and say how many more). For
   `--realism` and `--calibrate`, one block for the tier's decks. Keep it short; it goes before the results.
   `--quiet` keeps it (it is part of the result), or print it into the result file — check what `--quiet` does and
   say what you chose. Add a flag only if needed to turn it off (`--no-coverage`), defaulting to on.
   Make the coverage cheap: computed once per deck per run, not per game.
3. **Docs:** update `documents/realism.md`'s Phase 2 status (export refuses doubles; coverage printed) and the
   CLI's `--help` text. Leave the Spellbook part of Phase 2 to the other PR.

## Checks

- `cargo test --release -j 12` in `rust/` under the memory cap: all pass. `cargo build --release --all-targets`: 0
  warnings. `cardgen export --offline` gives no change to `data/`; `cardgen validate` passes.
- Prove the guard: make a temporary double (e.g. a scratch override that re-adds a cost static to a card with a
  cost hook), show `cardgen export` refuse it with the message, then remove it. Put the output in the report.
- Prove coverage doesn't change results: run `--realism --pool t3 --games 1000 --jobs 12` on origin/main and on
  your branch; the scorecard must be byte-identical apart from the coverage lines. Show the coverage block.
- Show the coverage line for each of JD's decks (`--deck cloud --pool t1 --games 50` etc., one per deck key in
  `data/my_decks.toml`, each only on one of its own tiers).

## Rules

- Commit on your branch as you go; do not push; no `Co-Authored-By` trailer.
- Keep a handoff file `audit/phase2/coverage-handoff.md`.
- Report in 15 lines or fewer, with the full report as text at the end of your reply (the harness may refuse your
  write of `rust/agents/report-phase2-coverage.md`; if it does, put the text in your reply).
- No recalibration and no deck changes.
