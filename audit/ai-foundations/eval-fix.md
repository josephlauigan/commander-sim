# The look-ahead's evaluation, fixed at the cause (2026-10-28)

Branch `ai-foundations`. Brief: `rust/agents/ai-eval.md`. Raw output: `raw/eval/`:
- `part1/`: the diagnostic runs (`EVD` lines, gzipped), `summary.txt`, the awk scripts and 5 traced games;
- `part2/`: each rule alone and together (Tier 6, 1,000 games), the noise diagnostic (`EVN` lines), `summary.txt`;
- `part3-must/`: the older must-do mana rules, each off;
- `full-eval/`, `full-AB/`, `full-base/`: the three-way comparison.

Code: `ai/search.rs` (the model and the decision rule, documented there), `ai/evdiag.rs` (the diagnostic),
`settings.rs` (knobs), all marked `// --- ai-eval`. Tests: `tests/ai_eval.rs`.

## Summary

- **The proven cause (Part 1, Tier 6 1,000 games, A and B off).** The look-ahead's "stop" wasn't stop. In 88-97% of
  main-1 playouts and 66-81% of main-2 playouts the stop line cast the same engine, commander or rock before the
  horizon. So `strength`'s 0.9 for a card in hand (cause 1) was not the issue (the hand term differed by -0.2 to
  +0.6). The real gap was 0.1-2.5 points (cause 2, mostly main 2). The hold bonus (cause 3) added +2.9 to +3.4 to
  the "stop (hold mana)" label, though that line kept the mana up only 4-6 points more often than the cast in main 1.
  Everything else was playout noise: the standard error of top minus stop at 6 rollouts is 6-10 points, and the pick
  changed on a second set of seeds in 60% of decisions.
