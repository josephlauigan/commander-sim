# Brief: "At the table" summaries for the pool decks

Practice mode's setup screen draws three opponents from a tier and shows a box when you hover one: a short, accurate
summary of how that deck plays, so JD can choose which of his decks to bring. Your job is the text of that box, for
the pool decks you are given.

## Where
- Worktree: `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/practice-setup` (`cd` there for every
  command). Another agent and the main session work in the same worktree on other files.
- Edit **only** the deck files of your tiers: `decklists/pool/<tier folder>/*.md` (not `retired/`). Nothing else.
- **No git commands that change anything** (no commit, stash, checkout, reset): the main session commits.
- Never run Python. Use `grep`, `sed -n`, `jq`. No builds or sims are needed.

## What to add
In each deck file, add this section directly **after** the `## Role in the pool` section (before `## Strategy`, or
before `## Decklist by type` when there is no Strategy section):

```
## At the table

- **Plan:** what the deck does in the first five or six turns, in one or two sentences.
- **Wins by:** how it actually wins, naming the cards that do it.
- **Watch out for:** the two to four cards or turns that decide games against it, and what to do about them.
- **Interaction:** its removal, counterspells, board wipes and protection, by name (what it can answer of yours).
```

Exactly those four bullets, those bold labels, in that order, each one line (no line breaks inside a bullet), about
90 to 140 words for the four together. Write for a player sitting down at the table: plain English, present tense, no
tuning history, no simulator talk, no win rates.

## Getting it right
- **Every card you name must be in that deck's current list** (the `## Import list (100)` block). Check each name
  with `grep -n "^1 <Card name>" <file>` before you write it. The existing `## Strategy` text can be out of date
  (cards that were cut are still mentioned): never copy a card name from it without checking.
- Read what a card does from `data/scryfall_cache.json`, e.g.
  `jq -r '.["doubling season"] | .oracle_text' data/scryfall_cache.json` (keys are lower case).
- Base the plan on the list: the commander, the ramp, the engines, the finishers, the combos named in the header.
- Don't edit the other sections, even when they are wrong. Instead, list every stale card mention you find in
  `## Strategy` or `## Role in the pool` (a card named there that is not in the Import list) in your report.

## Finish
- Check: each of your files has exactly one `## At the table` section with the four bullets, and every bolded or
  named card in it is in the list.
- Write your report to `rust/agents/report-pool-summaries-<your part>.md`: one line per deck (done), then the stale
  mentions (file: card names). Your final reply is at most 15 lines: decks done, anything unclear, the stale count.
