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

The **worth now** (`worth(g, p, c)`, card-equivalents, about 0-10) reads the profile at the current state:

```
early    = turn <= 4: 1.0 | turn 5-6: 0.55 | later: 0.25             (mana matters while the deck develops)
reps     = clamp(12 - round, 3, 8) / 2.5                              (turns an engine will run, in card units)
mana     = 2.2 x mana x early x (1 + 0.5 x [no land in hand]) + 0.5 x burst
cards    = 1.0 x cards x hand_need + 1.0 x cards_rep x reps            hand_need = 1.3 with <= 2 cards in hand, else 1
removal  = single: 1.5 + 0.35 x best opposing target's pval (0 targets: 1.0); sweep: 1.0 + 0.25 x (their board - 1.2 x mine)
damage   = 0.45 x damage x opponents + 0.45 x damage_rep x opponents x reps
board    = 0.5 x power + 0.2 x toughness + 0.4 x evasion + 0.3 x haste + 0.25 x other keywords + 0.6 x tokens
           + 0.6 x tokens_rep x reps + 0.5 x team x (my creatures + 1)
engine   = 0.6 x engine + 1.5 x lock
role     = combo_imp (9: completes a catalogued combo, 7: one short) x 0.8; a plan line's missing piece: +2
fit      = -0.4 x the mana it is short of next turn (cost against lands + rocks + next land drop), -1.5 when a colour
           of its cost has no source
worth    = mana + cards + removal + damage + board + engine + role + fit + 0.4 x tutor + 0.3 x protect
```

The **derived cast priority** (what replaces the table in `generic_prio`, same 0-90 scale so the consumers are
unchanged): `prio = clamp(30 + 6 x worth_cast, 1, 90)`, where `worth_cast` is `worth` without the held parts (a
counter, a removal or wipe spell, a protection spell keep 0: the response logic casts them, as before). The
state-dependent card rules that are code, not table rows, stay in front of it: key cards, plan rules, card code
`prio`, the commander rule, combo pieces (`50 + 4 x combo_imp`), Chrome Mox / Mox Diamond / Grim Monolith's
preconditions, the tutors' own value (`tutor_prio`, now fed by derived values), Smothering Tithe, Necropotence,
Rhystic Study, the Ad Nauseam / Citadel / Jeska / Braids / Agent timing functions, X spells, an Aura without a host.

**Derived bomb** (`threat(g, c)`, 0-10, static per card: how much the card does on the board if left alone, the scale
the old `bomb=` tags use): for a permanent, `0.9 x power + 0.8 x evasion + 0.5 x haste/lifelink/deathtouch +
0.5 x toughness over power + 1.6 x engine + 2.5 x lock + 1.2 x cards_rep + 1.0 x tokens_rep + 0.8 x team +
1.0 x mana (a rock or dork is a small threat)`, a commander +1; capped at 10. It replaces `bomb` at reads 3-9 for a
deck on the derived arm (the reader decides: a sacrifice by its controller, a counter or removal target by the one
choosing it).

`card_worth` on the derived arm: `10 x worth` (held interaction included: a counter is worth `1.5 +
0.4 x opponents' best spell ...`, i.e. its removal/counter component), so discard, piles and late tutors read the
same number. `tutor_value` drops the `10 x bomb` floor for `10 x worth + 4 x threat`. `bottom_pick` and the generic
`discard_worst` drop the card with the lowest worth (lands by the old land rule).

## Win-line role

`cardcode::combo_imp` (the catalogue and Spellbook combos the deck plays, at the current board: 9 completes, 7 one
short) is the role term; the plan file's derived wish list (`plan_data::derived_wish`, PR 1) adds the missing-piece
term for decks with plan lines. Tutors still take the wish list first, the derived value only orders what is left.

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

(filled in from the diagnostic)