- **The model (`ai/search.rs`).**
  - `strength` values a permanent for what it will make over the next two own turns past the playout (weights 1 and
    0.5), in card units (a card in hand = 0.9).
  - Engines produce at the measured step 3 rates (tax engines scale with the opponents who cast), or at `eng=N`.
  - Mana is converted at one rate, 0.44 of a card per mana (a Treasure's 0.4 over a card's 0.9, so one mana a turn =
    a land's 0.6).
  - The hold bonus goes to the playouts that actually kept the mana up.
  - The look-ahead overrules the heuristic only beyond two standard errors (the margin).
  - Playouts are deterministic (the heuristic's top play).
- **Removed:** A and B (`engine_first`), and the cheap-rock, Chrome Mox and mana-first must rules (each off lost
  nothing).
- **Three-way, Tier 6 (4,000 games), base / A+B / eval:**
  - Tymna out by T4: 40 / 63 / 59%. Rhystic Study cast by T4 (Kraum/Tymna): 8/14, 10/14, 11/15.
  - Kraum/Tymna cards seen by T6: 17.8 / 18.7 / 18.9.
  - Scorecard: 3 / 3 / 3 of 6 pass; spread 11.3 / 11.6 / 10.4.
  - Goldfish within 0.2 of a turn of A+B for every deck.
- **Tiers 0-5:** 22 → 13 of 46 targets pass. Most flips are seat shares that moved 0.4-2.1 points across a band edge.
  The margin causes the large deck moves (Tatyova +4.9, Windgrace +5.6, Krenko +4.1, Edgar -2.9, Kinnan t5 -4.3, Teysa
  -4.0), because close calls are now decided by each deck's hand-tuned heuristic.
- **JD's decks:** cloud t2 / t3 +2.8 / +2.6 and veyran t2 -2.4 left A+B's interval (both the margin). Scion 28.7 /
  28.6 / 29.2%.

## Part 1: the cause, measured

### What was recorded

Env `AI_FLOW_MUST=0` turned step 3's A and B off (since removed). Env `AI_EVAL_DIAG=1` writes every main-phase
look-ahead decision whose heuristic top option is a card engine (Rhystic Study, Mystic Remora, Esper Sentinel,
Necropotence, an `eng` card...), a commander cast from the command zone, or a mana permanent (rock, mox, dork). For
each option it records:
- the rollout average, and over the playouts that didn't end the game, `strength`'s terms for the decider (life,
  board `pval`, power, hand, lands, Treasures, combo) and the opponents' term;
- the hold and ramp bonuses;
- the playouts won, lost and cut;
- two facts per playout: had the top option's card been cast by the end, and was the held card still in hand and
  payable when the decider's turn ended.

The games are byte-identical with the diagnostic on (checked against step 3's base run). Tier 6 (`--pool t6`) and
Tier 3 (`--pool t3`), 1,000 games each: 15,155 and 22,534 decisions.

### Every decision, the top option against "stop" (both always played out on the same seeds)

Tier 6. "Raw" terms are means over open playouts, before `evaluate`'s tanh; "hold" is the bonus credited to stop.

| class | step | hold | decisions | top picked | stop picked | top - stop: score | open eval | board | hand | hold bonus | stop line cast the card | kept mana up: top / stop |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| commander | main 1 | yes | 2026 | 13% | 62% | -3.2 | +0.13 | +0.05 | +0.03 | 3.10 | 71% (top 70%) | 63% / 68% |
| commander | main 1 | no | 2986 | 33% | 33% | -0.3 | -0.09 | -0.09 | +0.04 | 0 | 72% (top 71%) | - |
| commander | main 2 | yes | 1478 | 19% | 61% | -2.2 | +0.74 | +0.70 | +0.12 | 3.09 | 74% (top 73%) | 69% / 100% |
| commander | main 2 | no | 1292 | 43% | 34% | +0.6 | +0.76 | +0.47 | +0.01 | 0 | 78% (top 75%) | - |
| engine | main 1 | yes | 583 | 23% | 56% | -1.4 | +0.26 | +0.25 | +0.16 | 2.89 | 88% | 60% / 64% |
| engine | main 1 | no | 1177 | 41% | 32% | +1.2 | +0.59 | +0.46 | +0.14 | 0 | 93% | - |
| engine | main 2 | yes | 413 | 33% | 43% | -2.2 | +2.55 | +1.92 | +0.60 | 2.93 | 66% | 66% / 99% |
| engine | main 2 | no | 523 | 51% | 29% | +2.0 | +2.31 | +1.73 | +0.49 | 0 | 71% | - |
| mana | main 1 | yes | 1237 | 17% | 52% | -2.3 | +0.46 | +0.30 | -0.00 | 3.35 | 95% | 66% / 70% |
| mana | main 1 | no | 1696 | 41% | 33% | +1.1 | +0.18 | +0.29 | -0.01 | 0 | 97% | - |
| mana | main 2 | yes | 871 | 21% | 51% | -2.6 | +1.53 | +1.48 | -0.02 | 3.41 | 81% | 76% / 99% |
| mana | main 2 | no | 873 | 48% | 33% | +1.4 | +1.02 | +1.03 | -0.22 | 0 | 76% | - |

(The commander's "cast" share is about 70% in both lines because a commander removed and returned to the command zone
counts as not cast. The other terms, life, power, lands, Treasures, combo and the opponents', moved by 0.3 or less.)

Tier 3 shows the same shape with fewer held counters (2,000 of 22,534 decisions hold mana): top - stop open eval
+1.3-1.5, the stop line casts the card in 72-93% of its playouts, and the heuristic's top play is picked in 37-47%.

### What decides these choices

1. **Not the hand term (cause 1 is not it).** A card in hand is 0.9 and a rock 0, as the review said, but "stop"
   doesn't keep the card: the stop line casts the same card before the horizon in 88-97% of main-1 playouts (engines
   and mana) and 66-81% of main-2 playouts. Both lines end with it on the battlefield, so the hand term differs by
   -0.2 to +0.6, not -0.9.
2. **The horizon (cause 2) shows as a small gap, not a sign error.** What casting now buys over casting a phase or a
   turn later is one cycle of output, and only in the playouts where the stop line never casts it. The systematic
   gap (open eval, top - stop) is +0.1 to +2.5 points, largest in main 2 (+1.0 to +2.6), where the stop line goes
   without the card in 19-34% of its playouts.
3. **The hold bonus (cause 3) is the largest systematic term, and it is credited to the wrong line.** It adds
   +2.9 to +3.4 to "stop (hold mana)" in every playout that doesn't win. But in main 1 the stop line keeps the held
   card's mana up only 4-6 points more often than the cast (64-70% against 60-66%): "stop" in main 1 goes to combat
   and lets the heuristic play main 2, where it taps out too. In the hold decisions the stop line was picked 51-62% of
   the time and the top play 13-33%.
4. **The rest is noise.** One decision's top - stop score difference has a spread of 12-18 points across decisions,
   from a win or a loss in one of six playouts, against the 0.1-2.5 points of signal. Without a held counter the top
   play is picked 33-51% of the time, about one in the number of options tried (3.4). A second measurement (Part 2)
   puts the standard error of the difference at 6-10 points within a decision.

So the look-ahead chose "stop" because the hold bonus went to the label rather than to the line that kept the mana
up, and because, without a held card, its choice among near-equal lines is a coin flip. Tymna, Rhystic Study and the
rocks lost twice a turn (main 1 and main 2), and each loss looked harmless in the playouts, because the heuristic
cast the card later in the same playout.

### Five decisions traced by hand (`part1/traces/`, `--calibrate within --pool tN --trace N`)

1. **Kraum/Tymna, game 11, round 3, main 1: Tymna (heuristic 95%) → stop (hold mana).** Open eval: Tymna -2.01,
   stop -2.36 (Tymna ahead by 0.35). Both lines kept the counter's mana up in 6 of 6 playouts, and the stop line
   cast Tymna in main 2 in 4 of 6. Stop got +4.53 (the hold value) and won, 2.17 against -2.01. Round 4, the same
   with Mox Diamond and Tymna (both lines held 6 of 6, stop +4.53).
2. **Kraum/Tymna, game 64, round 4, main 1: Sol Ring (52%) → stop (hold mana).** Open eval: Sol Ring -26.21, stop
   -25.22. Both lines cast Sol Ring (6 of 6). Stop kept the mana up in 6 playouts and Sol Ring in 1, a fair +5.00 for
   stop. In main 2 the look-ahead then picked Gleaming Splendor (-21.55) over Sol Ring (-37.93), whose one lost playout
   (-100) cost it 12 points: a single playout's game result decided it.
3. **Kinnan, game 8, round 3, main 1: Rhystic Study (top) → stop.** No hold. Open eval Rhystic 6.11, stop 8.93: the
   stop line cast Rhystic in main 2 in 5 of 6 playouts and drew the same cards. The opponents' term was 2.5 better in
   the stop playouts, which is noise. Main 2: Rhystic 5.28 against stop -0.83, cast.
4. **Kinnan, game 10, round 2, main 1: Mystic Remora (100%) → stop (hold mana).** Remora -2.08, stop -0.03 (cast in
   main 2 in 5 of 6) plus a hold of 0.75. Main 2: Remora 4.96 against -1.37, cast. The delay cost nothing here. It
   costs a turn when main 2 also goes the wrong way, as in 1.
5. **Marwyn (Tier 3), game 3, round 1, main 1: Llanowar Elves (98%) → stop.** Elves 7.82, stop 8.08, identical board
   (7.92): the stop line cast the Elves in main 2 in 6 of 6. A 0.26-point coin flip. Main 2: Elves 8.30 against 3.17,
   cast.

## Part 2: the evaluation, fixed

Three changes to `ai/search.rs`, for every deck. Each answers one measured cause; the output model is the brief's.

### 1. The board valued for what it will produce (`output_value`)

The unit is `strength`'s own: a card in hand is `HAND_W` = 0.9. A permanent that makes cards or mana every turn is
worth what it will make over `OUTPUT_TURNS`, the two own turns after the playout ends (weights 1 and 0.5: by the
second it may be gone or the game over):

    output_value = 0.9 × (1 + 0.5) × (cards a turn cycle + MANA_CARD × mana a turn)

- **One rate for mana:** `MANA_CARD` = 0.4 / 0.9 = 0.44 of a card. That is a Treasure's value in `strength` (one mana,
  once) over a card's. With it, a permanent making one mana a turn is worth 0.6 over the window, what `strength`
  already gives a land, so rocks, dorks and lands share one scale. Sol Ring: 1.2; Arcane Signet: 0.6.
- **Cards a cycle,** from the card itself:
  - `eng=N`: it draws N at upkeep (Sylvan Library, Phyrexian Arena);
  - else a table of the rates measured while out at the Tier 6 table (step 3): Rhystic Study 3.0, Mystic Remora
    2.7, Esper Sentinel 1.7, Kraum 1.2, Faerie Mastermind 0.7, Necropotence 4.0, The One Ring 2.0, Tymna 1.0, Thrasios
    0.3;
  - the tax engines (Rhystic, Remora, Sentinel, Kraum, Mastermind) draw off opponents' spells, so their rate scales
    with the live opponents who can cast. None in a goldfish, whose seats never cast.
- **Mana a turn:** `rock=N:...` gives N, a mana creature (`dork`) or Chrome Mox 1. A rock that doesn't untap (Mana
  Vault, Grim Monolith) or that is sacrificed for its mana (Lotus Petal, Lion's Eye Diamond) makes nothing past the
  turn.
- **No double count:** a card whose `pval` already prices its output (Rhystic Study 4, an `eng` card 2, The One Ring
  5, Consecrated Sphinx 6, Smothering Tithe 4) counts the larger of the two. Everything else adds its output to its
  `pval`.
- `pval` itself (which the table's threat assessment reads) is unchanged. Only `strength` reads the output.

### 2. The hold bonus, to the playouts that kept the mana up (`hold_credit`, `kept_up`)

The hold bonus (the heuristic's hold value, `hold_w` = 1) now goes to each open playout whose decider ended its turn
with the held card in hand and payable, whatever the option. A cast that leaves the counter's mana up earns it. A
"stop" in main 1 whose main 2 taps out doesn't. It is computed from the playout, not from the label. It isn't
switched off: in main 2, where "stop" really does keep the mana up (99-100% against 66-76%), stop still earns about
a third more of it.

### 3. The margin: depart from the heuristic only on evidence (`margin_holds`)

The coordinator asked for the noise to be measured and handled at the source. Measured at Tier 6, 1,000 games, on the
same decisions (`part2/noise-*.tsv.gz`, `EVN` lines; the second set of playouts spends no game budget, and the games
are byte-identical with it on):

| | old evaluation | the three rules |
| --- | --- | --- |
| decisions | 15,155 | 10,168 |
| standard error of top - stop within a decision (6 rollouts) | 7.7 (6.5-9.7 by group) | 7.6 (5.9-10.0) |
| mean |top - stop| | 8.0 | 7.4 |
| decisions where |top - stop| < its standard error | 47% | 53% |
| variance of the difference cut by the shared seeds (common random numbers) | 48% (39-54%) | 50% (37-54%) |
| bare argmax: pick changes on a second set of seeds | **60%** | 56% |
| margin of one standard error: pick changes on a second set of seeds | 48% | 41% |
| the heuristic's top play kept: bare argmax / margin | 30% / 59% | 39% / 69% |

- **Common random numbers are already in use:** every option of a decision plays the same six seeds
  (`playout_copy`). They cut the variance of the difference by about half, but the lines diverge after the first
  different play, so they can't do more.
- **More rollouts for close decisions** would have to bring a standard error of about 8 down to the signal's 1-2 points:
  about 100-170 rollouts, against 6. That is 15-30 times the look-ahead's cost on these decisions.
- **So the decision rule changes:** the look-ahead's pick stands only when it beats the heuristic's top play by more
  than two standard errors of the paired difference (per rollout, bonuses in, each capped at a win). Otherwise the
  heuristic's top play is made. This is a tie rule for noise, not a bonus: it applies to every option and every
  deck, and the look-ahead still overrides on evidence. The existing sure-win cap already sent exact ties to the
  heuristic. This extends the same idea to ties within the noise. The attack search's `HOLD_MARGIN` (casual speed)
  is a fixed-point version of it.
- **Why two:** about a 5% chance per alternative that noise alone lets the pick stand. One standard error was tried
  first and measured in full at Tier 6 (`full-eval-z1/`, 4,000 games): it kept the top play in only 69% of the
  diagnosed decisions, and Tymna was out by turn 4 in 53% against A+B's 63%. At 1,000 games (`part3-z/`), z = 1.5 gave
  60% and z = 2 61% (A+B 61%). Both moved Tier 3 less than z = 1 (poison wins 17.7% / 17.0% against 20.1%, base 15.9%).

### Each rule's share (Tier 6, 1,000 games, A and B off; `part2/`)

| run | Tymna out by T4 | Tymna cast by T4 | Kraum/Tymna seen by T4 / T6 | Rhystic cast by T4 (of seen, Kraum/Tymna) | Kraum/Tymna share | top play kept (diagnosed decisions) |
| --- | --- | --- | --- | --- | --- | --- |
| base (old evaluation) | 38% | 47% | 12.7 / 17.5 | 7 of 12 | 25.1% | 30% |
| output model only | 39% | 49% | 12.8 / 18.0 | 7 of 12 | 29.7% | 31% |
| hold rule only | 43% | 56% | 13.0 / 18.1 | 8 of 12 | 29.2% | 38% |
| output + hold | 45% | 57% | 13.1 / 18.5 | 9 of 13 | 32.0% | 39% |
| output + hold + margin, z = 1 | 56% | 67% | 13.3 / 18.9 | 8 of 14 | 30.3% | 69% |
| output + hold + margin, z = 2, A and B and the must rules removed (`part3-z/`) | 61% | 72% | 13.2 / 18.6 | 9 of 14 | 29.8% | - |
| A+B (step 3 final binary) | 61% | 73% | 13.0 / 18.4 | 9 of 12 | 27.1% | - |

The output model alone barely moves the flow, as Part 1 predicted: both lines end with the card cast, so valuing it
more lifts both. Its effect is in the playouts where one line lacks the permanent, mostly main 2. The hold rule moves
the held decisions to the same coin flip as the rest. The margin turns the coin flip into the heuristic's choice.

Tests (`tests/ai_eval.rs`): the look-ahead casts Rhystic Study and Sol Ring with four lands and nothing to hold
(A and B removed, `search::choose` directly); the output model's values; `kept_up`; the margin test.

### 4. The playout noise, cut at its source (`raw/eval/part4-noise/`)

JD, after the margin: close calls now fall back to the hand-tuned heuristic, and the look-ahead should decide them
itself. So the fallback was measured on every main-phase look-ahead decision (env `AI_EVAL_FALLBACK=1`, `EVF`
lines; a second set of playouts per decision that spends no game budget, games byte-identical with it on). Then two
ways to make the playouts less noisy were tested:
- **deterministic playouts** (`AI_EVAL_DET`): inside a playout the heuristic makes its top play (temperature near 0
  in `ai::temp`, which every sampled choice reads). `determinize` still re-deals the opponents' hands and shuffles the
  decider's library, and the game's own random effects (coin flips, random discards) stay random;
- **a shorter horizon** (`AI_EVAL_SHORT`): the playout ends with the current turn cycle, before the decider's next
  turn (its end-of-turn window still runs), scored by `strength` with the output values.

Tier 6 / Tier 3, 1,000 games each. "Handed back": the margin returned the decision to the heuristic. "Flips": the pick
changes on a second set of seeds. Time: the run without the diagnostic, 1,000 games on 16 threads.

| variant | handed back | look-ahead departs from the top play | SE, top vs best alternative | bare pick flips | final pick flips | time (s per 1,000 games) |
| --- | --- | --- | --- | --- | --- | --- |
| current (z = 2) | 51.4% / 46.3% | 7.7% / 10.2% | 7.6 / 5.7 | 54% / 49% | 12% / 14% | 157 / 95 |
| deterministic | 50.6% / 42.1% | 7.9% / 11.2% | 6.3 / 3.8 | 51% / 43% | 11% / 12% | 151 / 105 |
| short horizon | 48.3% / 41.8% | 10.1% / 13.2% | 4.9 / 3.6 | 51% / 45% | 13% / 14% | 129 / 60 |
| both | 47.0% / 36.6% | 10.9% / 15.3% | 3.8 / 2.4 | 47% / 38% | 12% / 12% | 128 / 74 |

By the kind of the heuristic's top play (current, Tier 6), the share handed back is 46-56% for every kind: engine 53%,
commander 56%, mana 53%, creature 53%, removal 55%, stop 46%, other 51%. Tier 3 is 38-52%. The fallback isn't
specific to engines. It is how close most main-phase options are at six rollouts.

The outcomes (1,000 games; Tier 3 also at 4,000 against step 3's base, which plays the old evaluation):

| variant | Tymna out / cast by T4 | Kraum/Tymna seen by T4 / T6 | Rhystic cast by T4 (Kraum/Tymna) | Tier 6 seats (PASS of 4), spread | Tier 3 at 4,000: deck shares moved (sum), largest scorecard move |
| --- | --- | --- | --- | --- | --- |
| current | 61% / 72% | 13.2 / 18.6 | 9 of 14 | 1 of 4, 11.1 | 4.6 points, 1.7 (seat 4) |
| deterministic | 59% / 73% | 13.4 / 18.6 | 9 of 14 | 4 of 4, 12.4 | 4.1 points, 1.3 (seat 4) |
| short horizon | 62% / 73% | 13.3 / 19.2 | 10 of 15 | 2 of 4, 7.8 | (1,000 games only: 13.7 points) |
| both | 62% / 74% | 13.5 / 19.2 | 9 of 15 | 2 of 4, 11.9 | 9.4 points (Atraxa +4.7), 1.9 (seat 4); poison wins +1.6 |

- The shorter horizon cuts the noise most, and the look-ahead decides more for itself, but it moves Tier 3 twice as
  far as the current look-ahead. Atraxa (infect) gains 4.7 points at 4,000 games, about 4 standard errors. A playout
  that ends before the decider's next turn sees less of what the turn-cycle-long attack plans pay. So it fails "no
  larger Tier 3 move", alone and combined.
- **Deterministic playouts are kept** (on by default, `AI_EVAL_DET=0` turns them off): the standard error falls by a
  sixth at Tier 6 and a third at Tier 3, the margin hands back 0.8 / 4.2 points fewer decisions, Tier 6 flow is the
  same and its scorecard better, Tier 3 moves no more than the current look-ahead's, and it costs the same. The margin
  stays as the tie-break for what is still inside the noise.
- What deterministic playouts can't remove is the hidden information: each of the six copies re-deals the opponents'
  hands and the decider's draws, so a win or loss in one copy still swings a decision by 15-20 points.

## Part 3: the workarounds removed, and the three states compared

### Removed

- **A and B** (eefe4e34, 40bc2a8e): `t6_table::engine_first`, its call in `brain::main`, and the knob. The engine
  list and the mana-permanent test survive only as the diagnostic's classes (`evdiag.rs`).
- **The older Tier 6 must-do mana rules,** each measured off, then all three (`part3-must/`, with the margin at z = 1,
  before it moved to 2; Tier 6 1,000 games and
  the goldfish, 2,000 games):

  | off | goldfish mean kill (scion / Kraum / Kinnan / Rog-Silas / Sisay / Dack) | Kraum/Tymna seen by T4 | Kinnan mana spent T2/T3/T4 | Tier 6 seats | spread |
  | --- | --- | --- | --- | --- | --- |
  | none | 9.8 / 8.6 / 8.3 / 9.1 / 7.2 / 9.7 | 13.3 | 2.45 / 3.75 / 5.02 | 30.4 / 26.0 / 23.1 / 20.5 | 12.3 |
  | cheap-rock rule (turns 1-4) | 9.8 / 8.6 / 8.2 / 9.0 / 7.2 / 9.7 | 13.2 | 2.52 / 3.79 / 5.14 | 29.4 / 27.9 / 21.7 / 20.9 | 9.8 |
  | `chrome_mox_now` | 9.8 / 8.6 / 8.3 / 9.1 / 7.2 / 9.7 | 13.2 | 2.44 / 3.76 / 5.05 | 29.2 / 27.3 / 24.8 / 18.7 | 11.7 |
  | `mana_first` (turns 1-3) | 9.8 / 8.6 / 8.3 / 9.1 / 7.2 / 9.7 | 13.1 | 2.48 / 3.80 / 5.16 | 29.2 / 28.8 / 22.6 / 19.4 | 9.8 |
  | all three | 9.7 / 8.6 / 8.2 / 9.0 / 7.2 / 9.7 | 13.3 | 2.47 / 3.87 / 5.04 | 29.5 / 28.4 / 22.7 / 19.4 | 10.3 |

  Nothing is lost with any of them off: goldfish means within 0.1 of a turn, the flow within 0.1, every deck share
  inside its interval (about ±3.5 points at 640 seatings). With all three off, Kraum/Tymna's share is 26.5% against
  30.3%. Its flow is the same (mana spent 2.01 / 2.72 / 3.34 against 2.02 / 2.78 / 3.36, Tymna out by turn 4 55%
  against 56%), and two 1,000-game runs differ by up to ±5 points per deck. The 4,000-game run below checks it.
  **All three were removed.** `must_option` keeps the plan's priority 90+ plays, the line-completing piece, the
  ready `combo:` line and Sisay's.

### The three states

- **base:** step 3 before A and B, the old evaluation (step 3's binary; `raw/step3/base/`, `full-base/`);
- **A+B:** step 3's final binary (`raw/step3/B/`; all-decks run fresh into `raw/eval/full-AB/`);
- **eval:** this branch at 51afa9f2: the output model, the kept-hold rule, the margin at z = 2, deterministic
  playouts, with A, B and the three must-do mana rules removed (`raw/eval/full-eval/`).

A and B touch Tier 6 players only, so for Tiers 0-5 base and A+B are the same games (checked byte for byte in step 3),
and for JD's decks the two differ only in Scion's cell (`--deck scion --pool t6` on both binaries; the A+B single-deck
run reproduces its all-decks cell, 28.6%).

#### Tier 6 flow (`--realism --pool t6 --games 4000`)

| | base | A+B | eval |
| --- | --- | --- | --- |
| Tymna out / cast by T4 (Kraum/Tymna) | 40% / 48% | 63% / 75% | 59% / 72% |
| Rhystic Study cast by T4, of games that saw it: Kraum/Tymna, Kinnan, Rog/Silas | 8/14, 7/11, 8/11 | 10/14, 8/11, 7/11 | 11/15, 7/11, 7/12 |
| cards seen by T4 / T6: Kraum/Tymna | 12.8 / 17.8 | 13.1 / 18.7 | 13.3 / 18.9 |
| cards seen by T4: Kinnan, Rog/Silas, Rog/Thrasios, Sisay, Dack | 11.8, 12.4, 11.9, 11.6, 11.2 | 11.9, 12.2, 11.9, 11.6, 11.2 | 11.9, 12.4, 12.0, 11.7, 11.2 |
| mana spent T2 / T3 / T4: Kinnan | 2.47 / 3.50 / 4.42 | 2.43 / 3.69 / 4.84 | 2.61 / 4.01 / 5.26 |
| mana spent T2 / T3 / T4: Kraum/Tymna | 1.88 / 2.30 / 2.90 | 1.86 / 2.55 / 3.13 | 2.02 / 2.71 / 3.23 |
| mana spent T4: Dack, Rog/Silas, Rog/Thrasios, Sisay | 3.30, 2.38, 3.51, 4.44 | 3.45, 2.56, 3.50, 4.55 | 3.93, 2.87, 4.34, 4.74 |

#### Tier 6 scorecard (4,000 games)

| | base | A+B | eval | target |
| --- | --- | --- | --- | --- |
| seats 1 / 2 / 3 / 4 | 29.4 / 26.6 / 22.5 / 21.5 | 29.7 / 26.3 / 21.9 / 22.1 | 29.1 / 26.4 / 22.8 / 21.7 | 31.8 / 27.3 / 22.1 / 18.8 |
| median round | 8 | 7 | 7 | 7 (6-8) |
| spread | 11.3 | 11.6 | 10.4 | 4 (0-8) |
| targets passing | 3 of 6 | 3 of 6 | 3 of 6 | |
| decks: Dack / Kinnan / Kraum / Rog-Silas / Rog-Thrasios / Sisay | 23.1 / 24.9 / 26.3 / 22.2 / 21.2 / 32.5 | 24.2 / 24.9 / 29.6 / 20.3 / 19.8 / 31.4 | 26.7 / 21.0 / 29.6 / 20.6 / 21.2 / 31.0 | |

#### Goldfish (`--goldfish t6,scion --games 2000 --seed 500000`): median / mean kill turn, kills by T3, no kill

| deck | base | A+B | eval |
| --- | --- | --- | --- |
| scion | 9 / 9.6, 5.2%, 1.6% | 9 / 9.8, 5.0%, 1.8% | 10 / 9.8, 4.7%, 1.7% |
| Kraum/Tymna | 8 / 8.8, 3.9%, 1.4% | 8 / 8.8, 2.9%, 0.7% | 8 / 8.8, 3.2%, 0.7% |
| Kinnan | 8 / 8.4, 2.5%, 0.8% | 8 / 8.4, 2.4%, 1.0% | 8 / 8.4, 2.5%, 1.1% |
| Rograkh/Silas | 9 / 9.1, 6.8%, 11.8% | 9 / 9.1, 5.8%, 12.7% | 9 / 9.1, 6.3%, 13.3% |
| Rograkh/Thrasios | 9 / 9.1, 0.1%, 0.3% | 9 / 9.2, 0.1%, 0.4% | 9 / 9.3, 0.0%, 0.2% |
| Sisay | 7 / 7.3, 0.1%, 0.2% | 7 / 7.4, 0.1%, 0.1% | 7 / 7.3, 0.3%, 0.1% |
| Dack | 10 / 9.6, 9.7%, 9.6% | 10 / 9.5, 9.7%, 9.2% | 10 / 9.7, 9.2%, 9.3% |

Goldfish flow (cards seen by T4 / T6; mana spent T4; engine out at T4): Kraum/Tymna 12.3 / 16.3, 2.41 (base); 12.4 /
16.7, 2.72, Tymna 74% (A+B); 12.6 / 16.8, 2.96, Tymna 73% (eval). Scion 11.8 / 14.8, 2.65 (base); 11.7 / 14.7, 2.65
(A+B); 11.8 / 14.7, 3.03 (eval). Kinnan's mana spent on T4: 4.58 / 5.05 / 6.08. Scion's median moved 9 → 10 with the
mean unchanged at 9.8 (a plateau: its kills split evenly across turns 9 and 10). In a goldfish the tax engines draw
nothing, and A+B and eval both cast them.

#### Realism, Tiers 0-5 (4,000 games; base = A+B → eval)

Targets passing: t0 4 → 4 of 9, t1 6 → 4, t2 2 → 1, t3 4 → 2, t4 3 → 1, t5 3 → 1: 22 → 13 of 46. Every row that flips
PASS/FAIL or moves more than 1.5 points:

| tier | statistic | base = A+B | eval | target (band) | flag |
| --- | --- | --- | --- | --- | --- |
| t1 | seat 1 share | 31.3% | 29.2% | 31.3% (29.8-32.8) | PASS → FAIL, -2.1 |
| t1 | seat 4 share | 20.1% | 20.8% | 18.9% (17.4-20.4) | PASS → FAIL, +0.7 |
| t1 | wins by damage | 66.2% | 64.0% | 70.4% (68.9-71.9) | -2.2 (FAIL both) |
| t1 | wins by commander damage | 31.4% | 33.9% | 18.1% (16.6-19.6) | +2.5 (FAIL both) |
| t2 | seat 3 share | 23.8% | 24.2% | 22.3% (20.8-23.8) | PASS → FAIL, +0.4 |
| t3 | seat 1 share | 31.3% | 30.4% | 32.3% (30.8-33.8) | PASS → FAIL, -0.9 |
| t3 | seat 4 share | 19.0% | 20.3% | 18.0% (16.5-19.5) | PASS → FAIL, +1.3 |
| t4 | seat 1 share | 32.2% | 30.1% | 32.8% (31.3-34.3) | PASS → FAIL, -2.1 |
| t4 | seat 3 share | 22.3% | 23.4% | 21.6% (20.1-23.1) | PASS → FAIL, +1.1 |
| t4 | wins by a combo (reference) | 31.9% | 30.3% | 17.9% | -1.6 |
| t5 | seat 2 share | 26.4% | 25.2% | 27.8% (26.3-29.3) | PASS → FAIL, -1.2 |
| t5 | seat 3 share | 23.1% | 25.0% | 22.4% (20.9-23.9) | PASS → FAIL, +1.9 |
| t0 | median round | 12 | 11 | 9 (8-10) | -1 (FAIL both) |

The deck shares that moved 2 points or more: t1 Tatyova 14.2 → 19.1 (+4.9), Teysa 18.7 → 14.7 (-4.0), Light-Paws
+2.2; t2 Windgrace 21.5 → 27.1 (+5.6); t4 Krenko 26.5 → 30.6 (+4.1), Edgar 49.5 → 46.6 (-2.9); t5 Kinnan 36.1 → 31.8
(-4.3). Their causes are in "Why things moved".

#### JD's decks (`--all-decks --pool all --games 1000`; base = A+B except Scion)

| deck | A+B (= base) | eval | moved outside A+B's interval? |
| --- | --- | --- | --- |
| seph t3 / t4 | 25.9 / 18.3 | 24.6 / 17.4 | no |
| veyran t1 / t2 / t3 | 15.3 / 13.1 / 9.9 | 13.4 / 10.7 / 11.6 | **t2: -2.4 (11.1-15.3)** |
| sauron t3 / t4 | 23.7 / 20.7 | 25.5 / 22.2 | no |
| galadriel t1 / t2 | 6.3 / 6.5 | 6.5 / 5.4 | no |
| yshtola t5 | 22.0 | 24.0 | no |
| alela t1 | 5.2 | 5.0 | no |
| kefka t3 | 18.8 | 18.8 | no |
| sheoldred t3 / t4 | 19.6 / 19.3 | 18.4 / 19.0 | no |
| cloud t1 / t2 / t3 | 18.9 / 11.3 / 16.0 | 20.2 / 14.1 / 18.6 | **t2: +2.8 (9.5-13.4); t3: +2.6 (13.9-18.4)** |
| thrasios t5 | 11.9 | 12.2 | no |
| sephfs t5 | 18.2 | 20.2 | no |
| karn t4 / t5 | 17.2 / 20.9 | 18.8 / 22.3 | no |
| scion t6 | 28.6 (base 28.7) | 29.2 | no |

Averages over each deck's tiers, A+B → eval: seph 22.1 → 21.0, veyran 12.8 → 11.9, sauron 22.2 → 23.9, galadriel 6.4 →
6.0, yshtola 22.0 → 24.0, alela 5.2 → 5.0, kefka 18.8 → 18.8, sheoldred 19.4 → 18.7, cloud 15.4 → 17.6, thrasios
11.9 → 12.2, sephfs 18.2 → 20.2, karn 19.0 → 20.6, scion 28.6 → 29.2. Both all-decks runs skipped 13 games to
internal simulator errors (integer overflows, one `unwrap` on `None`). Step 1's all-decks runs show the same 13-14,
so these errors predate this work.


#### Why things moved (`raw/eval/attrib/`: 1,000 games each, one rule switched off at a time)

| deck (tier) | A+B | eval | output off | hold rule off | margin off | deterministic off |
| --- | --- | --- | --- | --- | --- | --- |
| Tatyova (t1) | 15.7 | 19.2 | 19.5 | 19.7 | **15.6** | 21.4 |
| Teysa (t1) | 19.7 | 14.3 | 15.1 | 13.8 | 15.8 | 14.7 |
| Windgrace (t2) | 19.6 | 25.8 | 24.3 | 26.3 | **21.4** | 29.3 |
| Meren (t2) | 20.9 | 16.2 | 15.8 | 15.9 | 18.2 | 13.5 |
| Krenko (t4) | 25.2 | 29.2 | 30.4 | 29.3 | **24.2** | 28.8 |
| Edgar (t4) | 50.7 | 45.9 | 44.8 | 46.5 | **50.7** | 46.3 |
| Kinnan (t5) | 34.3 | 31.2 | 30.7 | 33.1 | **34.0** | 31.3 |
| cloud (t2), JD | 11.3 | 14.1 | 14.0 | 14.2 | **11.2** | 12.1 |
| cloud (t3), JD | 16.0 | 18.6 | 19.3 | 18.5 | **15.8** | 19.5 |
| veyran (t2), JD | 13.1 | 10.7 | 11.1 | 11.8 | **14.1** | 12.3 |

**The margin causes nearly every large move.** With it off, Tatyova, Krenko, Edgar, Kinnan (t5), cloud and veyran
return to their A+B values, and Windgrace and Meren most of the way. The output model, the hold rule and deterministic
playouts each move these decks by a point or two, inside the noise of 1,000 games. Teysa is the exception: no single
rule brings it back. Its loss comes from the rules together.

What the margin does is hand a close call back to the deck's heuristic. So a deck whose hand-tuned priorities play
better than the old look-ahead's noisy argmax gains: Tatyova's land and ramp order, Windgrace's lands, Krenko's goblin
order, and cloud's equipment, where `cloud::search_bonus` had to correct the look-ahead before. A deck the old
look-ahead's overrides were helping loses: Edgar, Kinnan, Veyran, Meren, and Kinnan again at Tier 6 (-3.9). Those
decks' heuristics are now what decide their close calls. Their priorities were tuned when the look-ahead overrode them
about 60% of the time. This is the "calibrated against the old evaluation" movement the brief expected. It wasn't
retuned. The traces the brief asks for in these cases weren't run (the coordinator stopped the sims). The attribution
above takes their place, and per-deck traces are listed for JD.

## Decisions (made without JD; the conservative call, with the alternative)

1. **Part 1 recorded every decision whose heuristic top was an engine, commander or mana permanent, not only the ones
   the look-ahead changed.** The changed ones alone are biased: the pick is by construction the luckiest option. The
   top-minus-stop comparison over all of them is not. The alternative (only the changed decisions, as the brief
   wrote) is in `part1/summary.txt` too.
2. **The output model is added to `strength`, not to `pval`.** `pval` also drives the table's threat assessment and
   removal targets, and changing it would move every tier's interaction. The alternative was to change `pval` itself.
   Cards whose `pval` already prices their output count the larger of the two, not both.
3. **The tax engines' rates scale with the opponents who can cast** (none in a goldfish). The alternative, a flat
   table rate, would have valued Rhystic Study at 4 cards against seats that never cast.
4. **The hold bonus was recomputed, not removed:** it goes to each playout that kept the held card's mana up through
   the turn. The alternative, `SEARCH_HOLD_W=0`, was ruled out by the brief.
5. **A third change beyond the brief's model: the margin.** Part 1 showed the bare argmax picks among near-equal lines
   at random, and the coordinator asked for the noise to be handled at the source. The alternatives measured were more
   rollouts (15-30 times the cost, rejected) and common random numbers (already in place). The margin is at
   **z = 2**: z = 1 left Tymna 10 points below A+B; 1.5 and 2 matched it; 2 is the conventional evidence level.
6. **Deterministic playouts kept, the shorter horizon rejected** (JD's follow-up). The short horizon cut the noise most
   but moved Tier 3 twice as far (Atraxa +4.7 at 4,000 games). Both stay as switches (`AI_EVAL_DET=0`,
   `AI_EVAL_SHORT=1`) for A/B runs.
7. **All three older must-do mana rules removed** (cheap rock, Chrome Mox, mana first): each off lost nothing, and all
   off lost nothing in the flow. Kraum/Tymna's 1,000-game share dipped 3.8 with all three off; at 4,000 games, eval's
   Kraum/Tymna is 29.6%, equal to A+B. The alternative was to keep them as a belt and braces, which is the patch layer
   the brief asked to remove.
8. **"base" and "A+B" reuse step 3's outputs** for the Tier 0-6 realism and the goldfish (the same seeds, and A and
   B are Tier 6 only). A+B's all-decks was run fresh; base's differs only in Scion's cell, run on its own. The
   alternative, rerunning everything three times, would have cost about 5 more hours for byte-identical results.
9. **No retuning.** Tiers 0-5 lost 9 passing targets (22 → 13 of 46), mostly seat shares 0.4-2.1 points across their
   bands. Per the brief, they are reported and explained, not refit.
10. **The earlier full-set runs were stopped and deleted** when JD asked for the noise work first (z = 1's Tier 6 and
    goldfish are kept in `full-eval-z1/`; the half-run all-decks was deleted).
11. **The diagnostics stay in the code** (`AI_EVAL_DIAG`, `AI_EVAL_FALLBACK`; off by default, no play change),
    so the next step can measure the look-ahead the same way. The alternative was to delete them after the report.
12. **The large moves are explained by attribution, not traces.** One rule was switched off at a time (1,000 games),
    and the coordinator then stopped the sims. Per-deck traces of the margin's hand-backs (Teysa, Edgar, Kinnan,
    Veyran) are left for JD to ask for.

## Decisions for JD

1. **Accept eval as the look-ahead for every tier?** At Tier 6 it matches A+B on the scorecard and on the engines'
   cards, and is 3-4 points behind on Tymna by turn 4. Below Tier 6 it costs 9 passing targets, because close calls
   now go to each deck's hand-tuned heuristic. Options: (a) accept and recalibrate Tiers 0-5 later; (b) keep the
   margin for Tier 6 and JD's decks only (the old bare argmax elsewhere), which brings back most of the Tier 0-5
   calibration but breaks "every deck"; (c) revert to A+B. The conservative call taken here is (a), not retuned.
2. **The noise that's left is hidden information:** every playout re-deals the opponents' hands, and the margin still
   hands back about half of all main-phase decisions. The next lever is more rollouts only for the decisions the
   margin hands back (about 2-3 times the look-ahead's cost), or a playout that keeps the known cards. Worth testing?
3. **Traces for the moved decks** (Teysa t1, Edgar t4, Kinnan t5/t6, Veyran t2): ask for them if the attribution
   isn't enough.
4. **The all-decks runs skip 13 games to integer overflows and an `unwrap` on `None`.** This predates this work (step 1
   had 13-14). It needs its own bug hunt.
