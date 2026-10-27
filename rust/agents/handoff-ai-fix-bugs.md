# Handoff: AI foundations step 1, the decision bugs (brief `ai-fix-bugs.md`)

Branch `ai-foundations`, worktree `.claude/worktrees/ai-foundations`. Report: `audit/ai-foundations/step1-fixes.md`;
raw output: `audit/ai-foundations/raw/step1/`.

## Done (2026-10-27)

- Six commits, one per bug, in the brief's order: G c2efec2e, C e5a84417, D ef54a6f4, E 3bae4ef6, B 1188ebd2,
  I 81be6eba. Each was measured in the goldfish (`raw/step1/<letter>/goldfish.txt`).
- Tests: `rust/crates/sim-core/tests/ai_fix_bugs.rs` (16). `cargo test --release --no-fail-fast` passed after every
  commit (3,533 after I), with no build warnings.
- Realism t0-t6 and all-decks, before (2445c5eb) and after (81be6eba). Then commit-by-commit attribution of the
  flagged JD cells (`raw/step1/attribution/`), and fresh-seed checks (`raw/step1/seed600000/`, plus the
  `*-seed600000.txt` files in attribution).
- Result: the Oracle and Breach decks' goldfish medians are about a turn faster, and their no-kill games fell
  sharply. Scion t6 rose 25.9% → 29.7%. The other moves are seed noise, checked on fresh seeds.

## Next

- Step 2, the line planner (REPORT.md, architecture-review.md section 8), then step 3, the Tier 6 recalibration.
- Follow-ups in step1-fixes.md: rituals go first in the cleanup (`card_worth` is 0 for them); `discard_worst`
  still uses its old rule.

## Gotchas

- Most shared files are not rustfmt-clean at HEAD (turn.rs, ai/mod.rs, combos.rs, brain.rs, pool.rs ...). Format
  only the new test file with `rustfmt --edition 2024`, and keep edits in shared files to 120 columns by hand.
- One sim or cargo command at a time. The "before" realism runs were split around the per-commit cycle; they use the
  same saved binary, so the order changes nothing.
- Same-seed before/after comparisons diverge early: G and E change every look-ahead playout. Check a move on a fresh
  seed range before believing it. JD's single 1,000-game cells swing by ±3 points.
- zsh doesn't word-split `set -- $var`: put sim loops over "deck tier" pairs in a bash script.
- Don't wait on a process with `pgrep -f '<pattern>'` (or kill one with `pkill -f`) when the pattern is in your own
  command line: it matches itself.
