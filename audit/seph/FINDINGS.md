# Seph/Atraxa (key `seph`): findings

The deck's living list of strengths, weaknesses and what was tested. Opponents: its matched tier, t4 only
(`data/my_decks.toml`).

## Strengths
- Mana is steady: about 4.15 mana on turn 4 and 7.0 on turn 8, with 0.6 missed land drops in turns 1-5.
- The game plan (first 6+ power bomb on the battlefield) is online in about 70% of games, by turn 6 in 38%, and by
  turn 8 in 60%.
- It wins mostly by damage (about 70%) and by combo (about 30%).

## Weaknesses
- Win rate vs t4 is about 28-29%.
- What eliminates it: Heliod via combo (about 14-15%), Krenko via combat (about 11-12%), Yuriko via triggers (about
  11%). These are fast combo and go-wide decks, and the grind engines don't answer them.
- The plan is not online at all in about 30% of games.

## Tested
### 2026-10-18: Mind Stone => The Soul Stone (vs t4, 1,500 games per list per profile, look-ahead AI)
Result: conservative 28.1% -> 29.1% (+1.0, noise ±2.5), loose 28.9% -> 28.5% (-0.5, noise ±2.4). Averaged over
t4: +0.25 pts, no clear effect. Raw output: `2026-10-17-soul-stone/swap-mindstone-t4.txt`.

Why:
- As a rock it is identical to Mind Stone: mana on turns 4, 6 and 8 did not change. Its {B} is no better in this
  sim, because the mana base already covers black.
- The harness ability arrives late. The Stone is drawn in about 26% of games and cast around round 6.5, and the
  harness needs 7 more mana plus a creature to exile, so it only matters in long games. By then the plan is usually
  already online or the game is already lost to Heliod/Krenko/Yuriko, and no axis about speed moved.
- Small signs in the right direction, all within noise but the same in both profiles: bombs put on the battlefield
  +0.10 to +0.13 per game, damage dealt +0.9 to +1.7, removed bombs brought back +0.02 to +0.04. It is a grind
  engine for long games, not a speed card.
- Lost cost: Mind Stone's late "sacrifice: draw a card" (cards in hand on turn 8 changed by -0.13 and +0.03, noise).

Verdict: a fine but not needed swap. It only pays off if games go long, and losses vs t4 come from speed.

## 2026-10-18: band calibration (proposed swaps, NOT applied; see audit/bands/REPORT.md)
Band 2 (T3-T4). Proposed: Force of Will, Farewell, Teferi's Protection (to Y'shtola), Consecrated Sphinx (to Veyran), Worldly Tutor (to Titania) out; Hornet Queen, Harmonize, Tireless Tracker (from Titania), Austere Command, Arcane Denial (spares) in. 8 -> 3 Game Changers (Demonic Tutor, Gifts Ungiven, Aura Shards). Final: **35.1% vs T3, 28.3% vs T4** (avg 31.7%; current list 36.2 / 29.8 at 1,000). Why: the deck barely misses its five GCs: Hornet Queen is a strong reanimation target (72% win when cast at T3) and Harmonize/Tracker replace the Sphinx's cards. AI bug found: the sim never cast Worldly Tutor (fixed ebd8d6f).

## 2026-10-18: AI audit re-measure (branch ai-audit; 2,000 games per profile, look-ahead)
**36.3% vs T3, 27.6% vs T4** (before 35.1 / 28.3): within noise. Hornet Queen and Muldrotha now valued as bombs; Momentary Blink protection; Abolisher timing; Tracker before the land drop.
