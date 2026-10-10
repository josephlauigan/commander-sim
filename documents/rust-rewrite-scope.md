# Rust rewrite: scope of work

This started as a scoping outline; the port is now under way on branch `rust-port`. The scope is **option B**: the
simulation core *and* practice mode move to Rust (section 4). The document lays out what has to be ported, in what
order, how to check the result, what could go wrong, and the decisions still open. Numbers are from the code as of
10 October 2026 (branch `jodah-final`).

## Contents

1. [Summary](#1-summary)
2. [Why consider it: where the time goes](#2-why-consider-it-where-the-time-goes)
3. [What exists today](#3-what-exists-today)
4. [Scope: option B](#4-scope-option-b)
5. [Design of the Rust version](#5-design-of-the-rust-version)
6. [Work breakdown](#6-work-breakdown)
7. [Validation: how to trust the new version](#7-validation-how-to-trust-the-new-version)
8. [Risks](#8-risks)
9. [Decisions still open](#9-decisions-still-open)
10. [Path and milestones](#10-path-and-milestones)

---

## 1. Summary

- **What would be ported:** about 32,400 lines of Python.
  - The simulation core is about 27,500: the engine, the turn loop, the AI, and 18,000 lines of card implementations.
  - Practice mode is about 4,900: the human seat's choices, rules checks, mana pool, sessions, Undo, saved games,
    Hint, the AI comparison and the server.
  - The browser page (HTML, CSS and JavaScript) stays as it is.
- **What it buys:**
  - **Simulations:** about 30–100 times faster per game. Today's ~2,300 look-ahead games an hour (22 workers) could
    reach roughly 70,000–200,000.
  - **Practice mode:** the AI opponents, Hint and the AI comparison all think much faster, on a computer and above all
    on the iPad.
  - **One engine:** the simulation and practice mode run the same rules and AI.
- **The iPad app:** with the server in Rust as well (section 4, recommended), the app becomes a small native shell (a
  web view in Swift) around a Rust library. Python no longer runs on the iPad, and the Briefcase app is retired.
- **Effort:** this is a re-implementation, not a translation (section 5).
  - By hand: roughly 6–10 months for one experienced developer.
  - With Claude doing most of the writing, as with the current code: more like 6–11 weeks of sessions.
  - Validation, not writing, takes most of that time.
- **The main risk is trust, not speed.** The Rust version will play slightly differently in hundreds of small ways.
  Every win rate has to be checked again before the new numbers replace the old ones. Saved practice games from the
  Python version won't load in the Rust one (section 9).
- **How it proceeds:** a Rust pilot slice first, in milestones M0–M6 with a check-in after each, then a go / no-go
  call on the measurements (section 10).

---

## 2. Why consider it: where the time goes

Profile of two Sephiroth vs Tier 3 games, look-ahead AI, loose profile (cProfile; it slows everything about equally):

| Share of time | Where | What it is |
|---|---|---|
| ~100% | look-ahead playouts | 352 decisions made 9,060 playouts; the real game itself is negligible |
| ~55% | `brain.main_options` (in playouts) | listing and scoring every legal play |
| ~20% | `can_pay`, `plan_pay`, `mana_units` | working out whether and how mana can be paid (826,000 calls) |
| ~15% | `search.clone` (`deepcopy`) | copying the game for each playout |
| ~14% | `brain.Situation` | reading the board at every decision (232,000 times) |
| ~13% | `brain.hook_options` | card-specific ability options |

(The shares overlap: some of these run inside others.)

The time is spread across branchy, object-heavy game logic. There is no single hot loop to move into Rust on its
own, so a partial port (Rust for one function, Python for the rest) would gain little: crossing between the two
languages hundreds of thousands of times per game would eat the saving.

Practice mode has the same cost: an AI opponent's turn, a Hint and each AI-comparison entry are look-ahead
decisions. The iPad's slow AI is this same problem.

---

## 3. What exists today

### Code to port

| Part | Python lines | What it is | Porting difficulty |
|---|---|---|---|
| `engine.py` | 3,117 | game state, mana, casting, the stack, counters, removal, zones, triggers, elimination | High: the core everything else touches |
| `ais.py` | 2,885 | game setup, mulligans, the turn loop, combat, your decks' plans | High: combat and the deck plans are full of special cases |
| `ai/` (brain, search, pool_ai, deck_plans, gc_prio, pool_decks) | 2,111 | the heuristic AI and the look-ahead | Medium: well contained |
| `cards/dsl.py` | 947 | the ability-language interpreter (58 effect kinds) | Medium |
| `cards/cardimpl.py` | 336 | the hook registry and dispatch | Medium: becomes a different mechanism |
| `cards/impl/*.py` | 18,065 | Python implementations of 658 cards (hooks on about 70 engine events), plus 79 hand-written cast priorities | High by volume: the bulk of the work |
| **Simulation core** | **~27,500** | | |
| `play/` rules side: `human.py`, `legal.py`, `mana.py`, `combat.py`, `choices.py`, `cards.py`, `abilities.py`, `controller.py` | 3,040 | the human seat: priority, legal-move checks, the mana pool, combat declarations, smaller choices, your decks' activated abilities by the rules | High: `cards.py` alone is 1,327 lines of card-by-card abilities |
| `play/` services: `session.py`, `view.py`, `advisor.py`, `shadow.py`, `catalog.py`, `images.py` | 1,130 | the game thread and its events, what each seat may see, Hint, the AI comparison, the setup screen's data, card images | Medium |
| `play/server.py`, `__main__.py`, `text.py` | 890 | the HTTP server (two-player over the local network too) and the terminal client | Medium |
| **Practice mode** | **~4,900** | | |
| Places in the engine and card code that ask the human seat (`human_choice`, `is_human`) | 97 call sites | where a person, not the AI, makes a choice inside the rules | Ported along with the code around them |

### Code that stays as it is

- **The browser page** (`play/static/`: about 1,270 lines of HTML, CSS and JavaScript). The Rust server keeps the
  same HTTP routes and JSON, so the page doesn't change. That also makes the server's behaviour easy to check
  (section 7).
- **Card data** (Python): `carddb.py`, `sources.py`, `scryfall.py`, `autotag.py`, `dsl_parse.py` and
  `pool_cards.py`, about 1,700 lines. They turn card names into card definitions: tags, compiled abilities and
  keywords. A Python export step writes every definition to one `cards.json` for Rust to load, so the Oracle-text
  compiler isn't ported.
- **Deck tools** (Python): `update_deck.py`, `pool_audit.py`, `pools.py --validate`, the deck files.
- **Running experiments and reports** (Python): `compare.py` and `poolmode.py` keep their statistics and printing,
  and call the Rust engine for the games (section 4).

### Tests

There are 676 tests in 50 files: 463 for the simulation and 213 for practice mode (22 `test_play*` files). Most build
a small table (`tests/table.py`), set up a position, and check the result, reaching into engine internals. They can't
run against Rust as they are:

- The simulation tests have to be ported, or Rust has to expose a scenario API close enough to `table()` and `perm()`
  for the Python tests to drive it.
- Many practice tests can be rewritten against the HTTP API instead. Those then test the Python and Rust servers
  alike.

### Things that make the port harder than the line count suggests

- **Loose, dynamic state.** There are 548 `getattr(obj, 'name', default)` calls and 565 uses of `.data`: state
  attached to the game, players and permanents on the fly. In Rust every one becomes a declared field, or an entry in
  a typed per-card state.
- **Tables keyed by object identity.** End-of-turn pumps, once-per-turn flags and similar tables are keyed by `id()`
  of a Python object. The look-ahead's `clone` re-keys them for each copy. In Rust they become ids into the game's own
  arrays, which is simpler, but every one has to be found.
- **Module-level globals.** `engine.CUR_G`, `PAY_FOR`, `LAST_COUNTER`, `AI_MODE`, the profile tables and others are
  set and restored around calls. They have to move into a context passed explicitly. (This is also why practice
  mode's AI comparison can't play copies on a second thread today; Rust removes that limit.)
- **207 deferred imports** (`importlib.import_module`) work around circular dependencies between modules. The Rust
  version needs a module layout without those cycles.
- **Random numbers.** Games are seeded with Python's `random.Random` (Mersenne Twister, seeded from strings such as
  `'play:500001'`) and use its `shuffle`, `random` and `choice`. Matching Python's games exactly would mean
  reproducing that generator and its seeding bit for bit (section 7).
- **The engine waits on a person.** In practice mode, the engine runs on a worker thread and stops at each of the
  human seat's decisions until the answer arrives. Undo and saved games rely on it: they replay the seed with the
  recorded answers, and the look-ahead's recorded decisions (the search tape). The Rust engine must keep the same
  shape: deterministic from a seed and a list of answers.

---

## 4. Scope: option B

| | Moves to Rust | Stays |
|---|---|---|
| **Simulation core** | engine, turn loop, combat, AI and look-ahead, ability-language interpreter, card implementations | card data export (Python), deck tools (Python) |
| **Practice mode** | the human seat's rules side, sessions and events, Undo and saved games, Hint, the AI comparison, the setup screen's data, card images, the server, two-player over the network | the browser page, unchanged |
| **Experiments** | the games themselves | the run harness and reports in Python, calling Rust |

Option A (core only, practice mode stays in Python) was set aside because it leaves two engines that must be kept
in step forever. Option C (everything, including card data and reports) rewrites code that has no speed problem.

### How the pieces connect

- **Simulations:** a Python module built in Rust (PyO3 and maturin). `poolmode.py` calls
  `rust_sim.play_pool_game(seed, seats)` and gets the result back. Each call is a whole game, so crossing languages
  costs nothing. A run can also hand Rust a whole batch of seeds, so the parallelism is in Rust (rayon) instead of
  Python's process pool. During the transition a flag (`--engine rust|python`) picks the engine.
- **Practice mode — the server in Rust too ("B2", chosen 10 October).** The practice server becomes a Rust program,
  serving the same page and the same JSON API. Python isn't needed to play.
  - **Computer:** `commander-practice` (a Rust binary) replaces `python3 -m commander_sim.play`.
  - **iPad:** a small Swift app with a web view, linking the Rust library. The app calls one function to start the
    server on the iPad, then shows `http://127.0.0.1:<port>`, which is what the Briefcase app does today. Building a
    Rust library for iOS and calling it from Swift is a well-trodden path. A Rust-built Python module inside a
    Briefcase app (B1, below) is not.
- **The alternative, "B1": keep the server in Python.** The Python server and session code drive the Rust engine
  through PyO3. Less to port (about 1,000 lines fewer), but the iPad app would need the Rust module built for iOS
  inside Briefcase, which is an untested path, and Python stays on the iPad. This choice is decision 1 in section 9.

---

## 5. Design of the Rust version

These are the main design choices. They are what makes this a re-implementation rather than a line-by-line port.

### The game

- **Card definitions:** one `CardDef` per card (types, cost, tags, abilities, keywords), loaded from `cards.json`
  and referred to everywhere by a small `CardId` number. The tags (247 distinct names in use) become an enum and bit
  flags with numeric fields, instead of a string dictionary consulted millions of times.
- **Game state:** a single `Game` struct with plain arrays: players, permanents (in a slot array with `PermId`
  handles), zones as `Vec<CardId>`. Nothing points at anything else directly, only by id, so the whole game is one
  `#[derive(Clone)]` value. That is what makes the look-ahead's copies cheap: copying a game becomes copying a few
  arrays.
- **Per-card state:** the `.data` dictionaries become a typed per-card state (an enum with one variant per card that
  needs state), or a small map from a fixed set of keys to numbers where that is enough.
- **Card behaviour:** the hook registry (`@on('Card Name', 'event')`) becomes a table from `CardId` to a struct of
  function pointers, one per event the card handles. Each of the 658 hand-implemented cards becomes a set of Rust
  functions. Grouping by file (`t1.rs` … `t5.rs`, `mine.rs` and so on) can follow the Python layout, to keep the
  two easy to compare.
- **The ability language:** the compiled ability data becomes Rust enums, with an interpreter over them. The
  compiler stays in Python.
- **Determinism:** ordered maps (`IndexMap`) wherever iteration order matters, stable sorts, and seeded generators
  passed explicitly. Nothing depends on memory addresses, since there are only ids.
- **Guards:** the step caps (`GAME_WORK`, `PLAYOUT_WORK`, `GAME_SEARCH_WORK`, `BOARD_LIMIT`) carry over unchanged.

### Who decides: one interface for the AI and the person

- Every decision point in the engine — the 97 human-choice sites, the main phases, attacks and blocks, the
  responses — calls a `Controller` for that seat. The AI controller decides on the spot. The human controller turns
  the decision into a `Request` (the same kinds, prompts and choices as today's) and blocks until the answer comes
  back.
- In a simulation every seat has the AI controller. The check "is this seat a person?" becomes "which controller does
  this seat have?", so practice mode code is inert in simulations, as it is now.
- The look-ahead's copies always use AI controllers for every seat, as `search.clone` arranges today.

### Practice sessions

- **The game thread:** a session runs its game on its own thread. Events (log lines, turns, requests, the end of the
  game) and the views of the table go to the server over a channel, and answers come back over another.
- **Views:** what each seat may see is built as the same JSON as `view.py` produces now. Hidden information stays
  out of it.
- **Undo and saved games:** a game is its seed, its settings, the answers given and the search tape. Replaying them
  rebuilds it exactly, as now. Rust only has to be deterministic with itself, not with Python.
- **Hint and the AI comparison:** both run the look-ahead on a copy of the game. With no global state, the comparison
  can run on its own threads while the person thinks. Today it has to squeeze in a playout at a time on the engine
  thread.
- **The server:** the same routes as `server.py` (`/api/options`, `/api/state`, answers, setup, save and load, Undo,
  two-player join), on a small Rust HTTP library. Seats are found by cookie, as now, and `--lan` for two players
  still works.
- **Card images:** found on Scryfall and cached under `data/images/`, with the same index format, so images already
  downloaded keep working.

---

## 6. Work breakdown

Each phase ends with something that can be checked. Sizes are relative: S is a few sessions, M about a week of
sessions, L two weeks or more.

### Part 1: the simulation core

#### Phase 0 — Decisions and baseline (S)

- Settle the decisions in section 9.
- Record the baseline: the Python version's speed, and a reference set of results for validation (the calibration
  runs and your decks × tiers matrix, with seeds).
- Decide which retired code doesn't need porting: Marchesa (`impl/marchesa.py`), Najeela's paths, GAA (retired),
  and the old non-look-ahead branches marked in the code.

#### Phase 1 — Card data export and the data model (M)

- A Python script that writes every card definition in use (all decks, all pools) to `cards.json`: types, cost,
  tags, compiled abilities, keywords, Game Changer flag.
- Rust: `CardDef`, the tag enum, loading and checking `cards.json`; `Game`, `Player`, `Perm`, `Land`, zones; the
  `Controller` interface (section 5), with only the AI controller for now.
- **Done when:** Rust loads every card the decks use, and a round-trip check matches the Python definitions field
  by field.

#### Phase 2 — Rules engine (L)

Port `engine.py`: mana sources and the payment planner, costs and cost changes, casting, the stack and responses,
counterspells, removal and wipes, entering and leaving the battlefield, triggers, damage, life, commander damage and
tax, drawing, tutors, state checks, elimination, the step caps. Each human-choice site becomes a `Controller` call.

- **Done when:** the rules tests (`test_rules`, `test_stack`, `test_triggers`, `test_steps`, `test_land_entry` and
  others) pass in their Rust form.

#### Phase 3 — Turn loop and combat (M)

Port the non-deck parts of `ais.py`: setup, seating and mulligans, the turn steps, upkeep and end step, land play,
combat (attacker choice, splitting attacks, blocks, combat damage, attack triggers, taxes, extra combats).

- **Done when:** a game between decks made only of tag-modeled cards (no hooks) plays to the end, and its
  play-by-play is sensible next to Python's.

#### Phase 4 — Ability-language interpreter (M)

Port `dsl.py`: the 58 effect kinds, targeting, triggered and static abilities, power and toughness from statics,
keyword checks, activated-ability options and the value estimate the AI uses.

- **Done when:** `test_dsl` passes in Rust, and every card that has only compiled abilities (no Python hook)
  behaves the same in the scenario tests.

#### Phase 5 — The AI (M)

Port `brain.py`, `pool_ai.py`, `deck_plans.py`, `gc_prio.py`, `pool_decks.py` and the priority functions of your
decks in `ais.py`, then `search.py`. Cloning becomes `game.clone()`; hiding hidden information, the playouts and the
evaluation port as they are; the search tape comes along for Undo.

- **Done when:** with a fixed position, the Rust heuristic lists the same options with the same scores as Python
  (a differential test, section 7), and `test_search` passes.

#### Phase 6 — Card implementations (L, the biggest phase)

Port the 658 hand-implemented cards, about 18,000 lines. Do it deck by deck so that results become usable step by
step:

1. The shared files: `common`, `rules`, `rules2`, `lands`, `combos`, `topdeck`, `partials`, `fixes`.
2. One tier at a time (`t1` … `t5`); a tier is usable once all five of its decks are complete.
3. Your decks: `mine`, `jodah`, `galadriel`, `yshtola`, `alela`, and the Seph, Veyran and Sauron plans in `ais.py`.

- **Done when:** `pool_audit.py` (pointed at the Rust card table) reports every card in use as modeled, and the
  card tests (`test_my_cards`, `test_game_changers`, the per-deck test files) pass in Rust.

#### Phase 7 — Running experiments (S–M)

Build the PyO3 module and connect it to the run harness: `--engine rust`, parallel batches in Rust, per-game results
back to `poolmode.py`'s statistics, `--trace` play-by-play, `--analyze`.

- **Done when:** every simulation command in the README works with `--engine rust`.

#### Phase 8 — Validating the simulation (M–L)

Section 7, steps 1–4. This is where the time goes: finding and fixing every difference that moves a win rate.

- **Done when:** the acceptance checks in section 7 pass. Rust becomes the default simulation engine.

### Part 2: practice mode

#### Phase 9 — The human seat's rules (L)

Port the rules side of `play/`: `human.py` (priority, playing lands, casting, abilities, passing), `legal.py` (the
legal-move checks and their reasons), `mana.py` (the human mana pool: tapping sources, paying from the pool,
emptying between steps), `combat.py` (declaring attackers and blockers), `choices.py` (mulligans, discards,
sacrifices, tutors, scry and surveil), and `cards.py` and `abilities.py` (your decks' activated abilities by the
rules). Then the human `Controller`, and the requests it sends at each of the 97 human-choice sites.

- **Done when:** the text client (a small Rust version of `text.py`) can play a full game against three AI seats,
  and the practice tests for these modules (`test_play_hand`, `test_play_mana`, `test_play_combat`,
  `test_play_choices`, `test_play_abilities`, `test_play_cards`, `test_play_respond`, `test_play_turn` and the
  per-deck `test_play_*`) pass in their Rust form.

#### Phase 10 — Sessions, Undo, Hint and the AI comparison (M)

Port `session.py` (the game thread, events, one or two people's answers in order), `view.py`, Undo and saved games
(replaying seed, answers and the search tape), `advisor.py` (Hint) and `shadow.py` (the AI comparison and the
review), now on worker threads of its own.

- **Done when:** `test_play_undo`, `test_play_save`, `test_play_hint`, `test_play_review` and `test_play_shadow`
  pass in their Rust form, and a saved game replays to the same position every time.

#### Phase 11 — The server and the setup screen (M)

Port `server.py` (the routes, seats by cookie, the two-player lobby over `--lan`), `catalog.py` (decks, tiers,
brackets and the results table for the setup screen) and `images.py` (card images from Scryfall, same cache
format). Serve `play/static/` unchanged.

- **Done when:** the browser page plays one- and two-player games against the Rust server with no change to the
  page, and the HTTP-level tests (`test_play_server`, `test_play_lan`, `test_play_catalog`, `test_play_images`) pass
  against it.

#### Phase 12 — The iPad app (M)

Under B2 (section 4): a small Swift app with a web view that links the Rust library, calls one function to start
the server, and shows the table. Bring over what the Briefcase app does today: the first-launch copy of card data
into the app's writable folder, saved games that survive updates, *Continue* after iOS closes the app, the hidden
two-player option, and the true-black palette.

- **Done when:** the app runs in the iPad simulator and on a device, and a game started, closed by iOS and
  reopened continues where it was.

#### Phase 13 — Cutover (S)

Make Rust the default for simulations and practice mode, retire the Python engine and `play/` (or keep them read-only
for a while), retire the Briefcase app, and update `documents/architecture.md`, `documents/practice-mode.md`, the
README and `pool-results.md`.

---

## 7. Validation: how to trust the new version

Matching Python's games exactly, seed for seed, would make checking easy: any difference in a play-by-play points
straight at a bug. But it needs Python's random generator reproduced bit for bit, the same order of iteration
everywhere, and identical floating-point arithmetic. One stray difference and every game after it diverges. That is
probably not worth the cost. A layered check works better:

1. **Ported tests:** the 463 simulation scenario tests, converted. They check rules and cards position by position.
2. **Differential tests on positions:** build the same position in both engines, and compare what each produces:
   the legal options, their utilities, the result of casting a given spell, the evaluation score. No randomness is
   involved, so these should match exactly (to rounding). A shared position format (JSON) for both engines makes
   this cheap to repeat for any card.
3. **Statistical agreement on full games:** run the reference set from phase 0 with both engines.
   - The pool calibration (within-tier and tier ordering) and your decks × tiers matrix, at 960 games per cell.
   - Acceptance: each cell agrees within its confidence interval; anything outside is investigated, card by card
     with `--analyze`, until it is explained or fixed.
4. **Swap agreement:** a handful of past `--swap` results (deltas you've acted on) reproduce with the same sign
   and roughly the same size.
5. **Practice mode, through its API:** the 213 practice tests, converted, and as many as possible rewritten to talk
   to the server over HTTP, so the same tests run against the Python and the Rust servers. The same request in the
   same position should offer the same choices, and the same answers should give the same views.
6. **Practice mode, by hand:** a checklist of games played in the browser on both versions, covering each of your
   decks, two players over the network, Undo, saving and loading, Hint and the review. Then the iPad on a device.

---

## 8. Risks

| Risk | Effect | Mitigation |
|---|---|---|
| Behaviour drift: hundreds of small differences between the engines | Win rates move, and old results can't be compared to new | Section 7; keep the Python engine runnable until the Rust results are accepted |
| Two engines to maintain during the port | Every card or rules fix made in Python must be repeated in Rust, or the port chases a moving target | Freeze Python feature work during the port, or keep a log of changes to re-apply |
| Practice mode lags behind the simulation during the port | For a while, simulations run on Rust and practice mode on Python, and the two can differ | Keep that window short: start part 2 as soon as phase 8 passes, and port fixes both ways until phase 13 |
| Saved practice games don't carry over | Games saved by the Python version won't replay in the Rust one, since the random numbers differ | Finish or abandon saved games before cutover; or keep the Python version available to open old saves (decision 3) |
| Slower everyday work | Adding a card or fixing a rule takes longer in Rust (types, compile times, the borrow checker) | Keep card data in Python; the card-function layout mirrors Python's |
| The redesign of the data model goes wrong early | Late rework across all cards | Pilot slice first (section 10); settle the `Game` struct, the hook table and the `Controller` interface before porting cards |
| Hidden dependence on Python behaviour (dict order, `id()` tables, float rounding, sort stability) | Subtle differences that only show in statistics | Ordered maps; stable sorts; the differential tests |
| A human-choice site is missed | The AI quietly makes a choice that should be the person's | Search the Python for all 97 sites and track each one in the port; the practice tests cover the common ones |
| The gain is smaller than expected | Months spent for a 10× gain that PyPy and caching could have come close to | Do those first; measure the pilot before committing |
| The new iPad app (B2) | A Swift shell and a Rust library for iOS are new to the project | Build the shell early, around the pilot, to find build problems before they matter |

---

## 9. Decisions still open

1. ~~**Where the practice server runs.**~~ Decided 10 October: in Rust (B2). The iPad app becomes a Swift shell around the
   Rust library.
2. ~~**How exact the match must be.**~~ Decided: statistical agreement.
3. **Saved practice games:** let old saves go at cutover, or keep the Python version available to open them.
4. ~~**Python work during the port.**~~ Decided: Python is frozen during the port (deck list edits only).
5. **What to drop:** retired decks and paths (Marchesa, Najeela, GAA, the old non-look-ahead branches).
6. **What the speed is for:** faster runs at today's AI, or a stronger AI at today's speed (more rollouts, a longer
   look-ahead, more candidates). The second changes results on purpose and needs its own recalibration. For
   practice mode the first is the obvious choice: the same AI, but quick, on the iPad too.

---

## 10. Path and milestones

The port started on 10 October 2026, on branch `rust-port` (from `jodah-final`, plus Sephiroth's flicker package).
The cheap Python speedups this section first recommended (PyPy, caching mana payment, a purpose-built clone) were
skipped, to start the port directly.

### The pilot (milestones M0–M6)

The pilot is phases 1–5, plus the cards of Tier 1 and one of your decks (**Sauron**: 16 cards with hand-written
code and a ~150-line plan in `ais.py`). It ends in a go / no-go. **Work stops for a check-in at the end of every
milestone**, and the next one starts only after that.

| Milestone | Phases | What gets done | Shown at the check-in | Size | Status |
|---|---|---|---|---|---|
| **M0** Setup and reference numbers | 0, start of 1 | Rust installed; the Cargo workspace (`sim-core`, `sim-py`, `practice`); the card export (`data/cards.json`, `data/decks.json`); the Python bridge (`rust_sim`); the Python reference run: Sauron vs Tier 1, loose profile, 960 games | Rust loads every card; the reference win rate | 1 session | **Done** 10 Oct: reference 28.9% (CI 26.1–31.8%) |
| **M1** Data model (**design review**) | 1 | `CardDef` and the tags as typed data; `Game`, `Player`, `Perm` in arrays with ids; the hook table; the `Controller` interface; loading `cards.json` into them | A short write-up and the core types, to review before anything is built on them | 1–2 sessions | **Done** 10 Oct, awaiting review: `documents/rust-design.md`; a game copies in 1.7 µs |
| **M2** Rules engine | 2 | `engine.py`: mana and payment, casting, the stack, counterspells, removal, zones, triggers, state checks, step caps; the rules tests in Rust | Rules tests passing, against the Python count | ~1–2 weeks | **Done** 10 Oct: engine.py ported; 54 Rust tests (26 rules, 9 stack and triggers); see rust/PORT_STATUS.md |
| **M3** Turn loop, combat, ability language | 3, 4 | The non-deck parts of `ais.py`; `dsl.py` | A full game in Rust with tag- and ability-language cards, next to Python's play-by-play | ~1 week | **Done** 10 Oct: whole games play in Rust; 77 Rust tests; `cargo run --example play_game` |
| **M4** AI and look-ahead | 5 | `brain`, `pool_ai`, `deck_plans`, `gc_prio`, the Sauron plan, `search`; the differential harness (the same position in both engines, options and scores compared) | Which options and scores match, and which don't | ~1 week | **Done** 10 Oct: 0 AI differences on 3,004 positions (rust/PORT_STATUS.md) |
| **M5** Pilot cards and go / no-go | parts of 6 and 7 | The shared card files Tier 1 and Sauron need, their cards, `--engine rust` in `poolmode`; the 960 reference games in Rust | **Go / no-go:** speed against Python, and the win rate against the reference with confidence intervals | ~1–2 weeks | **Go** 10 Oct: Rust 30.9% (CI 28.1–33.9%) against Python's 28.9% (26.1–31.8%), z = 0.96; 70 s for the 960 games against 24 min (1.5 CPU-s a game against 31, ~20×). audit/rust-port/ |
| **M6** iOS proof | 12 (early) | The core library built for iOS, and a minimal Swift web-view shell, with instructions | You build it on your Mac and it starts in the iPad simulator (any time after M1) | 1 session + your Mac | |

### After the pilot

On a "go": part 1 for the remaining tiers and your decks (phases 6–8), then part 2, practice mode (phases 9–13),
straight after. The milestones and check-ins for those are set at the go / no-go.
