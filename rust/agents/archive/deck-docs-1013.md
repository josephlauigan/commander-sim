# Task: deck-docs-1013 — bring one deck doc's prose in line with its new list

You are one of three agents; each edits ONE deck doc (your prompt names it). Your worktree's branch was made from
`sauron-weakness`, where the decklists already have JD's 2026-10-13 swaps (see the newest **Updated 2026-10-13** line at
the top of your doc, and `audit/sauron/2026-10-12-weakness/README.md` for the measurements). Read the README first.

## Hard rules
- **Never run Python** (a hook refuses it); no builds or sims are needed. Use `grep`, `sed`, `jq`.
- Edit only your doc. Don't touch the generated sections (`## Decklist by type`, `## Import list`) or the existing
  **Updated ...** log lines (they are history; leave them word for word).
- Write for JD (the deck's owner, who plays it in person). Match the doc's existing style: plain prose, short
  paragraphs, card names in bold where the doc does that. No "also in X deck" notes (decks never share cards).
- Every card you mention as in the deck must be in the doc's Import list; every cut card must leave the prose (unless
  the sentence says it's gone). Rules text: from `jq '.["card name lowercased"]' data/scryfall_cache.json`; don't
  invent card text. Numbers: only the ones in the README or the Updated lines; don't invent results.

## What to update
- The **Simulator** line at the top: keep its form, update it to the new measurement (Sauron: 32.6% over all tiers in
  the 1,000-per-tier paired confirmation, T1 33.7, T2 39.5, T3 32.1, T4 30.7, T5 27.0; say it's a paired test, a pool
  run is pending. Kefka: +0.5 within noise in the paired test. Veyran: -2.1 in the paired test). Keep the old figure
  out or mark it as the previous list's.
- Strategy, Key lines, Consistency (counts: lands, rocks, interaction, counters, wipes), Tutor targets, Bracket and
  Rule 0 (Game Changers and combos) — rewrite what the swaps made wrong, keep what is still right.
- Grep your doc for each cut card and each new card, and recount any number you change from the Import list.

## Done
Commit (`git add decklists/JD/<doc> && git commit -m "..."`, no Co-Authored-By line),
never push. Final report at most 15 lines: line 1 DONE, line 2 your model ID, commit hash, what sections changed, any
count you could not check.
