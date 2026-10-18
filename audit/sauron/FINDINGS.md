# Sauron, the Dark Lord — findings (living file)

What the simulator has taught us about the Sauron Grixis amass deck: strengths, weaknesses, and what was tried and why
it did or didn't work. Updated after every run. Newest evidence first. Matched tiers: t3–t5 (`data/my_decks.toml`).
Older tests: the dated folders here.

## Current state (2026-10-17, night)
Look-ahead, loose, current list (baselines of the Urza run, with the hold-mana search credit): **T3 28.2%, T4 31.6%,
T5 26.4% (28.7% over t3–t5).**

## Weaknesses (by evidence)
- **Combo tables at T4–T5:** the killers there are Heliod (combo 16% at T4), Kinnan (20% at T5), the Urza deck (16%)
  and Zur. At T3 it's creature decks in combat (Aurelia 17%, Marwyn 13%, Tergrid 12%).
- **Lowest win rate when cast** (adaptive card report, `audit/sauron/2026-10-17-adnaus/K0-analyze-adaptive.txt`):
  Disdainful Stroke, Arcane Denial, Spell Pierce (t3, t4), Inexorable Tide, Relic of Legends.

## What was tried, and why it did or didn't work
### Urza, Lord High Artificer (2026-10-17 night; branch `adnaus`; Full, from the pool's Urza deck)
| Swap (Urza in for) | Result over t3–t5 | Why |
|---|---|---|
| Spell Pierce / Disdainful Stroke (adaptive) | -0.3 / -0.3 | No effect. |
| Spell Pierce (look-ahead, before the hold credit) | -1.1 (t3 -0.6, t4 -0.3, t5 -2.5) | Noise. |
| **Spell Pierce (look-ahead, with the hold credit, 1,000 paired per tier)** | **+0.5 (t3 +2.1, t4 0.0, t5 -0.6; ±3.3–3.7)** | Noise. Urza cast in 65% of the games he's drawn (52% at T5), wins 36–55% when cast. T3 eliminations -2.6 (an extra body and the Construct against creature decks); at T4–T5 losing a cheap counter to combo decks costs a little (T5 damage -2.0). Mana on turn 8 unchanged: Sauron's artifacts are mostly rocks that already make mana, so Urza's {U} ability adds almost nothing. |
**Reading it:** a fair swap, not an upgrade. Urza wants cheap non-mana artifacts and artifact tokens; Sauron has few.

## 2026-10-18: band calibration (proposed swaps, NOT applied; see audit/bands/REPORT.md)
Band 2 (T3-T4). Proposed: +Force of Will (spare) for Slaughter Pact; 3 Game Changers. Final: **30.2% vs T3, 31.6% vs T4** (avg 30.9%; current 28.9 / 31.6 at 1,000). Why: a free counter protects Sauron and stops T3-T4 combo turns; Slaughter Pact had the lowest win rate when cast.

## 2026-10-18: AI audit phase 2 (branch audit-grixis; adaptive AI vs t3, 600 games x 2 profiles, before -> after)
Win rate 26.7% -> 27.0% (noise). Aggravated Assault is now cast in 17.5-18.5% of games at a 43% win rate (it was the tutors'
default target and rarely cast with a Sword that is no longer in the list); Army power at T6/T8/T10 mean 1.6/2.3/2.4 -> 1.7/2.7/2.2.
Why the Army is mostly absent (traced 37 games): it is killed, not unmade. About 30 Army deaths in 37 games: opponents' wipes
(Toxic Deluge, Deed, austere, dmg13) and spot removal; plus Sauron's own Nibelheim Aflame (the sweep source is the biggest creature,
so a smaller Army died). Fixed the self-sweep (the Army counts 3x in wipe_eval unless The Ozolith is out) and the tutors now fetch The
Ozolith first once an Army is growing. Weakness that remains: one big Army against a removal-heavy table; the Ozolith is the answer.
Other fixes: tutors by board state (Sol Ring when short, Ozolith, Archivist with Bowmasters, Cloak/Passage for a huge Army, then
Assault, closers, Not of This World, Barad-dur when flooded); Force of Will pitches draw spells before engine creatures; Sauron holds
mana for counters like the outside decks; Cost of Brilliance gives an opponent cards while Bowmasters is out; Archivist wheel also
for Kaervek, or Sauron with a small hand.

## 2026-10-18: AI audit re-measure (branch ai-audit; 2,000 games per profile, look-ahead)
**30.4% vs T3, 32.0% vs T4** (before 30.2 / 31.6): no change. Board-state tutors; its own sweepers spare the Army; Force of Will pitches draw spells first; mana held for counters.
