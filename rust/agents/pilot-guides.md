# Brief: make a deck document a detailed pilot guide (2026-11-07)

JD wants each of his deck documents to be a detailed pilot guide like the Tymna/Thrasios one
(`decklists/JD/tymna-thrasios-doomsday.md`, the model to follow). You get ONE deck. This brief replaces the
Rust-port parts of `rust/agents/RULES.md` (no worktree, no difftest): read only its "Keeping context small" part.

## Your file and limits
- Edit only your deck document, `decklists/JD/<file>.md` (named in your prompt), plus your report
  `rust/agents/report-pilot-<deck>.md`. Never touch the `## Import list` block or the `## Decklist by type` block,
  the commander/bracket table's facts, or any other file. Don't commit (the coordinator commits). No sims, no cargo,
  no Python (a hook refuses `python`, `pip`, `uv`). Use `jq`, `grep`, `sed -n` to read.
- The document describes only the CURRENT list. No dated change notes, no history (that lives in
  `decklists/CHANGELOG.md`). Name only cards that are in the import list, except opponents' cards in matchups.

## What the guide must have (Tymna's sections, adapted to your deck)
Keep what is already good; rewrite or add so the document has, in this order under `## Strategy`:
1. **The commander** (exact Oracle text quoted, what it does in this deck, when to cast it).
2. **How the deck wins**: each win line as its own `####` section, step by step with the cards and mana it needs
   (like Tymna's "Line 1 ... Line 4"), plus the fair/grind win.
3. **Game plan**: turns 1-3, 4-6, late; when to hold up versus go.
4. **Mulligans**: the keep table (7 / 6-5 / 4 cards) and **6-8 sample hands** from this list (`| # | Hand | Call |
   Why |`), each hand 7 real cards from the import list, with the call and a reason that checks out.
5. **What to tutor for**: a table by situation (situation, tutor, target), only tutors in the list.
6. **Protecting the win turn** (or the key permanent, for decks without a combo turn).
7. **Interaction on opponents' turns**: a table of the deck's answers (card, what to use it on).
8. **Matchups**: one row per opponent deck of your deck's tiers (below): their threat, your plan, the cards that
   matter. Read the opponent's list in `data/decks.json` (`.pool[] | select(.key=="...")`) or
   `decklists/pool/<tier folder>/<key>.md` to name their real threats.
9. **After a stopped attempt** (or after a wipe / losing the commander): how to rebuild.
10. **Life management**: a table of the life costs in the list and what gains it back.
11. **Common mistakes**: a numbered list, 8-10 items, specific to this deck.
Then `## Mana` (counts that match the list) and `## Bracket and Rule 0` (keep the facts).

## Accuracy rules (the coordinator spot-checks these)
- Every card you describe: read its Oracle text first: `jq -r '."<lower-case name>".oracle_text' data/scryfall_cache.json`
  (keys are lower case; a double-faced card's key is its full name). Mana costs from `.mana_cost`. Don't describe a
  card from memory.
- Every card you name for your deck must be in the import list
  (`jq -r '.mine[]|select(.key=="<deck>")|.cards[]' data/decks.json`).
- Use the sim evidence: `audit/<deck>/FINDINGS.md` (who kills the deck, how it wins, cards stuck in hand) for the
  matchups, mistakes and weak spots; `data/plans/<deck>.toml` for the intended lines. Say "the sims show" only for
  what FINDINGS shows.
- Plain, concrete English, short sentences, no filler; the style of the Tymna and Thanos documents.

## Decks and tiers
| Deck key | File | Tiers | Opponents to cover |
|---|---|---|---|
| aragorn | aragorn-uniter.md | t4, t5 | t4: chulane, edgar-markov, krenko, prosper, yuriko; t5: kinnan, miirym, urza, yawgmoth, zur |
| sheoldred | sheoldred-mono-black.md | t3, t4 | t3: atraxa, aurelia, korvold, marwyn, tergrid; t4: as above |
| yshtola | yshtola-esper-drain.md | t5 | t5: as above |
| cloud | cloud-midgar-equipment.md | t1, t2, t3 | t1: isshin, lathril, light-paws, tatyova, teysa; t2: brago, kaalia, lord-windgrace, meren, sythis; t3: as above |

(The full pool keys are in `data/decks.json` `.pool[] | {key, tier}`.)

## Report
At most 15 lines back to the coordinator: sections added or rewritten, line count before and after, anything you
couldn't verify. Full notes (every card claim you weren't sure of) in `rust/agents/report-pilot-<deck>.md`.
