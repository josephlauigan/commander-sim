# Sauron: Phyrexian Arena -> Cost of Brilliance (2026-10-11)

Would Sauron miss his Phyrexian Arena if it went to the Sheoldred deck (instead of buying her a copy)? Paired swap test
on Sauron's current list (Bedevil in Sheoldred's old slot; branch `sheoldred-deck` after main with PR #50): Arena out,
Cost of Brilliance (bulk) in. Look-ahead AI, loose profile, 1,000 paired games per tier (`run.sh`; per-tier lines in
`log.txt`).

| | T1 | T2 | T3 | T4 | T5 | All tiers |
|---|---|---|---|---|---|---|
| With Phyrexian Arena | 25.6% | 29.2% | 23.0% | 21.2% | 17.5% | **23.3%** |
| With Cost of Brilliance | 27.8% | 29.4% | 22.9% | 22.0% | 17.2% | **23.9%** |
| Change | +2.2 | +0.2 | -0.1 | +0.8 | -0.3 | **+0.6** |

Every tier is within its noise (about ±3), and the total (about ±1.4) too: **Arena is not critical to Sauron**; Cost
of Brilliance does as well. (Sauron's 23.3% here is below the 24.9% of the Sheoldred -> Bedevil test; the code has
changed since, PR #50's Kefka/Veyran trade among it, and the two are within about the noise.)
