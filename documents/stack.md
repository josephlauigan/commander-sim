# The stack: design

Status: phase 1 built 2026-10-02 (spells on the stack, responses of any depth); phases 2-6 to come. Replaces the engine's fixed response windows with Magic's stack and priority,
for simulations and practice mode alike (one engine).

## Goal

Play like the real game: every spell, activated ability (except mana abilities) and triggered ability goes on the
stack; players get priority in turn order, starting with the active player, and may respond with anything they can
legally do at instant speed; responses can be answered in turn, to any depth; the top item resolves only when every
player passes in succession, and the active player gets priority again after each resolution. Two or three players
can interact over the same stack (one counters, another counters the counter, a third responds with removal).

## What the engine does today

Interaction is a set of hand-built windows, each for one situation:

| Window | Where | What it allows |
|---|---|---|
| `counter_window` | `engine.cast_card` | each opponent once, in turn order, may counter the spell; the caster may counter back |
| `protect_response` | `engine.apply_removal` | the owner of a targeted permanent may protect it |
| `wipe_response` | `engine.apply_wipe` | each player may save their board |
| `combo_interrupted` | combos | opponents may remove a combo piece |
| `tide_response` | 7 places | Tishana's Tidebinder counters an ability or trigger |
| the end-of-turn window | before each turn | instant-speed plays (flash, draw, removal) |
| `play.human.respond` | practice mode | the person gets priority on each opponent's spell, when attacked, at end of turn |

Triggered abilities resolve the moment they trigger, in code order. Activated abilities are single functions that pay
their cost and apply their effect together. Nothing waits on a stack, so nothing can respond to an ability or a
trigger, and a counter war is one counterspell and one answer.

## The model

### Stack items (data, not closures)

```
StackItem: kind ('spell' | 'ability' | 'trigger'), controller, source (card or permanent), targets, ctx,
           effect: a key into a registry + its arguments (not a Python closure), copy (bool), id
g.stack: [StackItem]         top = last
g.pending_triggers: [StackItem]   triggered since the last time a player would get priority
```

Effects are registry keys with plain-data arguments so the look-ahead's `clone(g)` copies a game mid-stack safely (a
closure would keep pointing at the original game's objects).

### The priority loop

```
def priority(g):
    while True:
        put pending triggers on the stack: APNAP order; each player orders their own (the person chooses, the AI by value)
        passes = 0; p = g.active
        while passes < number of players alive:
            act = decide(g, p)                # cast / activate / pass
            if act is pass: passes += 1
            else: act puts an item on the stack (costs paid now); passes = 0; pending triggers go on top
            p = next player in turn order
        if not g.stack: return               # the step can end
        resolve(g.stack.pop()); check state-based actions
```

`cast_card` becomes "pay, put on the stack, run the priority loop" (the spell's controller gets priority first), and
resolution runs the spell's effect as today (`engine.resolve`). Counterspells, protection, removal in response, flash
creatures and abilities are all just actions taken with priority; the fixed windows above go away.

### Priority in every step

Untap and cleanup give no priority; every other step does (upkeep, draw, each main phase, beginning of combat, declare
attackers, declare blockers, combat damage, end of combat, end step). The end-of-turn window becomes the end step's
priority. Most passes are free: a player with no instant-speed action they can afford passes at once.

### Activated abilities

Each ability is split into **cost** (paid when activated: tap, mana, sacrifice, loyalty) and **effect** (on
resolution, rechecking its targets). The ~87 hand-written abilities in `cards/impl/`, the ability language's
`activated` and `loyalty` abilities, and your decks' practice-mode abilities (`play/cards.py ABILITIES`) are converted
to a small helper: `ability(cost=..., effect=..., targets=...)`. Mana abilities stay immediate (they don't use the
stack). While converting, any ability not yet split resolves at once, as today.

### Triggered abilities

Card-code hooks that are triggers (`etb`, `dies`, `self_dies`, `attack`, `blocks`, `combat_damage`, `upkeep`,
`end_step`, `landfall`, `cast`, `draw`, `discard`, `sacrifice`, `leaves`, `token_created`, `gain_life`,
`lose_life` ...) and the ability language's `triggered` abilities are queued as stack items instead of running at
once; hooks that are static or replacement effects (`cost`, `grant_kw`, `prevent_damage`, `uncounterable`, mana
hooks ...) stay immediate. Tag-driven triggers inside the engine (magecraft, Rhystic Study, ETB tags) are converted
one by one. A trigger records what it needs at the moment it triggered (the dying creature's power, the cast spell)
so it resolves correctly later.

### The AI's decisions with priority

A cheap first check keeps simulations fast: no affordable instant, flash card or ability that matters → pass at
once. Otherwise the AI weighs, against the item on top of the stack:

- **counter it** (today's thresholds and look-ahead `choose_counter`, now for abilities and triggers too, and at any
  depth);
- **protect** the permanent it targets (today's protection logic, moved here);
- **respond with removal or a trick** (kill the creature before its ability resolves, save one from a wipe);
- **use instant-speed value** (flash creatures, draw spells, blinks) in the end step before its own turn, as today.

### Practice mode

- The table shows the whole stack (top first), with who controls each item and what it targets.
- You get priority whenever the rules give it to you, with an **auto-pass** setting: *stop only when I can respond*
  (the default: you're asked only when you hold an instant, flash card or ability you can afford), *stop on every
  stack item*, or *stop on every step* (full control).
- Your simultaneous triggers: you choose their order (with a "same order as last time" shortcut).
- Responding to abilities and triggers works like responding to spells (Azorius Guildmage's counter, Stifle-type
  effects, removal before a trigger resolves).

## Phases

| Phase | What | Estimate |
|---|---|---|
| 1 | The stack and the priority loop for spells: responses of any depth from every player; counter wars; protection, removal and flash in response; fixed windows retired; AI response policy | 2-3 days |
| 2 | Activated abilities on the stack (cost/effect split for all hand-written and ability-language abilities, loyalty) | 2-3 days |
| 3 | Triggered abilities on the stack (queued hooks and ability-language triggers, APNAP order, your ordering choice, tag triggers converted) | 3-5 days |
| 4 | Priority in every step, combat steps included | 1-2 days |
| 5 | Look-ahead safety (cloning mid-stack) and speed; AI tuning; re-measure the results tables | 2 days |
| 6 | Practice mode: the stack panel, auto-pass settings, trigger ordering, responding to anything | 1-2 days |

Roughly two to three weeks. Each phase ends with the full test suite passing, the sim guard re-recorded, and practice
mode playable.

## Risks

- **Speed.** Priority for four players at every item could slow simulations several times over. The cheap "can I
  respond at all" check is the answer; measure after phase 1 and phase 3.
- **Results shift.** Every recorded result moves (expected and intended). The pool's tier ordering was tuned on the
  old model and may need retuning.
- **Conversion scale.** Hundreds of hooks and tag effects. Until converted, an item resolves immediately as today, so
  the engine works at every step of the conversion.
- **Card behaviour that relied on immediacy** (an attack trigger returning tokens that attack, a dies trigger that
  reads the creature's last power) needs those values captured at trigger time.
