# Tier 6 realism baseline, 2026-10-22

`commander-sim --realism --pool t6 --games 2200 --seed 500000` (loose profile, look-ahead AI, the six Tier 6 pool
decks, 4 a pod), against `data/realism/t6.toml`. Raw output: `t6-baseline-2200.txt`. 2,152 of 2,200 games were
decisive (2.2% hit the round cap), so a seat share is known to about ±2 points.

**Result: 3 of 10 targets pass.** As expected from `audit/t6/speed-2026-10-19/REPORT.md`, the table is too slow and its
seat edge too flat. The baseline also shows too many combo wins and too large a gap between decks.

| Statistic | Sim (95% CI) | Target | Band | Result |
| --- | --- | --- | --- | --- |
| Seat 1 share of decisive games | 27.5% (25.6-29.4) | 32.5% | 29.5-35.5 | FAIL |
| Seat 2 share | 27.5% (25.6-29.4) | 26.9% | 23.9-29.9 | PASS |
| Seat 3 share | 22.4% (20.7-24.2) | 22.7% | 19.7-25.7 | PASS |
| Seat 4 share | 22.7% (21.0-24.5) | 17.9% | 14.9-20.9 | FAIL |
| Median round of a decisive win | 9 (9-10) | 7 | 6-8 | FAIL |
| Win round spread (75th - 25th percentile) | 5 rounds | none | | no target |
| Wins by combo or alternate win | 76.0% (74.1-77.7) | 47.8% | 37.8-57.8 | FAIL |
| Wins by damage | 23.5% (21.7-25.3) | 39.9% | 29.9-49.9 | FAIL |
| Wins by commander damage | 0.6% (0.3-1.0) | 11.6% | 3.6-19.6 | FAIL |
| Wins by poison | 0.0% (0.0-0.2) | 0.7% | 0-3 | PASS |
| Deck win spread (best - worst) | 16.3 pts | 4.0 | 0-8 | FAIL |
| Games over 15 rounds | 11.2% (10.0-12.6) | none | | no target |

Each deck's share of the decisive games it sat in: Kraum/Tymna 32.8%, Kinnan 29.3%, Dack Fayden 25.0%, Sisay 24.3%,
Rograkh/Silas 22.2%, Rograkh/Thrasios 16.5%.

## Reading it

- **Speed and seat edge fail together.** The median win is round 9 against a target of 7. Seat 1 wins 27.5% against
  32.5%, and seat 4 wins 22.7% against 17.9%. A slow table gives the first player's extra turn less value, so the edge
  flattens. This matches the speed report (seat 1 27%, seat 4 22-24%, median 9), which found the decks are draw- and
  mana-limited rather than waiting.
- **Deck spread** is 16.3 points, twice the band. Kraum/Tymna (32.8%) and Rograkh/Thrasios (16.5%) are the same
  outliers as in the speed report.
- **Win types are the weakest targets.** They come from casual Playgroup logs of these commanders, not from
  tournaments. The sim's 76% combo share may be closer to real cEDH than the casual logs. Commander damage at 0.6%
  against 11.6% probably reflects casual pods where Kraum or Tymna attack freely. Treat these as provisional until
  there is a tournament source.
- **11.2% of games run past round 15**, in a format where real games rarely do. There is no target yet, but it points
  the same way as the slow median.

## Method notes

- The seat targets are each seat's share of seat wins in EDHTop16's monthly report, April-September 2026. They are
  close to the earlier per-event figures (34.1 / 27.3 / 22.5 / 17.4, `decklists/pool/TARGETS-t6.md`).
- There is no `systemd` in this container, so the run was capped with a cgroup v1 memory group at 12 GB, swap 0. The
  machine has 15 GB in total. The run used about 30 MB.
