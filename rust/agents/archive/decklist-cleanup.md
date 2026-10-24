# Brief: clean up the deck documents (decklist-cleanup)

JD (the user) wants each deck document under `decklists/JD/` and `decklists/Avery/` to describe **the deck as it is
now**, and nothing else. Three changes:

1. **No pull or pick lists.** Delete every "Where the cards come from" section (the per-colour / per-type tables of
   Bulk / Binder / Jodah / Buy / set codes), every "Buy list" section and every bought-card price table. Don't move
   these tables anywhere: they are gone (git history keeps them).
2. **No changelog in the deck doc.** Every dated note ("**Updated 2026-10-13.** Out: … In: …", "**Added …**",
   "**Simulator, …**", bare date lines like `2026-09-17`), every "Simulator status" / "What the simulator found"
   section, every "Flags and tuning levers" section, every "Bench" / "Cheap upgrades" note, and every history phrase
   inside the prose ("since 10-04", "(10-07)", "took Marchesa's slot", "left on 10-13", "JD's choice", "the new card
   is JD's, from his binder", "in Consecrated Sphinx's old slot", audit paths, win percentages, paired-test deltas)
   moves to the changelog part file (see below). Nothing dated stays in the deck doc.
3. **A detailed Strategy section.** Some decks' strategy is thin (a few bullets). Every deck gets a full,
   player-facing guide (see the template).

You work in the shared worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/decklist-cleanup`.
Other agents edit other deck files there at the same time: **edit only the files your task names. Do not commit,
do not run git commands that change anything, do not run cargo or any sim.** Never run Python (a hook blocks it).

## Rules

- **The `## Import list (100)` block must stay byte-identical**: don't touch a character inside the fence or the
  heading. The `## Decklist by type (100)` section stays too (same heading, same `**Group (n).** a, b, c` lines);
  only a deck that has none gets one built (your task says so).
- **Only cards in the import list.** Every card the deck doc names (outside the changelog) must be in the deck's
  current import list. Old docs still mention cards that have left (cut Swords, wipes, tutors…): remove those
  mentions or move them to the changelog. Check every card name you write with `grep -F` against the import list.
- **Check card text against Oracle.** The Scryfall cache `data/scryfall_cache.json` is keyed by lower-case card
  name: `jq -r '."sol ring" | .mana_cost, .type_line, .oracle_text' data/scryfall_cache.json`. Double-faced cards
  are keyed by the full `a // b` name too and keep each face in `.card_faces[]`. Before you describe what a card
  does, read its Oracle text. Don't invent interactions; if you aren't sure a line works by the rules, leave it out.
- **Keep what's good.** The existing Strategy / Key lines / Tutor targets / Consistency / Bracket prose is mostly
  accurate and tested. Keep its content (minus history and departed cards) and extend it; don't paraphrase good
  text into vaguer text.
- **Sim findings become advice.** Insights from simulator sections that help a player ("loses mostly to big creature
  decks, so land Propaganda early", "the wins come late; she is often removed and recast") can go into the Strategy
  as plain advice, without numbers, dates or tier names. The numbers go to the changelog.
- **Style**: match the existing docs: plain English, short sentences, `{2}{B}` mana symbols, **bold** for card
  names at the point they're introduced, card text in *italics* when quoted. Lines wrapped at about 120 characters or
  not wrapped at all, whichever the file already does. "colour" or "color": keep the file's own spelling.
- Say "JD" for the owner of the JD decks, "Avery" for Avery's decks, if needed at all (rarely: this is a deck guide).

## Deck doc template

```
# <Commander>, <title> — <Theme>            (keep the existing H1)

<2–4 sentence overview: what the deck does, how it wins, what it's weak to.>

| | |
|---|---|
| **Commander** | <name> ({cost}, P/T, colours) |
| **Bracket** | <n> (<k> Game Changers: a, b, c) — or "no Game Changers" |
| **Simulator** | deck key `<key>`; results in `decklists/pool/pool-results.md`, history in `decklists/CHANGELOG.md` |

## Strategy

### The commander
Oracle text quoted (both faces if it transforms), then what it means in practice: how much value per turn, what it
wants, what breaks it.

### Game plan
Early turns (1–3), middle turns (4–6), late game: what to do in each, which cards matter when.

### <One subsection per package / engine / role>
e.g. "Make them draw", "Punishers", "Protection", "Removal", "Recursion", "Finishers". Name **every nonland card**
in the list somewhere in Strategy, Key lines or Mana and consistency, with what it does here (a list of cards with
a phrase each is fine for the minor ones). Explain the synergies between cards, not only each card alone.

### How it wins
The kill lines, with rough numbers where they help (e.g. "Debt for X = 5 with Bond out kills …").

### Weak spots and how to play around them
What beats the deck (graveyard hate, wipes, fast combo, big creature decks…) and what to do about it.

## Key lines
Concrete sequences: openings, the big turn, sequencing tips, mulligan advice.

## Tutor targets by board state      (only if the deck has tutors; keep/extend the existing section)

## Mana and consistency
Land count and makeup, ramp, card draw, interaction counts: the current list only, no history.
(Existing "Consistency" tables can stay, cleaned of history.)

## Bracket and Rule 0
Bracket, Game Changers, combos, what to disclose; the commanderbrackets.com line if the file has one.

## Decklist by type (100)

## Import list (100)
```

Section names: use these. A deck without tutors has no "Tutor targets" section.

## Changelog part file

For each deck write `rust/agents/changelog-parts/<key>.md` (I assemble them into `decklists/CHANGELOG.md`):

```
## <Deck name as in the H1>

Deck key `<key>`, `decklists/<owner>/<file>.md`.

### Changes

- **2026-10-14.** Out: … In: … <why, measured result, audit path, who chose it> (verbatim from the old note,
  lightly cleaned)
- **2026-10-13.** …
  … newest first, oldest last; the "Added" / build entry is the last bullet. Fix entries the old file had
  interleaved or out of order. Keep every fact, figure and path from every dated note. Where the old doc's
  history tables ("Flags and tuning levers": "The 09-26 change — 6 out, 6 in" etc.) explain a change, fold their
  rows into that date's bullet (a nested list or table under the bullet is fine).

### Simulator history

Every simulator figure the old doc had (the "Simulator, …" header lines, "Simulator status", "What the simulator
found" sections and their tables), dated, newest first.

### Tuning notes

Non-history notes from "Flags and tuning levers", bench lists, cheap-upgrade lists, "weakest slots",
"replacements worth testing", known simulator simplifications. Omit the subsection if there's nothing.
```

The changelog part may name departed cards, dates and numbers freely. Don't copy the pull tables or buy tables
into it; one sentence of sourcing in the build entry is fine ("built from bulk and binder; 19 cards bought
2026-10-11 for about $36").

## Checks before you finish

For each deck file you edited:
- `git diff --stat` shows only your files (and your changelog parts are new files);
- the import list is unchanged (heading plus fenced block; some old files have sections after it, which go):
  `imp() { awk '/^## Import list/{f=1} f{print} f&&/^```/{n++; if(n==2) exit}'; }; diff <(git show origin/main:<file> | imp) <(imp < <file>)`
  prints nothing, and the import list is the last section of the file;
- `grep -nE '20[0-9]{2}-[0-9]{2}-[0-9]{2}|\b(0[1-9]|1[0-2])-[0-3][0-9]\b|audit/|[0-9]+\.[0-9]%|Bulk|Binder|Jodah|\$[0-9]' <file>` finds nothing outside the import list (if a hit is legitimate, e.g. a card named "… Jodah …", say so in the report);
- every card name in Strategy / Key lines / Tutor targets / Mana and consistency / Bracket is in the import list,
  and every nonland card in the import list is named outside the import list and decklist-by-type sections at least
  once (write a quick shell loop over the import list to check);
- `## Decklist by type` group counts add up to 100.

## Report

At most 15 lines: files changed, the checks above with results, anything you were unsure of (rules questions,
cards whose interaction you left out). Put longer notes in `rust/agents/report-decklist-cleanup-<group>.md`.
