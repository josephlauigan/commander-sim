# Adaptive rollouts for the close calls the margin hands back (2026-10-30)

Branch `ai-foundations`, from a08af77a (the current eval). Brief: `rust/agents/ai-rollouts.md`. Raw output:
`raw/rollouts/` (`progress.log` lists every run with its command and time; `run.sh`, `phase1.sh`, `phase2.sh` ran them;
`evf.awk`, `gf.awk`, `flow.awk`, `cmp.sh` summarise them; `*-evf.txt` are the EVF summaries). Code: `ai/search.rs`
and `settings.rs`, marked `// --- ai-rollouts`. Tests: `tests/ai_rollouts.rs`. Nothing was recalibrated or retuned.

## Summary

- **Built:** when the margin hands a main-phase decision back to the heuristic, every option of that decision gets 6
  more rollouts at a time (new seeds, the same for every option), and the pick is made again on all of them. It stops
  when the margin holds, when the look-ahead's best becomes the heuristic's top play, when the gain is at or below
  0.5 standard errors, or at 24 rollouts per option. The extra playouts count toward `search_work`.
- **Kept (on by default):** cap 24, batches of 6, early stop at 0.5 SE. The hand-back rate falls from **50.6% to
  34.1%** (Tier 6) and **42.1% to 27.4%** (Tier 3). Time per 1,000 games: **120 → 194 s** (1.62x) and **91 → 106 s**
  (1.17x). A decision uses 10.3-11.0 rollouts per option on average (6 before).
- **The cost of it:** the look-ahead now departs from the heuristic's top play twice as often (7.9 → 15.2% Tier 6,
  11.2 → 18.5% Tier 3), and the final pick changes on a second set of seeds more often (10.9 → 16.7%, 11.9 → 14.8%).
  The bare pick is a little steadier (51.2 → 47.8%, 42.9 → 39.1%).
- **Tier 6 (4,000 games):** 3 of 6 targets pass either way, spread 10.4 → 10.8. Tymna cast / out by T4 72 / 59% →
  67 / 55%. Everything else in the flow within 0.1 card and 0.15 mana. Goldfish within 0.2 turn for every deck.
- **Tiers 0-5 (4,000 games):** 13 → 18 of 46 targets pass (t1 +1, t2 +1, t3 +2, t4 +1). No row went PASS → FAIL except
  t3 wins by damage (58.7 → 57.7%). Several of the margin's large deck moves partly reverse (Teysa +2.6, Tatyova -1.5,
  Krenko -1.7, Windgrace -1.3).
- **JD's decks (1,000 games):** three cells left the current eval's interval: veyran t2 +2.9 (back to A+B), alela t1
  -1.3, thrasios t5 -2.4. Averages over their tiers move by 1.6 points or less except thrasios (-2.4) and karn (-1.6).

## What was built (`ai/search.rs`)

