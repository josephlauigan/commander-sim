# Tier 6 structure: why it barely beats Tier 5, and why its seat 1 edge is too small (Phase A, 2026-11-01)

Brief: `rust/agents/t6-structure.md`. Phase A only: code and existing outputs read, no sims. Branch `t6-structure`
(origin/main 216b460a). Outputs cited, all in the `overnight-stack` worktree unless marked:

- `audit/recal/stage2-calibrate-base.txt`: `--calibrate all --games 1500 --seed 500000`, look-ahead, loose. It holds
  the Tier 6 report (per seat, per deck) and the ordering (`T6 into T5`).
- `audit/recal/stage2-calibrate-knobs/t6.txt`, `t5.txt`: the same with the fitted knobs.
- `audit/realism/fit/check-2026-11-01/tN-builtin.txt`: `--realism --pool tN --games 2000 --seed 900000`, look-ahead.
- This worktree: `audit/realism/t6-seat-edge-2026-10-24.md` (the per-game ready-round measurement),
  `audit/realism/t6-lists-2026-10-24/REPORT.md`, `audit/realism/knobs/README.md`, `audit/phase2/spellbook-2026-10-29.md`,
  `audit/realism/t6-rograkh-thrasios-2026-10-26.md`.

## Summary

The two problems share one cause. **In the sim, Tier 6 decks do not reach a winning line much sooner or more
reliably than Tier 4-5 decks.** Above Tier 3 the sim's speed ladder is flat: the median win round is 8 at t4, 8 at t5 and 7
at t6, and the average game is 8.3, 8.5 and 8.0 rounds. Within Tier 6, only 21-46% of a deck's games ever reach a ready
line, and the win round's interquartile range is 5 rounds. Two things follow from that:

1. **Seat edge.** Going first only decides games when several players reach a line in the same round. A toy race model
   (below) gives the sim's own seat shares, 27 / 26 / 24 / 23, from the sim's ready share and spread. It gives the real
   31.8 / 27.3 / 22.1 / 18.8 only when about 90% of players get a line ready inside a spread of about one round. That
   holds at a median of 7 too. So the median target is met while the consistency that makes the seat edge is missing.
   The engine's turn structure is not the cause: the checks are in H5.
2. **Tier 6 into Tier 5.** A lone Tier 6 deck is no faster than the t5 table. It also meets more board answers there
   (ten sweepers across the five t5 lists against three across the six t6 lists). Two AI rules work against it as
   well: the t6 table rules don't treat a t5 player's win step as a win attempt, and the generic t5 AI counters
   almost any t6 spell. The decks whose lines are creature boards lose most (Rograkh/Thrasios 12.9%, Sisay 19.2%).
   The two Thassa's Oracle decks are above even share (Kraum/Tymna 30.8%, Rograkh/Silas 27.3%).

Ranked: H1 (consistency and speed) explains both problems; H2 (interaction asymmetries at a mixed table) explains most
of the gap into t5; H3 (passing while one tutor away) is a large part of H1's spread; H4 (removal on the developed
player) is a small damper; H5 (turn structure) and H6 (modelling gaps) are ruled out or point the other way.

## H1. Tier 6 is neither faster nor more consistent than the tiers below (rank 1, both problems)

**Speed ladder (look-ahead, loose, 11/01 build):**

| tier | median win round (realism, 2000 games) | win round IQR | average game (calibrate, 1500) | seat 1 |
|---|---|---|---|---|
| t3 | 10 | - | 10.1 | 30.1% |
| t4 | 8 | - | 8.3 | 28.6% |
| t5 | 8 | 4 | 8.5 | 29.4% |
| t6 | 7 | 5 | 8.0 | 27.5-28.5% (calibrate within: 29.0%) |

Tier 6 is one round faster than t5 at the median and has the widest spread. Its seat 1 share is lower than t3's,
t4's or t5's.

**Per Tier 6 deck (calibrate within, 1500 games):** median win round, ready share and median ready round, and the
ordering result into t5:

