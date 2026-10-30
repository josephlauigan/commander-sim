# Brief: Phase 2, PR 5a: clear the competing and shadowed doubles

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/phase2-competing` on
branch `phase2-competing`, based on `phase2-coverage` (PR #163). Run every command from there. Read `CLAUDE.md`
first: memory cap, no Python, loose profile. Another agent works in parallel on `phase2-splits` (the 161 splits,
notes only, no sims); you are the only one running sims: `--jobs 22` for sims, `-j 12` for cargo while the other
agent builds.

## Background

PR #161 removed every hard double (method and report: `rust/agents/phase2-doubles.md`,
`rust/agents/report-phase2-doubles.md`; read both and follow the same method). PR #163 made `cardgen export` refuse
any double not declared with `paths_ok` and froze the ones left in `data/paths_known.txt`, which only shrinks. JD
chose (2026-10-29) to clear the list before Phase 3. Your part is its **21 competing** and **25 shadowed** cards:

- **competing**: a compiled activated ability plus a card-code `options` slot for the same ability (mostly utility
  lands: Ghost Quarter, Tectonic Edge, the Castles, the horizon lands, Karn's Bastion ...). Find out what the sim does
  today: can the AI use both, or does one path never run? Then keep one path: usually the compiled ability, with
  the card's AI (when to use it) moved to or kept in an AI slot only if the generic AI can't use it well. Hand
  `options` that only decide *when* are AI code; the rule (what the ability does) must live in one place.
- **shadowed**: a compiled spell under a card-code `resolve` (the compiled spell never runs). Read the Oracle text:
  if the compiled spell is correct and complete, delete the hand `resolve` (keep AI slots); if the hand resolve is
  right and the compiled one is wrong or partial, decide as PR #161 did for its 18 shadowed spells (where the
  compiled spell is the AI's value of the card, `paths_ok` with the missing part named; otherwise `dsl = []`).

The other agent may hand you, through JD's coordinator, splits that are really doubles; you don't need to wait for
that.

## Work

1. For each of the 46 cards: Oracle text (`data/scryfall_cache.json`), compiled abilities (`data/cards.json`), card
   code (`cardgen audit --paths`, grep `impls/`). Decide, change, and delete its line from `data/paths_known.txt`.
2. Every card you change gets a test that its effect happens exactly once and the AI can still use it (e.g. Ghost
   Quarter destroys the target land and its controller may search for a basic, once; Damn's overload is a wrath).
3. `cardgen export` and `cardgen validate` pass; `audit --paths` regenerated as `audit/phase2/paths-after-competing.md`.
4. Update `documents/realism.md`'s Phase 2 paragraph with the new known-list counts.

## Checks

- `cargo test --release --no-fail-fast` passes; `cargo build --release --all-targets` 0 warnings.
- **Measure** as PR #161 did: `--realism --pool tN --games 4000 --jobs 22 --quiet` before (origin/phase2-coverage's
  binary, built first, in a separate target dir or a `git archive` copy) and after, for every tier whose pool plays a
  changed card; and `--deck KEY --pool all --games 2000 --jobs 22` before and after for each of JD's decks that
  plays a changed card or whose tiers change. One sim at a time, in the background; check rc=0 on every run.
- Before/after per deck share and targets; say whether each move is inside noise. Nothing is recalibrated (JD
  recalibrates once, after Phase 4).

## Rules

- Card data only through `data/card_overrides.toml` and `cardgen export`; no deck list changes.
- Don't seat `omnath` or `godo` in new tests. Format only your files (`rustfmt --edition 2024 <file>`), lines up to 120.
- Commit often, no Co-Authored-By line, never push. Scratch files in
  `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase2-competing/`.
- Keep `audit/phase2/competing-handoff.md`. Report in 15 lines or fewer, then the full report as text in your reply
  (the harness refuses report files from subagents): a table of every card (class, side kept, what was deleted or
  overridden, test), follow-ups for the ability language, the new paths summary, the before/after scorecards and
  JD's decks.