- `adapt_rollouts(vals, ex, ti, (bare, best), knobs, more)`: the rule, on the per-rollout values the decision already
  has. It runs only when the margin handed the decision back (`bare != ti`, `best == ti`). Each round asks `more` for
  the next batch of rollouts of every option (rollout indices 6..12, 12..18, ...: `playout_copy` seeds them from the
  decision's seed and the index, so each batch has new seeds and every option shares them), then picks again with
  `pick_with`, the same rule as before (the bare argmax of the capped sums, then the margin against the top play).
  Stops: the pick stands (the margin holds); the bare argmax is the top play itself (the look-ahead agrees with the
  heuristic: not a hand-back); `hopeless` (the paired mean gain over the top play is at or below `stop` standard
  errors, or at or below 0); or `cap` rollouts per option.
- `rollout_vals`: the batch, with the same bonuses `choose_inner` adds (ramp, the kept-hold credit). Karn's and
  Cloud's `search_bonus` are in `ex` as before.
- The first six rollouts and their pick are computed exactly as before. With the switch off, or with a cap of 6, the
  games are byte-identical to a08af77a: the Tier 6 1,000-game output of `off-t6-diag` equals `eval/part4-noise/
  det-t6-diag.txt` byte for byte, and its EVF summary is the same to the decimal (test
  `in_a_game_the_cap_bounds_the_rollouts_and_six_is_the_old_rule` checks the cap of 6 in a game).
- `LAST_SCORES` (the practice advisor's ranking) shows the means over all the rollouts used, scaled as before.
- Knobs (`settings.rs`): `AI_EVAL_ADAPT=0` turns it off; `AI_EVAL_ADAPT_CAP` (24), `AI_EVAL_ADAPT_BATCH` (6),
  `AI_EVAL_ADAPT_STOP` (0.5). The margin's z is `AI_EVAL_MARGIN`, as before.
- Diagnostics: `AI_EVAL_FALLBACK=1` EVF lines gain three columns: 16 the rollouts per option the decision used, 17
  whether the margin handed it back on the first six, 18 the rollouts the second set used. The second set of seeds is
  now decided by the same rule, adaptive rollouts included, so "bare flips" and "final flips" compare like with like;
  "bare" is the argmax over all the rollouts a set used. `AI_EVAL_DIAG` (EVD/EVN) still works; its playouts include
  the extra ones, and EVN's second set stays at six rollouts (Part 2's noise measure).

## Measurements

### Tier 6 and Tier 3, 1,000 games each, every setting (`raw/rollouts/{off,a24,a30,a24s1}-t{6,3}*`)

"Handed back": the margin returned the decision to the heuristic in the end; "at six": on the first six rollouts.
"Departs": the final pick isn't the top play. "SE": paired SE of top vs its best alternative, over the rollouts used.
Flips: the pick changes on a second set of seeds. Rollouts: per option, all decisions / those handed back at six /
share of those that ran to the cap. Time: the run without the diagnostic, `--jobs 22`.

| setting | handed back | at six | departs | SE | bare flips | final flips | rollouts | at six | to cap | s per 1,000 games | x current |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| current (off) | 50.6 / 42.1% | 50.6 / 42.1% | 7.9 / 11.2% | 6.26 / 3.83 | 51.2 / 42.9% | 10.9 / 11.9% | 6 / 6 | - | - | 120 / 91 | 1 / 1 |
| **cap 24, stop 0.5 (kept)** | **34.1 / 27.4%** | 49.7 / 41.7% | 15.2 / 18.5% | 5.05 / 3.09 | 47.8 / 39.1% | 16.7 / 14.8% | 10.95 / 10.27 | 16.0 / 16.2 | 37 / 38% | **194 / 106** | **1.62 / 1.17** |
| cap 30, stop 0.5 | 32.9 / 26.1% | 50.0 / 41.9% | 16.3 / 19.3% | 4.97 / 3.15 | 47.5 / 38.8% | 17.7 / 15.4% | 11.87 / 11.01 | 17.7 / 18.0 | 29 / 30% | 211 / 105 | 1.76 / 1.15 |
| cap 24, stop 1.0 | 41.4 / 33.8% | 50.1 / 41.9% | 13.4 / 16.6% | 5.74 / 3.44 | 50.3 / 42.2% | 15.7 / 14.5% | 8.35 / 7.92 | 10.7 / 10.6 | 13 / 12% | 170 / 83 | 1.41 / 0.92 |

What happened to the decisions handed back at six (cap 24, stop 0.5):

| | handed back at six | still handed back | more rollouts made the top play the best | departed on evidence |
| --- | --- | --- | --- | --- |
| Tier 6 | 21,512 | 14,767 (69%) | 3,525 (16%) | 3,220 (15%) |
| Tier 3 | 21,935 | 14,418 (66%) | 3,452 (16%) | 4,065 (19%) |

Where they stopped (Tier 6): 4,678 at six (hopeless at once), 5,919 at 12, 2,972 at 18, 7,943 at 24. The share of the
remaining decisions each batch resolves falls: 23% at 12, 16% at 18, 13% at 24.

By the kind of the heuristic's top play (`evdiag::kind`), current → kept, handed back / rollouts per option:

| kind | Tier 6 | Tier 3 |
| --- | --- | --- |
| commander | 54.5 → 37.1%, 11.7 | 47.3 → 28.0%, 10.5 |
| engine | 53.6 → 36.1%, 11.2 | 45.7 → 27.8%, 10.5 |
| mana | 53.2 → 36.1%, 11.3 | 45.4 → 29.5%, 10.7 |
| creature | 52.4 → 35.2%, 11.2 | 43.7 → 28.9%, 10.4 |
| removal | 55.4 → 36.0%, 11.8 | 42.7 → 26.4%, 10.4 |
| other | 50.1 → 33.8%, 11.0 | 39.5 → 26.5%, 10.1 |
| stop | 42.7 → 28.3%, 9.4 | 30.5 → 18.9%, 8.1 |

Bare flips by kind, Tier 6, current → kept: commander 54.0 → 50.0%, engine 53.0 → 52.2%, mana 54.5 → 51.0%, creature
52.3 → 48.7%, removal 53.0 → 51.7%, other 52.1 → 48.1%, stop 40.6 → 37.5%. Full tables: `*-evf.txt`.

### Why cap 24, batches of 6, stop 0.5

- **Cap 30** cuts 1.2 / 1.3 more points of hand-back for 9% more time at Tier 6 (1.76x against 1.62x). The fifth batch
  resolves about 11% of the decisions it runs on, against 23% for the second. Not worth it.
- **Stop at 1.0** saves 0.2x of the time but keeps only half the cut (41.4 / 33.8% handed back). 0.5 keeps most of
  the gain: decisions stopped at six are those whose gain is under half a standard error, and few of those ever clear
  two.
- **Batches of 6** (the brief's start) were not varied: with the early stop and the cap fixed, a smaller batch would
  only stop sooner within the same 6..24 range, and each batch is one more test (see "repeated looks" below).
- All three settings cut the hand-back clearly and stay under 2x at Tier 6 and Tier 3, so a setting is kept, and the
  switch is on by default.

### Tier 6 at 4,000 games (`full-realism-t6.txt` against `eval/full-eval/realism-t6.txt`)

| | current eval | adaptive | target |
| --- | --- | --- | --- |
| seats 1 / 2 / 3 / 4 | 29.1 / 26.4 / 22.8 / 21.7 | 27.6 / 27.0 / 23.5 / 21.9 | 31.8 / 27.3 / 22.1 / 18.8 |
| median round, spread, targets passing | 7, 10.4, 3 of 6 | 7, 10.8, 3 of 6 | 7 (6-8), 4 (0-8) |
| decks: Dack / Kinnan / Kraum / Rog-Silas / Rog-Thrasios / Sisay | 26.7 / 21.0 / 29.6 / 20.6 / 21.2 / 31.0 | 24.4 / 23.9 / 28.5 / 20.6 / 21.4 / 31.4 | |
| Tymna seen / cast / out by T4 (Kraum/Tymna) | 100 / 72 / 59% | 100 / 67 / 55% | |
| Rhystic Study seen / cast / out by T4, Kraum/Tymna, Kinnan | 15/11/9, 11/7/6 | 15/10/9, 11/7/7 | |
| cards seen T4 / T6: Kraum/Tymna, Kinnan, Rog/Silas, Sisay | 13.3/18.9, 11.9/15.9, 12.4/15.9, 11.7/14.8 | 13.3/19.0, 12.0/16.0, 12.5/16.0, 11.8/14.8 | |
| mana spent T2 / T3 / T4: Kinnan, Kraum/Tymna | 2.61/4.01/5.26, 2.02/2.71/3.23 | 2.67/4.11/5.31, 2.04/2.74/3.29 | |

Seat 1 moved 1.5 points further from its target; Kinnan +2.9 and Dack -2.3 (about ±1.6 points is one standard error
at 2,650 seatings). Tymna by T4 is 4-5 points lower: with more rollouts the look-ahead overrides the heuristic's Tymna
more often on what the playouts show; why the playouts rate Tymna lower than the heuristic does was not traced (listed
for JD).

### Goldfish (`--goldfish t6,scion --games 2000 --seed 500000`): median / mean kill turn, won by T3, no kill

| deck | current eval | adaptive |
| --- | --- | --- |
| scion | 10 / 9.8, 4.7%, 1.7% | 9 / 9.8, 4.8%, 1.5% |
| Kraum/Tymna | 8 / 8.8, 3.2%, 0.7% | 8 / 8.7, 3.9%, 0.6% |
| Kinnan | 8 / 8.4, 2.5%, 1.1% | 8 / 8.2, 2.6%, 0.8% |
| Rograkh/Silas | 9 / 9.1, 6.3%, 13.3% | 9 / 9.2, 6.5%, 12.8% |
| Rograkh/Thrasios | 9 / 9.3, 0.0%, 0.2% | 9 / 9.1, 0.0%, 0.2% |
| Sisay | 7 / 7.3, 0.3%, 0.1% | 7 / 7.2, 0.3%, 0.1% |
| Dack | 10 / 9.7, 9.2%, 9.3% | 10 / 9.7, 9.7%, 8.3% |

Every mean within 0.2 of a turn. Scion's median is back to 9 (its kills split evenly across turns 9 and 10).

### Tiers 0-5 at 4,000 games (`full-realism-tN.txt` against `eval/full-eval/realism-tN.txt`)

Targets passing, current eval → adaptive: t0 4 → 4 of 9, t1 4 → 5, t2 1 → 2, t3 2 → 4, t4 1 → 2, t5 1 → 1: **13 → 18 of
46** (A+B, before the eval, had 22). Every row that flips:

| tier | statistic | current eval | adaptive | flip |
| --- | --- | --- | --- | --- |
| t1 | seat 1 share | 29.2% | 30.3% | FAIL → PASS |
| t2 | seat 3 share | 24.2% | 23.4% | FAIL → PASS |
| t3 | seat 2 share | 25.6% | 26.8% | FAIL → PASS |
| t3 | seat 3 share | 23.7% | 22.4% | FAIL → PASS |
| t3 | median round | 10 | 9 | FAIL → PASS |
| t3 | wins by damage | 58.7% | 57.7% | PASS → FAIL |
| t4 | seat 3 share | 23.4% | 22.6% | FAIL → PASS |

Deck shares, current eval → adaptive (moves of 1.5 points or more in bold; one standard error is about 1.3 points at
3,200 seatings):
- t0: Anikthea 31.6 → 31.3, Captain America 24.0 → 23.2, Giada 31.8 → 31.4, Stella Lee 17.0 → **18.6**, Zurgo 20.5 → 20.5
- t1: Isshin 26.5 → 26.8, Lathril 18.9 → 18.7, Light-Paws 45.7 → 44.7, Tatyova 19.1 → **17.6**, Teysa 14.7 → **17.3**
- t2: Brago 11.7 → 12.9, Kaalia 28.3 → **26.0**, Windgrace 27.1 → 25.8, Meren 16.3 → 17.4, Sythis 41.9 → 43.2
- t3: Atraxa 22.6 → 23.4, Aurelia 27.9 → 27.0, Korvold 16.3 → 17.5, Marwyn 20.0 → 19.6, Tergrid 38.4 → 37.6
- t4: Chulane 14.3 → 14.4, Edgar 46.6 → 47.3, Krenko 30.6 → **28.9**, Prosper 15.6 → 15.3, Yuriko 18.5 → 19.7
- t5: Kinnan 31.8 → 32.8, Miirym 30.5 → 30.2, Urza 21.8 → 21.9, Yawgmoth 18.9 → 18.3, Zur 21.9 → 21.8

The margin's large moves (eval-fix.md, "Why things moved") partly reverse: Teysa 14.7 → 17.3 (A+B 18.7), Tatyova 19.1 →
17.6 (A+B 14.2), Krenko 30.6 → 28.9 (A+B 26.5), Windgrace 27.1 → 25.8 (A+B 21.5), Meren 16.3 → 17.4, Edgar 46.6 → 47.3
(A+B 49.5), Kinnan t5 31.8 → 32.8 (A+B 36.1). A third of the close calls the heuristic got back are now decided by the
look-ahead again, on evidence this time.

### JD's decks (`--all-decks --pool all --games 1000`, `full-all-decks.txt` against `eval/full-eval/all-decks.txt`)

| deck | A+B | current eval | adaptive | outside the current eval's interval? |
| --- | --- | --- | --- | --- |
| seph t3 / t4 | 25.9 / 18.3 | 24.6 / 17.4 | 25.9 / 16.8 | no |
| veyran t1 / t2 / t3 | 15.3 / 13.1 / 9.9 | 13.4 / 10.7 / 11.6 | 14.6 / 13.6 / 11.7 | **t2: +2.9 (8.9-12.8)**, back to A+B's 13.1 |
| sauron t3 / t4 | 23.7 / 20.7 | 25.5 / 22.2 | 26.1 / 21.8 | no |
| galadriel t1 / t2 | 6.3 / 6.5 | 6.5 / 5.4 | 5.7 / 5.3 | no |
| yshtola t5 | 22.0 | 24.0 | 23.7 | no |
| alela t1 | 5.2 | 5.0 | 3.7 | **-1.3 (3.8-6.5)** |
| kefka t3 | 18.8 | 18.8 | 19.2 | no |
| sheoldred t3 / t4 | 19.6 / 19.3 | 18.4 / 19.0 | 18.5 / 20.6 | no |
| cloud t1 / t2 / t3 | 18.9 / 11.3 / 16.0 | 20.2 / 14.1 / 18.6 | 20.9 / 14.0 / 17.5 | no (t2, t3 stay above A+B) |
| thrasios t5 | 11.9 | 12.2 | 9.8 | **-2.4 (10.3-14.3)** |
| sephfs t5 | 18.2 | 20.2 | 19.4 | no |
| karn t4 / t5 | 17.2 / 20.9 | 18.8 / 22.3 | 17.4 / 20.6 | no |
| scion t6 | 28.6 | 29.2 | 28.0 | no |

Averages over each deck's tiers, current eval → adaptive: seph 21.0 → 21.4 (over t3, t4), veyran 11.9 → 13.3 (t1-t3),
sauron 23.9 → 23.9 (t3, t4), galadriel 6.0 → 5.5 (t1, t2), yshtola 24.0 → 23.7 (t5), alela 5.0 → 3.7 (t1), kefka
18.8 → 19.2 (t3), sheoldred 18.7 → 19.6 (t3, t4), cloud 17.6 → 17.5 (t1-t3), thrasios 12.2 → 9.8 (t5), sephfs 20.2 →
19.4 (t5), karn 20.6 → 19.0 (t4, t5), scion 29.2 → 28.0 (t6). Three cells left the current eval's interval: veyran t2
(+2.9, back to A+B), alela t1 (-1.3) and thrasios t5 (-2.4). With 13 decks and 22 cells, one or two cells outside a
95% interval are expected from noise alone, so alela and thrasios need a second look before they are read as
real (listed for JD). Both runs skipped the same 13 games to internal simulator errors (predates this work).

### Time

Phase 1 is the clean comparison (same binary, `--jobs 22`, one run at a time): 1.62x at Tier 6 and 1.17x at Tier 3.
The full set ran at `--jobs 22`, the eval's at `--jobs 16`, so their times aren't comparable directly; on this machine
(24 cores, 8 of them P-cores) 22 threads are perhaps 1.2-1.3 times 16. Seconds, eval (16 threads) → adaptive (22):
goldfish 164 → 193, t6 612 → 764, t0 788 → 1,045, t1 681 → 944, t2 980 → 1,417, t3 336 → 398, t4 546 → 703, t5 408 →
680, all-decks 3,054 → 4,159 (1.36x on more threads). Tier 5 (1.67x on more threads) may be near or over 2x at equal threads; Tier 5 wasn't
timed alone (the brief times Tier 6 and Tier 3).

## Decisions made without JD (the conservative call, with the alternative)

1. **Every option gets the extra rollouts, not only the pick and the top play.** The brief says "the same options", and
   the bare pick is an argmax over all of them: extending two would compare them on more seeds than the rest. The
   alternative (only the two) costs about half; not measured.
2. **More rollouts that make the top play the look-ahead's best end the extension** and count as the look-ahead's own
   choice, not a hand-back (16% of the decisions handed back at six). The alternative, counting them as hand-backs,
   would report 42.3% / 34.0% instead of 34.1% / 27.4%.
3. **The early stop is "hopeless": the paired mean gain at or below 0.5 SE (or at or below 0).** The alternative
   measured, 1.0, keeps half the gain.
4. **The margin is tested again after each batch at the same z = 2.** Repeated looks raise the chance that noise alone
   clears the margin (up to four looks here); the final-pick flip rate rose 10.9 → 16.7% at Tier 6, partly from this and
   partly because departures doubled. The alternative, a stricter z for later looks (a sequential boundary), is a
   retune and is left for JD.
5. **The switch is on by default** (the brief: keep a setting that is worth it). `AI_EVAL_ADAPT=0` gives a08af77a's
   games exactly.
6. **`search_hold.rs`'s Ad Nauseam test is pinned to six rollouts** (`adapt = false`). With 24 rollouts the look-ahead
   finds casting Sheoldred, the Apocalypse ahead of holding five mana for Ad Nauseam by more than two standard errors
   (16.5 against 15.7 a playout) and casts it. The test asserted the six-rollout outcome, which was the margin handing
   a close call back. Whether holding is right there is JD's call; nothing was tuned to keep it.
7. **The diagnostic's second set of seeds uses the same rule**, adaptive rollouts included (it spends no game budget).
   The alternative, six rollouts on the second set, would compare a 6-24-rollout pick with a 6-rollout one.

## Decisions for JD

1. **Keep adaptive rollouts on (cap 24, batches of 6, stop 0.5)?** It hands back a third fewer close calls at
   1.6x / 1.2x the time, and moves Tiers 0-5 back toward A+B (13 → 18 of 46 passing). Against it: Tymna by T4 is 4-5
   points lower, and the final pick is less stable on a second set of seeds (16.7% against 10.9% at Tier 6). The
   alternative is `AI_EVAL_ADAPT=0` (the current eval).
2. **A sequential boundary for the repeated margin tests** (for example z = 2.4 at each of the four looks, or a higher z
   on the early ones): fewer noise departures, at the price of fewer departures overall. It is a retune; not done.
3. **The Ad Nauseam hold (decision 6):** with 24 rollouts the look-ahead casts Sheoldred over holding for Ad Nauseam.
   Keep the test pinned, or accept the new play and change the test?
4. **Runs not made (outside the brief), if wanted:** Tier 5's time alone, current vs adaptive, 1,000 games (to check
   it is under 2x at equal threads); the full set's current eval at `--jobs 22` for a clean time comparison; traces of
   the Tymna decisions that now depart at Tier 6; `--deck thrasios --pool t5` and `--deck alela --pool t1` at 4,000
   games, current against adaptive, to tell their moves from noise.
