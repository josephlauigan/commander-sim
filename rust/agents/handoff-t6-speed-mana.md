# Handoff t6-speed-mana (2026-10-19)
Branch t6-speed-mana. Task: mana (brief rust/agents/t6-speed.md).
## Steps
1. [x] build release, cardgen export --offline (status clean)
2. [x] stats (committed): t6stats::mana_snapshot (mn|T<n>, perm, hand, handc, n), mana_cast (T1-3), mana_held (end of main 2, T1-3),
       report line "mana T1-T5 ..." in t6report::render_mana. Baseline: audit/t6/speed-2026-10-19/mana-base600.txt (binary w/ stats, no rule change; win numbers identical to cal1000 profile)
3. [x] findings: mana T2 2.5-2.75, T3 3.5-4.2, T4 4.1-5.3; fast mana cast on turn it is castable except gated cards:
       Mox Diamond (needs 3 lands incl. hand), Chrome Mox (needs 2 spare cards), LED, rituals. Models (Petal, Opal, Amber, Spirit Guides, City, Tomb) checked OK.
4. [ ] v1 changes (uncommitted until measured): ai/t6_table.rs mana_first (dorks cmc<=2, 3-mana rocks, turns<=3, plan has no opinion);
       ai/pool.rs Chrome Mox spare>=1 and Mox Diamond n+bf>=2 on T6 turns<=3. Run audit/t6/speed-2026-10-19/mana-v1-600.txt
5. [ ] t3 lock (rust/agents/lock.sh writes to /tmp/claude-1000/t6ai-report/: mkdir it), tests (t6report, ai_t6_*), 2000-game confirm, report-t6-speed-mana.md
