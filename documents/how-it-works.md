# How it works

What a new reader needs to know about the simulator: the engine, how cards are modeled, the AI, and how runs are
set up. The code is the reference; each section names the files to read next. The older, longer description from the
end of the port is [`documents/archive/architecture.md`](archive/architecture.md): many of its details still hold,
but check them against the code.

## The pieces

```
rust/crates/sim-core/   the library: game state, rules, AI, card code
rust/crates/sim-cli/    commander-sim: runs games on worker threads and reports statistics
rust/crates/practice/   commander-practice: a person plays a seat (documents/practice-mode.md)
rust/crates/cardgen/    cardgen: builds data/cards.json and data/decks.json, edits decks, audits cards
```

There are four layers. **Card data** says what each card does. **The engine** holds the game and applies the rules.
**The AI** makes every decision. **The run harness** (sim-cli) plays many seeded games and turns them into numbers.

## The engine

`sim-core/src/state.rs` holds the game: `Game` (the table), `Player` (a seat: zones, life 40, commander tax,
commander damage taken, poison), `Perm` (a nonland permanent or token) and `Land`. Objects refer to each other by
small ids (`ids.rs`), so a `Game` is plain data and copying it is cheap. The look-ahead copies games all the time.

The rules are in `sim-core/src/engine/`: `turn.rs` (setup, mulligans, the turn loop and its steps), `cast.rs` (casting
and resolving), `stack.rs` (the stack, triggers, priority, counterspells), `mana.rs`, `combat.rs`, `removal.rs`,
`zones.rs`, `life.rs` (damage, state-based checks, elimination), `tutors.rs` and `hooks.rs` (calling card code).

- **A game** is up to 20 rounds of four seats (`play.rs` passes the cap). Unresolved after round 20, it ends as a
  timeout and the living player with the highest life plus twice their creature power wins.
- **Mulligans.** The AI keeps seven cards with 2 to 5 lands, else draws seven again. Cloud and the Tier 6 decks have
  their own keep rules (`impls/cloud.rs`, `impls/t6_plan_*.rs`).
- **The stack is real.** Spells, activated abilities (except mana abilities) and triggers go on the stack, and
  players get priority in turn order. A player with nothing affordable at instant speed passes at once, so most
  priority passes cost nothing.
- **Mana** is computed from sources rather than kept in a pool for the AI (`mana_units`, then a greedy payment plan,
  `plan_pay`). A person in practice mode has a real mana pool.
- **Players lose** at 0 life, after drawing from an empty library, at 21 commander damage from one commander, or at
  10 poison counters. A combo that resolves wins the game (win type `combo`).
- **Guards** in `settings.rs` keep every game finite: 20,000 engine steps a game (`game_work`; normal games use under
  1,000), 150 permanents on the table before the look-ahead stops (`board_limit`), 250 creature tokens a player
  (`engine/zones.rs`), and caps on look-ahead work per game. Interruptions are values (`flow.rs`: `Res`, `Stop`), not
  panics.

## The card model

Every card the engine knows is in `data/cards.json`, and every deck in `data/decks.json`. `cardgen export` writes both.
A card's definition comes from the first of these that has it:

1. `data/carddb.txt`: hand-checked tags, one line per card (`Grim Tutor|S|1BB|tut=any lose=3`).
2. Scryfall (`data/scryfall_cache.json`): the Oracle text compiled into abilities (the ability language), or, when
   nothing compiles, turned into tags by regular expressions.
3. `data/card_overrides.toml` then replaces a card's tags, types, cost or abilities and holds the audit's verdict.

Cards that need behaviour none of these can express get Rust card code in `rust/crates/sim-core/src/impls/`: a
function registered by card name and event (`hooks.rs` lists about 80 events: triggers, cost changes, cast locks,
options for the AI, resolve). `impls/mod.rs` registers the files in a fixed order. The ability-language interpreter is
`sim-core/src/dsl/`. Tags are listed in `sim-core/src/tag.rs`.

[`documents/cards.md`](cards.md) has the pipeline, the model statuses and how to add a card.

## The AI

Every seat is played by the AI, in two modes (`--ai`):

- **`adaptive`**: the heuristic AI (`ai/brain.rs`). For each main-phase decision it lists every legal play with a
  utility, then samples an order from a softmax of utility over temperature and makes the first play that works. A
  card's utility starts from its deck's priority for it and is adjusted for the board (ramp is worth less late, card
  draw less under pressure, blockers more in danger, bombs less into open counter mana). Hidden cards are estimated
  from public information only.
- **`lookahead`** (the default): the heuristic AI plus search (`ai/search.rs`). For main-phase plays, the first combat's
  attack plan and counterspells in the active player's main phase, it takes the best `top_k = 5` options plus "stop",
  plays each on `rollouts = 6` copies of the game (opponents' hands re-dealt from what the player can't see) to the end
  of the player's next turn (`horizon = 1`), and scores the result. Blocks, protection and the other responses stay
  heuristic. It is about 60 times slower than `adaptive`.

