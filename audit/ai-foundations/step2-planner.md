# AI foundations step 2: the line planner, built and measured (2026-10-27)

Branch `ai-foundations`. Brief: `rust/agents/ai-planner.md`. Code: `rust/crates/sim-core/src/ai/planner.rs` (new) and
the `// --- ai-planner` changes listed in `rust/agents/handoff-ai-planner.md`. Tests:
`rust/crates/sim-core/tests/ai_planner.rs` (9) and the knobs test in `sim-cli`. Raw output: `raw/step2/`.

## Summary

- **The planner works as specified, and it changes little.** Scion's goldfish median stays at 9 (mean 9.7 → 9.6).
  Kills by turn 3 rise from 3.6% to 5.2%, by turn 4 from 9.3% to 11.3%. Dack is the deck it helps most: median
  11 → 10, by turn 3 6.9% → 9.7%. Kraum/Tymna's mean falls 9.0 → 8.8. Kinnan, Sisay and Rograkh/Thrasios don't
  move.
  A fresh seed range (600000) gives the same picture: scion by turn 3 3.5% → 5.2%, Kraum/Tymna median 9 → 8.
- **The "3 turns behind the best case" gap was mostly in the measurement, not in the play.** The crude best case of
  `loss-accounting.md` counted a tutor's card twice, and counted tutors that can't find the piece:
  - a card put on top by a tutor was counted as "seen" when drawn, and the tutor also stood in for a missing piece
    (fixed: ab58377d);
  - Enlightened Tutor and Mystical Tutor stood in for Thassa's Oracle, and Intuition, Gamble and Ranger-Captain for
    anything. The new third variant counts only tutors that can legally find the missing piece, one each, a to-top
    tutor a turn late.

  With legal tutors, scion has a line and its mana in 45% of games, at a median own turn of 8. The kill comes the same
  turn in 66% of those games (59% before the planner), and trails by a mean of 1.1 turns (1.2 before).
- **What is left is mostly colours and draws, which no bound here sees.** Of 5 traced games where the legal best
  case said "line by turn T" and the deck didn't kill by T+2, none had a winning sequence this turn or next. Four
  lacked the colours (Thassa's Oracle needs two blue sources and the deck had one), one lost its Oracle to Gamble's
  random discard. The fixable part, fetch lands that ignored the line's colours, is now in the planner's land drop.
- **The planner fires late in the game.** Over 2,000 scion games it ran 54,725 times in main phases. It found a
  line this turn 1,960 times (78% of games; almost always the kill turn), a next-turn play 1,088 times, and nothing
  in 92% of calls. The node cap was hit 56 times, the depth cap never, and a found plan's first play failed twice.
  Before a line is within one tutor, the decisions are the utility AI's, as in step 1.
- **Tier 6 table, 4,000 games: unchanged within noise.** The median win round stays at 8, combo wins stay at
  90.9% (91.0%), and each deck is within 2 points. The brief expected faster games and more combo wins; the planner's
  gains are too small to show at the table. Scion against Tier 6: 29.3% → 29.6%.
- **Gate check: identical.** `--realism --pool t3 --games 1000` and `--deck cloud --pool t1 --games 500` give the
  same output with the planner off and on (byte for byte, apart from the elapsed-time line).
- **A bug found and fixed on the way.** The first version ignored the deck's plan slot (`t6_plans::prio` 0 = keep
  in hand). It cast Dack Fayden into open counters, and Dack won 41.0% of the Tier 6 table (from 22.1%). Fixed in
  ddf05cd5, with a test.
- **For JD:** the planner as briefed is cheap and does no harm, but it isn't the big fix the audit hoped for. The
  turns the audit attributed to the AI are mostly the decks' draws and colours. See "Decisions for JD".

## What was built

### The search (`ai/planner.rs`)

