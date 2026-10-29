# AI foundations: decisions made without JD

JD (2026-10-27): "Do all three sequentially" (step 1 bug fixes, step 2 line planner, step 3 Tier 6 recalibration).

1. **Goldfish seats stay fully inert** (JD's open question 3 got no answer). It is the strict worst case, and it
   keeps the 2026-10-27 baselines comparable. A "seats cast one cheap spell a turn" option can come later.
2. **The Treasure / floating-mana memory fix (d6e8b034, 2afe37db from phase2-doubles) is cherry-picked onto
   ai-foundations** so the t5 realism scorecard can run. The same commits sit on phase2-doubles; they will merge clean.
3. **Phase 2 PR 2 stays paused** while the three steps run: the machine runs one sim at a time.

## JD's answers on the eval fix (2026-10-29)

- **The eval is the look-ahead for every tier (eval-fix.md, decision 1a).** Tiers 0-5's 9 lost targets
  (22 → 13 of 46) are not retuned now: "We can wait recalibrate until the very end". One recalibration of Tiers 0-5
  comes after the rest of the AI-foundations work, so later look-ahead changes don't force a second one.
