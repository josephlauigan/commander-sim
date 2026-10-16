# Alela, Artful Provocateur — findings (living file)

What the simulator has taught us about Avery's Alela deck: strengths, weaknesses, and what was tried and why it did or
didn't work. Updated after every run. Newest evidence first; numbers are look-ahead, loose profile, vs Alela's matched
tier (T1) unless noted.

## Current state (2026-10-15)
List: Avery's physical deck as of 10/15 (22 out, 19 in vs the 10/14 list; 34 lands). Every card is modeled.
Win rate: **T1 9.8%** (4,000 games, ±0.5; `audit/alela/2026-10-15-newlist/analyze-t1.txt`). The previous list scored
12.0% on 10/14 (`audit/pool/2026-10-14-full/alela-t1.txt`), but on older engine code (since then: the land-mana fixes,
double strike dealing two damage steps, Calix/aura changes), so the −2.2 mixes the list change with code changes. A
paired old-vs-new run on today's code would separate them; not run yet.

## Strengths
- **The expensive answers and finishers win when cast:** Meteor Golem (26.6% win rate when cast), Daydream (25.2%),
  Cabal Patriarch (24.8%), Serra the Benevolent (21.6%), Notion Thief, Cavalier of Dawn and Venser (all 20–22%).
  Correlation, not causation: these get cast in games that are going well.
- **Wins come from the air:** 97% of damage is combat, 89% of eliminations by combat.

## Weaknesses (by evidence)
1. **Alela comes down late, and less often.** She resolves in 79.7% of games (old list 85.3%), by turn 5 in 45% (old
   53%). The first Faerie arrives by turn 6 in only 24% of games, and at all in 55.8% (old 60.3%). The new list has
   two fewer lands (34, was 36) and lost two early mana creatures (Initiates of the Ebon Hand, Page, Loose Leaf);
   only Sol Ring, Mind Stone, Dark Ritual and Burnished Hart remain.
2. **It dies to creature decks:** eliminated in 87.8% of games (old 85.4%), mostly Isshin combat (16.5%), Light-Paws
   commander damage (16.1%, up from 13.8%: the double strike engine fix likely matters here), Tatyova and Lathril.
3. **Alela is the removal magnet:** removed 0.39 times a game.
4. **Weak cards in the new list:** Cabal Patriarch is stuck in hand in 64.5% of the games it is drawn ({3}{B}{B}{B}
   with 8 Swamps); Tragic Poet, Burnished Hart, Dark Ritual and Ethersworn Canonist have the lowest win rates when cast
   (12.5–13.8%). Clever Concealment and Muddle the Mixture are almost never cast (reactive only).
5. Cards the old list had that did well when cast and are gone: Ajani Fells the Godsire (23.4%), Rhystic Study
   (21.7%), and the Tymna draw engine.

## What was tried, and why it did or didn't work
| Change | Result (T1) | Why |
|---|---|---|
| 10/15 list synced to Avery's physical deck (Skullclamp, Serra, Swords, Path, Fabricate, Chimes, Wall of Junk ... for Tymna, Rhystic Study, Opposition, Tetsuko, Ajani ...; 36 → 34 lands) | 12.0 → **9.8** (unpaired, across code changes) | Alela resolves later and less often (fewer lands and mana creatures); see weakness 1. Not a paired test. |
| 10/14 leftovers swap (Tymna, Exotic Orchard, Shattered Sanctum) | +1.2 (paired, 500 games) | Within noise. |
