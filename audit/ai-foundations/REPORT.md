# AI foundations audit: report (2026-10-27)

JD's question: is the foundation of the simulation sound, given that even in a goldfish (three seats that never act)
the cEDH decks kill at a median own turn of 7-10?

Sources:
- `architecture-review.md`: code reading, file:line evidence.
- `loss-accounting.md`: goldfish counters, 2,000 games for each of the seven Tier 6 decks, plus 30 traced games.
- `audit/scion/FINDINGS.md`: the two bugs found on 2026-10-27.

The main session checked the review's key claims against the code: ritual priorities, `must_option`, `strength`, and
the cleanup discard.

## Answer

**The rules engine isn't implicated. The decision-making layer is not sound for combo decks.**

The engine, which resolves cards, mana, the stack and zones, wasn't audited here. Nothing in the traces points at it:
the turns are lost to choices, not to wrong rules.

The AI picks one play at a time from a hand-tuned cast-order priority. A shallow look-ahead then improves that one
pick, over playouts that end after the deck's next turn. Nothing anywhere measures what a cEDH pilot optimizes: how
many turns until the kill. Four layers each patch the one below:
1. the utilities;
2. the deck and card hooks;
3. the Tier 6 must-do rules;
4. the look-ahead and its bonuses.

The patches now collide. The 2026-10-27 sure-win bug was one of those collisions.

## What the measurements say

| | scion | Kinnan | Kraum/Tymna | Rograkh/Silas | Sisay |
| --- | --- | --- | --- | --- | --- |
| goldfish median kill (look-ahead) | 10 | 8 | 9 | 10 | 7 |
| crude best case on the same draws, tutors counted (median turn) | ~6 | | | | |
| AI's loss beyond the best case (median turns) | 3 | 1 | 2 | 2 | |
| turns passed holding a castable card | 21% | 24% | | 36% | |
| games losing a combo piece to the cleanup discard | 18% | 5% | 33% | 24% | 2.5% |

Blank cells weren't reported for that deck.

- **A ready line is taken.** After the fix, `ready_not_taken` is 0-0.7% of turns. The time goes in getting to ready.
- **About half the turns "progress" without moving toward the kill.** Tutors fetch non-pieces (Enlightened Tutor
  fetching Esper Sentinel, Finale fetching Consecrated Sphinx). Scion spends turns on Dragons and value creatures.
  Sisay fetches Mount Doom or Ragavan before its line's pieces, in 6 of the 10 traced games.
- **Passing costs some turns but doesn't set the pace.** The adaptive AI passes on only 3% of turns and is a turn
  slower than the look-ahead.
- **The goldfish is harsher than a real table in one way.** Rhystic Study, Mystic Remora and Esper Sentinel draw
  nothing, and Exotic Orchard makes no mana, against seats that never cast. Even perfect play on these draws puts
  scion near turn 6, so turn-3 medians (D1) were never reachable from this list with these draws. This bound is crude:
  it counts each tutor as one piece.

## The defects, ranked by turns likely lost

| # | defect | where | kind |
| --- | --- | --- | --- |
| A | No turn plan and no distance-to-win in any layer; tutors rank below rocks and Scion in the cast order | `ai/pool.rs`, `brain.rs`, `search.rs:226-240` | foundation |
| B | Tutors pick targets badly: not mana when a line lacks only mana, re-fetch a card already on top, Enlightened Tutor scored as finding Oracle | `combos.rs:720`, `tutors.rs:401`, `pool.rs:233` | bug |
| C | Rituals switched off (Cabal, Rite of Flame, Desperate: priority 0) or below "stop" (Dark Ritual 1.5); a ritual in hand never counts as mana | `t5.rs:978`, `t6_mana.rs:564,585`, `brain.rs:317` | bug |
| D | The Tier 6 must-do rules force a rock or dork before a ready line and can make it unaffordable (every deck but Sisay) | `t6_table.rs:261-307` | bug |
| E | Win now and win next turn both score 100; since the cap, the tie goes to the higher-utility non-winning play | `search.rs:209,385` | bug (follow-up to d10bbb05) |
| F | The evaluation scores a rock at 0 and a card in hand at 0.9, so mana development looks like a loss | `search.rs:237`, `values.rs` | foundation |
| G | The cleanup discard throws away the highest-CMC card and ignores `discard_keep`, which itself keeps too few pieces | `turn.rs:796` | bug |
| H | Hold value, reserve penalty and hold bonus stay active in a goldfish (measured small) | `brain.rs` | goldfish only |
| I | `determinize` shuffles away a card the deck put on top itself | `search.rs:101` | bug, small |

## Recommendation

**Step 1: fix the bugs B-E, G and I, one commit each.** Measure each in the goldfish (Tier 6 + scion) and in the
`--realism` scorecard. Most of the code is shared, so the lower tiers move too. These are real rules-of-thumb errors
that a correct AI wouldn't make, so they should land whether or not step 2 happens. Expected: 1-2 turns off the
goldfish median.

**Step 2: a goal-directed line planner for Tier 6 and combo decks** (the review's section 8).
- It searches this turn's mana, tutors and pieces for a sequence that makes a catalogued line ready and affordable,
  and plays that sequence.
- If there is none, it picks the play that maximizes the chance of a ready line next turn.
- The existing utility AI stays as the fallback, and for every non-combo deck.
- Score a win by when it happens.

This is the real fix for A and F: a new module of a few hundred lines, run on game clones. Gate it to Tier 6 and
combo-tagged decks so Tiers 0-5 keep their calibrated pace.

**Step 3: recalibrate Tier 6.** A faster combo AI shortens Tier 6 games. Today the Tier 6 table is too slow (median
round 9 against 7) but already too combo-heavy (76% against 47.8%). So the table's interaction (counters, stax,
removal) has to be refit at the same time, and every Tier 6 baseline shifts, including scion's.

**Not recommended:** more priority patches. That's how the stack got here.

## Decisions for JD

1. Go ahead with step 1, the bug fixes, now?
2. Build the planner (step 2), gated to Tier 6 and combo decks? It is the larger piece, and it reopens the Tier 6
   calibration.
3. Should goldfish mode model a table's "free" resources? Right now Rhystic Study, Mystic Remora and Esper Sentinel
   are blanks, and Exotic Orchard makes no mana. Options: keep it as the strict worst case, or add an option where the
   seats cast one cheap spell a turn.
