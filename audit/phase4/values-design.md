# Phase 4, PR 3: card values from abilities (design)

`documents/realism.md`, Phase 4 ("Generic AI"); JD's plan, AI point 2: a card's worth is computed from what it does
on the board (mana, cards, removal, damage, board presence) at the current state, plus its role in a win line. It
replaces `bomb=` tags and the 0-90 priority tables, and a new card gets a sensible value the day it is added.

## What feeds on the old values today (generic policy: decks without their own hand code)

Fixed priority tables and `bomb` reads that decide something for a pool deck:

| # | where | what it decides | old input |
|---|---|---|---|
| 1 | `ai/pool.rs::generic_prio` | cast order (via `decks::deck_prio` -> `brain::card_utility`) | a 0-90 table by tag: Sol Ring 88, rocks/dorks 85 (turn <= 5) else 38, boots 45/25, ETB removal 45+6 x target, Tokx 50, Eng 64, Birgi 45, Rabble 55, Stampede 55/0, Drawcre 55/20, Clamp 55, anthems 60, board makers 62, draw 46/52, Treasure/token/ETB drain 50, Aura 55, Threedreams 60, Explore 50/30, Loam 45/0, Rishkar 55/0, Fable 58, creatures 42 + 2 x power (cap 16) + 4 flying, pillowfort 45 + opp power, stax 68/52, hooked 55, cost statics 68/52, sac outlets 50, combo piece 48, Sac/boots 40 |
| 2 | `engine/tutors.rs::card_worth` | discard to hand size (`ai::cleanup_discard`), Gifts / Intuition piles, graveyard worth | `deck_prio` (1), then held interaction 60 (rift) / 55 ctr / 50 mastery / 45 rem, wipe / 40 prot, X |
| 3 | `ai/decks.rs::tutor_pick` (fallback after the wish list) | which card a tutor finds | early: (`deck_prio`, `bomb`, mv); late: `tutor_value` |
| 4 | `ai/decks.rs::tutor_value` | tutor targets late, a tutor's own priority (`pool::tutor_prio`), pile tutors | `card_worth`, mana x 0.3 after turn 4, max(.., 10 x `bomb`) |
| 5 | `ai/decks.rs::gy_worth` | Farewell's graveyard mode, pile tutors | `bomb` or 1 + 0.3 x power |
| 6 | `ai/brain.rs::card_utility` | counter-risk discount on the cast | `bomb >= 5` |
| 7 | `ai/pool.rs::spell_importance` | how much an opponent wants to counter the spell | max(`bomb`, threat value + 1), 0.9 x power + flying |
| 8 | `engine/values.rs::pval` | removal and wipe targets, sacrifice and edict fodder, protection, threat | `bomb`, else 1 + 0.5 x power, then per-tag constants |
| 9 | `ai/decks.rs::rean_options`, `brain::rem_fodder` | sacrifice fodder for Dread Return and removal costs | `bomb == 0` |
| 10 | `engine/turn.rs::bottom_pick`, `engine/cast.rs::discard_worst` | mulligan bottom, effect discards | highest mana value (no value read) |

