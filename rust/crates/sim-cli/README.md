# commander-sim

The simulator's command-line program. It seats decks from `decklists/` against the opponent pools and plays the games
on worker threads (the games themselves are sim-core's). `commander-sim --help` lists every option with its current
text; the parser is `src/args.rs`.

Build it from `rust/` and run it from anywhere inside the repository (it finds the repository by looking for
`decklists/` and `data/`, or set `COMMANDER_SIM_ROOT`):

```
cargo build --release -p sim-cli          # -> rust/target/release/commander-sim
```

## What it runs

```
commander-sim --deck sauron --pool all                          # one of your decks against its own tiers
commander-sim --deck sauron --pool t3 --swap "Spell Pierce=>Negate"   # paired A/B: the list and the swap, same seeds
commander-sim --all-decks --pool all                            # every deck of yours, each against its own tiers
commander-sim --deck seph --pool t3 --analyze                   # how it wins and loses, plan timing, card report
commander-sim --deck seph --pool t3 --trace 12                  # play-by-play of one game (seed offset 12)
commander-sim --deck seph --cards                               # the cards, their tags, source and unmodeled text
commander-sim --calibrate within --pool t6 --games 2000         # pool balance; Tier 6 adds a per-deck speed report
commander-sim --realism --pool t6 --games 2200                  # the realism scorecard (documents/realism.md)
commander-sim --realism --pool t6 --knobs FILE                  # the same with AI knobs from a file
commander-sim --fit --pool t6 --ai adaptive --games 2000        # search the knobs for the values closest to the targets
```

Each of your decks plays only the opponent tiers listed for it in `data/my_decks.toml`; `--any-tier` overrides that
and is used only when JD asks (CLAUDE.md). The loose interaction profile is the default and the only one used.

| Option | |
| --- | --- |
| `--deck KEY` | one of your decks (`src/decks.rs` `MINE`) |
| `--swap "OUT=>IN"` | repeat for several: a paired run of the list and the variant. Runs a preflight first (`--no-preflight`, `--preflight-only`) and prints the swapped cards' breakdown after (`--no-explain`) |
| `--pool t0..t6\|all` | the tier the three opponents come from; `all` = the deck's own tiers |
| `--all-decks` | every deck of yours against its own tiers |
| `--calibrate within\|ordering\|all` | within-tier balance, each tier against the one below |
| `--realism` | with `--pool tN`: the tier's own decks, scored against `data/realism/tN.toml` |
| `--knobs FILE` | AI knobs over the built-in values (`src/knobs.rs`); applies to every run |
| `--fit`, `--fit-scope shared\|deck`, `--fit-gens N`, `--fit-out FILE` | search the knobs (sep-CMA-ES, `src/fit.rs`) and write the best as a knobs file |
| `--games N` / `--n N`, `--seed S` | games per run (default 1500) and the first seed (default 500000; games use S .. S+N-1) |
| `--profile p` / `--profiles a,b` | the AI interaction profile (default loose) |
| `--analyze`, `--trace GAME`, `--cards`, `--brief` | the deep report, one game's log, the card list, no per-axis table |
| `--ai lookahead\|adaptive` | the look-ahead search (default) or the heuristic AI alone (about 60 times faster) |
| `--temp T`, `--jobs N`, `--quiet` | the heuristic AI's randomness multiplier, worker threads (default every core), no progress bar |

Games are seeded, so the same command gives the same numbers. The progress bar goes to stderr, the report to stdout.

## Experiment tools

```
commander-sim swaptest <pool deck> <tier> <games> "Out Card>In Card; Out>In" [jobs]   # swaps in an opponent deck
commander-sim searchtest <deck> <tier> <games> [rollouts] [top_k] [jobs]             # heuristic vs look-ahead
commander-sim pools                                                                  # the pool decks
```

Deck lists come from the deck files (`decklists/JD/`, `decklists/Avery/`, `decklists/pool/t*/`, or `$SIM_DECKS`), in
file order; card definitions, commanders and colour identities from `data/cards.json` and `data/decks.json`. A card
that isn't in `data/cards.json` can't be played: run `cardgen export` after adding one to a list.
