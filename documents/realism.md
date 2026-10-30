# Realism: fitting the simulator to real games

The simulator is fitted to real games instead of being tuned against itself. Real data sets the targets: seat win
rates, how fast games are won, how they are won, and how close the decks in a tier are. The realism scorecard
(`commander-sim --realism`) measures the simulator against those targets, and every change to the engine, the AI or the
pool decks is judged by whether it moves the scorecard toward them.

The plan is JD's document [Commander Sim: Improvement and Refactoring
Plan](https://claude.ai/artifact/Fa1usjF6hHZiSUBXwLzGNJ) (a claude.ai Doc). This page summarises it and records where
the work stands. The working handoff is [`rust/agents/handoff-realism.md`](../rust/agents/handoff-realism.md).

## The phases

Each phase has a gate. The next phase starts only when the gate passes.

| Phase | Work | Gate |
| --- | --- | --- |
| 0 Ground truth | The realism scorecard and `--realism`, run in CI; a pod game log; EDHTop16 / TopDeck.gg import; Playgroup data; targets for Tier 6 first, then one casual tier | The scorecard runs nightly with targets for two tiers |
| 1 Fit and guard | Fit the AI's style knobs against the scorecard; generic rule checks (Pact upkeep, tutor then discard); sequential stopping and an interval on every delta | The Tier 6 seat edge is inside its target band |
| 2 One card model | `cardgen export` refuses cards modeled by two paths; combos from Commander Spellbook; model coverage per deck printed on every run | No double-modeled cards; coverage printed per deck |
| 3 Engine structures | A characteristics pass, one replacement step, a mana pool with restrictions, identity for exiled cards, one permanent arena | Paired tests level, scorecard holds, under 10% slower |
| 4 Generic AI | Plan files as data, card values from abilities, a learned evaluation, tree search on high-stakes choices; delete the per-deck hand code | Each deck level with its hand code in a paired test |

## The scorecard

```
commander-sim --realism --pool t6 --games 2200                 # look-ahead AI (the default)
commander-sim --realism --pool t3 --ai adaptive --games 8200   # heuristic AI, about 60 times faster
```

`--realism --pool tN` plays the tier's own pool decks against each other, four to a pod (the same games as
`--calibrate within`, `Ctx::tier_games` in `rust/crates/sim-cli/src/poolmode.rs`). It prints, for each statistic, the
sim's value, its 95% interval, the real target, the band, and PASS or FAIL. Then it prints the number of games, the
decisive share, the timeouts, each deck's share of the decisive games it sat in, and the sources of the targets.
`--pool all`, `--deck`, `--swap` and `--calibrate` are refused with it. A tier with no targets prints the sim's
numbers unscored. Code: `rust/crates/sim-cli/src/realism.rs`; tests in the same file and in
`rust/crates/sim-cli/tests/realism.rs`.

A **decisive** game has a winner that is not the round-cap tiebreak (the round cap is 20, `play.rs`). Every share is a
percentage of decisive games.

| Key | Statistic | How it is computed |
| --- | --- | --- |
| `seat1_win` .. `seat4_win` | Seat 1-4 share of decisive games | Wins from that seat / decisive games; Wilson 95% interval |
| `win_round_median` | Median round of a decisive win | Median of the winning rounds; interval from order statistics |
| `win_round_spread` | Win round, 75th minus 25th percentile | Quantile difference; no interval |
| `win_combo_alt` | Wins by a combo or an alternate win | Every win route from `t6report::win_route` that is not damage, commander damage or poison: a named combo line, or "other" (alternate win, mill) |
| `win_damage` | Wins by damage | Route `damage` (combat or other damage) |
| `win_commander_damage` | Wins by commander damage | Route `commander damage` |
| `win_poison` | Wins by poison | Route `poison` |
| `deck_win_spread` | Best deck minus worst deck, points | Each deck's share of the decisive games it sat in; best minus worst; no interval |
| `games_over_15` | Games over 15 rounds | Games that ended after round 15, as a share of all games |

Combo and alternate wins are scored together because a Thassa's Oracle win is a combo in the sim and often an
alternate win in a player's log. The plan's later statistics (commander casts and removals, wipes and counters per
game, per-commander win rates) are not in the scorecard yet.

## Target files

One file per tier: `data/realism/t0.toml` to `t6.toml`. Each holds `tier = "tN"` and one table per statistic that has a
target:

