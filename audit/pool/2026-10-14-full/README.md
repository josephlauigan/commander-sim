# Full pool run, 2026-10-14/15: every JD deck against every tier

Every JD deck (and Avery's Alela and Galadriel) against each opponent tier, T0 to T5: 66 cells, **4,000 games per
cell**, look-ahead AI, loose profile, seed 1015, `--any-tier` (JD asked for all tiers, to choose each deck's tiers).
A 95% interval is about ±1.4 points per cell. 25% is an even share at a four-player table. One file per cell
(`<deck>-<tier>.txt`, with the `--analyze` breakdown); `log.txt` has the run log.

Run on branch `calib-1014`, after the tier recalibration done the same night:
- **Tier 0 fully modeled** (all 132 missing precon cards; only Cursed Mirror is Approximate), then audited for
  generous code (9 faults fixed: seven lands that made free mana, Speaker of the Heavens, Idol of Oblivion,
  Bone Devourer, Thalisse, the bounceland rule).
- **Tier 2 evened out** (3 swaps each in Sythis, Kaalia, Lord Windgrace and Brago; `decklists/pool/t2-mid-b3/`).
- **DSL "X" fallback fixed** (16 cards silently used 3 for a counted amount, among them Kefka's Deadly Embrace and
  Shimmercreep, Cloud's Adelbert Steiner and Kor Spiritdancer, which played as a 6/2 in Light-Paws and Sythis).

Cells were run from frozen snapshots of the branch. Every cell run before a later fix was re-checked on the final code
(same seed, adaptive AI); the cells that changed were rerun, so all 66 cells reflect the final code.

## Win rate by deck and tier

| Deck | Bracket (deck doc) | T0 | T1 | T2 | T3 | T4 | T5 |
|---|---|---|---|---|---|---|---|
| Y'shtola | 4 | 43.0% | 44.4% | 55.2% | 43.6% | 39.3% | 23.3% |
| Sephiroth / Atraxa | 4 | 37.0% | 37.0% | 46.8% | 35.3% | 28.7% | 18.1% |
| Sephiroth FS | 3 | 41.6% | 41.9% | 48.6% | 38.7% | 32.5% | 20.4% |
| Sauron | 3 | 27.9% | 31.8% | 38.8% | 29.9% | 29.8% | 25.4% |
| Kefka | 4 | 22.4% | 30.8% | 39.9% | 27.9% | 30.1% | 24.2% |
| Sheoldred | 3 | 27.8% | 27.3% | 37.0% | 31.4% | 30.9% | 24.3% |
| Veyran | 3 | 18.9% | 23.0% | 26.9% | 23.2% | 21.4% | 13.6% |
| Titania | 2-3 | 15.5% | 20.3% | 30.3% | 27.7% | 25.6% | 17.5% |
| Cloud | 3 | 7.3% | 13.5% | 12.3% | 9.4% | 13.4% | 13.1% |
| Galadriel | 3 | 8.0% | 14.5% | 17.0% | 14.4% | 12.8% | 14.2% |
| Alela | 3 | 5.6% | 12.0% | 15.1% | 12.0% | 14.5% | 12.7% |

Tier calibration on the same code (look-ahead, each deck alone against three decks of the tier below; above 25% =
the upper tier is stronger): **T1 into T0 23.4%**, **T2 into T1 24.6%**, **T3 into T2 27.1%** (T4 into T3 and T5
into T4 unchanged since September: 24.1% and 28.7%). So T0, T1 and T2 are now about equally strong in the sim;
the clear steps are T2 → T3 → T5. Details: `audit/pool/2026-10-14-t0/`, `audit/pool/2026-10-14-t2/`.

## Reading the table

- **Tier 2 is every deck's easiest tier** (8-11 points above its T1 and T3), although T2 is now even within itself and
  about level with T1 head to head. T2 games are the longest (15.4 rounds, ~14% reach the round cap) and its decks
  are slow value decks; JD's decks are engines that need time, so they convert those games. Treat a T2 number as a
  soft table, not as the deck's strength.
- **Tier 0 is not an easy pod any more.** Fully modeled, the five WotC precons are about Tier 1 strength, and they
  punish the slow, low-interaction decks hardest (Cloud 7%, Alela 6%, Galadriel 8%).
- **Fair band** used below: 22-32% (holds its own without dominating).

## Recommended tiers

| Deck | Plays like | Recommended tiers | Now in `my_decks.toml` | Why |
|---|---|---|---|---|
| Y'shtola | **High B4** | **T5** | t4, t5 | Only T5 is fair (23%); 39-55% everywhere else. |
| Sephiroth / Atraxa | **Low-mid B4** | **T4** (T5 as a stretch) | t4, t5 | 29% vs T4; 18% vs T5. |
| Sephiroth FS | **Low B4** (bracket 3 on paper) | **T4-T5** | t1, t2, t3 | Dominates B3 tables (39-49%); fair vs T4 (33%) and T5 (20%). One Game Changer and a three-card loop keep it Bracket 3 by the rules, but expect to be the table's threat in a B3 pod. |
| Kefka | **B4** | **T3-T5** | t2, t3, t4 | Fair from T1 to T5 (24-31%) except soft T2 (40%); its six Game Changers make it B4. |
| Sauron | **High B3** | **T3-T5** | t2, t3, t4 | Fair at every tier (25-32%) except T2 (39%). |
| Sheoldred | **High B3** | **T3-T5** | t3, t4 | Fair at every tier (24-31%) except T2 (37%). |
| Veyran | **Mid B3** | **T1-T4** | t1, t2, t3 | 21-27% from T1 to T4; 14% vs T5. |
| Titania | **Mid B3** | **T2-T4** | t1, t2 | Better against T3/T4 (26-28%) than T1 (20%): Tier 1's fast aggro decks are its bad matchup. |
| Cloud | below every tier | (T1-T2 for now) | t1, t2 | 7-14% everywhere, even vs the precons. Needs a rework before a tier fits. |
| Galadriel | below every tier | (T1-T2 for now) | t1 | 8-17% everywhere. |
| Alela | below every tier | (T1 for now) | t1 | 6-15% everywhere. |

`data/my_decks.toml` is **not** changed here: these are recommendations for JD to accept or edit.
