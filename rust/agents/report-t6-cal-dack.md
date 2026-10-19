# Report t6-cal-dack (branch t6-cal-dack, off t6-cedh)
Result (2,000 games, seed 500000): Dack 34.8/36.3% -> 26.2% (conservative; see /tmp/claude-1000/t6cal-dack/f-2000.txt). Target 18-32: met. One seed only (500000).
Seat shares 26.0/26.8/24.8/22.4 (seats 1 and 4 still outside, not Dack-specific). Kraum/Tymna rose to 34.3% (over 32; other tasks).

## What was wrong
1. Modeling (windows), t6_combos_t6.rs: the "first flip wins the turn (35 piles)" combo gave the table one trigger window
   (first flicker) plus one removal check; a real loop has Dack's second enters trigger, Preston's copy triggers,
   Radiant Solar's ventures and further flickers. Added steps S_FLIP_DACK2, S_FLIP_PRESTON, S_FLIP_SOLAR and two more
   pieces checks. Effect at 600 games: none measurable (Dack 32.7/35.0; stopped 10-15% vs 10-11%): opponents rarely
   hold an answer. Kept, because it is the correct number of windows.
2. Pile table: read, no change. record_first_flip only records piles of exactly three of the seven, and an unknown
   left-card counts as a loss, so removed/exiled pieces never inflate wins. The 68% applies only to piles the AI
   verified; tries are made only when a pile wins, so 88% of tries win; the real lever is how often Dack is cast.
3. Archenemy view: the line [QUIET, Dack] is public-ready when Dack is castable, so opponents see it. No change.
4. Tuning (t6_plan_dack.rs, dack_bar): risk bar by round 0.3/0.4/0.5/0.65/1.0 -> 0.2/0.3/0.4/0.5/0.8. Dack waits
   for a safer table (or protection) before casting; tries per game 0.32 -> 0.22. This is the change that moved it.
## Numbers
600: windows only 32.7/35.0; with bar 24.9/24.9. 2000: 26.2%, tries 291 (0.22/game), stopped 9%, 81% of wins by the
35-pile line, 19% damage, median win round 7.
## Not done
Second seed, t3 lock diff (changes touch only t6 files). Dack's win share still rests on the hand-traced pile table.
