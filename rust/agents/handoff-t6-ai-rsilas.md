# Handoff t6-ai-rsilas (deck:rograkh-silas-turbo-breach)
Started 2026-10-18. Worktree .claude/worktrees/t6-ai-rsilas, branch t6-ai-rsilas.
## Done
- read brief, SPEC, baseline (19.9%/18.4% wins, 95% Oracle line, 44% mulligans, 0.33 tries, 41% stopped)
## Next
1. find where slots are called (keep/prio/wish/go_now), why Breach/Ad Nauseam never go, why stopped w/o card
2. fill plan file, tests, measure (300 games t6, jobs 3), t3 lock
## Step 1 (done, uncommitted->commit): first version of plan file (keep, prio, wish). Findings: Rograkh was cast on turn 5 (generic commander
cmd_turn=2, Silas first at 62%); mulligans: shared 2-5 lands rule vs 23 lands/99; Floodcaller/Wishclaw wasted mana with Oracle+Consultation in hand;
Ad line needs 8 mana (ad_ready), rituals never used. Next: measure (t6 300 games jobs 3), go_now, tests, t3 lock.