- **Goals:** the catalogued lines, by the test `combo_options` makes before it offers a line: ready, affordable
  (rituals in hand counted) and, at Tier 6, cleared by `go_ok` (the table and the plan slot's `go_now`)
  (`combos::plan_lines`).
- **Plays expanded:**
  - mana: a ritual that nets mana, a rock, Mox Diamond, Chrome Mox, Lotus Petal, a mana creature; lands, untapped
    rocks and dorks are paid from as any cast pays, and Spirit Guides count as mana in hand;
  - a land from hand while the land drop is still to come;
  - a tutor for a chosen card: a missing piece of a line missing at most as many piece groups as the tutors in
    hand, or the mana a complete line lacks (`combos::plan_mana_targets`, step 1's B fix), at most 6 cards a tutor;
  - a permanent piece of a line (Kinnan, Basalt Monolith, Underworld Breach, Lion's Eye Diamond, a commander piece in
    the command zone). Thassa's Oracle is left to its line.
- **Search:** breadth-first, on game copies (`search::clone`), at most **6 plays deep and 120 copies a call**. Two
  orders of the same plays that reach the same position count once (a fingerprint of the player's side).
  Opponents never act in a copy (`Game::plan_for` gates `take_priority`). What they may do is `go_ok`'s job, as in the
  real game, and their hidden hands are never consulted. Cards drawn in a copy are not used (only the hand at the
  start, and what the plan's tutors found).
- **This turn** (`plan_this_turn`): the shortest plan that reaches a line. Ties go to the most protection kept in hand,
  then the most cards left.
- **Next turn** (`plan_next_turn`): each position the search reached is carried to the next turn (`next_turn`):
  - mana empties, and the player's lands and permanents untap (not Mana Vault's kind);
  - summoning sickness ends;
  - the card it knows is on top is drawn (none under Necropotence, and no unknown draw is counted);
  - a land is played;
  - free mana artifacts in hand are cast.

  The plays are ranked by a line ready next turn, then protection kept, then cards left, then the fewest plays.
  Nothing is chosen when passing already leaves a line ready.
- **The forced tutor target** (`Player::plan_tutor`): `decks::tutor_pick`, the ability language's search
  (`dsl_search_pick`) and fetch lands (`turn::crack_fetch`) take the planner's card only when it meets their own
  restriction. Otherwise they pick as usual. A to-top tutor goes through `put_known_top`, so step 1's known-top record
  is kept.
- **Tutors modelled** (`tutor_rule`):
  - to hand: Demonic Tutor, Diabolic Intent, Beseech the Mirror, Eladamri's Call;
  - to the top: Vampiric Tutor, Imperial Seal, Mystical Tutor (an instant or sorcery), Enlightened Tutor (an
    artifact or enchantment, from its ability-language filter), Worldly Tutor (a creature);
  - not modelled: X tutors (Chord, Finale, Green Sun's Zenith: X is all the spare mana), pile tutors (Intuition,
    Gifts), Gamble (a random discard), creatures that tutor as they enter (Ranger-Captain), land searches.
- **The plan slot is obeyed:** a card whose `t6_plans::prio` is 0 or less is never the planner's play.
- **The land drop** (`choose_land`): at the start of the turn, each land in hand, and each land a fetch land can
  find, is played on a copy and scored:
  1. a line reached this turn (the shorter plan first);
  2. else a line ready next turn;
  3. then the coloured pips of the closest line the lands can make (`line_colours`).

  The usual choice stands unless another scores higher.

### Integration (`ai/brain.rs`, `engine/turn.rs`)

- **`brain::main`, before `t6_table::must_option`:**
  - a this-turn plan's first play is made, and the plan is made again at the next decision (a receding horizon);
  - a next-turn plan's first play is made as a must-do;
  - a next-turn plan that starts with an instant tutor keeps that tutor's mana up instead. A cast that would leave
    it unpayable is dropped from the options, and any other play is checked on a copy. The end-of-turn window then
    casts the tutor.
  - with no plan, the step 1 AI runs unchanged.
- **`brain::end_of_turn_window`:** the planner's instant tutor (or the instant ritual that pays for it) for a line
  ready next turn.
- **`turn::play_land`:** the planner's land drop.
- **Traces:** each planned play is logged as `[<name> decides: <play> [plan: a -> b -> combo: <line>; wins this
  turn]]`, `[... keeps mana for <tutor> [plan: ...]]`, `[... end of turn: <tutor> [plan: ...]]`, or `[... plays <land>
  for <land> [plan: the land drop the plan wants]]`.
- **Gating:** `planner::applies` requires a Tier 6 key (`t6_plans::T6_KEYS`), the planner on (`Settings::planner`:
  env `PLANNER=0` or a knobs file's `[ai] planner = false` turns it off), the real game (not a look-ahead playout,
  not a planner copy), and not a person's seat.
- **Cost:** none to speak of. The goldfish took 185 s with the planner against 188 s without, and the Tier 6 table
  11.8 min against 12.3 min: a planned play skips the look-ahead's 36 playouts.

## The goldfish

`--goldfish t6,scion --games 2000 --seed 500000 --jobs 16`, planner off (= step 1: the play is identical to
`raw/step1/I/goldfish.txt`; only the best-case lines differ) and on. Files: `raw/step2/goldfish-off.txt`,
`raw/step2/goldfish-on.txt`.

| deck | median off → on | mean | by T2 | by T3 | by T4 | no kill |
| --- | --- | --- | --- | --- | --- | --- |
| scion | 9 → 9 | 9.7 → 9.6 | 0.4% → 0.7% | 3.6% → 5.2% | 9.3% → 11.3% | 1.4% → 1.6% |
| Dack | 11 → 10 | 9.9 → 9.6 | 0.8% → 1.2% | 6.9% → 9.7% | 18.6% → 21.7% | 10.3% → 9.6% |
| Kinnan | 8 → 8 | 8.5 → 8.4 | 0.1% → 0.1% | 2.3% → 2.5% | 9.8% → 10.4% | 0.8% → 0.8% |
| Kraum/Tymna | 8 → 8 | 9.0 → 8.8 | 0.3% → 0.7% | 2.5% → 3.9% | 8.2% → 10.5% | 1.6% → 1.4% |
| Rograkh/Silas | 9 → 9 | 9.2 → 9.1 | 1.1% → 1.2% | 5.5% → 6.8% | 13.1% → 14.6% | 13.2% → 11.8% |
| Rograkh/Thrasios | 9 → 9 | 9.1 → 9.1 | 0.0% → 0.0% | 0.1% → 0.1% | 1.4% → 1.4% | 0.3% → 0.3% |
| Sisay | 7 → 7 | 7.4 → 7.3 | 0.0% → 0.0% | 0.1% → 0.1% | 2.1% → 2.2% | 0.2% → 0.2% |

The by-T4 column is read from each deck's "won by turn" line.

Fresh seeds (`--seed 600000`, `raw/step2/seed600000/`), off → on: scion by T3 3.5% → 5.2%, mean 9.8 → 9.7;
Kraum/Tymna median 9 → 8, by T3 1.9% → 3.5%; Dack by T3 8.9% → 11.4%. The direction holds; the size is about a
point or two of games by turn 3.

### Scion's loss accounting (share of own turns)

| bucket | off | on |
| --- | --- | --- |
| won | 10.0% | 10.1% |
| ready_not_taken | 0.0% | 0.0% |
| passed_castable | 20.6% | 20.8% |
| no_land_drop | 1.2% | 1.2% |
| mana_short | 10.4% | 9.7% |
| progressed | 48.9% | 49.5% |
| idle | 8.8% | 8.8% |

`mana_short` falls by 0.7 points, from the land drop. The other buckets don't move: the planner acts on the turns
where a line is within reach, and those turns were already `won` or `progressed`.

### The best cases (scion)

| variant | games | median own turn | kill − best: median, mean, share at 0 (off → on) |
| --- | --- | --- | --- |
| no tutors | 4.3% | 7 | 0, 0.93 → 0.68, 72% → 73% |
| each tutor one piece (`loss-accounting.md`'s, double count fixed) | 74% | 7 | 2, 3.09 → 3.02, 26% → 29% |
| **legal tutors only, one each** (new) | 45% | 8 | 0, 1.23 → 1.09, 59% → 66% |

Before the double-count fix, the tutor variant said 84% of games at a median of turn 6, with the kill 2-3 turns
behind; that was the "about 6" of the brief. Of that gap, the double count and the tutors that can't find the piece
account for most. Against the legal bound the AI is on time in two thirds of games, and the planner moved that from
59% to 66%. Kraum/Tymna: 49% → 55% at 0; Rograkh/Silas: 57% → 59%.

**Still crude:** the legal bound counts mana sources, not colours, and ignores summoning sickness. That is why it can
say "turn T" for a hand with one blue source and Thassa's Oracle (see the gap traces).

## Counters (instrumented, 2,000 goldfish games each; `plan|...` in the goldfish report)

| deck | main-phase calls | found this turn (played) | found next turn: played / mana kept for EOT tutor | nothing found | passing already ready | node cap | depth cap | EOT calls / EOT plays | land choices / changed |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| scion | 54,725 | 1,960 (1,958) | 465 / 623 | 50,590 | 1,087 | 56 | 0 | 17,818 / 243 | 6,473 / 1,398 |
| Kraum/Tymna | 49,704 | 2,739 (2,739) | 567 / 761 | 43,820 | 1,817 | 218 | 3 | 16,070 / 258 | 6,782 / 64 |
| Rograkh/Silas | 57,648 | 2,504 (2,503) | 585 / 804 | 51,542 | 2,213 | 199 | 0 | 19,098 / 226 | 6,114 / 58 |
| Kinnan | 56,165 | 2,415 (2,415) | 25 / 0 | 53,519 | 206 | 10 | 0 | 15,063 / 0 | 6,347 / 11 |
| Sisay | 49,154 | 2,193 (2,193) | 437 / 198 | 42,389 | 3,937 | 6 | 0 | 12,699 / 31 | 5,728 / 5 |
| Dack | 77,672 | 1,888 (1,888) | 3 / 0 | 75,764 | 17 | 1 | 0 | 19,516 / 0 | 5,357 / 34 |

The land columns for scion are from the final run; the rest are from the instrumented run before the colour
tie-break, in `raw/step2/variants/goldfish-counters-before-bound-fix.txt`. With the colour tie-break, "changed"
rises to 24-55% of games, depending on the deck.

**Overrides.** A found plan is never overridden by `must_option` or the look-ahead: the planner acts before them. What
keeps a plan from being found is counted instead:

- **The plan slot keeps a card in hand** (`plan|slot_holds`): at 62% of Kraum/Tymna's main-phase calls (30,609),
  67% of Rograkh/Thrasios's (42,695) and 12% of Sisay's (5,758). Their plan files set 0 for cards they want held.
  Scion has no plan priorities, so 0.
- **`go_ok` says wait at a ready line** (`plan|go_ok_waits`): 0 in the goldfish, as expected against inert seats.
- **`hold_up`:** the planner doesn't read it. A plan was played while the plan slot's `hold_up` asked to pass with
  the mana up 1,878 times for Kinnan and 3,529 for Kraum/Tymna (`plan|over_hold_up`). Most are this-turn plans:
  Kinnan found 2,415 of those and 25 next-turn plans. On a kill turn the `combo:` option already came before
  `must_option` in step 1.
- **A found plan's first play failed** at most twice in 2,000 games for any deck: the copy and the real game agree.

**Caps.** The node cap (120) was hit at 0.1% of scion's calls and 0.4% of Kraum/Tymna's. The depth cap (6) was hit 3
times in 14,000 games. Neither limits the search.

## Why the planner gains little: 5 traced gap games

`raw/step2/gap-traces/`: 200 scion games traced (`TRACE_HAND=1`). 100 had a legal best case; in 24 of them the deck
didn't kill by T+2. These traces were made after the double-count fix, before the colour tie-break. For each game,
the hand and the lands at turn T and T+1 were checked by hand for a winning sequence.

| game | legal best → kill | turn T by hand | why the planner missed it |
| --- | --- | --- | --- |
| 162 | T3 → T9 | Oracle + Consultation in hand on turn 4, lands Badlands, Tropical Island, Shifting Woodland, Simian Spirit Guide | **Not affordable:** one blue source for Oracle's UU. Scalding Tarn had found Badlands (Volcanic Island or Underground Sea was there). The colour tie-break would pick Underground Sea, but with these draws a second blue source still doesn't come until turn 9. |
| 195 | T5 → T18 | Oracle + Consultation on turn 3, lands Bayou, Tundra, Mana Vault | **Not affordable:** UU + B from one blue land and a colourless rock. Verdant Catacombs found Bayou, Polluted Delta found Tundra. Two lands for 13 turns. |
| 148 | T7 → T18 | Diabolic Intent from turn 3 with nothing to sacrifice until turn 7; Oracle found on turn 8 | **Not castable, then not affordable:** Diabolic Intent needs a creature (`needsac`), and the lands (Scrubland, Ancient Tomb, Bayou) make no blue. |
| 74 | T4 → no kill | Brain Freeze, Rite of Flame, Vampiric Tutor, Tainted Pact; two lands and Noble Hierarch | **Not affordable next turn:** Vampiric for Oracle leaves Oracle + Pact at UU + 1B against Sea, Volcanic Island and Hierarch. The planner computed it right and made no plan. Two lands for 18 turns. |
| 189 | T4 → no kill | Gamble on turn 1 tutored Thassa's Oracle and discarded it | **Out of reach:** Gamble (the utility AI's play; the planner doesn't model it) lost the piece to its random discard. The bound doesn't know the piece is gone. |

Also seen in the first round of traces, before the double-count fix: games 59, 62 and 124 had "line by turn 2-3",
and each was an Imperial Seal whose card was counted twice.

**Causes, largest first:**

1. Colours and land count the bound doesn't see (4 of 5).
2. The bound's own errors: the double count (fixed), and illegal tutors (the new variant).
3. Random discards and pieces gone.

None was depth, nodes, an unexpanded play kind, `go_ok` or an override.

**Fix made: the land drop reads the line's colours** (1d41a79f). It was the one cause the planner could act on.
Measured: scion by T3 +0.5 points over the planner without it, `mana_short` −0.9 points, Dack by T3 +2.4 points.
A second tie-break, preferring lands that make more of the line's colours, measured mixed (Kraum/Tymna's mean
+0.2) and was dropped (`raw/step2/variants/goldfish-colour-flex.txt`).

**The other fix tried: the "progress" fallback** (`PLANNER_PROGRESS=1`, off by default). When nothing leaves a line
ready next turn, it casts a tutor for the piece that brings the closest line nearer. It measured slower: scion's
mean was 9.8 against 9.6 (`raw/step2/variants/goldfish-progress-on.txt`). A tutor spent early can't fetch whichever
piece is missing later, and a tutor to the top costs the draw.

## The 10 traced games for three decks

`raw/step2/traces/<deck>-<n>.txt` (`--goldfish <deck> --seed 500000 --trace n`, planner on). Each game: kill turn,
legal best case, plan lines logged.

| deck | kills (own turn) | legal best case | planner lines a game |
| --- | --- | --- | --- |
| scion | 4, 7, 8, 8, 10, 16, 17; no kill in 0, 3, 6 | 4, 13, 17 (the rest none) | 0-6 |
| Kraum/Tymna | 4, 6, 6, 7, 9, 10, 11, 12, 12; no kill in 3 | 4, 6, 10, 10 (the rest none) | 2-5 |
| Rograkh/Silas | 5, 6, 7, 7, 7, 8, 8, 9, 11, 15 | 5, 6, 7, 8 (the rest none) | 1-4 |

- **Where a legal best case exists, the kill is on it or one turn later** in 9 of 11 games. The exceptions:
  Kraum/Tymna 6 kills 2 turns after it, and scion 3 has its best case on turn 13 and no kill by the round cap.
- **The planner's plays in these traces:**
  - the kill turn's line, sometimes after a tutor (scion 5, turn 16: Demonic Tutor for Tainted Pact, then the line);
  - an end-of-turn Mystical Tutor or Vampiric Tutor for the last piece, with its mana kept up through the main phase
    (scion 7, turn 16);
  - land drops for the line's colours (scion 7, turns 1-2).
- **The long games are draw-limited.** Scion 5: Gamble found Oracle on turn 3, and the second piece came with Demonic
  Tutor on turn 16. Scion 7: Oracle found on turn 11, Tainted Pact by Mystical Tutor at the end of turn 16.
- **Kills with no legal best case** (most of the Kraum/Tymna and Rograkh/Silas games) come through cards the bound
  doesn't count: Intuition, Gamble, Ad Nauseam's and Necropotence's cards, and Breach lines that dig with Brain
  Freeze.

## The Tier 6 table

`--realism --pool t6 --games 4000` (`raw/step2/realism-t6-{off,on}.txt`; off is identical to step 1's after run):

| statistic | off | on | target |
| --- | --- | --- | --- |
| seat 1 / 2 / 3 / 4 share | 28.6 / 25.7 / 23.7 / 22.1% | 29.4 / 26.6 / 22.5 / 21.5% | 31.8 / 27.3 / 22.1 / 18.8% |
| median round of a decisive win | 8 | 8 | 7 (6-8) PASS both |
| combo or alternate wins (reference) | 91.0% | 90.9% | 47.8% |
| deck win spread | 10.7 | 11.3 | 4 (0-8) |
| games over 15 rounds | 5.6% | 5.2% | - |
| targets passing | 1 of 6 | 3 of 6 | |

The two seat rows that flipped to PASS moved by 0.9 and 1.2 points, inside the run's noise.

Deck shares, off → on:

| deck | off | on |
| --- | --- | --- |
| Dack | 22.1% | 23.1% |
| Kinnan | 24.4% | 24.9% |
| Kraum/Tymna | 28.3% | 26.3% |
| Rograkh/Silas | 22.4% | 22.2% |
| Rograkh/Thrasios | 21.2% | 21.2% |
| Sisay | 31.8% | 32.5% |

The first, buggy version (`raw/step2/v1/`) moved Dack to 41.0%. The intermediate version (`raw/step2/v2/`, before
the colour tie-break) gave a median round of 8 and deck shares within 1.5 points of these.

`--deck scion --pool t6 --games 2000`: win rate 29.3% → 29.6%. Wins come at an own turn of 7.9 → 7.9, with a median
win round of 7 both. Combo attempts are 0.37 → 0.38 a game. "One tutor away, not ready" decisions now include the
planner's tutors (Demonic Tutor for Thassa's Oracle 43, Diabolic Intent for Thassa's Oracle 40). The first version
gave 25.5%: every opponent also cast its plan-slot-held cards.

Not tuned, as the brief says: the table's calibration is step 3.

## The gate check

| run | planner off vs on |
| --- | --- |
| `--realism --pool t3 --games 1000` | identical |
| `--deck cloud --pool t1 --games 500` | identical (only the "93s total" / "94s total" line differs) |

The planner is reached only for Tier 6 keys. It plays on copies, which carry their own random-number state. The real
game's random numbers are untouched (test `no_plan_in_a_hand_with_no_line_leaves_the_old_choice`).

## Tests (`tests/ai_planner.rs`)

1. `a_turn_two_oracle_kill_through_a_ritual`: two lands, Dark Ritual, Oracle + Consultation: the plan is to go, and
   the main phase wins.
2. `a_tutor_finds_the_missing_piece_and_the_line_wins_this_turn`: five lands, Oracle, Demonic Tutor: Demonic Tutor
   for Consultation, then the line.
3. `a_tutor_fetches_mana_when_the_line_lacks_only_mana`: two lands, the drop used: Demonic Tutor fetches a land or
   Lotus Petal for next turn, not Ad Nauseam.
4. `a_to_top_tutor_at_the_end_of_turn_then_the_kill`: Vampiric Tutor for Consultation in the window, then the line.
5. `no_plan_in_a_hand_with_no_line_leaves_the_old_choice`: same log, hand and next random number as with the
   planner off.
6. `the_forced_tutor_target_obeys_the_tutors_restriction`: Enlightened Tutor is never offered Oracle, a forced Oracle
   is not taken, and a legal forced card is (and known on top).
7. `planner_off_is_the_step_one_ai`: the env parse; no plan logged with the planner off.
8. `with_a_counter_visible_and_go_ok_false_the_planner_does_not_go`: three opponents with counters and blue open.
9. `the_plan_slot_keeps_a_card_in_hand_against_the_planner`: Dack Fayden held by its plan. The test fails without
   the fix.

`cargo test --release --no-fail-fast`: 3,543 passed, 0 failed. `cargo build --release --all-targets`: no warnings.

## Decisions (made without JD; the conservative call, with the alternative)

1. **Commits.** One commit holds the this-turn planner, the next-turn planner and the integration: they share one
   search, and neither half is testable without the other. Then come the plan-slot fix, the counters with the colour
   land drop, the audit fixes, and the report. The alternative was three commits of half-working code.
2. **Budget: depth 6, 120 copies a call, 6 cards a tutor (3 of them mana).** The caps were hit in under 0.5% of calls.
   The alternative was bigger caps; nothing points at them.
3. **Opponents don't act in planner copies**, and the table's risk is read through `go_ok`. The alternative,
   determinizing their hands and letting them respond, would add noise and a second model of the table.
4. **The planner doesn't run inside look-ahead playouts**, so the look-ahead still models its own future with the step
   1 policy. The alternative, running it there too, would cost about 36 times as many searches.
5. **Gamble, Intuition, X tutors and creature tutors aren't planner plays.** Their results can't be forced, or X
   spends all the mana. The utility AI still casts them.
6. **A next-turn plan that starts with an instant tutor keeps its mana up**, and the end-of-turn window casts it. The
   alternative was to cast it in the main phase, which is the same in a goldfish and worse at a table.
7. **The plan slot wins:** a card it holds (`prio` 0) is never the planner's play. This came from the Dack bug.
   `hold_up` is not read (see Counters). The alternative was to obey `hold_up` too, which would make the planner pass
   on kill turns for Kinnan and Kraum/Tymna.
8. **The land drop is the planner's** (lands are in the brief's play list, but the engine plays the land before main
   1). The usual choice stands unless another option scores higher.
9. **The "progress" fallback is off** (env `PLANNER_PROGRESS=1` turns it on). It measured slower.
10. **The audit's best case was fixed in place** (stats only): a tutored card isn't "seen" when drawn. This changes
    `loss-accounting.md`'s best-case figures. Play is unchanged, and the old variant's lines keep their wording. The
    alternative was to leave the double count in and add a corrected line beside it.
11. **The Tier 6 table was measured, not tuned** (step 3).

## Decisions for JD

1. **Is the step 2 result enough to go on to step 3?** The planner finds and plays the lines a pilot would on the
   turn they are available. But the decks' kill turns are set by their draws and colours, so the goldfish medians
   don't move. A turn-3 median is not reachable from these lists with this engine's draws.
2. **Measure deck changes on the colour side?** In the gap games, the deck that missed its kill was usually one blue
   source short of Thassa's Oracle's UU. That is a deck-building question for scion (blue sources, fetch targets),
   not an AI one.
3. **Goldfish seats still cast nothing** (decision 1 of step 1). Rhystic Study, Mystic Remora and Esper Sentinel stay
   blanks, which hurts scion's draws in particular.
