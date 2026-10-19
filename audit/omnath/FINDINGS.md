# Omnath, Locus of Rage (key `omnath`): findings

The deck's living list of strengths, weaknesses and what was tested. Built 2026-10-19 from Titania's mono-green
landfall list: Omnath, Locus of Rage leads, Titania, Protector of Argoth is in the 99, plus Wrenn and Six and a red
package (Terror of the Peaks, Goblin Bombardment, Decimate, Blasphemous Act, Scorching Dragonfire, Disintegrate,
Hellkite Tyrant). Matched tiers: t1-t3 for now (Titania's), to be re-set from these results.

## 2026-10-19: Omnath vs the Titania list it replaces (loose, look-ahead, 2,000 games per deck per tier)
| Tier | Titania | Omnath | Change |
|---|---|---|---|
| T1 | 17.4% (CI 15.8-19.1) | 20.9% (19.2-22.8) | **+3.5** (CIs apart) |
| T2 | 26.4% (24.6-28.4) | 27.4% (25.5-29.4) | +1.0 |
| T3 | 24.4% (22.6-26.4) | 25.4% (23.5-27.3) | +1.0 |
| **Average over t1-t3** | **22.7%** | **24.6%** | **+1.9** |
Same seeds and opponents per tier, but two different decks (not a paired swap), so each cell's noise is about ±2.6.
Raw: `2026-10-19-vs-titania/<deck>-<tier>.txt`.

Why (first read, from the runs' summaries; a deeper `--analyze` pass is still to do):
- Game length and how it wins are unchanged (wins around turn 14-15, losses 10-13; about 95% of wins by damage):
  the deck still wins late through the board, as Titania did.
- The gain is largest at T1, the slowest tier, where Omnath's 5/5 on every landfall (every fetch crack, every land
  Titania returns) has time to pile up; at T2-T3 the faster tables keep it to about +1.
- In the agent's first 100-game check, Omnath's first 5/5 came out in 60% of games.

## Weaknesses (to confirm with --analyze)
- Omnath costs seven ({3}{R}{R}{G}{G}) and needs two red: the mana base has 12 Mountains and 5 red duals.
- Like Titania, it loses to fast decks before the Elementals pile up.

## Cards to confirm (JD)
Terror of the Peaks, Blasphemous Act, Birds of Paradise, Command Tower, Exotic Orchard come from JD's spares and each
also appears in another deck: JD is checking he owns a second copy.
