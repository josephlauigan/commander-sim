# Architecture of the Commander pod simulator

This document explains how the simulator is built: what each part does, how a game is played, how the AI decides,
how cards are modeled, and how a run turns thousands of games into a win rate. It is written for someone who knows
Magic and Python and wants to change the code or trust its numbers.

It describes the code as of September 2026 (branch `lookahead-ai`). Function and module names are given instead of line
numbers, since those drift. The code is the `commander_sim` package; module paths below are relative to it
(`ai/search.py` is `commander_sim/ai/search.py`), and bare module names (`engine.py`) are in its top folder unless
section 3 places them elsewhere. Commands run from the repository root.

## Contents

1. [What the simulator is for](#1-what-the-simulator-is-for)
2. [The big picture](#2-the-big-picture)
3. [Modules at a glance](#3-modules-at-a-glance)
4. [The game model (engine.py)](#4-the-game-model-enginepy)
5. [A game from start to finish (ais.py)](#5-a-game-from-start-to-finish-aispy)
6. [Casting, the stack and responses](#6-casting-the-stack-and-responses)
7. [Combat](#7-combat)
8. [The heuristic AI (brain.py)](#8-the-heuristic-ai-brainpy)
9. [The look-ahead AI (search.py)](#9-the-look-ahead-ai-searchpy)
10. [How cards are modeled](#10-how-cards-are-modeled)
11. [The opponent pools](#11-the-opponent-pools)
12. [Running experiments (compare.py, poolmode.py)](#12-running-experiments-comparepy-poolmodepy)
13. [Randomness, pairing and reproducibility](#13-randomness-pairing-and-reproducibility)
14. [Guards against runaway games](#14-guards-against-runaway-games)
15. [Tests and tools](#15-tests-and-tools)
16. [Simplifications and their effect on results](#16-simplifications-and-their-effect-on-results)
17. [How to extend it](#17-how-to-extend-it)

---

## 1. What the simulator is for

The question the simulator answers: **how often does one of my Commander decks win a four-player game against
opponents of a given power level?**

- **Your decks** are the three lists in `decklists/mine/`: Sephiroth (Atraxa reanimator, key `seph`), Veyran
  (Izzet spellslinger, `veyran`) and Sauron (Grixis amass, `sauron`).
- **The opponents** are 25 outside decks in `decklists/pool/`, five in each of five tiers that follow the official
  Commander brackets:

  | Tier | Bracket | Decks |
  |---|---|---|
  | t1 | High Bracket 2 / Low Bracket 3 | Isshin, Lathril, Light-Paws, Tatyova, Teysa |
  | t2 | Mid Bracket 3 | Brago, Kaalia, Lord Windgrace, Meren, Sythis |
  | t3 | High Bracket 3 | Atraxa, Aurelia, Korvold, Marwyn, Tergrid |
  | t4 | Low Bracket 4 | Chulane, Heliod, Krenko, Prosper, Yuriko |
  | t5 | High Bracket 4 | Kinnan, Urza, Winota, Yawgmoth, Zur |

- **One game** seats your deck with three decks drawn from one tier, in a random seat order. With four players, 25%
  is an even share.
- **One run** plays hundreds or thousands of such games and reports the win rate with a confidence interval. A
  `--swap` run plays the same games with a changed list and reports the difference.

Everything is pure Python 3 with the standard library only. The only network use is fetching card data from Scryfall
the first time a card is seen (then cached in `data/scryfall_cache.json`).

---

## 2. The big picture

```
                         ┌──────────────────────────────────────────────┐
  command line  ───────► │ compare.py   argument parsing, parallel runs,│
                         │              progress, statistics, reports   │
                         │ poolmode.py  one deck vs a tier, A/B pairs,  │
                         │              matrix, calibration, --analyze  │
                         └───────────────┬──────────────────────────────┘
                                         │ seeds → games
                         ┌───────────────▼──────────────────────────────┐
                         │ ais.py       game setup, the turn loop,      │
                         │              combat, upkeep / end step,      │
                         │              your decks' hand-written plans  │
                         └──────┬───────────────────────┬───────────────┘
                   decisions    │                       │ rules
               ┌────────────────▼───────┐   ┌───────────▼────────────────┐
               │ brain.py  heuristic AI │   │ engine.py  game state,     │
               │ search.py look-ahead   │◄─►│  mana, casting, counters,  │
               │ pool_ai.py, deck_plans │   │  removal, zones, triggers  │
               └────────────────────────┘   └───────────┬────────────────┘
                                                        │ card behaviour
               ┌────────────────────────────────────────▼────────────────┐
               │ Card data:  carddb.py (hand tags) · sources.py (sources)│
               │   dsl_parse.py + dsl.py (Oracle text → ability data)    │
               │   cardimpl.py + impl/*.py (Python hooks for hard cards) │
               │   pool_cards.py (overrides)  ·  scryfall.py (+ cache)   │
               └─────────────────────────────────────────────────────────┘
```

The design has four layers:

1. **Card data** describes what each card does, in one of three forms: short *tags* (`rem=exile tgt=nl`), *ability
   data* compiled from Oracle text, or a *Python hook* for cards neither can express.
2. **The engine** holds the game state and applies the rules: paying mana, casting, countering, removing, dying,
   drawing, triggering.
3. **The AI** decides what each player does. A fast heuristic AI scores every legal play; the look-ahead AI, the
   default, tests the best few plays on copies of the game.
4. **The experiment layer** plays many seeded games in parallel and turns them into statistics.

The layers talk through a few narrow interfaces: the engine calls the AI only at decision points (main phases,
attacks, counter and protection windows), and calls card code only through tags, the ability interpreter, and named
hook events.

---

## 3. Modules at a glance

About 21,700 lines of Python in 36 modules, grouped into the `commander_sim` package:

```
commander_sim/          compare, poolmode, pools, decks, update_deck, pool_audit, engine, ais   (+ __main__.py)
commander_sim/ai/       brain, search, pool_ai, pool_decks, deck_plans
commander_sim/cards/    carddb, sources, scryfall, autotag, dsl_parse, dsl, cardimpl, pool_cards
commander_sim/cards/impl/   common, t1 … t5, combos, topdeck, fixes, lands, partials, rules, rules2, mine
commander_sim/tools/    searchtest, swaptest, linecov
data/                   scryfall_cache.json, cards_dsl.example.json (and cards_dsl.json if you add one)
```

`commander_sim/__init__.py` defines `ROOT` (the repository) and `DATA` (`data/`), which every module uses to find
deck files, the Scryfall cache and the audit notes. `python3 -m commander_sim` runs `compare.cli()`.

**Experiment layer** (`commander_sim/`)

| Module | Role |
|---|---|
| `compare.py` | Command-line entry point. Parses arguments, sets the AI mode, runs chunks of seeds on a process pool, draws the progress bar, holds the metric definitions and report printers. |
| `poolmode.py` | Everything measured against the pools: one deck vs one tier, paired A/B, the deck × tier matrix, `--analyze`, `--trace`, and the calibration checks. |
| `pools.py` | Loads the 25 pool deck files, validates them (size, singleton, colour identity, bans, Game Changers per tier), registers them for play, and draws seats for a game. |
| `decks.py` | Loads your three deck files from `decklists/mine/` into `DECKS`. |
| `update_deck.py` | Replaces a deck's list with a pasted one: validates it, rewrites the file's list sections, records the deck-guard fixture, and reports what changed and how the new cards are modeled. |
| `tools/searchtest.py`, `tools/swaptest.py`, `tools/linecov.py` | Paired tests for pool decks (with and without look-ahead, and with list swaps), and the test suite's line coverage. |
| `pool_audit.py` | How faithfully each card is modeled, per deck. |

**Game and rules** (`commander_sim/`)

| Module | Role |
|---|---|
| `engine.py` | The game state classes (`CD`, `Land`, `Perm`, `Player`, `Game`) and the rules: mana, costs, casting, counter windows, resolution, entering and leaving the battlefield, removal, wipes, tutors, drawing, life loss, elimination. |
| `ais.py` | Game setup and mulligans, the turn loop and its steps, combat, upkeep and end step, lands, and the hand-written plans of your three decks (priority functions, reanimation, combos, protection). |

**AI** (`commander_sim/ai/`)

| Module | Role |
|---|---|
| `brain.py` | The adaptive heuristic AI: reads the board, scores every legal play, samples one. Also the end-of-turn instant window, attack filtering and defender choice. |
| `search.py` | The look-ahead AI: clones the game, plays candidate moves forward, scores the result. |
| `pool_ai.py` | The generic AI for outside decks: cast priorities from tags, protection, attack filtering, special options. |
| `pool_decks.py`, `deck_plans.py` | Per-deck settings and plans for pool decks (play style, key cards, tutor wish lists, custom priority functions). |

**Card data** (`commander_sim/cards/`)

| Module | Role |
|---|---|
| `carddb.py` | About 340 hand-verified cards as `name | types | cost | tags` lines. Mostly your decks' cards. |
| `sources.py` | Resolves any card name to a card definition, choosing the source in priority order. |
| `scryfall.py` | Scryfall client with rate limiting and a local cache. |
| `autotag.py` | Regex tagger: Oracle text → tags. The fallback when nothing compiles. |
| `dsl_parse.py`, `dsl.py` | The card ability language: a compiler from Oracle text to JSON-like ability data, and the interpreter that runs it. |
| `cardimpl.py` | The hook registry (`@on(card name, event)`) and the dispatch helpers the engine calls. |
| `cards/impl/`: `common.py`, `t1.py` … `t5.py`, `combos.py`, `topdeck.py`, `fixes.py`, `lands.py`, `partials.py`, `rules.py`, `rules2.py`, `mine.py` | Python implementations of individual cards, grouped by where they appear or what they do. |
| `pool_cards.py` | Tag and ability overrides for pool cards, plus audit notes. |

---

## 4. The game model (engine.py)

### Card definitions: `CD`

A `CD` is the immutable definition of a card, shared by every copy in every game. It holds:

- **name, types and cost.** Types are one-letter codes: `L` land, `C` creature, `I` instant, `S` sorcery, `A`
  artifact, `E` enchantment, `P` planeswalker. The cost splits into generic mana and coloured pips (`3GWUB` →
  generic 3, pips `GWUB`).
- **tags**: a dictionary parsed from a string like `bomb=9 pow=7 fly vig dt lifelink leg atraxa`. Tags are the
  engine's main vocabulary. Some describe the card (`pow`, `tgh`, `fly`), some describe what it does (`rem=exile`,
  `draw=2`, `tut=any`, `ctr=any`), some name a role for the AI (`bomb=9` means "a top threat worth 9").
- **dsl**: an optional list of abilities in the ability language (section 10).
- keyword data from Scryfall: `kws`, `subtypes`, `protfrom` and `ward`.
- **bookkeeping**: `source` (where the definition came from), `unparsed` (Oracle lines nothing understood) and
  `game_changer`.

All definitions live in the global dictionary `engine.DB`, keyed by card name. `carddb.py` fills it at import time;
`sources.ensure_cards` adds every other card on demand.

### Game objects

| Class | What it is |
|---|---|
| `Land` | A land on the battlefield: its `CD`, whether it is tapped, and a small `data` dict. Lands live in `player.lands`, separate from other permanents. |
| `Perm` | A nonland permanent or token: controller (`owner`), original owner (`orig`), tapped / summoning-sick state, power and toughness, counters (`plus`), flags (flying, deathtouch, vigilance, lifelink, phased, attached), loyalty, and a free-form `data` dict for card-specific state. Tokens have `cd = None`. |
| `Player` | A seat. Zones as lists of `CD`s: `library` (top card is the last element), `hand`, `gy`, `exile`; `lands` and `perms`. Also life (starts at 40), commander tax and whether the commander is in the command zone, commander damage taken, treasures, clues and floating mana, per-turn counters, and `stats` for reports. |
| `Game` | The table. Holds `players` in seat order, the decision random generator `rng`, `round`, `active` player and current `step`, `over` / `winner` / `wintype`, and caches. Also `hooks` (permanents with Python implementations, in entry order), a step counter `work` with its cap `work_cap`, and the optional trace `log`. |

Cards in zones are `CD`s, not instances: two copies of a basic land in a library are the same object twice.
Only the battlefield has per-object state (`Land`, `Perm`).

### State-based checks

`check_state(g)` runs after almost every action. It fires the `sba` hook event and eliminates any player who:

- is at 0 life or less;
- drew from an empty library;
- took 21 commander damage from one commander;
- has 10 or more poison counters.

When one player is left, the game ends with that player as winner.

`eliminate` removes the player's permanents, records who dealt the final blow (`last_src`) and how (combat, burn,
drain, combo, commander damage, decked), for the reports.

### Mana

Mana is computed, not tracked as a pool:

1. **`mana_units(g, p)`** lists every source that could pay right now as `[source, colours, amount]`. Sources
   include:
   - untapped lands, with their colours from tags; hooks adjust for Blood Moon, Chromatic Lantern and the like;
   - mana rocks (`rock=amount:colour`) and untapped, non-sick mana dorks (`dork=colour`);
   - treasures;
   - floating mana from rituals and Birgi;
   - mana-producing tokens;
   - creatures that can convoke.

   The hook layer adjusts the list: locks (Cursed Totem, Collector Ouphe, Karn), bonuses (Kinnan, Urza),
   dynamic amounts (Priest of Titania) and hand mana (Elvish Spirit Guide).
2. **`plan_pay(units, generic, pips)`** is a greedy planner:
   - Coloured pips are assigned first, scarcest colour first, and each pip uses the least flexible source that
     makes it.
   - Generic mana is then filled, preferring non-treasure, painless sources that waste the least.
   - It returns how much of each unit is used, or `None` if the cost can't be paid.
3. **`pay(g, p, generic, pips)`** executes the plan: taps sources, sacrifices treasures, spends floating mana and
   applies side effects (painlands, Ancient Tomb, Talismans, `mana_tapped` hooks).

`can_pay` is `plan_pay` without paying, and the AI calls it constantly.

`cost_of(p, c)` gives a card's current cost after reductions, taxes and commander tax.

Floating mana empties at the start of each turn, not between phases.

### Interaction profiles

Two global profiles set how interactive the AI opponents are (`engine.PROFILES`, `set_profile`):

| Profile | Pool decks counter a spell of importance… | Instant-speed removal |
|---|---|---|
| `conservative` | 7 or more | Held for emergencies (a penalty on casting instants proactively) |
| `loose` | 6 or more | Used as freely as sorcery-speed removal |

A pool deck with many counterspells counters at a lower importance (`pool_ai.counter_threshold`: 0.3 lower per
counter in the list, at most 2.5).

Your decks have their own thresholds in the same table: Sephiroth 6 under both profiles; Veyran and Sauron 7
conservative, 6 loose.

Loose is the worst case for your decks, because opponents answer more. Most recent measurements use loose only.

---

## 5. A game from start to finish (ais.py)

### Setup

`play_pool_game(seed, seats)` builds a game from `[(deck key, card list, commander name), ...]` in seat order:

1. Each seat becomes a `Player`. The commander is taken out of the library into the command zone.
2. The game's decision generator is seeded with `'play:{seed}'`.
3. Each seat shuffles and mulligans with its own generator, seeded `'lib:{seed}:{deck key}'` (section 13 explains
   why).
4. **Mulligan rule** (`ais.mulligan`): keep a seven-card hand with 2 to 5 lands. Otherwise draw seven again, up to
   three redraws; the third and fourth redraws put one or two cards on the bottom (a spare land if flooded, else the
   most expensive spell).

### The round loop

`_run_rounds(g, players, max_rounds=20)`:

```
for round 1..20:
    for each player in seat order:
        skip if dead, or if a skip-turn effect applies (Ral Zarek)
        the player's end-of-turn window (from round 2)   ← instants at the end of the previous turn
        take_turn(player)
    stop if the game is over
if nobody won after 20 rounds:
    the winner is the living player with the highest life + 2 × creature power ("timeout")
```

The **end-of-turn window** (`brain.end_of_turn_window`) is how instant-speed play happens. Just before a player's
turn, that player may spend mana they held up but didn't use: instant-speed draw, tokens, flashback, removal and
activated abilities. Mana untaps next anyway, so holding it has no further value.

### A turn

A turn is five steps (`ais.STEPS`), each a function:

| Step | What happens |
|---|---|
| `start` | Untap, reset per-turn state, **upkeep**, then the **draw step** (skipped for Necropotence, dredge and the like), then the **land drop** (the AI picks which land). |
| `main1` | The main-phase AI plays spells and abilities until it decides to stop. |
| `combat` | Up to four combats: attack declaration, blocks, damage, extra combats. |
| `main2` | The main-phase AI again, knowing how combat went. |
| `end` | End-step triggers, discard to seven, cleanup of "until end of turn" effects. |

Upkeep (`ais.upkeep`) runs `cardimpl.turn_start` first, which handles:

- animated lands reverting;
- crewed vehicles ending;
- delayed draws;
- Pact of Negation payments;
- saga chapters and rebound spells.

It then applies the upkeep effects in `ais.upkeep` itself (Tabernacle, Glacial Chasm, …), the `upkeep` hooks and
the ability interpreter's upkeep triggers.

`continue_turn(g, p, step)` can resume a turn from any step. The look-ahead uses it to play a copied game forward
from the middle of a turn.

### How the game ends

A game ends when one player is left (a win by damage, drain, commander damage, poison, decking, or an opponent's
elimination), when a combo resolves (`ais.win` with win type `combo`), or after 20 rounds (`timeout`). The report
counts each win type.

---

## 6. Casting, the stack and responses

The engine has **no general stack**. A spell is cast, gets one response window, and resolves. Triggers resolve
immediately when they happen. This is the largest simplification in the model (section 16).

### The cast pipeline

```
AI picks a play
   │
   ├─ pay the cost (pay / pay_card)          the card leaves the hand before paying
   │
   ▼
cast_card(g, p, c, zone, ctx)
   ├─ move the card out of its zone (hand, graveyard, command zone, library)
   ├─ on_cast: cast triggers
   │     opponents' Rhystic Study, Sauron amass, Kaervek; magecraft; prowess;
   │     ability-language cast triggers; hook 'cast' events; copies (Jin-Gitaxias, Ral, Return the Favor)
   ├─ spell_imp: how important is this spell (to each opponent)?
   ├─ counter_window: each opponent in turn order may counter it
   │     └─ the caster may counter back (one counter war)
   └─ resolve(g, p, c, ctx, zone), then check_state
```

### Resolution

`resolve` dispatches in this order:

1. **A Python hook** registered for the card's `resolve` event handles the whole spell.
2. **An ability-language spell** (a nonpermanent with `dsl` abilities) is run by `dsl.resolve_spell`.
3. **A permanent spell** enters the battlefield (`enter`): the ETB effects of its tags, ability-language `etb`
   triggers and `etb` hooks all fire.
4. **Otherwise the tags are executed one by one**: `draw`, `treas`, `lr` (land ramp), `tut` (tutor), `rem`
   (removal), `wipe`, `rean` (reanimate) and many more.

A spell copy resolves with `zone='copy'`, so it never goes to a graveyard. A copy of a permanent spell becomes a
token.

### Counterspells

- **The threshold.** `spell_imp` rates how much a spell matters, from its tags, bomb value and whether it is a
  commander. A counterspell is only considered when that rating clears the would-be counterer's threshold (see
  "Interaction profiles" in section 4).
- **The heuristic decision.** `brain.wants_counter` gives a probability from how far the spell's importance is over
  the threshold and how many counters the player holds.
- **The look-ahead decision.** With look-ahead on, a counter against a spell cast in the active player's main phase
  is decided by `search.choose_counter` instead: it plays both futures, countered and not countered.
- **Which counter.** `pick_counter` chooses among the counters in hand: the free ones (Fierce Guardianship, Force of
  Negation, Pact of Negation, Mental Misstep) when their conditions hold, otherwise the cheapest one that can be paid
  for.
- **After the counter.** "Soft" counters (Spell Pierce, Mana Leak) let the caster pay. Side effects apply: Arcane
  Denial, An Offer You Can't Refuse, Mana Drain. The caster may then counter back once.

### Other response windows

Each is a small, explicit function rather than a stack:

| Window | Where | What it does |
|---|---|---|
| Protection | `ais.protect_response` → `pool_ai.protect` for pool decks | When removal targets a valuable permanent, its owner may answer with Heroic Intervention, a phase-out, a sacrifice in response, a blink or a redirect. |
| Wipe response | `ais.wipe_response` | Players answer a board wipe (Teferi's Protection, Heroic Intervention). |
| Combo interruption | `cards/impl/combos.interrupted` | Before a combo loop runs, each opponent gets one chance to remove a key piece at instant speed, or counter the last piece. |
| Reanimation response | `cardimpl.gy_response` | Graveyard hate (Soul-Guide Lantern, Tormod's Crypt) answers a reanimation spell. |
| Ward and removal taxes | `engine.apply_removal` | Ward costs, Sauron's ward (sacrifice a legend), taxes and prevention effects. |

`apply_removal` is the single place removal lands. It checks, in order:

1. whether the target is still there and targetable;
2. indestructible;
3. the owner's protection response;
4. ward and removal taxes;
5. damage prevention.

Only then does it destroy, exile, bounce, tuck or transform the permanent. It also applies riders such as the life
gain from Swords to Plowshares and the land from Path to Exile.

---

## 7. Combat

`ais.combat` runs up to four combat phases a turn:

1. **Attackers.** Every untapped, non-sick creature with power above zero is a candidate; mana dorks attack only
   when pumped to 3 power.
2. **The plan.** At the first combat, the look-ahead AI picks a plan: which opponent to attack and whether to send
   the filtered attackers, everyone, or nobody (`search.choose_attack`). Otherwise the heuristic picks the defender
   (`brain.choose_defender`: a lethal target if there is one, else the biggest threat with some randomness) and
   removes attackers worth keeping home as blockers (`brain.filter_attackers`, `pool_ai.attack_filter`).
3. **Restrictions.** Attack taxes and caps (Ghostly Prison, Propaganda, Crawlspace, Silent Arbiter), forced
   attacks, and beginning-of-combat triggers.
4. **Attack triggers.** Hooks, ability-language `attack` triggers, battle cry, mentor, exalted, ninjutsu and the
   Ring.
5. **Blocks and damage** (`resolve_combat`). The defender assigns blocks with the evasion rules in `can_block`:
   - flying and reach;
   - shadow;
   - landwalk;
   - protection;
   - "can't block";
   - the Ring's "can't be blocked by greater power".

   Damage respects first strike, double strike, deathtouch, trample, lifelink, prevention and commander damage.
   Combat-damage triggers fire per creature.
6. **Extra combats.** A deck-specific extra combat (Sauron's Aggravated Assault) or `extra_combats` granted by
   cards untaps the creatures and loops.

Blocks are always heuristic; the look-ahead doesn't choose them.

---

## 8. The heuristic AI (brain.py)

This is the adaptive AI. It runs alone in `--ai adaptive` mode, and it is also the policy that plays the look-ahead
AI's simulated futures.

### One main-phase decision

```
opts = main_options(g, p, post)        every legal play as (utility, label, fn), plus 'stop' (fn = None)
if look-ahead is on and there are ≥ 2 options:
    pick = search.choose(...)          (section 9)
else:
    order = gumbel_order(rng, opts, temperature)    sample an ordering from softmax(utility / T)
    try each play in that order; the first that succeeds is made
repeat (up to 18 plays per phase) until 'stop' is chosen or nothing works
```

The step that reads the board, **`Situation(g, p)`**, collects:

- the turn, the mana available and the hand size;
- how much damage opponents' creatures point at this player (`danger`);
- each opponent's threat level and who is leading;
- whether an opponent is one step from a combo;
- the risk that this player's next spell is countered (`counter_risk`) or its best threat removed
  (`removal_risk`).

**Hidden information is estimated from public information only.** `prob_holding(g, q, pred)` works out the chance
that opponent q holds a card of some kind, from q's decklist minus the copies already seen in the graveyard, exile
and battlefield. It is then combined with q's open mana.

**Utilities.** `card_utility` starts from the deck's own priority for the card and adjusts it for the situation:

- the priority comes from `seph_prio`, `veyran_prio` or `sauron_prio` for your decks, or `pool_ai.generic_prio` plus
  per-deck plans for pool decks, as a 0–90 score divided by 10;
- ramp is worth less late in the game;
- card draw is worth less when under pressure;
- blockers are worth more when in danger;
- the commander is worth less as its tax grows;
- bombs are worth less into open counter mana.

The other kinds of option get utilities in their own functions: removal and wipes, special plays (reanimation,
tutors, combos), activated abilities from hooks and the ability language, and equipment. The
`hold_value`/`reserve_penalty` pair makes the AI less likely to tap out when it holds a counterspell or protection
it can pay for.

**Temperature.** Each deck has a play style (`brain.STYLE` for your decks, `pool_decks.CONFIG` for pool decks) with
three knobs:

- `temp`: randomness. Lower is sharper and more predictable; `--temp` scales it globally.
- `aggression`: how much pressure it applies.
- `caution`: how much it holds up interaction and keeps blockers home.

Sampling from a softmax instead of always taking the top score makes play varied and human-like, and it stops one
tiny scoring error from deciding every game the same way.

### Your decks vs pool decks

Your three decks keep hand-written plans in `ais.py`:

- priority functions (`seph_prio`, `veyran_prio`, `sauron_prio`);
- reanimation targets;
- combo checks (Veyran's kitten combo, Sauron's Sword + Aggravated Assault);
- protection choices;
- special options in `brain.special_options`.

Pool decks use the generic `pool_ai` rules plus the per-deck configuration of section 11.

`--ai rigid` is an older fixed-priority AI (`ais.generic_main` and the per-deck `*_main` functions). It is still
selectable but no longer used for results.

---

## 9. The look-ahead AI (search.py)

The look-ahead is the default AI (`--ai lookahead`). Every deck at the table uses it for three kinds of decision:

- main-phase plays;
- the attack plan at the first combat;
- counterspells against spells cast in the active player's main phase.

Everything else stays heuristic: blocks, protection, wipe responses, the end-of-turn window, and the plays inside
simulated futures.

### Choosing a main-phase play

For a decision by player p with options `opts`:

1. **Candidates.** The `TOP_K = 5` options with the highest heuristic utility, plus "stop" (hold the mana / move to
   the next step).
2. **Copies.** For each of `ROLLOUTS = 6` rollouts and each candidate:
   - `clone(g)` makes a deep copy of the game;
   - `determinize` hides what p can't know: each opponent's hand is re-dealt at random from their hand + library,
     and p's own library is shuffled;
   - the candidate is played in the copy, and the rest of the phase is played by the heuristic AI;
   - `play_on` continues the game with the heuristic AI for everyone until the end of p's next turn
     (`HORIZON = 1`).
3. **Scoring.** `evaluate(copy, p)` scores the final position. Scores are summed across rollouts, and the candidate
   with the best total is played in the real game.

A main-phase decision therefore costs 36 playouts (6 candidates × 6 rollouts). An attack decision tries each
opponent × {filtered attackers, all attackers} plus "no attack". A counter decision tries two futures.

**Common random numbers.** All candidates in rollout r use the same random seed, so they are compared on the same
hidden hands and the same future draws. This removes most of the noise from the comparison: a candidate wins because
it is better, not because its copies drew better cards.

### The evaluation

```
evaluate(g, p) =  +100                   if p has won
                  −100                   if p has lost or is dead
                  95 · tanh(raw / 100)   otherwise

raw = strength(p) − 0.6 · strongest opponent − 0.4 · mean opponent strength + 12 · eliminated opponents

strength(q) = 0.25 · life (capped at 60) + Σ permanent values + 0.35 · creature power
            + 0.9 · cards in hand + 0.6 · lands + 0.4 · treasures + 12 · combo_progress(q)
```

- **The tanh bound** keeps every unfinished position below a win. Without it, a board of 250 goblins scored higher
  than actually winning, and Krenko stopped attacking.
- **`combo_progress`** is the share of the deck's closest modeled combo already held (hand, battlefield, or
  commander in the command zone), squared so the last pieces count most. It makes combo decks value assembling
  pieces, and opponents value breaking them up.

### Cloning a game

`clone` uses `copy.deepcopy` with a memo pre-filled with every `CD` in `engine.DB`, so card definitions are shared
rather than copied. Some tables are keyed by object id (end-of-turn pumps, once-per-turn flags, imprint lists); these
are re-keyed to the copied objects afterwards. Caches are dropped. The trace log is detached during the copy, so
simulated futures never print.

Copies are marked `in_search`, so a copy never searches itself: inside a copy, every player uses the heuristic AI.

### Cost and limits

A look-ahead game takes about 20 s of CPU, against about 0.2 s for the heuristic AI. A typical game makes about 125
look-ahead decisions and about 270,000 playout steps. Section 14 lists the caps that keep the worst cases bounded.

---

## 10. How cards are modeled

A card can be modeled in three ways, and one card may use more than one.

### 1. Tags (carddb.py, autotag.py)

A tag string names what the card does in the engine's vocabulary. Examples:

```
Anguished Unmaking|I|1WB|rem=exile tgt=nl lose=3
Arcane Signet|A|2|rock=1:A
Archon of Cruelty|C|6BB|bomb=9 pow=6 fly archon
Demonic Tutor|S|1B|tut=any
```

The engine and the AI read tags everywhere: `resolve` executes them, `mana_units` reads `rock` and `dork`,
`pval` reads `bomb`, and `card_utility` reads role tags. A tag that names a single card (`archon`, `atraxa`,
`veyran`) points at code written for that card.

Your decks' cards are hand-tagged in `carddb.py` and verified against Oracle text. For any other card,
`autotag.py` produces tags from Oracle text with regular expressions and lists the lines it didn't understand.

### 2. The ability language (dsl_parse.py, dsl.py)

Cards can be described as data: a list of abilities, each made of small effects.

```json
{"type": "triggered", "event": "etb", "source": "self",
 "effects": [{"do": "draw", "n": 2, "who": "you"}]}
{"type": "activated", "cost": {"mana": "2B", "tap": true},
 "effects": [{"do": "destroy", "what": {"sel": "target", "filter": {"type": "creature", "controller": "opp"}}}]}
```

- **Ability types:** spell, triggered, activated, loyalty, static (anthems, keyword grants, cost changes) and
  replacement (token doublers).
- **Effects:** about 40 kinds, including draw, damage, destroy, exile, bounce, tokens, counters, pump, search,
  reanimate, sacrifice, modal choices, extra combats and proliferate.

The two halves:

- **`dsl_parse.py`** compiles Oracle text into this form with a library of templated rules (`rule(pattern)`
  functions).
- **`dsl.py`** is the interpreter. Its entry points:
  - `fire(g, event, ...)` for triggers;
  - `resolve_spell` for instants and sorceries;
  - `pt`, `has_kw`, `cost_delta` and `token_mult` for static abilities;
  - `ability_options` to offer activated and loyalty abilities to the AI;
  - `card_value`, the AI's estimate for a card no priority table knows.

The engine switches the interpreter on for a game (`g.dsl_on`) the first time a card with ability data enters.
Games with only hand-tagged cards skip it entirely.

### 3. Python hooks (cardimpl.py, impl/*.py)

For cards neither tags nor the ability language can express (locks, taxes, commander engines, combos, unusual
replacement effects), Python functions are registered by card name and event:

```python
@CI.on('Smothering Tithe', 'draw')
def tithe(g, src, p): ...
```

**Events.** `cardimpl.py`'s docstring lists them all:

- triggers: `etb`, `leaves`, `dies`, `cast`, `attack`, `blocks`, `combat_damage`, `upkeep`, `end_step`, `draw`,
  `landfall`, `sacrifice`;
- AI options: `options`, which offers activated abilities;
- static queries: `cost`, `can_cast`, `attack_tax`, `attack_cap`, `trigger_copies`;
- zone hooks: `hand_blocks`, `gy_options`, `land_options`, `resolve` and more.

**Dispatch.** When a hooked permanent enters, it is added to `g.hooks`. The engine calls:

- `CI.fire(g, event, ...)` for triggers;
- `CI.total(g, event, ...)` to sum numeric answers (taxes, extra mana);
- `CI.allowed` to ask whether something may be cast;
- `CI.hooked` to walk the listeners.

A per-game cache maps each event to its listeners.

**Where the implementations live:**

| Module | Contents |
|---|---|
| `cards/impl/common.py` | Staples shared by many decks: Auras, equipment, sagas, planeswalker helpers. |
| `cards/impl/t1.py` … `cards/impl/t5.py` | Each tier's commanders and signature cards. |
| `cards/impl/combos.py` | Combos as a unit: pieces, readiness check, mana, and the result. |
| `cards/impl/topdeck.py` | Scry, surveil, top-of-library manipulation, and `desire` (which card a player wants next). |
| `cards/impl/lands.py` | Utility lands and creature lands. |
| `cards/impl/rules.py`, `cards/impl/rules2.py` | Rules that were approximated, made exact: mana sources, taxes, counter interactions, locks, emblems, crew, evoke, dash and similar mechanics. |
| `cards/impl/fixes.py` | Replacements for cards the compiler reads wrongly. |
| `cards/impl/partials.py` | Completing cards the audit listed as Partial. |
| `cards/impl/mine.py` | Cards in your decks that need more than tags: the Ring, equipment, lands and full card text. |

`cardimpl.load()` imports these modules in a fixed order. A later registration for the same card and event
replaces an earlier one.

### Choosing the source for a card

`sources.ensure_cards(names)` fills `engine.DB`, choosing each card's source in this order:

1. **`data/cards_dsl.json`**: ability data written by hand for a card. It always wins. The repo ships only
   `data/cards_dsl.example.json`.
2. **`carddb.py`**: hand-verified tags.
3. **Scryfall text compiled into ability data** (`dsl.build_cd`), with Scryfall's keyword list and the tags the
   regex tagger found alongside.
4. **Scryfall regex tags only** (`autotag`), when nothing compiled.

Then, when the pools are registered, **`pool_cards.apply()`** applies overrides to pool cards (replacement tags or
abilities). It skips your hand-tagged cards, because your decks' AI is built around their tags.

**Python hooks sit on top of any source.** A card with hooks also keeps its tags or ability data; the hook adds or
replaces specific behaviour.

`SIM_DSL_ALL=1` (or `--dsl-all`) runs every card from compiled Oracle text instead of hand tags. It is useful as a
cross-check of the hand tags.

### The card audit

`pool_audit.py` rates every card:

- **Full**: hand-verified.
- **Full-auto**: every Oracle line compiled.
- **Approximate**: modeled with a simplification.
- **Partial**: the main effect is in, some text is missing.
- **Unmodeled**: the simulator never casts it, or it does nothing.

The rating comes from `documents/card-audit.md`, notes recorded with `note()` in the impl modules, and the compiler's
unparsed lines. `--mine` audits your decks. After the September 2026 pass, every card in your decks rates Full.

---

## 11. The opponent pools

### Deck files

Every deck, yours and the pool's, is a Markdown file. The simulator reads:

- the `## Import list` block: `1 Card Name` lines. `decks.load` reads it.
- the `- **Commander:**` line.
- the `**Game Changers (N)**` claim.
- the `## Sim modeling notes` section (pool decks), which the audit uses.

Everything else (strategy, role in the pool, change log) is for people.

`pools.PoolDeck` reads a pool deck's file. Its tier comes from the folder name (`t3-high-b3` → `t3`) and its key from
the file name.

### Validation

`python3 -m commander_sim.pools --validate` checks every pool deck for:

- 100 cards, singleton except basics;
- the commander present and a legal commander;
- every card name resolving exactly on Scryfall;
- colour identity;
- Commander bans;
- the number of Game Changers per tier (`pools.GC_RULES`: t1 none, t2 one or two, t3 exactly three, t4 and t5
  unrestricted), matching what the file claims.

### Registration

**`pools.register()`** makes the pool decks playable. It:

1. loads the card implementations;
2. ensures card data for every card;
3. applies `pool_cards` overrides;
4. registers each deck's seat (colour identity, display name);
5. attaches its AI configuration.

It is idempotent: every worker process calls it.

### Per-deck AI

`pool_decks.CONFIG[deck key]` may set:

- **`style`**: `temp`, `aggression`, `caution`.
- **`key_cards`**: fixed cast priorities for named cards.
- **`wish`**: a tutor wish list, or a function returning one, such as the missing pieces of the deck's combo.
- **`prio_fn`**: a custom priority function, usually in `deck_plans.py`, for decks whose plan the generic rules can't
  see (Yawgmoth, Heliod, Chulane, Yuriko, Kaalia).
- **`cmd_prio` and `cmd_turn`**: when to cast the commander.

Decks without an entry use `pool_ai.DEFAULT_STYLE` and the generic priority. The generic priority rates fast mana
highest early, engines and anthems next, and holds counters, removal and protection for their response logic.

### Calibration

The pools are only useful if the tiers are ordered and each tier is internally balanced. Two checks measure this
(`--calibrate`):

- **Within-tier balance** (`poolmode.within_tier`): games of four decks drawn from one tier. Each deck should win
  15–35% of the games it plays; outside that it is flagged HIGH or LOW.
- **Tier ordering** (`poolmode.tier_ordering`): each deck of a tier alone against three decks from the tier below.
  It should win more than 25%.

Results, tuning decisions and history are in `decklists/pool/pool-results.md`. Opponent lists are only changed with
explicit approval; each change is noted in the deck file.

---

## 12. Running experiments (compare.py, poolmode.py)

### Commands

```
python3 -m commander_sim --deck seph --pool t3 --games 1500 --jobs 24             one deck vs one tier
python3 -m commander_sim --deck seph --pool t3 --swap "Out=>In" --jobs 24         paired A/B
python3 -m commander_sim --all-decks --pool all --profile loose --jobs 24         deck × tier matrix
python3 -m commander_sim --deck seph --pool t4 --analyze                          how it wins and loses
python3 -m commander_sim --deck seph --pool t2 --trace 7                          play-by-play of one game
python3 -m commander_sim --calibrate within|ordering|all --games 240 --jobs 24    pool balance checks
```

`--jobs` defaults to 1. Set it to your core count, especially with the look-ahead AI.

### The run

`compare.main` parses the arguments and calls `set_ai`, which switches the look-ahead on for every deck
(`search.KEYS = {'*'}`) or off. It then hands pool work to `poolmode.main`, which chooses the mode:

- **run**: one deck against one tier;
- **A/B**: the baseline list and the swapped list on the same seeds;
- **matrix**: every deck against every tier;
- **analyze**: the detailed report;
- **calibrate**: the two pool checks.

### Parallelism

The seeds (500000, 500001, …) are split into chunks (`compare._chunks`): 1–25 games per chunk for look-ahead, 25–250
for the heuristic AI. Chunks run on a `multiprocessing.Pool` of `--jobs` workers (`_run_chunks`):

1. Each worker sets the profile and AI.
2. It registers the pools.
3. It plays its seeds and returns counters: wins, turns, per-opponent seat counts, wins and eliminations, per-axis
   sums and per-seed outcomes.

`_merge` adds the counters together. Results don't depend on chunk size or worker count, because each game depends
only on its seed.

A game that raises an error is skipped and counted, and `report_errors` prints the seeds so they can be replayed with
`--trace`. A game that runs out of engine steps ends as a timeout (section 14).

### Statistics

- **Win rate** with a Wilson 95% interval (`poolmode.wilson`). At 240 games the interval is about ±6 points; at
  1500, about ±2.5.
- **Paired A/B difference** (`poolmode.paired_delta`): the mean of the per-seed differences (variant win − baseline
  win) and its standard error. Pairing cancels most of the luck, so a paired difference is much tighter than the
  difference of two independent win rates. `compare.verdict` summarises it across profiles.
- **Strength axes** (`compare.METRICS`, `axis_values`) for your decks. Each is reported baseline → variant with a
  two-standard-error noise band and `*` for changes beyond it. The axes:
  - mulligans, missed land drops and mana on turns 4 and 6;
  - when the game plan came online;
  - cards drawn, tutors and spells cast;
  - counters, removal and wipes cast;
  - protection used, threats lost and spells countered;
  - damage dealt, by kind.
- **`--analyze`** reports how wins happen and who kills you with what, plan timing, damage by source, and a card
  report.

---

## 13. Randomness, pairing and reproducibility

Every random choice comes from a generator seeded by a string, never by time or Python's randomised string hashing:

| Stream | Seed | Used for |
|---|---|---|
| Seats | `'pool:{seed}'` | Which three opponents, and the seat order (`pools.draw_seats`) |
| Library | `'lib:{seed}:{deck key}'` | That seat's opening shuffle and mulligans |
| Play | `'play:{seed}'` | Every in-game decision and random effect |
| Look-ahead | `crc32(decision number, round, …)` | Each decision's rollout seeds (`search._decision_seed`) |

This design has three consequences:

- **Any game replays exactly.** `--trace N` plays seed 500000 + N with a full log, and the result matches the run.
- **Swaps are paired.** The seat draw depends only on the seed and the deck keys, never on card lists, and each
  seat shuffles its own library from its own stream. A swapped list therefore faces the same opponents, in the same
  seats, with the same opponent draws. `poolmode.swap_in_place` keeps every other card at its list position, so your
  own library order differs only where the swapped card sits. Games diverge once play differs, but early games are
  identical and the rest stay strongly correlated.
- **Parallel runs are deterministic.** The same command gives the same numbers on any machine and with any `--jobs`.
  The look-ahead uses crc32 rather than `hash()` because Python randomises string hashes per process.

---

## 14. Guards against runaway games

Some decks can loop, or build a board so large that every step crawls. Several caps keep every game finite. None
changes normal games.

| Guard | Where | Limit | When it is hit |
|---|---|---|---|
| Engine steps per game | `engine.GAME_WORK`, `tick` | 20,000 (normal games use under 1,000) | `OutOfWork` is raised and the game ends as a timeout; `ais.STOPPED` records where. |
| Engine steps per playout | `search.PLAYOUT_WORK` | 2,000 | The playout is scored where it stands. |
| Look-ahead decisions per game | `search.GAME_DECISIONS` | 1,000 (a game uses about 125) | The heuristic AI plays on. |
| Playout steps per game | `search.GAME_SEARCH_WORK` | 3,000,000 (a game uses about 270,000) | The heuristic AI plays on. |
| Board size | `search.BOARD_LIMIT`, `g.board_cap` | 150 permanents on the table | No look-ahead at that size; a copy that grows past it stops. |
| Tokens per player | `engine.TOKEN_CAP` | 250 creature tokens | Further tokens aren't created (the board is lethal many times over). |
| Rounds | `_run_rounds` | 20 | Timeout adjudication. |

`OutOfWork` derives from `BaseException` so that no per-game `except Exception` handler swallows it.

`search.STATS` counts decisions, playouts, changed choices, cut playouts and capped games, for diagnosing the
look-ahead.

---

## 15. Tests and tools

Run the tests with `python3 -m unittest discover -s tests -t .` (118 tests, about a minute).
[tests/README.md](../tests/README.md) describes each file, how to run one test, and how to write a new one.

| File | What it checks |
|---|---|
| `test_rules.py` | Core rules on hand-built positions: mana, commander tax, state-based losses, counterspells, removal, wipes, tutors, mulligans, combat keywords. |
| `test_my_cards.py` | Key cards of your three decks against their Oracle text. |
| `test_search.py` | The look-ahead AI: independent copies, re-dealt hidden hands, evaluation bounds, a whole reproducible decision. |
| `test_dsl.py` | The ability compiler and interpreter. |
| `test_cli.py` | Statistics, and every command run as a module with a few games; results don't depend on `--jobs`. |
| `test_update_deck.py` | The deck updater, on copies of real deck files. |
| `test_pool_games.py` | Games from every tier, alone and with each of your decks, play to the end; one look-ahead game. |
| `test_pool_sampling.py` | Seat drawing, pairing and reproducibility. |
| `test_validator.py` | Deck validation rules. |
| `test_my_decks.py` | Your deck files parse to the recorded lists (`tests/fixtures/my_decks_parsed.json`). |

`tests/table.py` builds a position for a rule test: seat decks, empty the hands, put chosen cards in hand or onto the
battlefield, then drive the engine directly.

`python3 -m commander_sim.tools.linecov` measures line coverage (86% in September 2026). It records subprocesses and
worker processes too, using only the standard library.

Other tools:

- `python3 -m commander_sim.pools [--validate]`: list or validate the pools.
- `python3 -m commander_sim.pool_audit [--deck KEY] [--mine] [--md FILE]`: modeling coverage.
- `python3 -m commander_sim.tools.swaptest <deck> <tier> <games> "Out>In; …"`: paired test of pool list changes without editing
  the file.
- `python3 -m commander_sim.tools.searchtest <deck> <tier> <games> [rollouts] [top_k] [all]`: a pool deck with and without
  look-ahead, paired.
- `python3 -m commander_sim.cards.autotag "Card"` and `python3 -m commander_sim.cards.dsl "Card"`: preview how a card will be modeled.
- `python3 -m commander_sim --deck veyran --cards`: every card in a deck, its source, tags, abilities and unmodeled text.

---

## 16. Simplifications and their effect on results

The simulator abstracts Magic on purpose. The main simplifications, and which way they push results:

- **No general stack.** One response window per spell, one counter war, and triggers resolve at once. Instant-speed
  play outside those windows happens in the end-of-turn window before each player's turn. Decks built on stack
  tricks are modeled through explicit windows (counters, protection, combo interruption) rather than true priority
  passes.
- **Greedy mana payment.** Payment is near-optimal but not exhaustive. It can occasionally tap a source a later spell
  that turn needed.
- **Hidden information by estimate.** The heuristic AI infers opponents' hands from public information. The
  look-ahead samples plausible hands, so it never peeks.
- **The look-ahead horizon is short.** One turn cycle. Long plans (setting up a two-turn combo, sandbagging) are only
  as good as the heuristic plays inside the playouts and the evaluation's `combo_progress` term.
- **Blocks are heuristic, and there are no politics or deals.** Players attack the biggest threat or a lethal
  target; nobody negotiates.
- **Combos resolve abstractly.** Once a combo is assembled and survives its interruption window, the loop's end
  result is applied (usually a win) without playing each iteration.
- **Timeouts.** A game unresolved after 20 rounds is scored for the player with the most life plus board. Wins by
  timeout are reported separately.

The numbers are best read comparatively: tier against tier, list against list, profile against profile. A paired
A/B difference that holds under both profiles is the most reliable output.

---

## 17. How to extend it

**Add or change a card in your deck.** Run `python3 -m commander_sim.update_deck <deck> <new list>` with the
whole new list. It validates the list, rewrites the deck file's list sections and the deck-guard fixture, and reports
how each new card is modeled.

- A card without a `cards/carddb.py` line is fetched from Scryfall and modeled automatically. Check it with
  `python3 -m commander_sim --deck <key> --cards` and `python3 -m commander_sim.pool_audit --mine`.
- If the automatic model is wrong, you have three options: add a `cards/carddb.py` line, write its abilities into
  `data/cards_dsl.json`, or add a hook in `cards/impl/mine.py`.
- If the card needs a new decision from your deck's AI, add it to the deck's priority function in `ais.py`.

**Model a new pool card.**

- If it compiles, it may already work (check `python3 -m commander_sim.pool_audit`).
- If it compiles wrongly, add a fix in `cards/impl/fixes.py` or an override in `cards/pool_cards.py`.
- If it needs code, register hooks with `@CI.on(name, event)` in the impl module for its tier (or `cards/impl/common.py`
  if it's shared), and record its status with `note()`.

**Add a pool deck.**

1. Put its file in the tier folder, following an existing file's layout.
2. Run `python3 -m commander_sim.pools --validate`.
3. Add a `pool_decks.CONFIG` entry (`ai/pool_decks.py`) if the generic AI misplays it.
4. Rerun `--calibrate within` for its tier.

**Add a combo.** Declare it with `@combo(name, groups, mana=…)` in `cards/impl/combos.py`. The function checks readiness
and returns the key permanents and the cards still to cast. The look-ahead's `combo_progress` and the pool AI's wish
lists pick it up automatically.

**Add a hook event.** Fire it from the engine at the right moment with `CI.fire` (triggers) or `CI.total` (numeric
queries), and document it in `cards/cardimpl.py`'s docstring.

**Tune the AI.**

- Heuristic utilities live in `ai/brain.py` (`card_utility`, the option builders).
- Deck priorities live in `ais.py` for your decks and `ai/pool_decks.py` / `ai/deck_plans.py` for pool decks.
- Look-ahead settings are the constants at the top of `ai/search.py`.

Before and after an AI change, run the calibration and a paired A/B on a fixed seed range, so the change's effect is
measured, not guessed.
