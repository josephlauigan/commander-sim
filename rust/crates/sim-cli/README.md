# commander-sim

The simulator's command-line program: the run harness that was `python3 -m commander_sim` (compare.py, poolmode.py,
pools.py, decks.py and the experiment tools), with the games played by sim-core on worker threads. No Python.

Build it from `rust/`:

```
cargo build --release -p sim-cli          # -> rust/target/release/commander-sim
```

Run it from anywhere inside the repository (it finds the repository by looking for `decklists/` and `data/`; or set
`COMMANDER_SIM_ROOT`). The options are the Python harness's, without `--engine`.

```
commander-sim --deck sauron --pool t1 --profile loose --games 960              # one deck against one tier
commander-sim --deck sauron --pool t3 --swap "Spell Pierce=>Negate"            # paired A/B: the list and the swap
commander-sim --deck seph --pool all --profile loose                           # every tier (a deck x tier table)
commander-sim --all-decks --pool all --profile loose --games 960               # your decks x tiers matrix
commander-sim --deck seph --pool t3 --analyze                                  # how it wins and loses, card report
commander-sim --deck seph --pool t3 --trace 12                                 # play-by-play of game 12 (seed 500012)
commander-sim --deck jodah --cards                                             # the cards, tags and Game Changers
commander-sim --calibrate within|ordering|all --games 2000 --ai adaptive       # pool balance checks
```

| option | |
|---|---|
| `--deck KEY` | seph, veyran, sauron, galadriel, yshtola, alela, jodah |
| `--swap "OUT=>IN"` | repeat for several; a paired run of the list and the variant (same seeds, seats and draws) |
| `--pool t1..t5\|all` | the tier the three opponents are drawn from |
| `--all-decks` | every deck of yours against `--pool` (default all) |
| `--calibrate within\|ordering\|all` | within-tier balance, each tier's decks against the tier below |
| `--games N` / `--n N` | games per run (default 1500) |
| `--seed S` | first seed (default 500000; games use S .. S+N-1) |
| `--profiles a,b` / `--profile p` | conservative, loose (default both) |
| `--analyze`, `--trace GAME`, `--cards`, `--brief` | the deep report, one game's log, the card list, no per-axis table |
| `--ai lookahead\|adaptive` | look-ahead search (default) or the heuristic AI alone (about 60x faster) |
| `--temp T` | the heuristic AI's randomness multiplier |
| `--jobs N` | worker threads (default: every core) |
| `--quiet` | no progress bar |

The reports, seats, seeds and statistics are the Python harness's: the same command gives the same numbers as
`python3 -m commander_sim ... --engine rust` did. The progress bar and the estimate go to stderr, the report to stdout.

Experiment tools:

```
commander-sim swaptest <pool deck> <tier> <games> "Out Card>In Card; Out>In" [jobs]   # swaps in an opponent deck
commander-sim searchtest <deck> <tier> <games> [rollouts] [top_k] [jobs]             # heuristic vs look-ahead
commander-sim pools                                                                  # the pool decks
```

Deck lists come from the deck files (`decklists/JD/`, `decklists/Avery/`, `decklists/pool/t*/`, or `$SIM_DECKS`), in
file order; the card definitions, commanders and colour identities from `data/cards.json` and `data/decks.json`.
A card that isn't in `data/cards.json` can't be played: re-export the cards after adding one to a list.

Not available: `--ai rigid` (the Rust engine has no rigid AI) and `--dsl-all` (it needs the Python card compiler).
