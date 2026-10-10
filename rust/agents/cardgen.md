# Task: `cardgen`, card data and deck lists without Python

Worktree: `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/agent-a81b136543e20d594`
(branch `worktree-agent-a81b136543e20d594`). A previous agent was stopped just before porting the Oracle-text
compiler; its work is in WIP commit d0a2f17 (new crate `rust/crates/cardgen`, data files).

The engine loads data/cards.json and data/decks.json, which only Python's `python3 -m
commander_sim.tools.export_cards` produces today. Definitions come from cards/carddb.py (DB_TEXT), `card()` /
`full()` / `note()` calls in cards/impl/*.py (via cards/pool_cards.py), the Scryfall cache (cards/sources.py,
scryfall.py), cards/dsl_parse.py (Oracle text → ability language), cards/dsl.py (compile step), cards/autotag.py;
deck lists from decks.py, deck_files.py, pools.py and the decklist files; update_deck.py edits decks;
pool_audit.py reports what's modeled.

1. One-time extraction (a script in rust/tools/ may use Python to read Python) of every definition source into
   human-editable files in `data/` (keep the user able to add a card by hand).
2. Port the pipeline to `crates/cardgen` (library + binary: `cardgen export`, `cardgen audit`, `cardgen deck ...`):
   data files + Scryfall cache → Oracle parser, compile step, autotag, Game Changer flags, deck lists → cards.json /
   decks.json (reuse sim-core/src/export.rs types). Don't hit the Scryfall API in tests; keep Python's cache format.
3. Equivalence: `cardgen export` reproduces the current data files (field by field for every card; report and
   explain any difference); add a test.
4. Port update_deck.py (same behaviour) and pool_audit.py (card code is Rust now: look up hooks in sim-core's
   registry).
5. `rust/crates/cardgen/README.md` (add a card, edit a deck, rebuild, audit); update rust/README.md.

No difftest needed. Commit message: "Rust port: cardgen — card data and deck lists without Python".
