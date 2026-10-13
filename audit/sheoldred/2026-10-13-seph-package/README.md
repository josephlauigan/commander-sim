# Sheoldred: a Sephiroth aristocrats package (2026-10-13)

Baseline: PR #60 (cloud-midgar) plus the sacrifice-outlet work on this branch (Phyrexian Tower fixed; JD's decks
sacrifice to free outlets for value; pool decks play as before). Paired test, look-ahead, loose, 1,000 games per list
per tier, 7 threads (`run.sh`, `pkg-<tier>.txt`). Cuts from `analyze-t*.txt` (the baseline list).

| Out | In |
|---|---|
| Dark Deal | Sephiroth, Fabled SOLDIER |
| Apex Witchstalker | Phyrexian Altar |
| Masterful Flourish | Carrion Feeder |
| Commander's Sphere | Common Crook |
| Potioner's Trove | Callous Inspector |
| Swamp | Phyrexian Tower |

| | T1 | T2 | T3 | T4 | T5 | All tiers |
|---|---|---|---|---|---|---|
| Current list | 29.5% | 41.4% | 30.7% | 31.7% | 24.6% | |
| Package | 26.5% | 38.8% | 32.7% | 29.4% | 23.8% | |
| Change | -3.0 | -2.6 | +2.0 | -2.3 | -0.8 | **-1.3** (about ±1.6) |

No clear effect, leaning slightly worse. Why (`analyze-pkg-t3.txt`, 480 games): only 2.0 creatures sacrificed per game
and Sephiroth flipped in 4.6% of games. The deck has no token makers, so the outlets have little to eat; the cuts
(two rocks, Witchstalker's life and landcycling, Dark Deal, Masterful Flourish) cost about what the drains add.
Not applied.