```toml
tier = "t6"

[seat1_win]
value = 32.5        # the real value, in the statistic's unit
lo = 31.5           # the band the sim must land in
hi = 33.5
n = 387427          # the real sample size
source = "EDHTop16 monthly seat win rates, Apr-Sep 2026, share of seat wins"
date = "2026-10-22" # when the source was read
note = "optional"
```

`value`, `lo`, `hi`, `n`, `source` and `date` are required; `note` is optional. A missing or unknown field, an unknown
statistic, a wrong `tier`, or a value outside its own band is refused before any game is played. A statistic with no
table has no target. Each file's comments say where every number came from and why.

| Tier | Targets |
| --- | --- |
| t6 (cEDH) | Seats, median win round, deck spread: 6 scored targets. The win types are reference values (`scored = false`: casual logs, printed, not scored or fitted) |
| t3 (High Bracket 3) | Seats, median win round, win types: 9 targets. Atraxa is left out of every target (see the file) |
| t0, t1, t2, t4, t5 | None yet; the files hold only `tier` and comments naming the future sources |

Bands (JD, 2026-10-23): every share target is held to its real value +/- 1 point. A sim value that far from its target
can only be told apart from noise with about 8,000 decisive games (about 8,200 games at Tier 6).

`data/realism/pod_games.toml` is the template for logging JD's own pod games (date, seats in turn order, winning seat,
the round it ended, how it was won, notes). Nothing reads it yet.

## The nightly run

`.github/workflows/realism.yml` runs the scorecard every night (07:17 UTC) and on demand ("Run workflow" on the Actions
tab):

- the adaptive AI on every tier, 8,200 games each (about a minute a tier);
- the look-ahead AI on Tier 6, 2,200 games by default (about 90 minutes; the `lookahead_games` input sets it, and
  8,200 resolves +/- 1 point in about 5-6 hours).

The tables go to the run's summary page and to an artifact. A failing target does not fail the job; an error does.
`.github/workflows/ci.yml` builds and tests on every push and pull request.

## Status (2026-10-24)

**Phase 0** is nearly done. The Tier 6 and Tier 3 targets exist and the nightly run is live on `main`. The Tier 3
targets are on branch `realism-casual-targets` (commit `9d98dc0e`); the gate passes once they are on `main` and a
nightly run scores them.

**Tier 6 baseline** (`audit/realism/t6-baseline.md`): 2,200 look-ahead games, loose profile, seeds 500000-502199.
**3 of 10 targets pass** (seat 2, seat 3, poison).

| Statistic | Sim | Target (band) |
| --- | --- | --- |
| Seat 1 / 2 / 3 / 4 | 27.5 / 27.5 / 22.4 / 22.7% | 32.5 / 26.9 / 22.7 / 17.9% (each +/- 1) |
| Median win round | 9 | 7 (6-8) |
| Combo or alt / damage / commander damage / poison | 76.0 / 23.5 / 0.6 / 0.0% | 47.8 / 39.9 / 11.6 / 0.7% |
| Deck spread | 16.3 pts (Kraum/Tymna 32.8%, Rograkh/Thrasios 16.5%) | 4 (0-8) |
| Games over 15 rounds | 11.2% | no target |

The table is too slow, and a slow table flattens the seat edge. This matches `audit/t6/speed-2026-10-19/REPORT.md`:
the decks are draw- and mana-limited, not waiting.

**Tier 3 baseline** (`audit/realism/t3-baseline.md`): 8,200 adaptive games. **1 of 9 targets pass** (poison). The median
win is round 13 against 8, 33% of games run past round 15, combo wins are 1.9% against 20%, and seat 1 wins 26.5%
against 31.3%. Korvold (13.9%) and Marwyn (11.7%) are the weakest decks in the sim, although on Playgroup they win most
often.

**Phase 1, step 1: the knobs as data** (branch `realism-phase1`, commit `538ca94a`). `--knobs FILE`
(`rust/crates/sim-cli/src/knobs.rs`) overrides the AI's hand-set values from a TOML file:

```toml
tier = "t6"            # optional: the tier it was fitted on
[table]
temp_scale = 1.0       # every deck's temperature times this (as --temp); 0.05-3.0
cthresh_default = 6    # outside decks' counter threshold; 0-20
instant_extra = 0      # extra importance instant removal needs before it is used proactively; 0-10
[cthresh]
seph = 6               # one of JD's decks' counter threshold
[search]               # the look-ahead: combo_w, ramp_w, hold_w, top_k, rollouts, horizon
combo_w = 12.0
[style.kinnan-bonder-mana-combo]
temp = 1.0             # 0.05-3.0
aggression = 0.6       # 0-1
caution = 0.5          # 0-1
```

