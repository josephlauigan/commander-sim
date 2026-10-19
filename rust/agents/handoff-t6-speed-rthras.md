# Handoff t6-speed-rthras (PAUSED by lead 2026-10-19, no builds/sims until "resume")
Task: Rograkh/Thrasios end-of-opponent-turn dig for Tier 6; check Cradle mana is used. Branch t6-speed-rthras.
## Done
- Release build + cardgen export OK. Baseline 600 (seed 500000): Thrasios 14.0 / 15.2 (conservative / loose), median win 13.
- Stats only (no decisions): `mn|dig|eot|own|land` in thrasios_go (impls/tymna_cards.rs), `t6stats::eot_window` called at the top of brain::end_of_turn_window, rendered in t6report render_mana ("Thrasios digs per seated game ..."). Win rates unchanged with the stats (14.0/15.2). UNCOMMITTED code edits are in the worktree; last build + run with them worked (/tmp/claude-1000/t6speed-rthras/base600c.txt).
## Finding so far
The end-of-turn dig already exists (thrasios_options post None, utility 3; window = end of the turn before the deck's own turn; mana is the same pool as at the 3 opponent end steps, so one window is equivalent). Rograkh/Thrasios: 8.8 windows/game, Thrasios in play in 5.2, 2.7 untapped mana on average, those windows offer 0.56 digs each (~2.9 digs/game offered) vs 2.22 EOT + 1.02 own-turn digs done: the spare mana is already spent. Only 41/322 digs hit a land. So the dig is mana-limited, not window-limited. Cradle mana counts (total_mana).
## Next (after resume)
- Idea: dig is 4 mana for ~1 card; speed needs earlier Thrasios/Cradle, not more digs. Options: scry policy for lands (keep lands on top while under 6 mana), dig before land drop when the top is a land. Measure 600 then 2000; t3 lock diff; full cargo test --release; report rust/agents/report-t6-speed-rthras.md.
