# Brief: update deck docs for the 10/18 band swaps

Repository: `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/soul-stone` (work only here). Read
`rust/agents/RULES.md` first. Do NOT commit; the main session reviews and commits. No Python, ever.

## What changed

JD approved card swaps across his decks (`decklists/CHANGELOG.md`, the 2026-10-18 entries, says exactly what went out
and in for each deck). The `## Import list` and `## Decklist by type` sections of each deck doc are ALREADY updated by
`cardgen deck`. Your job is the rest of each doc you are given: the prose (strategy, card notes, combos, tutor targets,
the Bracket / Game Changer lines, counts like "five rocks" or "seven Game Changers", tables).

Game Changers after the swaps (only these count):
- Y'shtola (9): Necropotence, Notion Thief, Consecrated Sphinx, Smothering Tithe, Mystical Tutor, Grand Arbiter
  Augustin IV, Force of Will, Farewell, Teferi's Protection (Bolas's Citadel left). Bracket 4.
- Seph FS (4): The One Ring, Demonic Tutor, Vampiric Tutor, Bolas's Citadel. Bracket 4 by GC count.
- Seph/Atraxa (`sephiroth-phyrexian-reanimator.md`) (3): Demonic Tutor, Gifts Ungiven, Aura Shards. Bracket: 3 by GC
  count unless the doc names another reason for 4 (infinite combos? keep the doc's own reasoning if it gives one).
- Sauron (3): Cyclonic Rift, Orcish Bowmasters, Force of Will.
- Sheoldred (1): Ad Nauseam.
- Kefka (3): Underworld Breach, Jeska's Will, Rhystic Study. (Its doc may argue Bracket 4 from GCs: with 3 it is
  Bracket 3 by GC count unless the doc gives another reason such as a two-card combo; say which.)
- Tymna/Kraum (1): Underworld Breach.
- Veyran (2): Consecrated Sphinx, Mystical Tutor.
- Titania (1): Worldly Tutor.

## Rules for the docs (JD's)

- A deck doc describes the CURRENT list and how to play it, in detail. No history ("was swapped in", "used to"), no
  "pull list" or "buy list", no notes that a card is in or came from another deck (decks never share cards).
- Remove or rewrite every mention of a card that left the deck. Add short, accurate notes for the cards that came in
  (what the card does — check its Oracle text with `rust/target/release/commander-sim --deck <key> --cards` or the
  Scryfall cache in `data/scryfall_cache.json` via `jq` — and how it fits this deck's plan).
- Keep the doc's existing voice, structure and plain-English style. Change only what the swaps require.
- Card counts in headings ("**Instants (13).**") are already right; fix counts in prose.

## Your files

Given in your task message. Edit only those.

## Report

Write `rust/agents/report-band-docs-<your letter>.md` early (after your first file) and update it per file: for each
doc, the mentions you changed (old → new, one line each). Final reply: 10 lines or fewer.