Every value is optional; a value left out keeps its built-in one. Unknown keys and deck keys that are not in
`data/decks.json` are refused. A file that restates every built-in value gives byte-identical output to a run without
one (`audit/realism/knobs/t6-identity-2000.txt` against `t6-none-2000.txt`).

**The knobs screen** (`audit/realism/knobs/README.md`): each knob alone at both ends of its range, for every Tier 6
deck, 2,000 adaptive games each. No setting moves the seat edge (seat 1 stays 25-28% against 32.5%; a seat share is
known to about +/- 2 points at 2,000 games), and none moves the median win round off 9-10.

**The lists are not the cause either** (`audit/realism/t6-lists-2026-10-24/REPORT.md`). Each Tier 6 pool deck was
compared with 8 recent EDHTop16 winning lists of its commander: the pool lists already hold 49-86 of the 51-87 cards
those lists share (6 or more of 8), and the 15 missing ones changed nothing (seat 1 27.0% -> 26.0%, median 10). The
table without Dack Fayden, and with Mox Diamond's land rule relaxed (`[table] t6_moxd_lands`), is as slow, and the sim
mulligans no more than real players of these commanders. The time goes in turning tutors and draws into a finished
line: lines are "one tutor away" at round 6-7 but ready in only 18-45% of games. The AI's option list also offers
plays that Dack Fayden's taxes make unaffordable (the engine refuses them; every refused play in 8 traced games was in
a game with Dack), but the table without Dack is as slow.

**Phase 1, step 2: `--fit`** (`rust/crates/sim-cli/src/fit.rs`). sep-CMA-ES over the knobs scaled to [0, 1], the same
seeds for every candidate, starting from the built-in values; loss = the sum over scored targets of ((sim - target) /
half the band)^2. `--fit-scope shared` fits one style for the tier plus the table knobs, `deck` one style per deck;
`--fit-stats` limits the loss to chosen statistics; the result is a knobs file. The win types count for a quarter in
the loss (`WIN_TYPE_WEIGHT`, JD 2026-10-24: their Tier 6 targets come from casual logs; to be revisited).

```
commander-sim --fit --pool t6 --ai adaptive --games 2000 --fit-gens 15            # every target
commander-sim --fit --pool t6 --ai adaptive --games 2000 --fit-scope deck \
    --fit-stats seat1_win,seat2_win,seat3_win,seat4_win,win_round_median        # the gate's statistics only
```

The first Tier 6 fits (`audit/realism/fit/`) cut the loss from 752 to 326 (shared) and 367 (per deck), almost all of
it in the win types. On fresh seeds (900000-908199, 8,200 games) the win-type shift holds (combo 68.7% -> 59.6%), the
seat edge doesn't move (seat 1 26.0% -> 25.6%), and the median win round gets worse (10 -> 11). The built-in values are
unchanged. Fitting only the gate's statistics (`--fit-stats` seats and median) reached seat 1 28.4% on its own seeds, but 25.6% (shared) and
26.2% (per deck) on fresh ones, the same as the built-in 26.0%: the knobs cannot move the Tier 6 seat edge.

**The first fix that moved the speed** (2026-10-24 night): the AI never sacrificed a commander as a cost, so Rograkh,
Son of Rohgahh, a 0-cost commander real lists sacrifice freely, was never Diabolic Intent's fodder and Rograkh/Silas
sat on the tutor one step from its line. With a 0-cost commander allowed as fodder (`ai::brain::spare_creature`), the
Tier 6 median win round goes from 10 to 9 on the same 8,200 seeds; seat 1 is unchanged. The calibrate-within report now
shows, per deck, what it chooses while one tutor away and which tutors it holds when it passes.

## Data sources

