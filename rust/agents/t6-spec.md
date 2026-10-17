# Brief: Tier 6 Stage 2 — behaviour spec for one cEDH deck

This is a research and writing task, not a port. Ignore the porting parts of `RULES.md`; its rules on memory caps
and short reports still apply. **Never run Python** (a hook blocks it): use `curl`, `jq` and shell tools.

## Context

We are adding a Tier 6 (full cEDH, Bracket 5) opponent pool to the Commander simulator, so JD's Godo deck can be
tested against real cEDH play. Stage 2 turns each deck's primer into a spec. The spec is what the AI plan
(Stage 5), the combo definitions (Stage 3) and the scenario tests are written from. A spec that guesses is worse
than one that says "the primer doesn't say".

Worktree: `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/t6-cedh` (branch `t6-cedh`).

- The deck lists: `decklists/pool/staging-t6-cedh/<deck>.md` (the `## Import list` is the exact 100 cards).
- Godo's list and our own notes: `decklists/JD/godo-cedh-equipment.md` on branch `godo-cedh`
  (`git show origin/godo-cedh:decklists/JD/godo-cedh-equipment.md`).
- The targets and why: `decklists/pool/staging-t6-cedh/TARGETS.md`.

Your deck is named in the prompt that started you.

## What to do

1. Read your deck's list.
2. Find the deck's primer and other good write-ups. Start from https://cedh-decklist-database.com (it links each
   archetype's primer). Moxfield blocks automated reads (HTTP 403): use other copies, the database's own text,
   articles (e.g. on the site linked from the database or from EDHTop16), and Reddit r/CompetitiveEDH threads.
   Open every page you cite; a search snippet is not a source.
3. Check the Oracle text of every card you name in a win line or decision against Scryfall
   (`curl -s 'https://api.scryfall.com/cards/named?exact=<url-encoded name>' | jq -r .oracle_text`, 100 ms
   between calls).
4. Write `audit/t6/<deck>/SPEC.md` (create the folder) to the template below.
5. Write a full report to `rust/agents/report-t6-spec-<deck>.md`. **Do not commit** (several agents share this
   worktree; the lead commits). Do not edit any other file.

## Rules for the spec

- Every claim is tagged **[primer]** (a source says it; link it in that section or in Sources) or **[inference]**
  (your reasoning from the cards). Prefer primer claims. Keep inferences few and say why.
- Win lines use only cards **in our list**. If the primer's main line needs a card our list doesn't have, say so.
- Mana: give the total and the colours for each line, counting from an empty board where it matters.
- Plain, short sentences. Tables where items have the same attributes. No filler.

## Template

```
# <Deck> — Behaviour Spec (Tier 6)

Sources: <each page opened, with its title and date if shown>. Spec written 2026-10-17.

## Game plan
One paragraph: how the deck wins, how fast it aims to (turn), and its stance (race / hold up / lock).

## Win lines
| Line | Pieces (in our list) | Mana (total, colours) | Steps | Where it can be answered (step, by what) |
One row per line, main line first. Below the table: tutors that find each piece.

## Mulligan
Keep rules, ship rules, and what a typical keep looks like.

## Turn by turn
Turns 1–2, turns 3–4, later: what it deploys first and in what order.

## Interaction
What it holds (list the cards), what it spends each on, and when it lets something resolve.
How it decides who to stop when two players threaten to win.

## When it goes for the win
The conditions (table's open mana, cards in hand, its own protection) under which it tries a win line,
and when it waits.

## What it fears
Cards and strategies that beat it, and how it plays around each.

## Against Godo
How this deck stops Godo's lines (Godo + Helm of the Host from the ETB; Dualcaster Mage + copy spell), using the
cards in its list: which answers, at which step. (For the Godo spec itself: how Godo beats each of the five Tier 6
decks, and what it must play around.)

## Decision scenarios (become AI tests)
At least 8. Each: the position (board, hand, mana, what opponents show), the right action, the reason, and the tag.

## Combo tests (become scripted tests)
One per win line: the starting board and hand, and the expected result.

## Open questions
Where the sources are silent or disagree.
```

## Report

At most 15 lines in your final message: the spec path, the sources used, how many claims are [primer] vs
[inference], and anything the lead must decide.
