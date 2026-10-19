# Tier 6 calibration (Stage 6), 2026-10-19

Final state of t6-cedh after Stage 6: `commander-sim --calibrate within --pool t6 --games 2000` on two seeds, both
profiles. Raw output: `t6-within-s500000.txt`, `t6-within-s900000.txt`. Targets: `decklists/pool/TARGETS-t6.md`.

## Result

| Deck | s500000 cons / loose | s900000 cons / loose | Target 18-32 |
| --- | --- | --- | --- |
| Kraum/Tymna | 31.7 / 30.3 | 28.8 / 29.5 | in |
| Dack Fayden | 26.2 / 29.4 | 28.2 / 29.6 | in |
| Kinnan | 26.9 / 28.2 | 27.7 / 28.6 | in |
| Sisay | 25.3 / 22.0 | 26.4 / 23.7 | in |
| Rograkh/Silas | 22.5 / 23.1 | 24.2 / 25.3 | in |
| Rograkh/Thrasios | 17.5 / 17.1 | 14.8 / 13.5 | **below** |

| Target | Result | Status |
| --- | --- | --- |
| Each deck 18-32% | five of six in; Rograkh/Thrasios 13.5-17.5 | **missed by one deck** |
| Best-worst gap 8 points or less | 14-16 points (about 7-9 without Rograkh/Thrasios) | **missed** |
| Seat shares 30-38 / 24-31 / 19-26 / 14-21 | 26.5-27.5 / 24.6-26.5 / 23.4-26.4 / 22.4-22.7 | **seats 1 and 4 out** |
| No deck wins mostly through a backup line | every deck wins mostly through its spec's main line | met |
| The real-data order (Kraum/Tymna, Sisay, Kinnan near the top; Rograkh/Thrasios a little behind; Rograkh/Silas last) | Kraum/Tymna top, Kinnan and Sisay middle, Rograkh/Silas below them: matches, except Rograkh/Thrasios is last instead of near the top | partly |

Average game 10.1-10.3 rounds, round-cap draws 2-3%.

## What changed in Stage 6

Modeling fixes (rules or engine errors found while calibrating):

- **Combos need a real kill** (main #100, merged): Kinnan + Basalt counted as a win without a mana sink. Kinnan fell
  from 32-34% to 27-28%; JD then withdrew the "Kinnan may stay the strongest" exception.
- **Sisay**: Marvin's two ability sets (Kefka's tap copies and Ioreth's untap) overwrote each other; every legend
  counted as a stoppable combo piece; the lines ignored pieces Sisay can fetch; her fetch ranked above a ready combo.
- **Rograkh/Thrasios**: the plan's "on the battlefield" check skipped lands, so Gaea's Cradle and Talon Gates in play
  were invisible; lines C (Badgermole Cub + a clone with Oboro and Cradle) and D (two Cubs, earthbent lands, Witness +
  Snap) modeled; Chord, Green Sun's Zenith and Sowing Mycospawn fetch the line's missing piece.
- **Dack**: more answer windows in the 35-pile flip shortcut (no measurable effect: opponents rarely hold an answer).
- **Staff of Compleation** (Dack and Godo): "destroy target permanent you own" had been parsed as "you control" and
  destroyed its own Sol Ring; now card code aimed at your permanents an opponent controls.

Plan tuning: Dack waits for a safer table before casting Dack (34.8/36.3 -> 26-30%); Kraum/Tymna goes for Oracle only
with cover (34.3 -> 29-32%); Rograkh/Thrasios keeps hands that find Cradle/Gates, stops recasting Rograkh at high tax
and doesn't recast Thrasios into open removal (13-15 -> 13.5-17.5%).

## The open gaps, and why

1. **Rograkh/Thrasios is below range (13.5-17.5%).** Not a modeling error found in four passes
   (reports `rust/agents/report-t6-cal-rthras*.md`). Its main line needs Gaea's Cradle, Talon Gates, Oboro and six
   creatures together; it wins at median round 12, the slowest in the tier. Removal on it is fair (49 removals in 60
   traced games vs Sisay 51, Kinnan 46). Real-data share is 25.7%, so the sim underrates it.
2. **The seat edge is flat.** No seat bug (report `report-t6-cal-seats.md`): every seat mulligans, ramps and resolves
   its commander the same; the opening follows the rules. The table simply wins late: only ~21% of wins come by round
   6, so going first is worth little. Real cEDH games end around turns 4-6.
3. Both gaps share one cause: **the Tier 6 decks win later than real cEDH decks** (median round 7-9 for the fast decks,
   12-13 for Sisay and Rograkh/Thrasios). Fixing that means faster assembly of each deck's lines (tutor chains, fast
   mana sequencing, going for it with less cover), a larger project than this stage.

## Effect on Godo

Godo's baseline (Stage 7, `audit/godo/`) is ~49-51% against this table, far above its real 21% and outside the 15-35%
plausibility range in TARGETS-t6.md. Godo wins at median round 6 while the table wins around round 10: Godo races
slow opponents. Treat Godo numbers against Tier 6 as comparative (swap A/B tests) until the table's speed is fixed.
