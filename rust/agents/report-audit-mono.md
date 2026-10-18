# Report: audit-mono (sheoldred, titania)

Branch audit-mono. Win rates (adaptive, seed 900000): sheoldred t3 32.0/32.1 -> 33.0/33.0 (10,000 games, conservative/loose);
titania t2 29.2 -> 28.9 (6,000 games, conservative; noise).

## Sheoldred
- 1 Flourish / Battle-Rage cast with no creature: NOT fixed here (core agent's brain.rs fallback).
- 2 Ad Nauseam: the deck taps out on its own turn, so the "end of the opposing turn" gate almost never had mana. For key
  sheoldred only, `plans::adnaus_cast_now` is also true in its own second main phase (room for 4 cards still required).
  Traces: "Sheoldred casts Ad Nauseam ... 6 cards, life now 24".
- 3 Dark Deal: own-hand limit loosened from 2 to 3 cards (other than Dark Deal).
- 4 Seizan: `seizan_prio` now uses `draw_toll > 0` (Sheoldred, Underworld Dreams or Fate Unraveler).
- 5 Necrosquito: `necro_prio` 55 base, +3 per fodder (token / noncreature artifact, max 3), +6 with Carnage Altar, Deadly
  Dispute or Blood Artist; cap 70.
- 6 Rooftop Assassin: its dsl removed (override) and a hook destroys the best opposing creature "dealt damage this turn".
  New `Game::dmg_marks` / `mark_damaged` / `was_damaged_now` (state.rs), filled in `t0_giada::damaged` (combat blocks and
  damage removal) and `bites::deal`; only Rooftop reads it. Priority 82 with such a target, else 46 (body).
- 7 Memory Jar: priority 82 when Sheoldred / Dreams / Unraveler is out (both sevens punish), else 38.
- 8 reactive cards: not bugs. 9 land count: deckbuilding.

## Titania
- 1 Blossoming Defense: NOT fixed here (core agent).
- 2 Fblthp: override cost {1}{G}{G} (X = 1: the deck has only Forest, X>=1 fetches exactly one). X=0 (a 1/2 body that finds
  nothing) is never cast; Domain power stays 1.
- 3 Silhana Starfletcher: `dork=G` (add_tags override).
- 4 Worldly Tutor: `titania::worldly_pick` via `tutor_pick_named`: mana creature at <= 3 mana, land-finder then Satyr
  Wayfinder (no fetch in gy) while Titania is away, Ghalta / Altisaur / Cactarantula once she is out or 5+ mana.
  Traces: Llanowar Elves on R2 (was Satyr Wayfinder), Ghalta late.
- 5 Arachnogenesis: fires at a fifth of life, or 3+ attackers, or 12 damage (was a third).
- 6 plan online / early crack: not a bug. In 60 traces only 8 games crack a fetch land before the first Titania, all
  at <= 4 lands for mana.
- 7 stuck-by-nature: not bugs. 8 End-Raze: Worldly no longer picks it (wish list excludes it).

Shared code touched: state.rs (field + two methods), t0_giada.rs `damaged`, bites.rs `deal` (record only, no behaviour
change), ai/plans.rs `adnaus_cast_now` (sheoldred gate), ai/decks.rs `tutor_pick_named` (titania arm).
