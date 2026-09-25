# Commander pod simulator

Pure Python 3 (3.9 or newer), standard library only. Nothing to install.

## Run
Each run seats one of your decks against three outside decks from `decklists/pool/` (25 decks in five power tiers,
see `decklists/pool/README.md`), drawn without replacement from one tier and re-drawn every game, in a random seat
order. Opponent draws, seats and every seat's opening shuffle depend only on the seed and the deck keys, so a
`--swap` comparison faces identical opponents and draws (paired runs).

    cd commander-sim
    python3 compare.py --deck seph --pool t3 --games 1500 --jobs 24             # one deck vs one tier
    python3 compare.py --deck seph --pool t3 --swap "Blood Artist=>Grim Tutor"   # paired A/B vs that tier
    python3 compare.py --deck seph --pool all --jobs 24                          # all five tiers
    python3 compare.py --all-decks --pool all --jobs 24                          # deck x tier matrix
    python3 compare.py --deck seph --pool t4 --analyze                           # how it wins / loses there
    python3 compare.py --deck seph --pool t2 --trace 7                           # play-by-play of game 7
    python3 compare.py --calibrate within|ordering|all --games 240 --jobs 24     # pool balance checks
    python3 compare.py --deck veyran --cards                                     # every card: source, tags, unmodeled text
    python3 tools_searchtest.py <pool deck> <tier> 240 6 5 all                   # a pool deck with and without look-ahead
    python3 tools_swaptest.py <pool deck> <tier> 800 "Out>In; Out>In"            # test pool list changes without editing
    python3 pools.py --validate                                                  # decklist checks
    python3 pool_audit.py [--deck yuriko] [--md FILE]                            # card coverage per deck

- `--deck`     seph | veyran | sauron
- `--pool`     t1..t5 or all (required, unless --all-decks or --calibrate)
- `--swap`     "Card Out=>Card In" (repeat for several swaps)
- `--games`    games per list per profile (default 1500; `--n` is the same)
- `--seed`     first seed (default 500000)
- `--jobs`     worker processes; set to your core count
- `--profiles` conservative,loose (default both); `--profile` runs one
- `--ai`       lookahead (default) or adaptive. Look-ahead: every deck at the table chooses its main-phase plays,
               attacks and counterspells by trying the best few candidates on copies of the game and playing each
               copy forward to the end of its next turn (search.py). It costs about 20 s of CPU per game (1500 games
               on 24 cores: about 20 minutes per profile). adaptive is the heuristic AI alone, about 100x faster:
               fine for quick checks, but it misplays some decks badly (see decklists/pool/pool-results.md).
- `--temp`     adaptive randomness multiplier (default 1.0; 0.5 = sharper, more predictable play,
               2.0 = looser play). Per-deck styles (aggression, caution) live at the top of brain.py
- `--analyze`  deep report instead of a win rate: how it wins, how it loses (who and by what), game-plan timing,
               damage by source, and a card report. Add --swap to analyze a variant.
- `--brief`    skip the per-axis breakdown (verdict only)
- `--trace N`  print a play-by-play log of game N (current list, or the variant with --swap), then exit

Each run reports your win rate against the 25% even share with a 95% interval, the average turn of your wins
and losses, how often your plan came online, and for each opponent how often it was seated, how often it won,
and how often it eliminated you. A/B runs report a paired noise band (per-seed pairs) and a breakdown along
seven strength axes (outcome, mana & consistency, speed, card flow, interaction, resilience, threat &
pressure), baseline -> variant under each profile, with * marking changes larger than the noise band.

The baseline is the list in the deck's .md file in `decklists/mine/` (the "## Import list" block);
point SIM_DECKS at another folder to use lists kept elsewhere. The outside decks are in `decklists/pool/`
(one folder per tier, plus `retired/` for decks no longer drawn).

The original four-deck mode (your decks against each other) was removed in September 2026; everything is
measured against the pools.

## The opponent pools
- Results and calibration: `decklists/pool/pool-results.md`. Card coverage and what is approximated: run
  `python3 pool_audit.py`, or read the tables in `pool-results.md`.
- How outside cards are modeled: `pool_cards.py` (tag and ability overrides, audit notes), `cardimpl.py`
  (hook events) with the implementations in `impl_common.py`, `impl_t1.py` .. `impl_t5.py`, and combos in
  `impl_combos.py`. The generic AI for outside decks is `pool_ai.py`; per-deck settings in `pool_decks.py`
  and `deck_plans.py`. Your decks keep their own hand-tuned AI.
- The look-ahead: `search.py` (game copies, candidate plays, playouts, position scoring). Engine step caps
  (`engine.GAME_WORK`, `search.PLAYOUT_WORK`), a per-game decision cap and a board-size limit keep every game
  bounded; the seeds are deterministic, so any game replays exactly.
- Tests: `python3 -m unittest discover -s tests -t .` (validator, pool sampling and pairing, pool games with
  and without look-ahead, and a guard that your four deck files still parse to the recorded lists).

## How the adaptive AI decides (brain.py)
Each decision: read the board (mana, hand, incoming damage, who leads, combos close to going off,
and what opponents probably hold, estimated from public information), score every legal play,
sample one from a softmax distribution, re-read the board, repeat. `--trace` shows each distribution.

## Card audit
CARD_AUDIT.md lists every card with how faithfully the sim models it (Modeled / Approximate /
Partial / Not modeled), checked against Oracle text. Update it when you add cards.

## Cards from Scryfall
Any card that isn't hand-tagged in carddb.py is fetched from Scryfall (https://scryfall.com) and
auto-tagged from its Oracle text the first time it's used, then cached in scryfall_cache.json.
You can put any card in a deck file or a --swap; no manual tagging needed.

    python3 autotag.py "Talrand, Sky Summoner" "Grim Tutor"   # preview how cards will be modeled
    python3 compare.py --deck veyran --cards                   # every card: source, tags, unmodeled text,
                                                               # plus the Game Changer count from Scryfall
Auto-tagged cards print their tags and any "not modeled" Oracle lines whenever they're used.
If a card matters and its tags look wrong, add a hand-written line to carddb.py: it always wins.
Needs an internet connection the first time a card is looked up.

## Card ability language (how any card is modeled)
Cards are described as data: a list of abilities (spell effects, triggered, activated, loyalty,
static, replacement), each made of small effects (draw, damage, destroy, token, counters, search,
reanimate, pump, grant keyword, sacrifice, extra combat, ...). The interpreter in dsl.py runs them.
Oracle text from Scryfall is compiled into this form automatically (dsl_parse.py).

    python3 dsl.py "Grave Pact" "Chandra, Torch of Defiance"   # show the compiled abilities
    python3 compare.py --deck veyran --cards                    # every card's abilities + unmodeled text
    python3 compare.py --deck seph --dsl-all --n 1500            # run EVERY card from its Oracle text
                                                                 # (ignores hand tags; good cross-check)
To model a card yourself (or fix one), write its abilities into cards_dsl.json -- see
cards_dsl.example.json. Entries there override every other source. No code changes needed.
Card source priority: cards_dsl.json > carddb.py hand tags > Scryfall compiled > Scryfall regex tags.

## New cards (hand tags)
Tag each incoming card in carddb.py first:  Name|types|cost|tags
e.g. `Grim Tutor|S|1BB|tut=any lose=3`. Copy the pattern of a similar card.
Untagged text doesn't exist in the sim; cards with no recognised tags are never cast.
Cards that do something new need AI code in ais.py.

## Reading results
Win rates are directional; the paired before/after comparison is the reliable part.
Trust changes that hold under both AI interaction profiles.
