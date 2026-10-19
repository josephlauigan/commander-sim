# Brief: pool deck files describe the current list only

JD's rule for deck docs: a deck file describes the deck **as it is now**; history lives in a changelog. The pool
deck files (`decklists/pool/<tier folder>/*.md`) still have text about cards that were cut and paragraphs of tuning
history. Clean them up.

## Where
- Worktree: `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/practice-setup` (`cd` there for every
  command). Another agent and the main session work in the same worktree on other files.
- Edit **only** the deck files of your tiers (not `retired/`), and write the history you move out to your own file
  `rust/agents/pool-changelog-<your part>.md` (the main session merges both parts into `decklists/pool/CHANGELOG.md`).
- **No git commands that change anything** (no commit, stash, checkout, reset). Never run Python; use `grep`,
  `sed -n`, `jq`. No builds or sims.

## The rules for each file
1. **Never touch** `## Decklist by type` or `## Import list (100)` (the simulator reads them).
2. **Every other part** (the header lines under the title, `## Role in the pool`, `## Strategy`,
   `## Sim modeling notes`, `## At the table`) describes the current list only:
   - **No card that is not in the Import list**, not even as "no longer plays X" or "no X". Check each name with
     `grep -n "^1 <Card name>" <file>`. Rewrite the sentence around the cards the deck really has, or drop it.
   - Read what a card does from `data/scryfall_cache.json` (`jq -r '.["card name"] | .oracle_text'`, lower case).
   - **Header counts must match the list:** `**Game Changers (N):**` lists exactly the cards in the list whose
     cache entry has `"game_changer": true`
     (`jq -r 'to_entries[] | select(.value.game_changer==true) | .value.name' data/scryfall_cache.json`), and
     `**Lands:**` matches the land count in `## Decklist by type`. `**Combos:**` names only combos whose pieces
     are all in the list.
   - Keep the tier/bracket line, the axis, and the meaning of each section; keep the existing style (plain English,
     present tense). Keep `## At the table` as it is unless it breaks a rule here.
3. **History moves out, word for word.** Tuning paragraphs ("Tuned in September 2026 ... made way for ..."),
   `**Updated <date>.**` lines, before/after win rates and "was X, now Y" notes leave the deck file. Put them in your
   changelog file, one section per deck, in this format (newest first; use the date the text gives, e.g.
   `2026-09` when only the month is known):

   ```
   ## <the deck file's title line, without "# ">

   Deck key `<file name without .md>`, `decklists/pool/<tier folder>/<file>`.

   ### Changes

   - **2026-09.** <the moved text>
   ```
   Decks with no history get no section.

## Finish
- Check with grep: no card outside the list in the prose; header counts right; no tuning paragraphs left.
- Report to `rust/agents/report-pool-cleanup-<your part>.md` (one line per deck: what changed). Final reply at most
  15 lines.