**Deck plans.** JD's and Avery's decks have hand-written priority functions and plans (`ai/decks.rs`, `ai/seph.rs`,
`ai/veyran.rs` and the deck files in `impls/`). Pool decks use the generic rules in `ai/pool.rs` plus per-deck entries
in `ai/plans.rs` (key cards, tutor wish lists, custom priorities). The seven Tier 6 decks (the six pool decks and
`godo`) also have plan files `impls/t6_plan_<deck>.rs` and shared cEDH table rules in `ai/t6_table.rs` (who is about
to win, when to spend answers, when to go for the win).

**Styles.** Each deck has a style (`ai/mod.rs`): `temp` (randomness), `aggression` (pressure) and `caution` (holding up
interaction, keeping blockers home). JD's decks have their own; pool decks default to 1.0 / 0.6 / 0.5
(`ai/pool.rs`, `DEFAULT_STYLE`).

**Profiles.** The interaction profile sets how readily the AI counters and removes (`settings.rs`). Loose, the default
and the only one in use: a spell needs importance 6 to be countered, and instant removal is used as freely as
sorcery-speed removal. Conservative: importance 7, and instant removal needs 2 more. Sephiroth's threshold is 6 in both.

**Knobs.** `--knobs FILE` overrides these hand-set values (styles per deck, `temp_scale`, the counter thresholds,
`instant_extra`, Tier 6's Mox Diamond land rule `t6_moxd_lands`, the look-ahead's `combo_w`, `ramp_w`, `hold_w`,
`top_k`, `rollouts`, `horizon`) from a TOML file. The
format is in `rust/crates/sim-cli/src/knobs.rs` and [`documents/realism.md`](realism.md). Style overrides go into
`Settings::styles`, which is read before the built-in tables. `--fit` (`rust/crates/sim-cli/src/fit.rs`)
searches them against a tier's realism targets and writes the best as a knobs file.

## The run harness

`commander-sim` (`rust/crates/sim-cli/`) seats one of JD's decks with three pool decks drawn from one tier, in a random
seat order, and plays `--games N` seeds (default 1500, starting at `--seed 500000`) on `--jobs` threads (default every
core). Results are merged in seed order, so they don't depend on the number of threads.

- **Seeds.** Each game uses named random streams: `pool:{seed}` draws the opponents and the seat order
  (`sim-cli/src/pyrand.rs`), `lib:{seed}:{deck key}` shuffles each seat's library and runs its mulligans, and
  `play:{seed}` drives every in-game decision. The same command gives the same numbers on any machine.
- **Paired runs.** The seat draw depends only on the seed and the deck keys, and each seat shuffles from its own
  stream. So a `--swap` variant faces the same opponents, in the same seats, with the same opponent draws, and the
  report gives the paired difference with its noise band. [`documents/testing-decks.md`](testing-decks.md) has the
  details.
- **The loose profile only.** `--profiles` defaults to `loose` (JD, 2026-10-19), the more interactive table and the
  worst case for JD's decks. Report one number per tier.
- **Tiers.** The pool is `decklists/pool/`, 36 decks in seven tiers: t0 Bracket 2 precons, t1 High B2 / Low B3, t2 Mid
  B3, t3 High B3, t4 Low B4, t5 High B4 (five decks each), and t6 cEDH (six decks). `decklists/pool/README.md` lists
  them. Each of JD's decks plays only against its own tiers (`tiers = [...]` in `data/my_decks.toml`); a tier outside
  them is refused unless `--any-tier`.
- **Pool checks.** `--calibrate within` plays a tier's decks against each other (a deck winning over 35% or under 15%
  of its games is flagged); `--calibrate ordering` plays each deck against three from the tier below. `--realism`
  scores the `within` games against real targets ([`documents/realism.md`](realism.md)).

Reports are in `sim-cli/src/report.rs` and `stats.rs`: win rates with a Wilson 95% interval (about +/- 2.5 points at
1,500 games), paired differences from per-seed pairs, and seven strength axes (outcome, mana and consistency, speed,
card flow, interaction, resilience, threat and pressure).

## Simplifications

The simulator abstracts Magic on purpose. Read its numbers comparatively (list against list, tier against tier), and
read the realism scorecard for how far its games are from real ones.

- No politics or deals. Blocks are heuristic.
- The look-ahead sees one turn cycle ahead; longer plans rest on the heuristic AI and the evaluation's combo-progress
  term.
- Combos resolve as a unit: once the pieces are together and survive one interruption window, the result applies.
- Mana payment is greedy and can occasionally tap a source a later spell that turn needed.

## Tests

`cargo test --release` in `rust/` runs every test (about 3,000; about 10 minutes), under the memory cap in
[`CLAUDE.md`](../CLAUDE.md). `-p sim-core`, `-p sim-cli`, `-p practice` or `-p cardgen` runs one crate. The card tests
are `sim-core/tests/cards_*.rs`; rules tests are `rules.rs`, `stack.rs` and `game.rs`; practice mode's are
`sim-core/tests/play_*.rs` and `practice/tests/`. `sim-core/src/testkit.rs` builds positions for tests. Examples are in
`sim-core/examples/` (`all_decks`, `bench_games`, `clone_cost`, `debug_game`, `play_game`).
