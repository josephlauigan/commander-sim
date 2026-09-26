# Commander pod simulator

Measures a Commander deck against pools of outside decks in simulated four-player games. You get a win rate with a
confidence interval, and paired before/after comparisons for list changes.

Pure Python 3 (3.9 or newer), standard library only. Nothing to install. Run every command from the repository
root.

For how it works inside, see [documents/architecture.md](documents/architecture.md).

## Repository layout

```
commander_sim/            the simulator (a Python package)
  compare.py              command line: python3 -m commander_sim ...
  poolmode.py             runs against the pools: one tier, A/B, matrix, calibration, --analyze
  pools.py                the pool decks: loading, validation, seating
  decks.py                your decks (decklists/mine/)
  pool_audit.py           how faithfully each card is modeled
  engine.py               game state and rules
  ais.py                  the turn loop, combat, and your decks' play plans
  ai/                     decision-making
    brain.py              the heuristic AI
    search.py             the look-ahead AI
    pool_ai.py            the AI for pool decks, with pool_decks.py and deck_plans.py
  cards/                  card data
    carddb.py             hand-verified tags
    sources.py            where each card's definition comes from
    scryfall.py           Scryfall client
    autotag.py            Oracle text to tags
    dsl_parse.py, dsl.py  the card ability language (compiler and interpreter)
    cardimpl.py           the Python hook registry
    pool_cards.py         overrides for pool cards
    impl/                 Python implementations of individual cards
  tools/                  searchtest.py, swaptest.py, linecov.py
data/                     scryfall_cache.json, cards_dsl.example.json
decklists/mine/           your decks
decklists/pool/           the 25 opponent decks in five tiers, results (pool-results.md), retired/
documents/                architecture.md, card-audit.md
tests/                    rule, card, AI and command tests (tests/README.md)
```

## Run

Each game seats one of your decks with three outside decks drawn from one tier of `decklists/pool/` (see
`decklists/pool/README.md`), re-drawn every game, in a random seat order. Opponent draws, seats and every seat's
opening shuffle depend only on the seed and the deck keys, so a `--swap` comparison faces identical opponents and
draws (paired runs).

```
python3 -m commander_sim --deck seph --pool t3 --games 1500 --jobs 24             # one deck vs one tier
python3 -m commander_sim --deck seph --pool t3 --swap "Blood Artist=>Grim Tutor"  # paired A/B vs that tier
python3 -m commander_sim --deck seph --pool all --jobs 24                         # all five tiers
python3 -m commander_sim --all-decks --pool all --jobs 24                         # deck x tier matrix
python3 -m commander_sim --deck seph --pool t4 --analyze                          # how it wins / loses there
python3 -m commander_sim --deck seph --pool t2 --trace 7                          # play-by-play of game 7
python3 -m commander_sim --calibrate within|ordering|all --games 240 --jobs 24    # pool balance checks
python3 -m commander_sim --deck veyran --cards                                    # every card: source, tags, unmodeled text
```

Options:

- `--deck`: `seph`, `veyran` or `sauron`.
- `--pool`: `t1` to `t5`, or `all`. Required, unless you use `--all-decks` or `--calibrate`.
- `--swap`: `"Card Out=>Card In"`. Repeat it for several swaps.
- `--games`: games per list per profile (default 1500). `--n` is the same.
- `--seed`: the first seed (default 500000).
- `--jobs`: worker processes (default 1). Set it to your core count.
- `--profiles`: `conservative,loose` (default both). `--profile` runs one. Loose opponents counter and remove more
  freely, so it is the worst case for your decks.
- `--ai`: `lookahead` (default) or `adaptive`.
  - Look-ahead: every deck at the table chooses its main-phase plays, attacks and counterspells by trying the best
    few candidates on copies of the game, and playing each copy forward to the end of its next turn. It costs about
    20 s of CPU per game: 1500 games on 24 cores take about 20 minutes per profile.
  - `adaptive`: the heuristic AI alone, about 100 times faster. Fine for quick checks, but it misplays some decks
    badly (see `decklists/pool/pool-results.md`).
- `--temp`: the heuristic AI's randomness multiplier (default 1.0; 0.5 is sharper, more predictable play; 2.0 is
  looser). Per-deck play styles are at the top of `commander_sim/ai/brain.py`.
- `--analyze`: a deep report instead of a win rate. It covers how the deck wins, how it loses (to whom and by what),
  game-plan timing, damage by source, and a card report. Add `--swap` to analyze a variant.
- `--brief`: skip the per-axis breakdown (verdict only).
- `--trace N`: print a play-by-play log of game N (the current list, or the variant with `--swap`), then exit.

Other commands:

```
python3 -m commander_sim.pools [--validate]                              # list or check the pool decklists
python3 -m commander_sim.pool_audit [--deck yuriko] [--mine] [--md FILE] # card coverage per deck
python3 -m commander_sim.tools.swaptest <pool deck> <tier> 800 "Out>In; Out>In"   # pool list changes, without editing
python3 -m commander_sim.tools.searchtest <pool deck> <tier> 240 6 5 all          # a pool deck with and without look-ahead
python3 -m commander_sim.cards.autotag "Talrand, Sky Summoner"           # preview how a card is tagged
python3 -m commander_sim.cards.dsl "Grave Pact"                          # show a card's compiled abilities
python3 -m unittest discover -s tests -t .                               # the tests (see tests/README.md)
python3 -m commander_sim.tools.linecov                                   # their line coverage
```

## Reading results

Each run reports:

- your win rate against the 25% even share, with a 95% interval;
- the average turn of your wins and losses;
- how often your game plan came online;
- for each opponent, how often it was seated, how often it won, and how often it eliminated you.

A/B runs report a paired noise band (from per-seed pairs) and a breakdown along seven strength axes: outcome, mana
and consistency, speed, card flow, interaction, resilience, and threat and pressure. Each axis shows baseline →
variant under each profile, with `*` marking changes larger than the noise band.

Win rates are directional; the paired before/after comparison is the reliable part. Trust changes that hold under
both interaction profiles.

The pool results and calibration are in `decklists/pool/pool-results.md`.

## Your decks

The baseline is the list in each deck's `.md` file in `decklists/mine/` (the `## Import list` block). Point
`SIM_DECKS` at another folder to use lists kept elsewhere. After editing a list, update
`tests/fixtures/my_decks_parsed.json` too; a test guards against accidental edits.

## Cards

Any card can go in a deck file or a `--swap`; no manual tagging is needed. Each card's definition comes from the
first of these that has it:

1. **`data/cards_dsl.json`**: ability data you write for a card (see `data/cards_dsl.example.json`). It always wins,
   and needs no code changes.
2. **`commander_sim/cards/carddb.py`**: hand-verified tags, one line per card (`Name|types|cost|tags`, for example
   `Grim Tutor|S|1BB|tut=any lose=3`).
3. **Scryfall.** The card's Oracle text is compiled into the ability language, or, if nothing compiles, turned into
   tags by regular expressions. The lookup needs an internet connection the first time a card is used, then it is
   cached in `data/scryfall_cache.json`.

Cards that need behaviour none of these can express get Python hooks in `commander_sim/cards/impl/` (your decks' cards
in `mine.py`). `python3 -m commander_sim --deck veyran --cards` shows every card's source, tags, abilities and any
Oracle text that isn't modeled; `python3 -m commander_sim.pool_audit --mine` lists your cards that aren't fully
modeled. `documents/card-audit.md` records the hand check of your decks' cards against Oracle text.

`--dsl-all` runs every card from its compiled Oracle text instead of hand tags, as a cross-check.
