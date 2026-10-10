# Rust port: the data model (M1)

This is the design the rest of the port builds on: how cards, game state, card code, people at the table and
interruptions are represented in Rust, and why. It is the M1 milestone of `documents/rust-rewrite-scope.md`
(section 10). The code is in `rust/crates/sim-core/src/`. Each module starts with a comment saying how it maps to the
Python.

## Contents

1. [The main choice: a game is plain data](#1-the-main-choice-a-game-is-plain-data)
2. [Cards](#2-cards)
3. [Game state](#3-game-state)
4. [Card code: the hook table](#4-card-code-the-hook-table)
5. [Interruptions: `Res` instead of exceptions](#5-interruptions-res-instead-of-exceptions)
6. [People at the table: `Humans`](#6-people-at-the-table-humans)
7. [Run settings and random numbers](#7-run-settings-and-random-numbers)
8. [What M1 does not do yet](#8-what-m1-does-not-do-yet)
9. [Points for review](#9-points-for-review)
10. [Appendix: Python's runtime attributes](#10-appendix-pythons-runtime-attributes)

---

## 1. The main choice: a game is plain data

In Python, objects point at each other: a permanent holds its owner `Player`, an Equipment holds the creature it's
attached to, the game holds both. Copying a game for the look-ahead means `deepcopy` following every pointer, then
re-keying the tables indexed by `id()` (`search.clone`). That copy was about 15% of all the time in the profile.

In Rust nothing points at anything. Every object is named by a small number (`ids.rs`):

| Id | What it names | Python equivalent |
|---|---|---|
| `CardId` | a card definition in the `CardDb` | a shared `CD` object |
| `PlayerId` | a seat, by turn order | a `Player` |
| `PermId` | a permanent or token in `Game::perms` | a `Perm` |
| `LandId` | a land in `Game::lands` | a `Land` |

Permanents and lands live in two arrays on the `Game`, the "arenas". An entry is added when the permanent is
created and never removed or reused. Python keeps every permanent alive until the game ends (`g.alive_objs`) for
the same reason: so a table keyed by a permanent can't be inherited by a new one. A player's battlefield is the list
of ids they control, in entry order (`Player::perms`, `Player::lands`), and a permanent knows whether it's on the
battlefield (`Perm::on_bf`).

So `Game` derives `Clone`, and copying a game is copying a few arrays. **Measured:** a mid-game table (four players
with full zones, 60 permanents, 32 lands) copies in **1.7 µs**, against about 4 ms for Python's `deepcopy`
(`cargo run --release -p sim-core --example clone_cost`). That's about 2,000 times faster.

---

## 2. Cards

`cards.rs`, `tag.rs`. Card definitions are built once per process from `data/cards.json` (the Python export) into a
`CardDb`, which every game shares through an `Arc`. They never change during a game: the Python only adjusts tags
while cards load, and the export comes after that.

- **`CardDef`** has every field of Python's `CD`: types (bit flags), cost, mana value, power and toughness, `bomb`,
  colours, identity, keywords, subtypes, protection, ward, starting loyalty, Game Changer, and the compiled abilities.
  The compiled abilities stay as JSON until the ability language is ported (M3).
- **Tags** are the engine's main vocabulary, consulted millions of times per run. The export has 277 distinct tags,
  and each takes exactly one kind of value: 241 are bare flags (`fly`), 22 numbers (`pow=7`), 14 short strings
  (`rem=exile`). So `Tag` is an enum, and a card's `Tags` is a bit set (whether it has a tag is one AND) plus short
  lists for the values. Python's dict order is kept for the rare code that walks the tags.
- `tag.rs` was generated from the export, and from now on it's kept by hand. A tag the code tests but no card has
  yet (there are a few dozen in the Python) is added when that code is ported. A test checks that every tag in the
  export is known, with the same kind of value, and that every card's tags read back exactly as exported.

---

## 3. Game state

`state.rs`.

- **`Game`**: the players, the two arenas, the stack and the trigger queue, the round, the active player and step,
  the step counter and its caps, the decision random generator, the trace log, and the shared `CardDb` and
  `Settings`.
- **`Player`**: the zones as lists of `CardId` (the top of the library is last, as in Python), the battlefield as
  ids, life, commander damage (an array by seat instead of a dict by deck key), floating mana in one struct
  (Python's `floatR`, `floatA`, `floatU` ...), and the per-turn counters.
- **`Perm`**: every slot of Python's `Perm`, plus `on_bf`, and the end-of-turn pump and keywords (`eot_pt`,
  `eot_kw`).

Three rules for turning Python's loose state into fields:

1. **Attributes Python sets on the fly become declared fields** with the same default. The runtime inventory found
   81 such attributes on the game and 121 on players (section 10). M1 declares the core ones; the rest are added
   as the code that uses them is ported, each with the same name.
2. **Tables Python keys by `id(permanent)` become fields of the permanent.** End-of-turn pumps and granted keywords
   are `Perm::eot_pt` and `Perm::eot_kw`, not dicts on the game. That's simpler, and it can't go stale.
3. **Card-specific state on a permanent (`m.data`) is `PermData`**: a short list of typed keys (`DataKey`, the 64
   keys the inventory found) and values (`Val`: number, card, permanent, player, turn stamp, list ...). It reads
   like the Python dict: `get`, `set`, `truthy` (Python's `if m.data.get('x')`), `int`, `remove`.

Names that sit in game state (token names, subtypes, deck keys, stat names) are interned (`sym.rs`): a `Sym` is a
`&'static str`, so copying a game copies pointers, not strings.

---

## 4. Card code: the hook table

`hooks.rs`. Python registers a function per card and event with `@on('Card Name', 'event')`. 658 cards have such
code, on 77 different events.

- **`Event`** lists all 77. `TRIGGER_EVENTS` (24 of them) are triggered abilities and go on the stack; the rest are
  asked for a value at once: cost changes, locks, options.
- **`CardImpl`** is one card's code: a typed function slot per event. `etb` takes the game, the source permanent,
  the player and the permanent that entered; `cost` returns a mana change; and so on. A hook with the wrong shape
  is a compile error. M1 has slots for 19 events; the rest are added as the engine code that fires each event is
  ported.
- **`Registry`** maps a `CardId` to its `CardImpl`. Card modules register by name at startup, and a name that isn't
  in the card database is an error, so a typo can't silently leave a card unimplemented.
- **Triggers are data.** Python's `Trigger` holds the hook function and its arguments. Here a `Trigger` holds the
  event and its arguments (`Call`), and the hook is looked up again in the registry when it resolves. So a copied
  game can carry pending triggers and stays plain data.
- **AI options are data too.** Python's `(utility, label, fn)` becomes `Opt { utility, label, act }`, where `act`
  is an engine verb (cast this card from this zone) or a card ability (its function plus an argument). The
  look-ahead finds an option again in a copy by its label and which occurrence of that label it is, as Python does.

---

## 5. Interruptions: `Res` instead of exceptions

`flow.rs`. Python cuts a call short with two exceptions:

- **`OutOfWork`:** a game or a look-ahead playout used up its step budget.
- **`TriggerProbe`:** to find out whether a hook really triggers, the engine runs it in probe mode, and reaching
  `trigger_window` raises.

Here every engine function that can be cut short returns `Res<T>` (`Result<T, Stop>`), and `?` carries the stop up
to whoever handles it. A probe happens at every triggered ability, so it has to be cheap: a returned value costs
almost nothing, where a panic would cost an unwind each time. The price is a `?` after most engine calls, which is
ordinary Rust.

`Game::tick` is the step counter. Past `work_cap`, or past `board_cap` permanents (look-ahead copies only), it
returns `Stop::OutOfWork`, as Python's `tick` raises.

---

## 6. People at the table: `Humans`

`control.rs`. The AI's decisions stay inline in the engine and AI code, as in Python. A seat played by a person has
a `Human`, and the engine asks it at the person's decisions:

- `ask(game, seat, Request) -> Answer` waits for the answer. A `Request` has the same kind, prompt and choices as
  Python's `controller.Request`.
- `tell(seat, kind, text)` passes on a message that needs no answer.

The practice server's `Human` (phase 9) will send each request to the browser and block the engine thread until
the answer arrives. Python's 22 helpers (`choose`, `yes_no`, `pick_cards`, `scry` ...) become functions on top of
`ask`, as `choices.py` builds them on the controller.

**A copied game never has people in it:** the look-ahead plays every seat with the AI. Python's `search.clone`
drops `g.controllers` by hand. Here cloning `Humans` always gives an empty table, so it can't be forgotten.

---

## 7. Run settings and random numbers

`settings.rs`, `rng.rs`.

- **Settings:** Python's module globals become one `Settings` value that each game shares through an `Arc`:
  - the profile's counter thresholds (`CTHRESH`, `instant_extra`);
  - the AI mode;
  - the temperature scale (`--temp`);
  - the look-ahead's knobs (rollouts, top K, horizon, step caps).

  Two runs with different settings can share a process, and copying a game doesn't copy them.
- **Random numbers:** the streams are kept as Python has them, because their separation is what pairs A/B runs.
  Each seat shuffles and mulligans from its own stream (`lib:{seed}:{deck}`), and play decisions use another
  (`play:{seed}`), so changing one deck's list leaves every other seat's library and hand the same.
  - **The generator differs from Python's Mersenne Twister.** It's xoshiro256**, seeded from a hash of the stream's
    name: small to copy, fast, and the same on every machine.
  - That's possible because the port is checked statistically, not seed for seed (scope doc, section 7).
  - Rust games are deterministic among themselves: the same seed and answers replay the same game, which Undo and
    saved games need.

---

## 8. What M1 does not do yet

- **No rules:** casting, paying, combat and zone changes are M2 and M3. `Game::new_perm` makes a permanent, but
  entering the battlefield (which fires triggers) is the engine's `enter`, in M2.
- **Only some hook slots:** 19 of the 77 events (section 4).
- **Some state still to declare:** 54 game and 76 player attributes, many card-specific (section 10).
- **The ability language stays JSON** until M3.

---

## 9. Points for review

These are the choices that would be expensive to change later:

1. **Ids and arenas instead of references** (section 1). The alternative, shared references with interior
   mutability (`Rc<RefCell<…>>`), would let the code read more like the Python. But copying a game would mean
   rebuilding the whole object graph, which is the cost this port exists to remove.
2. **Declared fields for Python's on-the-fly attributes** (section 3, rule 1), rather than one generic map of
   `name -> value` per object. Declared fields are faster and typos fail to compile. The cost is about 130 more
   fields to declare as the port goes on, many used by a single card.
3. **Typed hook slots** (section 4), rather than one generic signature with the arguments packed in an enum.
   Typed slots catch mistakes at compile time. The cost is a wide `CardImpl` struct, about 77 slots in the end.
4. **`Res` everywhere** (section 5), rather than panics for `OutOfWork` and probes.
5. **A different random generator** (section 7), so Rust games can't be compared with Python's seed for seed.
   Section 7 of the scope doc already chose statistical agreement for that reason.

---

## 10. Appendix: Python's runtime attributes

Found by playing 700 games across every deck and tier and recording every attribute that appeared
(`python3 -m commander_sim.tools.state_inventory OUT.json`). "Declared" means `state.rs` has a field for it, or a
replacement noted in brackets.

**Game: 27 of 81 declared.** Declared: `players`, `rng`, `round`, `active`, `step`, `over`, `winner`, `wintype`,
`elim`, `monarch`, `hooks`, `stack`, `stack_pushes`, `trig_queue`, `resolving`, `work`, `work_cap`, `board_cap`, `log`
and others; `eot_pt` and `eot_kw` (on `Perm`); `alive_objs` and `hid_no` (the arenas); `hook_cache`, `static_idx` and
`coat_cache` (caches, not needed). To declare as their code is ported:

animated, attach_to, aura_put, auras, batch, bf_ver, blink_depth, blocking, bond_depth, bounced_spell, breach_dry,
cast_target, chatter_depth, coat, combo_decks, combo_spell, cur_cast, deaths_turn, destroyer, died_turn, dsl_depth,
dying, enter_no, entered, eot_returns, flutes, fog, garland, goldfish, in_combat, jar_due, last_cast_etb,
last_removed, last_x, lattice_lock, line_no_setup, lineage, marchesa_due, marchesa_on, ninja_atk, no_fang, noregen,
rem_src, resto_target, returns, returns_turn, selfpt, skip_etb, sphinx_depth, thief_chain, trig_current, trig_mode,
uro_escaping, zur_due

**Player: 45 of 121 declared.** Declared include the zones, life, commander damage and tax, the floating mana (one
`Floating` struct for `floatR`, `floatA`, `floatU`, `floatG`, `floatB`, `floatC`) and the per-turn counters.
To declare as their code is ported:

adv_done, agent_ids, arch_t, art_tok, attackers, attacking, borrowed, cast_names, chasm_age, clamp_n, clamp_t,
cmd_pending, combo_turn, conduit_lock, death_kind, delayed_draws, discarded_turn, drain_mana, draw_n, draw_st,
elspeth_emblem, emblems, energy, experience, extra_land_now, first_bomb, flag_turn, gained_turn, grudge, gy_start,
hit_turn, hope_lock, impulse, impulse_long, incubator, is_cast_n, jin_turn, last_kind, last_turn_end, left_turn,
locked_name, loop_turn, lost_names, lost_turn, loyalist_turn, marchesa_batch, milestone, milled_keys, miracle,
muld_used, najeela_boost, nissa_emblem, oath_return, ozolith_counters, pact_debts, pacts, ral_copy, rebound,
removed_bombs, ring_bearer, ring_level, scry_turn, seen_names, stolen, suspended, te_used, territory_type, to_top,
turn_casts, unbl_all, unearthed, urabrask, valakut_cards, yawg, yawg_gy, yawg_loop

The inventory covers the paths those 700 games reached; rare paths may add a few more as their code is ported.
