# Architecture of the Commander pod simulator

This document explains how the simulator is built: what each part does, how a game is played, how the AI decides,
how cards are modeled, and how a run turns thousands of games into a win rate. It is written for someone who knows
Magic and a little Rust and wants to change the code or trust its numbers. Practice mode (playing a deck by hand) is
in [practice-mode.md](practice-mode.md).

It describes the code as of October 2026, after the move from Python to Rust. Function and module names are given
instead of line numbers, since those drift. The code is the Cargo workspace in `rust/`; paths below are relative to
`rust/crates/` (`sim-core/src/ai/search.rs`), and commands run from the repository root. The history of the move is in
[rust-rewrite-scope.md](rust-rewrite-scope.md); the data model it chose is in [rust-design.md](rust-design.md).

## Contents

1. [What the simulator is for](#1-what-the-simulator-is-for)
2. [The big picture](#2-the-big-picture)
3. [Crates and modules at a glance](#3-crates-and-modules-at-a-glance)
4. [The game model](#4-the-game-model)
5. [A game from start to finish](#5-a-game-from-start-to-finish)
6. [Casting, the stack and responses](#6-casting-the-stack-and-responses)
7. [Combat](#7-combat)
8. [The heuristic AI](#8-the-heuristic-ai)
9. [The look-ahead AI](#9-the-look-ahead-ai)
10. [How cards are modeled](#10-how-cards-are-modeled)
11. [The opponent pools](#11-the-opponent-pools)
12. [Running experiments](#12-running-experiments)
13. [Randomness, pairing and reproducibility](#13-randomness-pairing-and-reproducibility)
14. [Guards against runaway games](#14-guards-against-runaway-games)
15. [Tests and tools](#15-tests-and-tools)
16. [Simplifications and their effect on results](#16-simplifications-and-their-effect-on-results)
17. [How to extend it](#17-how-to-extend-it)

---

## 1. What the simulator is for

The question the simulator answers: **how often does one of my Commander decks win a four-player game against
opponents of a given power level?**

- **Your decks** are the seven lists in `decklists/JD/` and `decklists/Avery/` (`data/my_decks.toml`): Sephiroth (Atraxa
  reanimator, key `seph`), Veyran (Izzet spellslinger, `veyran`), Sauron (Grixis amass, `sauron`), Y'shtola (Esper
  Drain, `yshtola`), Jodah (WUBRG legends, `jodah`), Galadriel (Bant Rebels, `galadriel`) and Alela (Esper Faeries,
  `alela`).
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

It is written in Rust. The simulator itself never uses the network: card data is built ahead of time by `cardgen`,
which fetches a card from Scryfall the first time it is seen and caches it in `data/scryfall_cache.json`. Only
practice mode fetches card images.

---

## 2. The big picture

```
                         ┌──────────────────────────────────────────────┐
  command line  ───────► │ sim-cli: commander-sim                       │
                         │   args, poolmode (run, A/B, matrix,          │
                         │   calibrate, analyze), play (worker threads),│
                         │   report, stats                              │
                         └───────────────┬──────────────────────────────┘
                                         │ seeds → games (sim-core::run::Engine)
                         ┌───────────────▼──────────────────────────────┐
                         │ sim-core engine/turn.rs   game setup, the    │
                         │   turn loop, upkeep / end step; combat.rs    │
                         └──────┬───────────────────────┬───────────────┘
                   decisions    │                       │ rules
               ┌────────────────▼───────┐   ┌───────────▼────────────────┐
               │ ai/brain.rs  heuristic │   │ engine/  state, mana,      │
               │ ai/search.rs look-ahead│◄─►│  casting, stack, removal,  │
               │ ai/pool.rs, plans.rs,  │   │  zones, life               │
               │ seph.rs, veyran.rs ... │   └───────────┬────────────────┘
               └────────────────────────┘               │ card behaviour
               ┌────────────────────────────────────────▼────────────────┐
               │ Card data (data/cards.json, written by cardgen)         │
               │   tags · compiled abilities (dsl/) · Rust card code     │
               │   (hooks.rs, impls/*.rs)                                │
               └─────────────────────────────────────────────────────────┘
```

The design has four layers:

1. **Card data** describes what each card does, in one of three forms: short *tags* (`rem=exile tgt=nl`), *ability
   data* compiled from Oracle text, or *Rust card code* for cards neither can express.
2. **The engine** holds the game state and applies the rules: paying mana, casting, the stack, removing, dying,
   drawing, triggering.
3. **The AI** decides what each player does. A fast heuristic AI scores every legal play; the look-ahead AI, the
   default, tests the best few plays on copies of the game.
4. **The experiment layer** plays many seeded games on worker threads and turns them into statistics.

The layers talk through a few narrow interfaces: the engine calls the AI only at decision points (main phases,
attacks, priority windows), and calls card code only through tags, the ability interpreter, and named hook events.

---

## 3. Crates and modules at a glance

```
rust/crates/sim-core/    the library: engine, AI, cards
rust/crates/sim-cli/     commander-sim, the experiment layer
rust/crates/practice/    commander-practice, practice mode (documents/practice-mode.md)
rust/crates/cardgen/     cardgen, the card data builder
data/                    cards.json, decks.json (written by cardgen), scryfall_cache.json, hand-edited sources
```

**sim-core** (`sim-core/src/`)

| Module | Role |
|---|---|
| `state.rs`, `ids.rs`, `sym.rs` | The game state (`Game`, `Player`, `Perm`, `Land`, `StackItem`), the ids that name things (`CardId`, `PlayerId`, `PermId`, `LandId`), and interned strings. |
| `cards.rs`, `tag.rs`, `export.rs` | Card definitions (`CardDef`), the tag vocabulary (`Tag`), and loading `data/cards.json` and `data/decks.json`. |
| `engine/` | The rules. `values.rs` (what a permanent is and is worth), `life.rs` (life, damage, state-based checks, elimination), `mana.rs` (sources, payment, costs), `zones.rs` (drawing, tokens, entering and leaving), `cast.rs` (casting and resolution), `stack.rs` (triggers, the stack, priority, counterspells), `removal.rs` (removal and wipes), `tutors.rs`, `combat.rs`, `turn.rs` (setup, mulligans, the turn loop and its steps), `hooks.rs` (calling card code). |
| `ai/` | `brain.rs` (the heuristic AI), `search.rs` (the look-ahead), `pool.rs` (the generic AI for outside decks), `plans.rs` (per-deck configuration and plans for pool decks), `decks.rs`, `seph.rs`, `veyran.rs`, `topdeck.rs` (your decks' plans and library ordering), `act.rs` (making a chosen option happen), `tape.rs` (practice mode's record of look-ahead decisions). |
| `dsl/` | The ability language: `model.rs` (the typed abilities) and `mod.rs` (the interpreter). |
| `hooks.rs`, `cardcode.rs`, `impls/` | The hook table (`Event`, `CardImpl`, `Registry`) and card code: `common.rs`, `t1.rs` to `t5.rs`, `combos.rs`, `topdeck.rs`, `fixes.rs`, `lands.rs`, `partials.rs`, `rules.rs`, `rules2.rs`, `mine.rs` and the per-deck files. |
| `human/`, `control.rs` | The person's seat in practice mode: the requests the engine asks, and the rules check on the answers. |
| `flow.rs`, `rng.rs`, `settings.rs`, `run.rs` | Interruptions (`Res`, `Stop`), random numbers, run-wide settings (profile, AI mode, look-ahead knobs), and `Engine`, which loads the data once and plays a seeded game. |
| `testkit.rs` | Building positions for tests and examples. |

**sim-cli** (`sim-cli/src/`): `args.rs` (the command line), `decks.rs` (deck files and the pool), `play.rs` (seat drawing, worker threads, one game's results), `poolmode.rs` (the run modes), `report.rs` and `stats.rs` (reports and statistics), `tools.rs` (`swaptest`, `searchtest`, `pools`), `progress.rs`, `pyrand.rs` (the seat-order generator, section 13).

**cardgen** (`cardgen/src/`): `sources.rs` (where each card's definition comes from), `scryfall.rs` (client and cache), `autotag.rs` (regex tagger), `dsl_parse.rs` and `dsl.rs` (the compiler from Oracle text to abilities), `export.rs` (writes the two JSON files), `decks.rs` (deck files and validation), `update_deck.rs` (`cardgen deck`), `audit.rs` (how faithfully each card is modeled).

---

## 4. The game model

### Card definitions: `CardDef`

A `CardDef` is the immutable definition of a card, built once per process from `data/cards.json` and shared by every
game through the `CardDb`. It holds:

- **name, types and cost.** Types are bit flags: land, creature, instant, sorcery, artifact, enchantment,
  planeswalker (one-letter codes `L C I S A E P` in the data files). The cost splits into generic mana and coloured
  pips (`3GWUB` becomes generic 3, pips `GWUB`).
- **tags**: a bit set plus short value lists, from a string like `bomb=9 pow=7 fly vig dt lifelink leg atraxa`.
  Tags are the engine's main vocabulary (`tag.rs`). Some describe the card (`pow`, `tgh`, `fly`), some describe what
  it does (`rem=exile`, `draw=2`, `tut=any`, `ctr=any`), some name a role for the AI (`bomb=9` means "a top threat
  worth 9").
- **abilities**: an optional list in the ability language (section 10).
- keyword data from Scryfall: keywords, subtypes, protection and ward.
- **bookkeeping**: where the definition came from, the Oracle lines nothing understood, and the Game Changer flag.

### Game objects

Nothing in a game points at anything else: objects are named by small numbers, so a `Game` is plain data and
copying one (for the look-ahead) is copying a few arrays. See [rust-design.md](rust-design.md).

| Type | What it is |
|---|---|
| `Land` | A land on the battlefield: its `CardId`, whether it is tapped, and a small data list. Lands live in `Game::lands`, separate from other permanents. |
| `Perm` | A nonland permanent or token: controller, original owner, tapped / summoning-sick state, power and toughness, counters, flags (flying, deathtouch, vigilance, lifelink, phased, attached), loyalty, and card-specific state (`PermData`). Tokens have no card definition. |
| `Player` | A seat. Zones as lists of `CardId` (the top of the library is the last element): library, hand, graveyard, exile; lands and permanents as ids. Also life (starts at 40), commander tax and whether the commander is in the command zone, commander damage taken, treasures, clues and floating mana, per-turn counters, and stats for reports. |
| `Game` | The table. Holds the players in seat order, the two arenas of permanents and lands, the stack and the trigger queue, the decision random generator, the round, the active player and step, the winner and win type, the step counter with its caps, the optional trace log, and the shared `CardDb` and `Settings`. |

Cards in zones are ids, not instances: two copies of a basic land in a library are the same id twice. Only the
battlefield has per-object state (`Land`, `Perm`).

### State-based checks

`check_state(g)` runs after almost every action. It fires the state-based hook event and eliminates any player who:

- is at 0 life or less;
- drew from an empty library;
- took 21 commander damage from one commander;
- has 10 or more poison counters.

When one player is left, the game ends with that player as winner.

`eliminate` removes the player's permanents and records who dealt the final blow and how (combat, burn, drain,
combo, commander damage, decked), for the reports.

### Mana

Mana is computed, not tracked as a pool:

1. **`mana_units(g, p, convoke)`** lists every source that could pay right now. Sources include:
   - untapped lands, with their colours from tags; hooks adjust for Blood Moon, Chromatic Lantern and the like;
   - mana rocks (`rock=amount:colour`) and untapped, non-sick mana dorks (`dork=colour`);
   - treasures;
   - floating mana from rituals and Birgi;
   - mana-producing tokens;
   - creatures that can convoke.

   The hook layer adjusts the list: locks (Cursed Totem, Collector Ouphe, Karn), bonuses (Kinnan, Urza),
   dynamic amounts (Priest of Titania) and hand mana (Elvish Spirit Guide).
2. **`plan_pay(g, units, generic, pips)`** is a greedy planner:
   - Coloured pips are assigned first, scarcest colour first, and each pip uses the least flexible source that
     makes it.
   - Generic mana is then filled, preferring non-treasure, painless sources that waste the least.
   - It returns how much of each unit is used, or `None` if the cost can't be paid.
3. **`pay(g, p, generic, pips, convoke)`** executes the plan: taps sources, sacrifices treasures, spends floating
   mana and applies side effects (painlands, Ancient Tomb, Talismans, mana-tapped hooks).

`can_pay` is `plan_pay` without paying, and the AI calls it constantly. `cost_of(g, p, c)` gives a card's current
cost after reductions, taxes and commander tax. Floating mana empties at the start of each turn, not between phases.

### Interaction profiles

Two profiles set how interactive the AI opponents are (`settings.rs`, `--profile`):

| Profile | Pool decks counter a spell of importance… | Instant-speed removal |
|---|---|---|
| `conservative` | 7 or more | Held for emergencies (a penalty on casting instants proactively) |
| `loose` | 6 or more | Used as freely as sorcery-speed removal |

A pool deck with many counterspells counters at a lower importance (`ai/pool.rs`: 0.3 lower per counter in the
list, at most 2.5).

Your decks have their own thresholds in the same table: Sephiroth 6 under both profiles; Veyran and Sauron 7
conservative, 6 loose.

Loose is the worst case for your decks, because opponents answer more. Most recent measurements use loose only.

---

## 5. A game from start to finish

### Setup

`Engine::play(seed, seats, settings, max_rounds, trace)` (`run.rs`) builds a game from the seats (deck key, card
list, commander name) in seat order, through `setup_game` in `engine/turn.rs`:

1. Each seat becomes a `Player`. The commander is taken out of the library into the command zone.
2. The game's decision generator is seeded from `play:{seed}`.
3. Each seat shuffles and mulligans with its own generator, seeded from `lib:{seed}:{deck key}` (section 13 explains
   why).
4. **Mulligan rule** (`mulligan`): keep a seven-card hand with 2 to 5 lands. Otherwise draw seven again, up to
   three redraws; the third and fourth redraws put one or two cards on the bottom (a spare land if flooded, else the
   most expensive spell).

### The round loop

`run_rounds(g, 20)`:

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

The **end-of-turn window** (`end_of_turn_window` in `ai/brain.rs`) is how a player spends mana they held up. Just
before a player's turn, that player may use instant-speed draw, tokens, flashback, removal and activated abilities.
Mana untaps next anyway, so holding it has no further value. (Responses to spells and abilities, and priority in
every step, are in section 6.)

### A turn

A turn is five steps, each a function in `engine/turn.rs`:

| Step | What happens |
|---|---|
| `start` | Untap, reset per-turn state, **upkeep**, then the **draw step** (skipped for Necropotence, dredge and the like), then the **land drop** (the AI picks which land). |
| `main1` | The main-phase AI plays spells and abilities until it decides to stop. |
| `combat` | Up to four combats: attack declaration, blocks, damage, extra combats. |
| `main2` | The main-phase AI again, knowing how combat went. |
| `end` | End-step triggers, discard to seven, cleanup of "until end of turn" effects. |

Upkeep (`upkeep`) handles first:

- animated lands reverting;
- crewed vehicles ending;
- delayed draws;
- Pact of Negation payments;
- saga chapters and rebound spells.

It then applies the upkeep effects in `upkeep` itself (Tabernacle, Glacial Chasm, …), the upkeep hooks and the
ability interpreter's upkeep triggers.

`continue_turn(g, p, step)` can resume a turn from any step. The look-ahead uses it to play a copied game forward
from the middle of a turn.

### How the game ends

A game ends when one player is left (a win by damage, drain, commander damage, poison, decking, or an opponent's
elimination), when a combo resolves (win type `combo`), or after 20 rounds (`timeout`). The report counts each win
type.

---

## 6. Casting, the stack and responses

Spells, activated abilities (except mana abilities) and triggered abilities all go on a real stack
(`StackItem` in `state.rs`; `engine/stack.rs`). Players get priority in turn order, starting with the active player,
and may respond with anything they can do at instant speed. The top item resolves when every player passes in
succession. Every step except untap and cleanup gives priority; most passes are free, because a player with nothing
affordable at instant speed passes at once. [stack.md](stack.md) is the design and records how it was built.

A stack item is data, not a closure (`kind`: spell, ability or trigger; controller; source; targets; context; an
effect named by a key plus plain arguments), so a look-ahead copy can be taken in the middle of a stack.

### The cast pipeline

```
AI picks a play
   │
   ├─ pay the cost (pay)                     the card leaves the hand before paying
   │
   ▼
cast_card(g, p, c, zone, ctx)               engine/cast.rs
   ├─ move the card out of its zone (hand, graveyard, command zone, library)
   ├─ on_cast: cast triggers
   │     opponents' Rhystic Study, Sauron amass, Kaervek; magecraft; prowess;
   │     ability-language cast triggers; hook 'cast' events; copies (Jin-Gitaxias, Ral, Return the Favor)
   ├─ spell_imp: how important is this spell (to each opponent)?
   ├─ the spell goes on the stack and the priority loop runs: counters, protection, removal in response,
   │     flash creatures, abilities, to any depth
   └─ if it was not countered: resolve(g, p, c, ctx, zone), then check_state
```

Triggered abilities wait in a queue until whatever is resolving has finished, then go on the stack in APNAP order
(each player orders their own: a person chooses, the AI by value). A hook is probed first to find out whether it
really triggers (`Stop::Probe`, section 14), then pushed, then run for real, top first.

### Resolution

`resolve` dispatches in this order:

1. **A card-code hook** registered for the card's resolve event handles the whole spell.
2. **An ability-language spell** (a nonpermanent with abilities) is run by the interpreter (`dsl`).
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
- **The heuristic decision.** `wants_counter` (`ai/brain.rs`) gives a probability from how far the spell's importance
  is over the threshold and how many counters the player holds.
- **The look-ahead decision.** With look-ahead on, a counter against a spell cast in the active player's main phase
  is decided by `search::choose_counter` instead: it plays both futures, countered and not countered.
- **Which counter.** `pick_counter` chooses among the counters in hand: the free ones (Fierce Guardianship, Force of
  Negation, Pact of Negation, Mental Misstep) when their conditions hold, otherwise the cheapest one that can be paid
  for.
- **After the counter.** "Soft" counters (Spell Pierce, Mana Leak) let the caster pay. Side effects apply
  (`counter_side_effects`): Arcane Denial, An Offer You Can't Refuse, Mana Drain. A counterspell is itself a spell on
  the stack, so it can be countered in turn.

### Other response windows

Some situations keep a small explicit function on top of the stack:

| Window | Where | What it does |
|---|---|---|
| Protection | `protect_response` (`ai/brain.rs`; `ai/pool.rs` for pool decks) | When removal targets a valuable permanent, its owner may answer with Heroic Intervention, a phase-out, a sacrifice in response, a blink or a redirect. |
| Wipe response | `wipe_response` | Players answer a board wipe (Teferi's Protection, Heroic Intervention). |
| Combo interruption | `impls/combos.rs` | Before a combo loop runs, each opponent gets one chance to remove a key piece at instant speed, or counter the last piece. |
| Reanimation response | the graveyard-response hook | Graveyard hate (Soul-Guide Lantern, Tormod's Crypt) answers a reanimation spell. |
| Ward and removal taxes | `apply_removal` | Ward costs, Sauron's ward (sacrifice a legend), taxes and prevention effects. |

`apply_removal` (`engine/removal.rs`) is the single place removal lands. It checks, in order:

1. whether the target is still there and targetable;
2. indestructible;
3. the owner's protection response;
4. ward and removal taxes;
5. damage prevention.

Only then does it destroy, exile, bounce, tuck or transform the permanent. It also applies riders such as the life
gain from Swords to Plowshares and the land from Path to Exile.

---

## 7. Combat

`combat` (`engine/combat.rs`) runs up to four combat phases a turn:

1. **Attackers.** Every untapped, non-sick creature with power above zero is a candidate; mana dorks attack only
   when pumped to 3 power.
2. **The plan.** At the first combat, the look-ahead AI picks a plan: which opponent to attack and whether to send
   the filtered attackers, everyone, or nobody (`search::choose_attack`). Otherwise the heuristic picks the defender
   (`choose_defender`: a lethal target if there is one, else the biggest threat with some randomness) and
   removes attackers worth keeping home as blockers (`filter_attackers`, and `attack_filter` in `ai/pool.rs`).
   **Splitting** (`split_attack`): the attackers may then go at different players. An attacker moves when it
   finishes off a player who can't block it, when the chosen defender has a blocker that would eat it (as many
   attackers as there are such blockers), or when an outside deck held it back from the defender but it is safe
   elsewhere. The look-ahead's "everyone" plan stays at one player; its "filtered" plan includes the splits. Each
   defending player then gets restrictions, blocks and damage in turn (in the rules they happen at once); attack
   triggers fire once for the whole attack. A person in practice mode can split attackers too.
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

## 8. The heuristic AI

This is the adaptive AI (`ai/brain.rs`). It runs alone in `--ai adaptive` mode, and it is also the policy that plays the look-ahead
AI's simulated futures.

### One main-phase decision

```
opts = main_options(g, p, post)        every legal play as an Opt (utility, label, action), plus 'stop'
if look-ahead is on and there are ≥ 2 options:
    pick = search::choose(...)         (section 9)
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

**Hidden information is estimated from public information only.** `prob_holding` works out the chance
that opponent q holds a card of some kind, from q's decklist minus the copies already seen in the graveyard, exile
and battlefield. It is then combined with q's open mana.

**Utilities.** `card_utility` starts from the deck's own priority for the card and adjusts it for the situation:

- the priority comes from the deck's own function (Sephiroth's and Veyran's in `ai/seph.rs` and `ai/veyran.rs`, the
  others in `ai/decks.rs` and the deck files in `impls/`) for your decks, or `generic_prio` in `ai/pool.rs` plus
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

**Temperature.** Each deck has a play style (in `ai/brain.rs` for your decks, `ai/plans.rs` for pool decks) with
three knobs:

- `temp`: randomness. Lower is sharper and more predictable; `--temp` scales it globally.
- `aggression`: how much pressure it applies.
- `caution`: how much it holds up interaction and keeps blockers home.

Sampling from a softmax instead of always taking the top score makes play varied and human-like, and it stops one
tiny scoring error from deciding every game the same way.

### Your decks vs pool decks

Your seven decks keep hand-written plans (`ai/seph.rs`, `ai/veyran.rs`, `ai/decks.rs`, and the per-deck files in
`impls/` for Galadriel, Y'shtola, Alela and Jodah):

- priority functions;
- reanimation targets;
- combo checks (Veyran's kitten combo, Sauron's Sword + Aggravated Assault);
- protection choices;
- special options in `special_options`.

Pool decks use the generic rules in `ai/pool.rs` plus the per-deck configuration of section 11.

There is no rigid AI any more: `--ai` takes `lookahead` or `adaptive`.

---

## 9. The look-ahead AI

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
   - `clone(g)` copies the game (it is plain data, so this is a few array copies, microseconds);
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

`Game` derives `Clone`. Card definitions and settings are shared through an `Arc`, objects refer to each other by id,
and tables that were keyed by object identity are fields of the object, so nothing needs re-keying. A copy carries no
people (practice mode's seat is played by the AI inside it). The trace log is detached during the copy, so simulated
futures never print.

Copies are marked as searching, so a copy never searches itself: inside a copy, every player uses the heuristic AI.

### Cost and limits

A look-ahead game costs far more than a heuristic one (the heuristic AI alone is about 60 times faster). A typical game
makes about 125 look-ahead decisions and about 270,000 playout steps. Section 14 lists the caps that keep the worst cases bounded.

---

## 10. How cards are modeled

A card can be modeled in three ways, and one card may use more than one. `cardgen export` builds the data for the
first two into `data/cards.json`; the third is Rust code in `sim-core`.

### 1. Tags (`data/carddb.txt`, `cardgen/src/autotag.rs`)

A tag string names what the card does in the engine's vocabulary. Examples:

```
Anguished Unmaking|I|1WB|rem=exile tgt=nl lose=3
Arcane Signet|A|2|rock=1:A
Archon of Cruelty|C|6BB|bomb=9 pow=6 fly archon
Demonic Tutor|S|1B|tut=any
```

The engine and the AI read tags everywhere: `resolve` executes them, `mana_units` reads `rock` and `dork`,
`pval` reads `bomb`, and the AI's utilities read role tags. A tag that names a single card (`archon`, `atraxa`,
`veyran`) points at code written for that card. `sim-core/src/tag.rs` lists every tag and the kind of value it takes.

Your decks' cards are hand-tagged in `data/carddb.txt` and verified against Oracle text. For any other card,
`autotag.rs` produces tags from Oracle text with regular expressions and lists the lines it didn't understand.

### 2. The ability language (`cardgen/src/dsl_parse.rs`, `sim-core/src/dsl/`)

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

- **`dsl_parse.rs`** (in cardgen) compiles Oracle text into this form with a library of templated rules, at export
  time. The result is stored in `cards.json`.
- **`sim-core/src/dsl/`** is the interpreter; `model.rs` is the typed form of the abilities, parsed once when the data
  loads. Its entry points:
  - firing triggers on an event;
  - resolving instants and sorceries;
  - static abilities (power and toughness, keywords, cost changes, token multipliers);
  - offering activated and loyalty abilities to the AI;
  - an estimate of a card's value for the AI, for a card no priority table knows.

A card can also get hand-written abilities in `data/card_overrides.toml` (`dsl = [...]`), which replace the compiled
ones.

### 3. Rust card code (`hooks.rs`, `impls/*.rs`)

For cards neither tags nor the ability language can express (locks, taxes, commander engines, combos, unusual
replacement effects), a function is registered by card name and event. Each card's code is a `CardImpl`: one typed
slot per event, so a hook with the wrong shape doesn't compile. `Registry` maps a card to its `CardImpl`; a name that
isn't in the card database is an error at start-up, so a typo can't silently leave a card unimplemented.

```rust
r.card(db, "Smothering Tithe")?.draw = Some(tithe);     // in the register function of an impls/ file
fn tithe(g: &mut Game, src: Src, p: PlayerId) -> Res { ... }
```

**Events.** `Event` in `hooks.rs` lists them all (77):

- triggers: `etb`, `leaves`, `dies`, `cast`, `attack`, `blocks`, `combat_damage`, `upkeep`, `end_step`, `draw`,
  `landfall`, `sacrifice` and more (these go on the stack);
- AI options: `options`, which offers activated abilities;
- static queries: `cost`, `can_cast`, `attack_tax`, `attack_cap`, `trigger_copies`;
- zone hooks: hand and graveyard options, land options, `resolve` and more.

**Dispatch.** When a hooked permanent enters, it is tracked on the game. The engine (`engine/hooks.rs`,
`cardcode.rs`) calls the registered slots to fire triggers, to sum numeric answers (taxes, extra mana), to ask
whether something may be cast, and to walk the listeners. A pending trigger is stored as data (the event and its
arguments) and its hook is looked up again when it resolves, so a copied game can carry pending triggers.

**Where the implementations live** (`sim-core/src/impls/`):

| Module | Contents |
|---|---|
| `common.rs` | Staples shared by many decks: Auras, equipment, sagas, planeswalker helpers. |
| `t1.rs` … `t5.rs` | Each tier's commanders and signature cards. |
| `combos.rs` | Combos as a unit: pieces, readiness check, mana, and the result. |
| `topdeck.rs` | Scry, surveil, top-of-library manipulation, and which card a player wants next. |
| `lands.rs` | Utility lands and creature lands. |
| `rules.rs`, `rules2.rs` | Rules that were approximated, made exact: mana sources, taxes, counter interactions, locks, emblems, crew, evoke, dash and similar mechanics. |
| `fixes.rs` | Replacements for cards the compiler reads wrongly. |
| `partials.rs` | Completing cards the audit listed as Partial. |
| `mine.rs` | Cards in your decks that need more than tags: the Ring, equipment, lands and full card text. |
| `marchesa.rs`, `zur.rs` | Cards first written for the Marchesa and Zur decks (removed 2026-10-07) that other lists still run (Marchesa's return trigger in Jodah's 99; the lock Auras, Zur's attack trigger and the protection and wipe responses Y'shtola's AI uses). |
| `galadriel.rs`, `yshtola.rs`, `alela.rs`, `jodah.rs` | One deck each: its cards, its AI priorities and responses. |

`impls/mod.rs` registers the modules in a fixed order. A later registration for the same card and event replaces an
earlier one.

### Choosing the source for a card

`cardgen export` (`cardgen/src/sources.rs`) builds each card's definition in this order; the first source that has the
card wins:

1. **`data/carddb.txt`**: hand-verified tags.
2. **Scryfall text compiled into ability data**, with Scryfall's keyword list and the tags the regex tagger found
   alongside.
3. **Scryfall regex tags only** (`autotag.rs`), when nothing compiled.

Then **`data/card_overrides.toml`** applies overrides to a card (replacement tags or abilities, keyword data, the
audit's verdict). It skips the tags and abilities of your hand-tagged cards, because your decks' AI is built around
their tags.

**Rust card code sits on top of any source.** A card with hooks also keeps its tags or ability data; the hook adds or
replaces specific behaviour. Details and the file formats are in `rust/crates/cardgen/README.md`.

### The card audit

`cardgen audit` rates every card:

- **Full**: hand-verified.
- **Full-auto**: every Oracle line compiled.
- **Approximate**: modeled with a simplification.
- **Partial**: the main effect is in, some text is missing.
- **Unmodeled**: the simulator never casts it, or it does nothing.

The rating comes from `documents/card-audit.md`, the verdicts in `data/card_overrides.toml`, `data/audit.toml` and the
compiler's unparsed lines. `--mine` audits your decks. After the September 2026 pass, every card in your decks rates
Full.

---

## 11. The opponent pools

### Deck files

Every deck, yours and the pool's, is a Markdown file. The simulator reads:

- the `## Import list` block: `1 Card Name` lines.
- the `- **Commander:**` line.
- the `**Game Changers (N)**` claim.
- the `## Sim modeling notes` section (pool decks), which the audit uses.

Everything else (strategy, role in the pool, change log) is for people.

A pool deck's tier comes from its folder name (`t3-high-b3` becomes `t3`) and its key from the file name. Your decks
are named in `data/my_decks.toml`. `cardgen export` puts every deck's list, commander and colour identity into
`data/decks.json`, which is what the engine plays from; `sim-cli/src/decks.rs` reads the deck files for the
run harness.

### Validation

`cardgen validate` checks every pool deck for:

- 100 cards, singleton except basics;
- the commander present and a legal commander;
- every card name resolving exactly on Scryfall;
- colour identity;
- Commander bans;
- the number of Game Changers per tier (t1 none, t2 one or two, t3 exactly three, t4 and t5 unrestricted),
  matching what the file claims.

### Per-deck AI

`ai/plans.rs` holds, for each pool deck that needs one:

- **a style**: `temp`, `aggression`, `caution`.
- **key cards**: fixed cast priorities for named cards.
- **a wish list**: for tutors, or a function returning one, such as the missing pieces of the deck's combo.
- **a custom priority function**, for decks whose plan the generic rules can't see (Yawgmoth, Heliod, Chulane,
  Yuriko, Kaalia).
- **when to cast the commander**.

Decks without an entry use the default style and the generic priority (`ai/pool.rs`). The generic priority rates
fast mana highest early, engines and anthems next, and holds counters, removal and protection for their response
logic.

### Calibration

The pools are only useful if the tiers are ordered and each tier is internally balanced. Two checks measure this
(`commander-sim --calibrate`):

- **Within-tier balance**: games of four decks drawn from one tier. Each deck should win 15–35% of the games it
  plays; outside that it is flagged HIGH or LOW.
- **Tier ordering**: each deck of a tier alone against three decks from the tier below. It should win more than 25%.

Results, tuning decisions and history are in `decklists/pool/pool-results.md`. Opponent lists are only changed with
explicit approval; each change is noted in the deck file.

---

## 12. Running experiments

### Commands

```
commander-sim --deck seph --pool t3 --games 1500                       one deck vs one tier
commander-sim --deck seph --pool t3 --swap "Out=>In"                   paired A/B
commander-sim --all-decks --pool all --profile loose                   deck × tier matrix
commander-sim --deck seph --pool t4 --analyze                          how it wins and loses
commander-sim --deck seph --pool t2 --trace 7                          play-by-play of one game
commander-sim --calibrate within|ordering|all --games 240              pool balance checks
```

`--jobs` defaults to every CPU core. Each run starts with one line giving the number of games, the AI, the workers
and an estimated time, and a progress bar that moves as games finish. The full option list is in
`rust/crates/sim-cli/README.md`.

### The run

`sim-cli/src/main.rs` parses the arguments (`args.rs`), builds a `Settings` (profile, AI mode, temperature) and an
`Engine` (the card data, loaded once and shared), and hands the work to `poolmode.rs`, which chooses the mode:

- **run**: one deck against one tier;
- **A/B**: the baseline list and the swapped list on the same seeds;
- **matrix**: every deck against every tier;
- **analyze**: the detailed report;
- **calibrate**: the two pool checks.

### Parallelism

The seeds (500000, 500001, …) are played on `--jobs` worker threads (`play.rs`), each taking the next unplayed seed.
A finished game is boiled down to a small result record (win, turns, per-opponent seat counts, wins and eliminations,
per-axis values); the records are merged in seed order, so results don't depend on the number of threads, because each
game depends only on its seed.

A game that panics is caught, counted and its seed printed, so it can be replayed with `--trace`. A game that runs out
of engine steps ends as a timeout (section 14).

### Statistics

(`sim-cli/src/stats.rs`, `report.rs`)

- **Win rate** with a Wilson 95% interval. At 240 games the interval is about ±6 points; at 1500, about ±2.5.
- **Paired A/B difference**: the mean of the per-seed differences (variant win − baseline win) and its standard
  error. Pairing cancels most of the luck, so a paired difference is much tighter than the difference of two
  independent win rates. The verdict summarises it across profiles.
- **Strength axes** for your decks. Each is reported baseline → variant with a two-standard-error noise band and `*`
  for changes beyond it. The axes:
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

Every random choice comes from a generator seeded by a string, never by time or by a randomised hash:

| Stream | Seed | Used for |
|---|---|---|
| Seats | `'pool:{seed}'` | Which three opponents, and the seat order (`sim-cli/src/play.rs`, using `pyrand.rs`) |
| Library | `'lib:{seed}:{deck key}'` | That seat's opening shuffle and mulligans |
| Play | `'play:{seed}'` | Every in-game decision and random effect |
| Look-ahead | a hash of the decision number, round, … | Each decision's rollout seeds (`search.rs`) |

The game's generator (`sim-core/src/rng.rs`) is xoshiro256** seeded from a 64-bit FNV-1a hash of the stream's name.
The seat draw uses a copy of the Mersenne Twister (`pyrand.rs`) so a seed seats the same opponents it did in the
Python era, which keeps the old results comparable.

This design has three consequences:

- **Any game replays exactly.** `--trace N` plays seed 500000 + N with a full log, and the result matches the run.
- **Swaps are paired.** The seat draw depends only on the seed and the deck keys, never on card lists, and each
  seat shuffles its own library from its own stream. A swapped list therefore faces the same opponents, in the same
  seats, with the same opponent draws. The swap keeps every other card at its list position, so your own library
  order differs only where the swapped card sits. Games diverge once play differs, but early games are identical and
  the rest stay strongly correlated.
- **Parallel runs are deterministic.** The same command gives the same numbers on any machine and with any `--jobs`.

---

## 14. Guards against runaway games

Some decks can loop, or build a board so large that every step crawls. Several caps keep every game finite. None
changes normal games. The limits are in `settings.rs` (`Settings`, `Search`) unless noted.

| Guard | Limit | When it is hit |
|---|---|---|
| Engine steps per game (`game_work`) | 20,000 (normal games use under 1,000) | `Stop::OutOfWork` ends the game as a timeout. |
| Engine steps per playout (`playout_work`) | 2,000 | The playout is scored where it stands. |
| Look-ahead decisions per game (`game_decisions`) | 1,000 (a game uses about 125) | The heuristic AI plays on. |
| Playout steps per game (`game_search_work`) | 3,000,000 (a game uses about 270,000) | The heuristic AI plays on. |
| Board size (`board_limit`) | 150 permanents on the table | No look-ahead at that size; a copy that grows past it stops. |
| Tokens per player (`TOKEN_CAP`, `engine/zones.rs`) | 250 creature tokens | Further tokens aren't created (the board is lethal many times over). |
| Rounds | 20 | Timeout adjudication. |

Interruptions are values, not exceptions (`flow.rs`): an engine function that can be cut short returns `Res<T>`, and
`Stop::OutOfWork` travels up through `?` to whoever handles it. `Stop::Probe` is how the engine finds out whether a
trigger really triggers: it runs the hook until its trigger window and stops it there.

`search::stats()` counts decisions, playouts, changed choices, cut playouts and capped games, for diagnosing the
look-ahead. All integer counters that could go below zero use checked arithmetic, because release builds have
overflow checks on: a counter that wraps panics instead of growing a vector without bound.

---

## 15. Tests and tools

Run all the tests with `cargo test --release` from `rust/` (about 990 tests; the practice HTTP suite is the slowest).
Run one crate with `-p sim-core`, or one test file with `--test <name>`.

| Where | What it checks |
|---|---|
| `sim-core/tests/rules.rs`, `stack.rs`, `land_entry.rs`, `game.rs` | Core rules on hand-built positions: mana, commander tax, state-based losses, counterspells, the stack, removal, wipes, tutors, mulligans, combat keywords, lands' enter-tapped conditions, whole games. |
| `sim-core/tests/cards_*.rs` | Cards against their Oracle text, by file: one per tier (`cards_t1` … `cards_t5`), the shared files, lands, rules, and one per deck (`cards_sauron`, `cards_seph`, `cards_veyran`, `cards_yshtola`, `cards_galadriel`, `cards_alela`, `cards_jodah`, `cards_zur`). |
| `sim-core/tests/ai.rs`, `dsl.rs` | The AI (including the look-ahead: independent copies, evaluation bounds, a reproducible decision) and the ability interpreter. |
| `sim-core/tests/play_*.rs` | A person at the table: mana, the main phase and the rules check, targets and abilities, attacks and blocks, priority on other players' turns, choices, and each deck's card-specific choices. |
| `sim-core` unit tests | Inside the modules (`#[test]`), for the pieces they define. |
| `cardgen/tests/` | `equivalence.rs`: the committed `data/*.json` is what `export` writes, every listed card has data, and the deck-guard fixture (`tests/fixtures/my_decks_parsed.json`) is current; `workflow.rs`: `export`, `deck` and `validate` on copies of the real files. |
| `practice/tests/` | Sessions and the HTTP server against a fake Scryfall; bot games from every deck; see [practice-mode.md](practice-mode.md). |

`sim-core/src/testkit.rs` builds a position for a rule test: seat decks, empty the hands, put chosen cards in hand or
onto the battlefield, then drive the engine directly.

Other tools:

- `cardgen validate`, `cardgen audit [--deck KEY] [--mine] [--md FILE]`: pool validation and modeling coverage.
- `commander-sim pools`: list the pool decks.
- `commander-sim swaptest <deck> <tier> <games> "Out>In; …"`: paired test of pool list changes without editing the file.
- `commander-sim searchtest <deck> <tier> <games> [rollouts] [top_k] [jobs]`: a pool deck with and without look-ahead,
  paired.
- `commander-sim --deck veyran --cards`: every card in a deck, its tags, abilities and unmodeled text.
- Examples in `sim-core/examples/`: `all_decks` (a quick game of every deck, `all_decks adaptive 1`), `bench_games`,
  `clone_cost`, `debug_game`, `play_game`.

---

## 16. Simplifications and their effect on results

The simulator abstracts Magic on purpose. The main simplifications, and which way they push results:

- **Priority is mostly free.** The stack and priority are real (section 6), but a player with nothing affordable to do
  passes at once, and the AI answers a spell or ability only with the responses it models (counterspells, protection,
  removal, flash, activated abilities). Instant-speed play that doesn't answer something happens in the end-of-turn
  window before each player's turn.
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

**Add or change a card in your deck.** Run `cardgen deck <deck> <new list>` with the whole new list (or edit the
deck file), then `cardgen export`. The first validates the list, rewrites the deck file's list sections and the
deck-guard fixture, and reports how each new card is modeled.

- A card without a `data/carddb.txt` line is fetched from Scryfall and modeled automatically. Check it with
  `commander-sim --deck <key> --cards` and `cardgen audit --mine`.
- If the automatic model is wrong, you have three options: add a `data/carddb.txt` line, write its abilities (or
  tags) into `data/card_overrides.toml`, or add card code in `sim-core/src/impls/mine.rs`.
- If the card needs a new decision from your deck's AI, add it to the deck's priority function in `ai/` (or the
  deck's file in `impls/`).

**Model a new pool card.**

- If it compiles, it may already work (check `cardgen audit`).
- If it compiles wrongly, add a fix in `impls/fixes.rs` or an override in `data/card_overrides.toml`.
- If it needs code, register a slot in the `register` function of the impls file for its tier (or `common.rs` if it's
  shared). `rust/PORTING_CARDS.md` shows the pattern, and `impls/t1.rs` is the model file.

**Add a pool deck.**

1. Put its file in the tier folder, following an existing file's layout.
2. Run `cardgen validate`, then `cardgen export`.
3. Add an entry in `ai/plans.rs` if the generic AI misplays it.
4. Rerun `--calibrate within` for its tier.

**Add a combo.** Add it to the combo table in `impls/combos.rs`: the pieces (groups of card names), a readiness check
that returns the key permanents and the cards still to cast, and the mana. The look-ahead's `combo_progress` and the
pool AI's wish lists pick it up automatically.

**Add a hook event.** Add a variant to `Event` and a typed slot to `CardImpl` in `hooks.rs`, and call it from the
engine at the right moment (`engine/hooks.rs`).

**Tune the AI.**

- Heuristic utilities live in `ai/brain.rs` (the option builders and the utility functions).
- Deck priorities live in `ai/seph.rs`, `ai/veyran.rs`, `ai/decks.rs` and the deck files for your decks, and in
  `ai/plans.rs` for pool decks.
- Look-ahead settings are the `Search` defaults in `settings.rs`.

Before and after an AI change, run the calibration and a paired A/B on a fixed seed range, so the change's effect is
measured, not guessed.
