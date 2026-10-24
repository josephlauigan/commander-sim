# commander-sim

A Magic: The Gathering Commander game simulator, in Rust. It plays JD's decks (and Avery's) in four-player games
against pools of outside decks in seven power tiers, t0 to t6, and reports win rates and paired before/after
comparisons for list changes. Practice mode lets a person play one of those decks against the same AI in a browser.
`cardgen` builds the card data and edits the deck lists.

The simulator is being fitted to real games: a realism scorecard compares its games with real ones (seat win rates,
speed, how games are won, deck balance), and changes are judged by whether they move it toward those targets. See
[The realism loop](#the-realism-loop).

## Quick start

From the repository root (macOS or Linux, with `make` and `curl`):

```
make setup                 # installs Rust (rustup) if it's missing, then the pinned compiler (rust/rust-toolchain.toml)
make build                 # builds the three programs into rust/target/release/
eval "$(make -s env)"      # puts the programs and cargo on PATH in this shell
make test                  # the full test suite
```

`make` alone (or `make shell`) opens a shell with the programs on PATH; `make help` lists the targets (`make sim`,
`make practice`, `make cardgen`, `make clean`). `rust/Cargo.lock` pins every dependency. Without make: `cd rust &&
cargo build --release`, and `cargo test --release` for the tests. On this machine, run builds, tests and sims under the
memory cap in [`CLAUDE.md`](CLAUDE.md) (`make build` and `make test` apply it where systemd is available).

The programs are `commander-sim` (simulations), `commander-practice` (practice mode) and `cardgen` (card data and
decks). They run from anywhere inside the repository and find `data/` and `decklists/` themselves (or set
`COMMANDER_SIM_ROOT`).

## Main commands

```
commander-sim --deck sauron --pool all                                  # the deck against each of its own tiers
commander-sim --deck sauron --pool t3 --games 1500                      # one of its tiers
commander-sim --deck sauron --pool t3 --swap "Spell Pierce=>Negate"     # paired A/B: the list and the variant
commander-sim --deck sauron --pool t3 --swap "A=>B" --ai adaptive       # the same, a quick read (seconds)
commander-sim --deck sauron --pool t4 --analyze                         # how it wins and loses there
commander-sim --deck sauron --pool t3 --trace 7                         # play-by-play of game 7
commander-sim --all-decks --pool all                                    # every deck against its own tiers
commander-sim --deck sauron --cards                                     # every card: tags, abilities, Game Changers
commander-sim --realism --pool t6 --games 2200                          # the sim against real-game targets
commander-sim --realism --pool t6 --ai adaptive --knobs my-knobs.toml   # the same with the AI's values from a file
commander-sim --calibrate within|ordering|all --games 240               # pool balance checks
commander-sim pools                                                     # list the pool decks
commander-sim swaptest <pool deck> <tier> 800 "Out>In; Out>In"          # pool-deck list changes, without editing
commander-sim searchtest <deck> <tier> 240                              # a deck with and without the look-ahead

commander-practice                                                      # practice mode at http://127.0.0.1:8765
commander-practice --text --deck sauron --tier t3                       # the same game in the terminal

cardgen deck sauron new-list.txt --dry-run                              # check a new list, show what would change
cardgen deck sauron new-list.txt --log "Why."                           # write it, and log it in the changelog
cardgen export                                                          # rebuild data/cards.json and data/decks.json
cardgen audit --mine                                                    # JD's and Avery's cards that aren't fully modeled
cardgen validate                                                        # check the pool decklists
```

`commander-sim --help` lists every option (`rust/crates/sim-cli/src/args.rs`). The ones that matter most:

- `--deck`: `seph`, `veyran`, `sauron`, `galadriel`, `yshtola`, `alela`, `kefka`, `omnath`, `sheoldred`, `cloud`,
  `thrasios`, `sephfs`, `godo` or `karn`.
- `--pool`: `t0` to `t6`, or `all` (the deck's own tiers). Each deck plays only its own tiers (`data/my_decks.toml`);
  `--any-tier` lifts that, only when JD asks.
- `--swap "Card Out=>Card In"`: repeat for several. Every swap run first does a preflight (`--no-preflight`,
  `--preflight-only`) and ends with the paired breakdown (`--no-explain` turns it off).
- `--games N` (default 1500), `--seed S` (default 500000), `--jobs N` (default every core), `--quiet`.
- `--ai lookahead` (the default: each deck tests its best few plays on copies of the game) or `adaptive` (the heuristic
  AI alone, about 60 times faster). `--temp T` scales the heuristic AI's randomness.
- `--profiles`: `loose` by default and the only profile in use (JD, 2026-10-19). Loose opponents counter and remove more
  freely, the worst case for JD's decks.
- `--realism`: with `--pool tN`, play the tier's own decks and score the games against `data/realism/tN.toml`.
- `--knobs FILE`: the AI's hand-set values (deck styles, counter thresholds, look-ahead weights) from a TOML file
  (`rust/crates/sim-cli/src/knobs.rs`).

How a change to one of JD's decks is tested, and how to read the result: [`documents/testing-decks.md`](documents/testing-decks.md).

## Opponent tiers

`decklists/pool/` holds 36 outside decks: five each in t0 (Bracket 2 precons), t1 (High B2 / Low B3), t2 (Mid B3), t3
(High B3), t4 (Low B4) and t5 (High B4), and six in t6 (cEDH). `decklists/pool/README.md` lists them and the rules each
tier was built to; `decklists/pool/pool-results.md` has the calibration results.

Each game seats one of JD's decks with three decks drawn from one tier, re-drawn every game, in a random seat order.
The draw and every seat's opening shuffle depend only on the seed and the deck keys, so a `--swap` comparison faces
the same opponents and draws.

| JD's decks | Tiers |
| --- | --- |
| yshtola, sephfs, thrasios | t5 |
| sheoldred, sauron, seph | t3-t4 |
| kefka | t3 |
| cloud, veyran, omnath | t1-t3 |
| karn | t4-t5 |
| godo | t6 |
| alela (Avery's) | t1 |
| galadriel (Avery's) | t1-t2 |

## The realism loop

Real data sets the targets: seat win rates (EDHTop16 for cEDH, Playgroup.gg for casual tiers), the median winning
round, the mix of win types, and the spread between a tier's decks. `commander-sim --realism --pool tN` plays a tier's
own decks and prints each statistic with its 95% interval, the target and its band (+/- 1 point for shares), and PASS
or FAIL. Targets are in `data/realism/tN.toml` (Tier 6 and Tier 3 so far), and `.github/workflows/realism.yml` runs the
scorecard every night. The work goes in phases with gates: ground truth, fit and guard, one card model, engine
structures, a generic AI. Today Tier 6 passes 3 of 10 targets and Tier 3 passes 1 of 9: the sim's games are too slow
and their seat edge too flat. [`documents/realism.md`](documents/realism.md) has the statistics, targets, sources,
status and plan, and links JD's plan document.

## Your decks

The baseline is the list in each deck's `.md` file (the `## Import list` block): JD's in `decklists/JD/`, Avery's in
`decklists/Avery/`. `data/my_decks.toml` maps each deck key to its file, display name, commander, colour identity and
opponent tiers; to add a deck, add a table there. `SIM_DECKS` points at another folder of lists.

To change a list, edit the file or give `cardgen deck` the whole new list, as copied from a deck site or typed one
card per line:

```
cardgen deck sauron new-list.txt --dry-run    # check it and show what would change
cardgen deck sauron new-list.txt              # write it
cardgen deck sauron -                         # or paste the list, then Ctrl-D
cardgen deck sauron new-list.txt --log "Why." # also log it (decklists/CHANGELOG.md; pool decks: decklists/pool/CHANGELOG.md)
cardgen export                                # then rebuild the card data
```

It checks the list first (100 cards, singleton, colour identity, bans, every name found on Scryfall) and changes
nothing if there is a problem. Then it rewrites the `## Import list` and `## Decklist by type` sections, records the new
list for the deck guard test, and reports the cards out and in, how completely each new card is modeled, and which
lines of the strategy text still mention cards that left. When the pasted list has set codes and collector numbers
(`1x Sol Ring (sld) 2093 *F*`), it also writes them to `data/printings/<deck>.txt` so practice mode shows those exact
cards. It works for a pool deck too (by its key), keeping the tier's Game Changer rule.

Each deck's strengths, weaknesses and tested swaps are in `audit/<deck>/FINDINGS.md`.

## Practice mode

Play one of your decks by hand against three AI opponents from a tier, in the browser. The opponents are the same
decks and AIs as the simulations. The game never suggests moves unless you ask for a Hint; it tells you when a move
isn't legal, and why. After the game, a review shows where your choices and the AI's differed.

```
commander-practice                                  # opens http://127.0.0.1:8765
commander-practice --port 9000 --no-browser         # another port; open it yourself
commander-practice --text --deck sauron --tier t3   # the terminal client
commander-practice --lan                            # a friend on your network can take a seat
```

Without `--lan` the server listens on this computer only. The iPad app in `ios-shell/` runs the same server inside a
native app (`ios-shell/README.md`). How to play, the setup options, the tools (Hint, Undo, Save, Review) and how it is
built: [`documents/practice-mode.md`](documents/practice-mode.md).

## Where things are

| Folder | What it holds |
| --- | --- |
| `rust/` | The Rust workspace: `crates/sim-core` (engine, rules, AI, card code), `crates/sim-cli` (`commander-sim`), `crates/practice` (`commander-practice` and its page), `crates/cardgen` (`cardgen`). See [`rust/README.md`](rust/README.md) |
| `data/` | `cards.json` and `decks.json` (written by `cardgen export`), their hand-edited sources (`carddb.txt`, `card_overrides.toml`, `audit.toml`, `my_decks.toml`), the Scryfall cache, `printings/`, and `realism/` (the scorecard targets and the pod-game log) |
| `decklists/` | `JD/` and `Avery/` (the decks under test), `pool/` (the opponent tiers), and the changelogs |
| `documents/` | How the project works: [realism](documents/realism.md), [how it works](documents/how-it-works.md), [testing decks](documents/testing-decks.md), [cards](documents/cards.md), [practice mode](documents/practice-mode.md); `archive/` holds the port-era documents |
| `audit/` | Evidence: results of runs, each deck's `FINDINGS.md`, the realism baselines (`audit/realism/`) |
| `ios-shell/` | The iPad app: Swift around the Rust practice server |
| `tests/fixtures/` | The deck guard fixture (`my_decks_parsed.json`) |
| `.github/workflows/` | `ci.yml` (build and test on every push and PR), `realism.yml` (the nightly scorecard) |

## Cards

The engine loads every card from `data/cards.json`, which `cardgen export` builds from `data/carddb.txt` (hand-checked
tags), Scryfall's Oracle text (compiled into abilities) and `data/card_overrides.toml`. Cards that need more get Rust
card code in `rust/crates/sim-core/src/impls/`. `cardgen audit` rates each card Full, Full-auto, Approximate, Partial or
Unmodeled. Details: [`documents/cards.md`](documents/cards.md) and
[`rust/crates/cardgen/README.md`](rust/crates/cardgen/README.md).

## Standing rules

[`CLAUDE.md`](CLAUDE.md) is the rule book for Claude Code sessions here. In brief:

- No Python, in any form; a hook refuses it.
- Run every sim, test and example under the memory cap
  (`systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 <command>`); exit code 137 means it was hit.
- The loose profile only. Each of JD's decks only against its own tiers.
- Swap runs keep their explain breakdown; update `audit/<deck>/FINDINGS.md` after every run.
- Before proposing a card: check Commander legality, colour identity, and that the card going out is in the current
  list.
- Subagents' work is checked (diff read, tests re-run, claims spot-checked) before it is merged or reported.
