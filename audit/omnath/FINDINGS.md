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

## 2026-10-19: trade with Tymna/Thrasios (paired swap, loose, look-ahead, 1,000 games per tier)
Worldly Tutor, Blossoming Defense, Bite Down out (to Tymna/Thrasios); Glorybringer, Thundermaw Hellkite, Smaug the
Magnificent in (from Tymna/Kraum). Raw: `2026-10-19-thrasios-trade/omnath-<tier>.txt`.
| Tier | Current | Swapped | Change |
|---|---|---|---|
| T1 | 21.0% | 23.7% | +2.7 (±3.5) |
| T2 | 27.2% | 32.3% | **+5.1 (±3.8)** |
| T3 | 26.4% | 27.8% | +1.4 (±3.8) |
| **Average over t1-t3** | **24.9%** | **27.9%** | **+3.1** |

Why (T2 breakdown): the dragons are cast in 87-90% of the games they're drawn and win 42-48% when cast; damage dealt
+4.0 per game (combat +3.6), eliminated 69% -> 65% of games. Blossoming Defense was stuck in hand 46% of its draws. The
cost: without Worldly Tutor, Omnath resolves 0.37 turns later (tutors 0.22 -> 0 per game). Hasty fliers give the deck
pressure while it waits for landfall, which it lacked (Weakness: loses to fast decks before Elementals pile up).
Also settles "Cards to confirm": Tymna's Terror of the Peaks copy frees up for Omnath if the trade goes ahead.