| deck | median win round | ready (share of games, median round) | one tutor away | into t5 |
|---|---|---|---|---|
| Dack Fayden | 5 | 22%, round 4 | 22% | 23.1% |
| Rograkh/Silas | 6 | 33%, round 6 | 38% | 27.3% |
| Kraum/Tymna | 7 | 46%, round 6 | 53% | 30.8% |
| Kinnan | 7 | 22%, round 7 | 48% | 23.2% |
| Sisay | 8 | 31%, round 7 | 35% | 19.2% |
| Rograkh/Thrasios | 9 | 21%, round 8 | 31% | 12.9% |

The two slowest decks are the two worst into t5: Sisay at the t5 median, Rograkh/Thrasios a round behind it. Dack and
Kinnan don't fit speed alone; H2 covers them.

**The seat edge needs consistency, not a lower median.** The 10/24 per-game measurement
(`t6-seat-edge-2026-10-24.md`, older build, median 8) found:
- the first player to get a line ready wins 76% of the time from every seat;
- 72% of player-games never have a ready line;
- 1.22 players get one in the games where anyone does;
- ready rounds average 7.8 with a standard deviation of 2.8.

Toy race (`scratchpad/t6-structure/race.awk`, awk, 200,000 games each). Each player gets a line ready with
probability `pr`, at a round drawn from N(mu, sd). From then on it wins on each of its own turns with probability `s`,
and the first success in turn order wins:

| pr | mu | sd | s | seats 1-4 | median, IQR |
|---|---|---|---|---|---|
| 0.30 | 7 | 2.8 | 0.76 | 26.5 / 25.2 / 24.6 / 23.8 | 6, 3 |
| 0.45 | 7 | 2.8 | 0.76 | 27.1 / 25.7 / 24.2 / 23.0 | 6, 4 |
| 0.90 | 7 | 2.8 | 0.76 | 29.1 / 26.3 / 23.4 / 21.2 | 5, 3 |
| 0.45 | 7 | 1.0 | 0.76 | 30.3 / 26.3 / 23.1 / 20.3 | 7, 1 |
| 0.90 | 7 | 1.0 | 0.35 | **32.7 / 26.9 / 22.1 / 18.4** | 7, 2 |
| 0.90 | 6 | 1.0 | 0.50 | 34.4 / 27.0 / 21.4 / 17.2 | 6, 1 |

The sim-like rows reproduce the sim's flat seats. The real seats (31.8 / 27.3 / 22.1 / 18.8) need most of the table
ready inside a tight window, and the attempts are then often stopped (s about 0.35-0.5). The success rate barely
matters when the spread is wide. This is why no style knob moved seat 1 (`audit/realism/knobs/README.md`, cthresh
0-20 and instant_extra 0-10 included): the knobs change `s` and the median, not `pr` or `sd`. It is also why
`--fit` meets the median target and still misses the seats.

**The targets don't tell the tiers apart.** `data/realism/t6.toml` and `t5.toml` both set a median win round of 7.
The t6 value is JD's estimate, backed by casual Playgroup logs; `decklists/pool/TARGETS-t6.md` says "no public dataset
gives the turn cEDH games end on". The t5 value is Playgroup's average for the five t5 commanders, built at any power.
Fitting both tiers to 7 drives them to the same speed. `win_round_spread` has no target in either tier, and that is
the statistic the seat edge depends on.

## H2. The interaction at a t6-vs-t5 table works against the lone t6 deck (rank 2, Tier 6 into Tier 5)

**The lists** (tags in `data/cards.json`, counted over `data/decks.json`):

| | counters per list (that can hit a creature spell) | sweepers per list | spot removal per list |
|---|---|---|---|
| t5 (5 lists) | Kinnan 9 (4), Urza 9 (4), Zur 7 (3), Miirym 4 (2), Yawgmoth 0 | Zur 4, Miirym 2, Yawgmoth 2, Kinnan 1, Urza 1 = **10** (Cyclonic Rift x4, Toxic Deluge x2, Damnation, Supreme Verdict, Blasphemous Act, Winds of Rath) | about 3.4 |
| t6 (6 lists) | 9-11 each (5-7), Dack 1 | Dack 2, Sisay 1 = **3** | about 3.8 (Dack 8) |

