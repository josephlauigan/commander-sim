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
## Step 2: run1 (keep+prio+wish only): 14-16% wins, mull 35.7% (keep too strict -> loosened: no action card ok if 2+ fast), tries 0.26-0.29.
Trace finding: lookahead picks "stop (hold mana)" over Lotus Petal / Rograkh (hold_bonus in ai/search.rs ignores hold_up) -> hook added
in ai/search.rs hold_bonus (adds t6 hold_up; t6-only since hold_up is None outside t6). hold_up now -50 when developing. Next: re-measure.
## Step 3: run2 (hold_up develop-first): 13.8% / 17.3% wins, mull 30%, median win round 6 (was 7). Added: lands==0 keep only with 3 fast+tutor,
Squelcher cast before Oracle line + go_now true with Squelcher out. Next: run3, tests (tests/ai_t6_rograkh_silas.rs written, not yet run), t3 lock, report.
