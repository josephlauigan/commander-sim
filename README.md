# Commander pod simulator

Measures a Commander deck against pools of outside decks in simulated four-player games. You get a win rate with a
confidence interval, and paired before/after comparisons for list changes. You can also play one of your decks by
hand against the same AI opponents in your browser ([Practice mode](#practice-mode)).

Pure Python 3 (3.9 or newer), standard library only. Nothing to install. Run every command from the repository
root.

For how it works inside, see [documents/architecture.md](documents/architecture.md).

## Repository layout

```
commander_sim/            the simulator (a Python package)
  compare.py              command line: python3 -m commander_sim ...
  poolmode.py             runs against the pools: one tier, A/B, matrix, calibration, --analyze
  pools.py                the pool decks: loading, validation, seating
  decks.py                your decks (decklists/JD/, Avery/; deck_files.py says which is where)
  update_deck.py          replace a deck's list with a pasted one
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
  play/                   practice mode: play a deck by hand (session, rules check, server, the browser page)
  tools/                  searchtest.py, swaptest.py, linecov.py
data/                     scryfall_cache.json, cards_dsl.example.json; images/ and saves/ (practice mode, not in git)
decklists/JD/             JD's decks: Jodah, Sauron, Sephiroth, Veyran, Y'shtola
decklists/Avery/          Avery's decks: Alela, Galadriel
decklists/pool/           the 25 opponent decks in five tiers, results (pool-results.md), retired/
documents/                architecture.md, card-audit.md, practice-mode.md
tools/                    knight_dragon.py (draws practice mode's loading animation)
tests/                    rule, card, AI and command tests (tests/README.md)
```

## Run

Each game seats one of your decks with three outside decks drawn from one tier of `decklists/pool/` (see
`decklists/pool/README.md`), re-drawn every game, in a random seat order. Opponent draws, seats and every seat's
opening shuffle depend only on the seed and the deck keys, so a `--swap` comparison faces identical opponents and
draws (paired runs).

```
python3 -m commander_sim --deck seph --pool t3 --games 1500                       # one deck vs one tier
python3 -m commander_sim --deck seph --pool t3 --swap "Blood Artist=>Grim Tutor"  # paired A/B vs that tier
python3 -m commander_sim --deck seph --pool t3 --swap "Blood Artist=>Grim Tutor" --ai adaptive --profile loose
                                                                                  # the same, a quick read (seconds)
python3 -m commander_sim --deck seph --pool all                                   # all five tiers
python3 -m commander_sim --all-decks --pool all                                   # deck x tier matrix
python3 -m commander_sim --deck seph --pool t4 --analyze                          # how it wins / loses there
python3 -m commander_sim --deck seph --pool t2 --trace 7                          # play-by-play of game 7
python3 -m commander_sim --calibrate within|ordering|all --games 240              # pool balance checks
python3 -m commander_sim --deck veyran --cards                                    # every card: source, tags, unmodeled text
```

Options:

- `--deck`: `seph`, `veyran`, `sauron`, `yshtola`, `jodah`, `galadriel` or `alela`.
- `--pool`: `t1` to `t5`, or `all`. Required, unless you use `--all-decks` or `--calibrate`.
- `--swap`: `"Card Out=>Card In"`. Repeat it for several swaps.
- `--games`: games per list per profile (default 1500). `--n` is the same.
- `--seed`: the first seed (default 500000).
- `--jobs`: worker processes (default: every CPU core).
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

## Practice mode

Play one of your decks by hand against three AI opponents from a tier, in your browser. The opponents are the same
decks and AIs as the simulations. The game never suggests moves; it only tells you when a move isn't legal, and why.
After the game, a review shows where your choices and the AI's differed.

```
python3 -m commander_sim.play                                            # opens http://127.0.0.1:8765
python3 -m commander_sim.play --port 9000 --no-browser                   # another port; open it yourself
python3 -m commander_sim.play --text --deck sauron --tier t3             # the same game in the terminal
python3 -m commander_sim.play --lan                                      # also lets a friend on your network join
```

Stop the server with Ctrl+C. Without `--lan` it listens on this computer only (127.0.0.1), so it can't be reached
from a phone or another machine.

**Setting up a game.** Pick your deck and a tier (each shows your simulated win rate against it), then the
opponents (three drawn at random, or pick three), your seat, how much the opponents interact (loose or
conservative), the opponent AI, and a seed. *Opponent AI*: look-ahead is the stronger opponent but thinks for a few
seconds at its decisions; adaptive is much faster. The first game with a set of decks downloads their card images
from Scryfall (about a minute, once); after that they load from `data/images/`.

**Your turn.** Click the table:
- a land or mana rock taps it for mana (a menu asks for the colour when it makes several); click a land you just
  tapped to untap it;
- a card in your hand casts it, or plays it as your land;
- your commander in the command zone, or a card in your graveyard you're allowed to play, casts it;
- a permanent offers its abilities (Equip, loyalty abilities, sacrifice outlets ...);
- **Pass priority** (bottom right) moves on.

Targets and other choices light up on the table: click one, or pick from the list in the panel on the right. To
attack, click your creatures (each gets a sword), then *Attack with N*; to block, click a blocker for the attacker
that's pulsing, or *no block*.

**Opponents' turns** play out one action at a time. Use *Pause*, *Next action*, *1× / 2× / 4×* or *Skip ahead* in
the header. You get priority on every spell an opponent casts (it's shown on the stack, with who has passed), when
you're attacked, and at the end of each turn: respond with instants, counterspells or abilities, or pass.

**The stack.** Spells, activated abilities and triggered abilities all go on the stack, and everyone gets priority in
turn order, as in the real game. You can counter a counterspell, answer an ability (Azorius Guildmage, Tishana's
Tidebinder), or kill a creature before its trigger resolves. When several of your triggers happen at once, you choose
the order (*Same order as last time* when the same ones come up again). **Stop** in the header (and *Stop for priority* on the setup screen) sets when the game asks you:
- *when I can respond* (the default): opponents' spells; abilities and triggers when you hold something to answer
  with; attacks on you; the end of each other turn; and the declare blockers step of your combats when you could
  do something;
- *on every spell, ability and trigger*;
- *at every step (full control)*: also every upkeep, draw step, beginning of combat, declare attackers and
  blockers step, and end step, yours included.

A change applies from your next decision, and Undo and saved games replay it exactly.

**Practice tools** (each can be switched off on the setup screen):
- 💡 **Hint**: what the AI would do now, with the look-ahead's score for each option.
- ↶ **Undo**: takes back your last action (the game replays from its seed, so it's exact).
- 💾 **Save**: keeps the game in `data/saves/`; the setup screen lists saved games to continue or review.

**After the game**, *Review the game* lists each decision where your choice and the AI's differed, with both scores
(where the game stood at the end of your next turn, from −100 lost to +100 won). *Try it* goes back to that decision
with the AI's choice played, so you can see how it goes. *Play this seed again* deals the same game.

**Playing with a friend** (two people, two AI opponents, each person on their own computer on the same network):
1. On your computer, start the server with `--lan`. It prints the address your friend will use.
2. On the setup screen pick *Me and a friend on another computer*, your deck, the tier and the options, then
   *Open the table for your friend*. The page shows a link (`http://<your address>:8765/?join=<code>`) and a
   six-digit code.
3. Your friend opens the link in their browser, picks their deck (a different one from yours) and clicks *Join*.

Seats are drawn at random. Each of you sees the table from your own seat: the other person's hand is a card count,
and a card they tutor is named only on their screen. While the game waits on the other person, your panel says so.
Hint and the review are each person's own. Undo and *Try it* send the game back for both of you, so the other
person is asked first (a banner with *Yes, go back* / *No*). Only the computer running the server can start, save,
load or end games; a saved two-player game, when loaded, waits for your friend to join again with the same deck.
If your friend's page doesn't load, your firewall may be blocking the port (for example `sudo ufw allow 8765/tcp`).
Anyone on the network can open the page, but only someone with the code can take the second seat, so use it on a
network you trust.

How it's built: [documents/practice-mode.md](documents/practice-mode.md).

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

The baseline is the list in each deck's `.md` file (the `## Import list` block), in its owner's folder:
`decklists/JD/` (Jodah, Sauron, Sephiroth, Veyran, Y'shtola) and `decklists/Avery/` (Alela, Galadriel).
`commander_sim/deck_files.py` maps each deck key to its folder and file; to add a deck or move one
between folders, change it there. Point `SIM_DECKS` at another folder (flat, or with the same owner folders) to use
lists kept elsewhere.

To change a list, give the updater the whole new list, as copied from a deck site or typed one card per line:

```
python3 -m commander_sim.update_deck sauron new-list.txt --dry-run    # check it and show what would change
python3 -m commander_sim.update_deck sauron new-list.txt              # write it
python3 -m commander_sim.update_deck sauron -                         # or paste the list, then Ctrl-D
python3 -m commander_sim.update_deck sauron new-list.txt --log "Why."  # also add an "Updated <date>" line
```

It checks the list first (100 cards, singleton, colour identity, bans, every name found on Scryfall) and changes
nothing if there is a problem. Then it rewrites the `## Import list` and `## Decklist by type` sections and
records the new list for the deck guard test. It also reports:
- the cards out and in;
- how completely the simulator models each new card;
- which lines of your strategy text still mention cards that left (it doesn't rewrite prose).

It also works for a pool deck (by its key), where it keeps the tier's Game Changer rule.

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