A lone t6 deck faces fewer counterspells at a t5 table than at its own, but about four times the sweepers. Every t6
creature line sits on a board: Oboro + Gaea's Cradle, Witness + Snap + Cradle, Derevi + Emiel + Cradle, Ioreth +
Marvin, Kinnan + rocks. Yawgmoth's -1/-1 counters also kill Rograkh (0/1), the 1/1 dorks and Gene Pollinator. That
fits the deck pattern into t5:
- the instant-speed Thassa's Oracle decks are above 25% (Kraum/Tymna 30.8%, Rograkh/Silas 27.3%);
- the board-combo decks are below it (Rograkh/Thrasios 12.9%, Sisay 19.2%, Kinnan 23.2%).

Dack's within-t6 strength comes from stax that taxes cEDH's cheap spells. It wins 77% of its games by the first
flip, and its line is ready in only 22% of games. Against t5's higher curves its stax probably matters less (to
measure, S1).

**Code asymmetry 1: a t5 win step is not a win attempt to the t6 table rules.** `g.t6_going` is set only in
`combos::attempt` for a Tier 6 player (`impls/combos.rs` 473-488). The t6 counter rule "a win step is always
countered, free counters included" (`ai/t6_table.rs::counter_verdict`, line 117) therefore never fires on a t5
player's combo steps. They are judged as ordinary spells instead:
- a permanent piece by `piece_gain`;
- otherwise by `Roll(val, thr + 0.5 .. 1.5)`;
- or `Pass` while someone else is the archenemy and `val < 7`.

`archenemy_score` (t6_table.rs 72-92) reads combo-line progress, open mana, tutors and Ad Nauseam. It has no board
power or damage term, so a Miirym dragon board or a Yawgmoth drain board never becomes the archenemy. t5 wins are 31%
damage in the sim (t5 realism check), and Miirym's are 86% damage on Playgroup.

**Code asymmetry 2: the t5 decks counter almost anything a t6 deck casts.**
- A non-t6 player's counter decision goes to the look-ahead (`ai::search_counter`, `engine/stack.rs` 793-818): for
  the active player's main-phase spell the rollouts see the t6 line win and counter it.
- Elsewhere it is `wants_counter(val, thr, nc)`, with `thr = cthresh_default - min(0.3 x counters in list, 2.5)`
  (`ai/mod.rs` 374). Loose sets cthresh_default to 6, so it is 3.5 for t5 Kinnan and Urza (9 counters each) and 3.9 for
  Zur. At value 4 that is a 78% counter chance; at value 3 it is 50%.
- At a t6 table the t6 decks hold counters for the archenemy (`Pass` / `held`). A lone t6 deck at a t5 table has its
  tutors, rocks and engines countered by three opponents who don't hold back.
- This doesn't show at t6 tables, where every deck plays the table rules.

**What is not established without a sim:** the stop rate of t6 attempts at t5 tables, who stops them, how the t6
deck dies, and the t5 winners' speed against it. The ordering prints only win rates (S1 measures the rest).

## H3. The decks pass while one tutor away, and Kinnan waits without cover (rank 3; part of H1's spread)

From the calibrate-within report (1500 games), "while one tutor away (not ready) it chose" and "tutors held when it
passed":

| deck | passes while one tutor away | tutors held when it passed |
|---|---|---|
| Rograkh/Thrasios | stop (hold mana) 153, stop 124 | Finale of Devastation 159, Green Sun's Zenith 132 |
| Kraum/Tymna | 103 + 65 | Mystical Tutor 94, Diabolic Intent 50, Enlightened 25 |
| Rograkh/Silas | 74 + 41 | Mystical Tutor 83, Diabolic Intent 26, Demonic Tutor 18 |
| Kinnan | 46 + more | Nature's Rhythm 32, Invasion of Ikoria 27, Finale 24 |

Some of these passes are right. Mystical Tutor is best cast in the end-of-turn window, and Diabolic Intent needs a
creature to sacrifice. The sorcery tutors Finale and Green Sun's Zenith held 291 times by Rograkh/Thrasios are not.
The planner tutors only when the tutor makes a line ready next turn (`PLANNER_PROGRESS` is off by default, measured
slower in `audit/ai-foundations/step2-planner.md`).

Ready lines are held in only a few cases:
- Kraum/Tymna went later in 89 of 458 ready games and never went in 54, from its `[[go_now]]` cover rules.
- The other decks went later in 0-19 games.
- No deck repeats Dack's old hand-AI hold (`audit/recal/dack-check.md`; that hand AI is deleted).

