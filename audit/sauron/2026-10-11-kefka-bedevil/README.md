# Sauron: Kefka, Court Mage → Bedevil (2026-10-11)

JD moved Kefka out of Sauron to lead his own deck; Bedevil took the slot. `run.sh` plays the list with Bedevil
against the same list with Kefka put back, paired (same seeds, seats and opponents), look-ahead AI, loose profile,
1,000 games per list per tier. Delta = Kefka minus Bedevil.

| Tier | Bedevil | Kefka | Delta | Noise |
|---|---|---|---|---|
| T1 | 24.1% | 26.9% | +2.8 | ±3.3 |
| T2 | 30.6% | 32.2% | +1.6 | ±3.5 |
| T3 | 23.1% | 23.4% | +0.3 | ±3.3 |
| T4 | 19.5% | 22.4% | +2.9 | ±3.1 |
| T5 | 20.4% | 21.9% | +1.5 | ±2.8 |

Average +1.8 points for Kefka. No single tier clears its noise, but all five point the same way, so the swap
costs Sauron about two points.

The first T4 run crashed with a stack overflow (seeds 500298 and 500841): Gamble's priority valued the hand,
Underworld Breach's priority valued the graveyard, a Gamble there asked for Gamble's priority again, and so on.
Fixed with a re-entry guard in `ai/decks.rs` `gamble_prio`; only games that hit the loop change (they crashed
before), so the T1–T3 and T5 runs, made before the fix, stand. T4 was rerun with the fix (about 10 minutes for
all five tiers on 22 threads).