Not in scope (stay): JD's decks' priorities (`ai/decks.rs::deck_prio`'s named arms, `seph`, `veyran`,
`impls/<jd deck>.rs`), plan-file `[[prio]]` rules and `key_cards` (they still win), the Tier 6 plan slots, card code's
own `prio` hooks (a card's own timing, written with its card code), and the `bomb` reads in Sephiroth's code and the
reports (`note_bomb`, `removed_bombs`, `first_bomb`: statistics only).

## The formula (`ai/values.rs`)

Everything is read from what the card does: its compiled abilities (`dsl`) and the role tags that cardgen and the
hand tags compile from Oracle text (`rock=1:C` = a tap for one mana, `draw=2`, `rem=destroy`, `tok=2 tokp=1` ...).
Each card gets a **profile** of what it does, once per card (no state):

- `mana`: mana a turn from a permanent (`rock=N`, `dork`, `lr` land ramp, `fastmana`, a compiled `add_mana` with a
  tap cost or a `mana` static); `burst`: one-shot mana (a ritual, Treasure, `fastmana` spells).
- `cards`: cards drawn once (`draw=N`, compiled `draw`); `cards_rep`: cards a turn from an engine (`eng`, `rhystic`,
  `spelldraw`, `drawcre`, a triggered or activated `draw`).
- `removal`: single-target reach (`rem` with its target kind, compiled `destroy` / `exile` / `bounce` / `tuck` on a
  target); `sweep`: mass removal (`wipe`, compiled `... all`); `counter` (`ctr`).
- `damage`: life lost by each opponent once (`drain`, `ping`, `lose_life`, `damage` to players); `damage_rep`
  repeatable (a trigger or activation: `bartist`, `ping`, `kiln`).
- `body`: the card's own board presence: power, toughness, evasion (flying, trample, menace, unblockable, shadow),
  haste, deathtouch, lifelink, vigilance, protection keywords; `tokens`: power made in tokens (`tok` x `tokp`,
  `mktok`, `treas`, compiled `token`), `tokens_rep` repeatable (`tokup`, `tokatk`, `spelltok`, a triggered token);
  `team`: power added to each creature (`anth`, `crusade`, `warleader`, compiled `anthem` static).
- `engine`: the number of repeatable abilities (triggered, static, activated, card code that handles engine events),
  `lock`: card code or statics that tax or lock (cost, can't cast, attack tax).
- `tutor`, `protect`: a search; protection (`prot`, `boots`, `cloak`, hexproof / indestructible grants).

The **worth now** (`worth(g, p, c)`, card-equivalents, clamped to 0-10) reads the profile at the current state (as
implemented after the diagnostic's fixes, below):

```
early    = turn <= 4: 1.0 | turn 5-6: 0.55 | later: 0.25             (mana matters while the deck develops)
reps     = clamp(12 - round, 3, 8) / 2.5                              (turns an engine will run, in card units)
mana     = 5.5 x mana x early x (1.25 with no land in hand, else 1) + 0.5 x burst
cards    = cards x hand_need + cards_rep x reps                        hand_need = 1.3 with <= 2 cards in hand, else 1
removal  = single (a permanent's, or any when held): 2.5 + 0.35 x best opposing target's pval by p (none: 1.5 held, 0 to cast)
           sweep: max(0.5, 1.5 + 0.25 x (their board / opponents - 1.2 x mine)); a counter (held): 4.0
damage   = 0.45 x opponents x (damage + damage_rep x reps)
board    = 0.5 x power + 0.2 x toughness + 0.4 x evasion + 0.3 x haste + 0.25 x other keywords + 0.6 x tokens
           + 0.35 x tokens_rep x reps + 0.5 x team x (my creatures + 1)
engine   = an engine the profile reads (cards, damage or tokens a turn): 0.3 x engine; else 0.9 x reps x min(engine, 2)
           + 1.5 x lock
role     = 0.8 x combo_imp (9: completes a catalogued combo, 7: one short); at least 1 for any catalogued piece
misc     = 3.5 for a tutor, 2.5 for protection (a permanent's only with a creature to protect)
fit      = -0.4 x the mana it is short of next turn (its cost now against lands + rocks + a land in hand), -1.5 when
           a colour of its cost has no land source (with two lands or more)
worth    = mana + cards + removal + damage + board + engine + role + misc + fit
```

The **derived cast priority** (what replaces the table in `generic_prio`, same 0-90 scale so the consumers are
unchanged): `prio = clamp(20 + 8 x worth, 1, 90)` with the held parts left out (a counter, a removal or wipe spell, a
protection spell keep 0: the response logic casts them, as before), 0 when the worth is 0 (the interpreter's value
then decides, as before). The state-dependent card rules that are code, not table rows, stay in front of it: key
cards, plan rules, card code `prio`, the commander rule, combo pieces (`50 + 4 x combo_imp`, and 48 for a catalogued
piece, as floors), Chrome Mox / Mox Diamond / Grim Monolith's preconditions, the tutors' own value (`tutor_prio`,
fed by derived values), Smothering Tithe, Necropotence, Rhystic Study, the Ad Nauseam / Citadel / Jeska / Braids /
Agent timing functions, X spells, the gates that return 0 (Stampede under six creatures, Loam, Rishkar, a Tokx spell
under five mana, an Aura without a host), and the plan knobs `aura_prio` and `hooked_prio`.

**Derived bomb** (`threat(g, c)`, 0-10, static per card: how much the card does on the board if left alone, the scale
the old `bomb=` tags use): for a permanent, `0.9 x power + 0.8 x evasion + 0.5 x (haste + min(other keywords, 2)) +
0.5 x min(toughness over power, 3) + 1.6 x min(engine, 2) + 2.5 x lock + 1.2 x cards_rep + 0.6 x min(tokens_rep +
damage_rep, 4) + 0.8 x team + min(mana, 3)`, capped at 10; 0 for a spell; the override table wins. It replaces
`bomb` at reads 3-9 for a deck on the derived arm, read as "a bomb" from 5 up (cardgen's old line: power 5); the
reader decides (a sacrifice by its controller, a removal target by the caster).

`card_worth` on the derived arm: `10 x worth` with the held parts, where the deck's own priority is 0, so discard,
piles and late tutors read the same number. `tutor_value` keeps its floor, read as `10 x threat` for a derived bomb.
`bottom_pick` and the generic `discard_worst` drop the spell with the lowest worth (lands by the old land rule).

## Win-line role

`cardcode::combo_imp` (the catalogue and Spellbook combos the deck plays, at the current board: 9 completes, 7 one
short) is the role term in worth. A plan's lines act through its derived wish list (`plan_data::derived_wish`, PR 1),
which tutors take first and the cleanup discard keeps, ahead of any value: the derived value only orders what is
left, and plan `[[prio]]` rules and key cards still win the cast order.

## Override table

`data/values_overrides.toml`, embedded at build time; rows of

```toml
[[card]]
name = "Sol Ring"
worth = 9.0          # optional: the worth now (replaces the formula's)
threat = 3.0         # optional: the derived bomb
reason = "net +1 mana on turn 1 compounds; the formula's mana term caps at the rest of the rocks"
```

A row without a reason, or naming a card not in `data/cards.json`, refuses to load (a test). Overrides are for real
one-offs only; a wrong value for a general reason is a formula fix.

## Switch

Env `AI_VALUES`: unset or `0` the old tables; `1` every generic-policy deck on derived values; a comma-separated list
of deck keys puts only those on it (the paired test of half a tier). JD's decks and Tier 6 never read it.

## Stages

- A: derived values computed beside the old ones; unset = byte-identical on t1, t2, t4, t5 (300 games). Diagnostic
  (`examples/values_diag.rs`): old and derived values side by side for every pool card, Spearman rank correlation per
  tier; worst disagreements below with their cause.
- B: per tier t1-t5, halves A and B of the tier's decks (handoff), `AI_VALUES=0` against `AI_VALUES=<half>`, 2000
  games per arm, seeds 500000-501999; the half's summed share and its CI.
- C: default derived where level or better; delete the replaced tables there.

## Diagnostic findings (Stage A)

`examples/values_diag.rs` (state: round and turn 3, three of each deck's lands out, empty hand; cards that key cards,
plan rules or card code decide are left out). Spearman rank correlation of old and derived values, per tier:

| tier | cards | cast priority | card worth | bomb (permanents) | old bombs (>= 5) | derived (>= 5) | both |
|---|---|---|---|---|---|---|---|
| t1 | 299 | 0.74 | 0.61 | 0.33 | 12 | 45 | 10 |
| t2 | 273 | 0.73 | 0.61 | 0.37 | 13 | 28 | 12 |
| t3 | 287 | 0.75 | 0.63 | 0.39 | 16 | 43 | 15 |
| t4 | 289 | 0.78 | 0.54 | 0.32 | 8 | 30 | 8 |
| t5 | 299 | 0.85 | 0.65 | 0.52 | 24 | 37 | 23 |

(First pass: cast priority 0.60-0.79; the general fixes below raised every tier.) The bomb correlation is low because the
old tag is 0 for most permanents (ties): the derived threat calls more creatures bombs (Hero of Bladehold 7.8,
Aurelia 10, Pearl-Ear 7.8, Hellrider 6.8, Reflector Mage 6.4: repeatable token, damage or removal triggers the tags
never rated); it misses four old bombs, two of them in the override table (below).

Worst disagreements and their cause (fixed for a general reason unless said):

- **Card code with no tags scored 0** (Exploration, Coat of Arms, Impact Tremors, Birthing Pod, planeswalkers,
  old 55 "hooked"): the engine-event list was narrower than `card_threat_value`'s; now the same 17 events, any live
  card code on a permanent, and a planeswalker's loyalty count as an engine; an engine the profile can't read is
  worth `ENGINE_W` (0.9) a turn while it runs.
- **`equip_cost`'s mana read as mana made** (Swords at 12, Reaver Cleaver at 19): Equipment statics now read as
  power on the equipped creature, evasion and protection; `equip_cost` is nothing.
- **Mana from a trigger or a sacrifice read as a tap** (Reaver Cleaver's Treasure, Skirk Prospector): only a tap
  ability with no other cost makes `mana`; the rest is one-shot `burst`. Attack and combat-damage triggers count
  half (a creature must attack or connect).
- **Cost statics read as engines** (the Medallions, old 68/52): a cost reduction for you is mana (0.8 a pip), a tax
  on the table is a lock.
- **Land-search ramp** (Harrow, Kodama's Reach, Rampant Growth; old 0, the interpreter's 30): a search for a land
  to the battlefield is mana (one a land; to hand half), a land sacrificed by the spell or its additional cost takes
  one back; worth is capped at 10.
- **Found in the first Stage B run (t1, t2), fixed, derived arms rerun**: a removal permanent with no target scored
  1.5 and was cast at nothing (Seal of Cleansing, Song of the Dryads at 32-42 with no target); now 0 to cast (1.5 held).
  Mana sources sat below development creatures (a one-mana rock 60 against the old 85; Sythis, a ramp-first deck,
  -6.0 [-9.4, -2.6] on t2): `MANA_W` 4.0 -> 5.5 (a one-mana source on turn 3: 75, Sol Ring 90, turn 5+: 44 and less).
- **Replacement doublers** (Doubling Season at 9.5): half an engine (worth what the deck feeds it).
- **Repeatable tokens** weighted down (Sigil of the Empty Throne at 8.6 -> 0.35 a power a turn).
- Left as derived (the derivation is the better reading): Grave Pact and Dictate of Erebos (66 vs 44: an edict engine
  is removal a turn), Vito and Fiend Artisan (combo role), Lightning Greaves (old 25 without creatures: derived
  protection only counts with a creature out), the colour-fit penalty (Mavren Fein, Charismatic Conqueror at 24 vs 62
  in the diagnostic's state with a colour missing), Shadow of Mortality (its life discount isn't in `cost_of`: it
  reads as a 15-drop until the life total drops).
- Overrides (real one-offs, `data/values_overrides.toml`): Archon of Cruelty (threat 9) and Sheoldred, the
  Apocalypse (threat 7): their triggers are engine code under their own tags (`archon`, `sheoA`), which no profile
  field can read. Iroas (old 7, derived 2.9) is not overridden: a God without devotion is no creature.
- `spell_importance` (how much the table wants to counter a spell) stays on the tag: it is computed once per spell on
  the stack and read by every seat, so no reader is known there; a derived reading would change how the other decks
  treat the half's spells, not the half's own decisions.