Two rules can hold more often at a t5 table, where four of the five t5 lists are blue and hold free counters:
- Kinnan's plan file has "no free counter and no counter in hand while an opponent has blue up: don't go" before
  round 6 (`data/plans/kinnan-bonder-mana-combo.toml`).
- Sisay and Rograkh/Silas have no `[[go_now]]` rule, so they use `t6_table::go_ok`. It goes only when the table's stop
  risk is at most 0.6 in rounds 1-3 and 0.75 in rounds 4-5.

The mulligan (`impls/t6_keep.rs`) keeps on mana alone and never values a tutor or line piece. It ships 36-52% of
hands, and a mulligan costs Kinnan 25 -> 13% and Kraum/Tymna 33 -> 22%. This is a smaller factor: the 10/24 check
found the sim mulligans no more than real players of these commanders.

## H4. Removal goes to the developed player and to commanders (rank 4, small damper on the seat edge)

- Per game, seat 1 takes 1.07 removal and 0.93 counters; seat 4 takes 0.78 and 0.84.
- Dack resolves 1.75 removals per seated game (Fiend Hunter and All-Fates Stalker exile, 2632 in 1002 games), mostly
  on Sisay and Kinnan.
- The commander engines (Kinnan, Sisay at threat value 8) draw the table's removal (Kinnan was lowered from 8 to 5 on
  10/24 for this).

This punishes whoever is ahead, which is usually seat 1, and widens the spread in H1. The table without Dack was as
slow and as flat (10/24), so this is secondary.

## H5. Turn structure and engine seat effects (ruled out)

- **The first player draws** on turn 1 (`engine/turn.rs` draw step), which is correct for multiplayer Commander.
- **Mulligans** are symmetric (`turn.rs::mulligan`: a free first mulligan, then bottom one and two).
- **Gemstone Caverns'** opening applies only to players who don't start (`card_models::caverns_opening`).
- **Turn order:** the round loop plays seats 0-3 in order every round (`turn.rs::run_rounds`). Responses go in turn
  order from the caster.
- **No seat term** in the archenemy, counter or go rules.
- **Combo stop rates by seat** are flat: 30 / 35 / 28 / 33%. The first-ready player converts at 76% from every seat.
- **One quirk.** `run_rounds` skips `end_of_turn_window` for every seat in round 1 (`if r > 1`). Seats 2-4 should get
  a window at the end of the turn before their first. This costs the later seats a round-1 instant window, so if
  anything it helps seat 1; it isn't the cause. Worth a one-line fix later (`r > 1 || i > 0`).
- **Tournament draws** (24% of real cEDH games) leave out long games. That explains about 1 point of the seat edge
  (`t6-seat-edge-2026-10-24.md`, item 5).

## H6. Modelling gaps (ruled out as the cause; they point the other way)

- Commander Spellbook (`audit/phase2/spellbook-2026-10-29.md`): the t6 lists have 2 uncovered 2-3 card winning lines
  (Breach + Lotus Petal + Brain Freeze, in Kraum/Tymna and Rograkh/Silas; the sim plays the LED version). The t5
  lists have **20**, all in Yawgmoth. If anything the sim's t5 is under-modelled, which would weaken t5.
- The Partial cards in t6 lists:
  - Rograkh/Thrasios: Biomancer's Familiar and Training Grounds (the Oboro cost reducers), plus Sowing Mycospawn,
    Springheart Nantuko, Vibrance.
  - Sisay: Derevi, Shang-Chi, Agatha's Soul Cauldron and three more.
  - Kinnan: Imposter Mech, Volatile Stormdrake.
  - Dack: six, including All-Fates Stalker and Bilbo's Gambit.

  None blocks a modelled line: the Oboro reducers only cut line A's mana from Cradle at 6 to 5, and Sisay's Derevi
  line runs as a catalogue combo. t5's lists have no Partial cards. They have 6-11 Approximate each.

## Rograkh/Thrasios at 12.9% into Tier 5

All of H1-H3 apply at once:
- It is the slowest t6 deck: median win round 9, ready in 21% of games at round 8, one round slower than the t5
  table's median.
