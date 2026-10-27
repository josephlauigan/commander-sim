# Handoff: AI foundations step 1, the decision bugs (brief `ai-fix-bugs.md`)

Branch `ai-foundations`, worktree `.claude/worktrees/ai-foundations`. Report: `audit/ai-foundations/step1-fixes.md`;
raw output: `audit/ai-foundations/raw/step1/`.

## Done

- The "before" binary: built at 2445c5eb (the decisions log), kept outside the repo; its goldfish run is
  `raw/step1/before/goldfish.txt` (identical to the audit's `raw/goldfish-lookahead.txt`).

## Next

- One commit per bug, in the brief's order: G, C, D, E, B, I. Each: build, its tests, the goldfish, a table row.
- After the last: `cargo test --release --no-fail-fast`, `cargo build --release --all-targets` (no warnings), then
  realism t0-t6 and all-decks for the before and after binaries (detached units), and the report.

## Gotchas

- Most shared files are not rustfmt-clean at HEAD (turn.rs, ai/mod.rs, combos.rs, brain.rs, pool.rs ...): format only
  the new test file with `rustfmt --edition 2024`; keep the edits in shared files to 120 columns by hand.
- One sim or cargo command at a time: the realism runs block builds, so the "before" realism runs were split around
  the per-commit cycle (same saved binary, so the order changes nothing).
