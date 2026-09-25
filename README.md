# Commander pod simulator

Pure Python 3 (3.9 or newer), standard library only. Nothing to install.

## Run
    cd commander-sim
    python3 compare.py --deck seph --swap "Phyrexian Metamorph=>Grim Tutor" --n 10000 --jobs 5

- `--deck`     seph | veyran | sauron | najeela
- `--swap`     "Card Out=>Card In" (repeat for several swaps)
- `--n`        games per list per profile (default 1500; 10000 for close calls)
- `--jobs`     worker processes; set to your core count (`sysctl -n hw.ncpu`)
- `--ablate`   also test each swap on its own
- `--goldfish` add solo speed numbers
- `--profiles` conservative,loose (default both)
- `--ai`       adaptive (default) or rigid. Adaptive AIs read the board and choose plays from a
               probability distribution; rigid AIs follow fixed priority lists (the original model)
- `--temp`     adaptive randomness multiplier (default 1.0; 0.5 = sharper, more predictable play,
               2.0 = looser play). Per-deck styles (aggression, caution) live at the top of brain.py
- `--analyze`  deep report on one deck instead of a comparison: how it wins, how it loses (who
               and by what), game-plan timing, damage by source, and a card report (win rate when
               cast, cards stuck in hand, cards opponents remove most). Add --swap to analyze a variant.
- `--brief`    skip the per-axis breakdown (verdict only)
- `--trace N`  print a play-by-play log of game N with the variant list (current list if no --swap), then exit

Every comparison prints, after the verdict, a breakdown along seven strength axes
(outcome, mana & consistency, speed, card flow, interaction, resilience, threat & pressure),
baseline -> variant under each profile, with * marking changes larger than the noise band.

The baseline is the list in the deck's .md file in this folder (the "## Import list" block).
Keep the .md files next to the scripts, or point SIM_DECKS at another folder.

## Using the opponent pools
`opponents/` holds 25 outside decks in five power tiers (see `opponents/README.md`). Pool mode seats your
deck against three of them, drawn without replacement from one tier and re-drawn every game, in a random
seat order. Opponent draws, seats and every seat's opening shuffle depend only on the seed and the deck
keys, so a --swap comparison faces identical opponents and draws (paired runs).

    python3 compare.py --deck seph --pool t3 --games 10000 --jobs 24          # one deck vs one tier
    python3 compare.py --deck seph --pool t3 --swap "Blood Artist=>Grim Tutor"  # paired A/B vs that tier
    python3 compare.py --deck seph --pool all --games 10000                   # all five tiers
    python3 compare.py --all-decks --pool all --games 10000 --jobs 24         # deck x tier matrix
    python3 compare.py --deck seph --pool t4 --analyze                         # how it wins / loses there
    python3 compare.py --deck seph --pool t2 --trace 7                         # play-by-play of pool game 7
    python3 compare.py --calibrate within|ordering|all --games 2000 --jobs 24   # pool balance checks
    python3 tools_swaptest.py <pool deck> <tier> 800 "Out>In; Out>In"          # test list changes without editing
    python3 pools.py --validate                                                # decklist checks
    python3 pool_audit.py [--deck yuriko] [--md FILE]                          # card coverage per deck

- `--pool` t1..t5 or all; `--games` (same as --n); `--seed` first seed (default 500000);
  `--profile conservative|loose` to run one profile (default both).
- Each run reports your win rate against the 25% even share with a 95% interval, the average turn of your
  wins and losses, how often your plan came online, and for each opponent how often it was seated, how
  often it won, and how often it eliminated you. A/B runs report a paired noise band (per-seed pairs).
- Pool games play by a few extra rules the four-deck mode doesn't have (so old-mode results never change):
  haste from Boots/Greaves and granted haste, fetch lands cracking, attacking planeswalkers, keyword grants
  from compiled cards. Your decks keep their own AI; only outside decks use the pool AI.
- Results and calibration: `opponents/pool-results.md`. Card coverage and what is approximated: run
  `python3 pool_audit.py`, or read the tables in `pool-results.md`.
- How outside cards are modeled: `pool_cards.py` (tag and ability overrides, audit notes), `cardimpl.py`
  (hook events) with the implementations in `impl_common.py`, `impl_t1.py` .. `impl_t5.py`, and combos in
  `impl_combos.py`. The generic AI for outside decks is `pool_ai.py`; per-deck settings in `pool_decks.py`.
- Tests: `python3 -m unittest discover -s tests -t .` (validator, pool sampling and pairing, and an exact
  old-mode regression guard).

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