- All four of its lines are creature boards on Gaea's Cradle or Oboro (A Oboro + Cradle + Talon Gates; B Witness +
  Snap + Cradle; C/D Badgermole Cub + clone). That puts them in front of t5's ten sweepers and Yawgmoth's -1/-1
  counters. Rograkh is 0/1.
- It has the fewest answers to a board: Chain of Vapor and Snap, the second of which is also its line B piece.
- It passes most while one tutor away (277 passes, with Finale and Green Sun's Zenith held 291 times).
- Its go rules start Oboro and Witness lines from round 3-4 without cover (`[[go_now]]`).
- Within t6 it is already the weakest (19.8%). At a t5 table it loses the race and the board.

## Sim plan (Phase B, for the coordinator to approve)

Costs come from the existing outputs at `--jobs 22`:
- calibrate t6 with knobs: 10,500 games in 1875 s, about 5.6 games a second;
- ordering and `--deck` cells: about 4.5 min per 1500 games;
- look-ahead realism t6: about 4 games a second, so 2000 games take about 8 min.

Every run is under the 24G cap, loose, the look-ahead AI, `--jobs 22` (fewer if another agent runs). Diagnostic patches
are scratch only: built into a scratch `CARGO_TARGET_DIR` and never committed.

| # | run | games | cost | tests | confirms if |
|---|---|---|---|---|---|
| S0 | Scratch build with one diagnostic change: print the Tier 6 report in `print_run` for any tier (drop `&& tier == "t6"`, `sim-cli/src/poolmode.rs` 373) | - | 10 min build (`-j 6`) | - | - |
| S1 | `commander-sim --deck KEY --pool t5 --games 1500 --seed 500000` for the six t6 keys (the same games as the ordering cells) | 9000 | 30 min | H1, H2, H3 | t6 decks' median win round is at or behind the losses' turn (t5 speed); creature-line decks show high stop rates by removal or sweepers and more "removal taken" than at t6; Kinnan, Sisay and Silas show more "ready, went later" than within t6 |
| S2 | `--trace` of 8 lost games each for Rograkh/Thrasios and Sisay from S1 (`--deck KEY --pool t5 --trace N`) | 16 | 2 min | H2, H3 | the losses show a sweeper or Yawgmoth on the board line, or a sorcery tutor held with the mana to cast it |
| S3 | Knobs file `[cthresh]` = 6 for the five t5 keys (their effective threshold is 3.5-4.8 today; Yawgmoth has no counters), paired on S1's seeds. No code change | 9000 | 30 min | H2 (2) | t6 into t5 rises by 2+ points overall (paired) |
| S4 | Scratch patch: `g.t6_going = Some(p)` for every player's combo attempt, not only t6, so t6 decks treat t5 win steps as win attempts. Paired on S1's seeds | 9000 | 30 min | H2 (1) | t6 into t5 rises by 2+ points and t5 combo wins against the t6 deck fall |
| S5 | Scratch diagnostic counter: each seat's first ready round (game time) and the number of players ready per game, on `--realism --pool t6 --games 4000 --seed 900000` | 4000 | 17 min | H1 (seats) | under 50% of player-games are ever ready and the ready round's standard deviation is 2.5 or more (as on 10/24). Then the race model with the measured pr, mu, sd and s reproduces the sim's seats within 1.5 points |
| S6 | `PLANNER_PROGRESS=1` (an existing switch) on S5's run and on S1's Rograkh/Thrasios cell | 4000 + 1500 | 22 min | H3 | ready share up and the win-round IQR down; seat 1 up (paired against S5) |

Total about 2 h 20 min of machine time, plus the build. Order: S0 -> S1 -> S2 (they decide whether S3 and S4 are
worth it) -> S3, S4 -> S5 -> S6.

**Decisions this may need from JD** (not taken here):
- The Tier 6 speed target: keep a median of 7, or set a `win_round_spread` target or a faster median for t6. A
  cEDH-only source for game length would settle it.
- Whether the t5 lists' cEDH-grade free counters plus ten sweepers are what "High B4" should play.
- Whether a lone cEDH deck should be the archenemy of a casual table in the AI's eyes. Real casual tables do gang up,
  but not with look-ahead counter decisions.