| Source | What it gives | Licence and credit |
| --- | --- | --- |
| [Playgroup.gg](https://playgroup.gg) | `api/public/v1/commanders.json`: per commander (5,142), games, wins, average game length, wins by combat, combo, commander damage, other damage, alternate win, mill, poison. `/api/public/v1/commanders/<id>.json`: average win turn. Commander pages: seat win rates. Turn-order page: 284,950 four-player single-winner games, seats 29.2 / 25.7 / 23.6 / 21.5%. Casual pods logged by players; no bracket field, no per-game export | Stats are CC BY 4.0 (playgroup.gg/legal): credit Playgroup.gg |
| [EDHTop16](https://edhtop16.com/reports/monthly_seat_winrates) | cEDH seat 1-4 win rates and draw rate by month since March 2024. The Tier 6 seat targets are April-September 2026, sample-weighted, each seat's rate divided by the four seats' total | Public page |
| [EDHTop16](https://edhtop16.com) decklists | Each commander page lists its top entries of the last year (events of 50+); each decklist page carries every card's `commanderPlayRate` (the share of that commander's decks running it). Used for the Tier 6 list comparison | Public pages |
| [TopDeck.gg](https://topdeck.gg) | REST API v2 (free key, 100 requests a minute): standings, decklists, round tables; no seat field | Must show a visible credit and link back to TopDeck.gg |

The Tier 6 median win round (7) is JD's speed target (`decklists/pool/TARGETS-t6.md`), consistent with Playgroup's
average win turn for the Tier 6 commanders (6-7). The Tier 6 deck spread (4, band 0-8) comes from EDHTop16's six-month
decisive shares of the five established Tier 6 decks (23.9-27.9%). The Tier 6 win types come from Playgroup's casual
logs of the Tier 6 commanders, not from tournaments. Since 2026-10-24 they are reference values: the scorecard prints
them with their band and the result `reference`, and they are left out of the pass count and of `--fit`. A targets
table takes `scored = false` for that.

## JD's decisions

- **2026-10-23.** The seat targets are EDHTop16's monthly-report shares. Every share target is held to +/- 1 point.
  Win types stay scored, although their source is casual logs. Fitting may overrule any hand-set value or threshold,
  the loose profile's included. Each deck may get its own fitted settings, to close the deck spread.
- **2026-10-24.** Pool decks may be updated to fit or reflect the real data (for example, the Tier 3 Atraxa can become
  the infect deck players actually log, and rejoin the targets).
- **2026-10-24 (later).** cEDH keeps its own metrics: Tier 6 is scored only on tournament-sourced targets (seats,
  deck spread) and the speed target. The win types, whose only source is casual logs, become reference values until a
  cEDH source exists (EDHTop16 and TopDeck.gg do not record how games are won). This replaces the 2026-10-23 "win types
  stay scored" and the quarter weight for Tier 6. Other tiers keep scoring their win types.
- **2026-10-24, overnight (made for JD, to confirm).** Tier 3 is the casual tier for the Phase 0 gate (most of JD's
  decks play it). Atraxa is left out of every Tier 3 target (45% of its Playgroup wins are poison; the pool's Atraxa is
  superfriends). The Tier 3 median win round uses Playgroup's average win turn (8.1, band 7-9). The knobs file is TOML
  with every value optional.
- **Phase 1 route** (JD, 2026-10-24): real lists first, then fit. The default is to adjust the existing pool lists
  toward what real lists of the same commanders run; a list may be replaced outright where that makes the most sense.
  Every decision taken for JD overnight is in `rust/agents/overnight-2026-10-24.md`.
- The standing rules in [`CLAUDE.md`](../CLAUDE.md) still apply: loose profile only, each of JD's decks only against its
  own tiers, no Python, the memory cap.
- **Still open** in the plan document: start logging pod games now (by hand or from practice mode)? Which engine work
  first (the permanent arena or the characteristics pass)? Pause new card work during the card-model move?

**Phase 2, one card model** (2026-10-29; the Commander Spellbook part is a separate PR). The gate "no double-modeled
cards" **passes** (PR 5a, 2026-10-29): `cardgen audit --paths` finds 0 hard, shadowed, competing or undeclared split
doubles, `data/paths_known.txt` is empty, and every card on two paths has a `paths_ok` reason (209 cards).

- `cardgen audit --paths` (PR #153) lists every path a card reaches the sim by (compiled abilities, executing tags,
  card-code rule and AI slots) and classes the doubles. PR #161 removed every hard double (21 -> 0) and declared 29
  deliberate splits with `paths_ok = "why"` in `data/card_overrides.toml`.
- `cardgen export` now refuses a new double and writes nothing: a hard, shadowed or competing double, or an
  undeclared split, with no `paths_ok`. The message names each card, its class and its paths, and the three fixes
  (delete one path, a `dsl` override, or `paths_ok`). The check is `cardgen::paths::guard`; the test
  `the_export_guard_passes_the_committed_data` (`rust/crates/cardgen/tests/paths.rs`) runs it on the committed data,
  so `cargo test` fails on a double even without a re-export.
- The doubles already in the data are listed in `data/paths_known.txt`, one "class | card" line each, and accepted
  only with that class: it started at **21 competing, 25 shadowed, 161 undeclared splits** (207 cards). A hard double
  is never accepted from it. The list only shrinks: the test fails on a line whose card is no longer that double.
- PR 5b judged the 161 undeclared splits: 152 are real splits (two paths for different parts of the card) and got
  `paths_ok`; the 45 lands among them are compiled abilities that never run for a land (a land is no permanent in
  the sim) next to City of Traitors' `land_etb` check, which is set on every land. 9 were real doubles.
  `sim-core/tests/paths_splits.rs` checks the splits that act on the same event happen once.
- PR 5a cleared the rest, and **the list is now empty** (0 competing, 0 shadowed, 0 undeclared splits).
  - The 15 competing lands, Dragon's Hoard, Weathered Wayfarer and Desperate Ritual lost a compiled ability that
    never ran or was wrong (`dsl = []`; `dsl::ability_options` never offered a land's compiled activation).
  - Demon of Fate's Design, Gingerbrute, both Staffs, Damn and 23 shadowed spells are declared splits: two abilities
    with one path each, or a hand-written resolve whose partial compiled spell stays as the AI's value of the card
    until the AI phase.
  - The 9 doubles from PR 5b keep their card code: the enters tag is gone (Battle Angels of Tyr's `treas`; `tok` on
    Karn, Scion of Urza, Pawn of Ulamog, Requiem Angel, Sifter of Skulls and Springheart Nantuko; `lr` on
    Sakura-Tribe Elder and Sowing Mycospawn), and Doubling Season's `etb` hook is deleted (the engine's enter
    doubles a planeswalker's loyalty once). `sim-core/tests/paths_competing.rs` checks each effect happens once.
- Every `commander-sim` run that plays decks (`--deck`, `--pool`, `--all-decks`, `--calibrate`, `--realism`,
  `--goldfish`) prints the model coverage first: for each deck, its 99 + commander by `cardgen audit` status (Full,
  Full-auto, Approximate, Partial, Unmodeled) and up to five of its Unmodeled and Partial cards (commander and engine
  cards first); a `--deck` run's opponent tiers get one summed line each. `--quiet` keeps it. It is read from the card
  data once per run and plays no game, so results are unchanged (`rust/crates/sim-cli/src/coverage.rs`).

**Phase 3, engine structures, PR 1: one permanent arena** (2026-10-29, branch `phase3-arena`; gate changed by JD to
"the test suite passes", no sims until Phase 4). Every permanent, land or not, is a `Perm` in `Game::perms` with one
id type (`PermId`): the `Land` type, `Game::lands` and `LandId` are gone. `Player::lands` and `Player::perms` stay two
lists in the same order as before. The ability language now reads lands (after the other permanents): their compiled
triggers, statics and activated abilities run, so the engine's `land_etb_fx` is deleted (temples and gain lands use
their compiled enter trigger; the four surveil lands now surveil), eight lands' activated abilities are offered
(Castle Ardenvale, High Market, Maelstrom of the Spirit Dragon, Skemfar Elderhall, Hanweir Battlements, Room of
Refuge, Sinister Hideout, Urza's Factory), and Seraph Sanctuary's Angel trigger runs. Fetches keep the `f` tag's
`crack_fetch`, channel lands channel from the hand only, Treasure Vault lost its compiled ability (`dsl = []`: the
language has no {X} activation cost). Design and the decks touched: `audit/phase3/arena-design.md`,
`audit/phase3/arena-handoff.md`.

## What's next

1. Finish Phase 0: put the Tier 3 targets on `main` and let the nightly run score them. The same method (Playgroup's
   win types, seat blocks and average win turn for each pool commander) fills the other casual tiers.
2. Phase 1, the speed: make the AI price costs as the engine charges them (taxes such as Damping Sphere): the AI
   starts lines and plays it then can't pay for (`rust/agents/overnight-2026-10-24.md`, items 16 and 21); then Sisay's
   and Rograkh/Thrasios's lines (both win at a median round of 13), Dack Fayden's 35%, and Kraum/Tymna's held lines.
3. Re-fit after each fix (`--fit`, fresh-seed check), and confirm with the look-ahead AI at about 8,200 games; write
   `audit/realism/t6-fit-<date>.md`. JD to decide whether the win types keep full weight in the loss.
4. Once Phase 1 passes, make a target miss fail the nightly job.
5. Optionally, a reader for `pod_games.toml` that checks the records and summarises seats, rounds and win types.
