# AI foundations: architecture review (2026-10-27)

This is a code-reading review of how the AI makes a main-phase decision, aimed at the goldfish gap: cEDH decks kill
at a median own turn of 7-10 against seats that never act, where real turbo decks kill around turns 3-4. Nothing was
run. Paths are under `rust/crates/sim-core/src/` unless noted. Turn costs are estimates from the code, not
measurements; section 9 lists the counters that would confirm them.

## Verdicts

| # | Question | Verdict |
| --- | --- | --- |
| 1 | Decision stack | **Flawed.** Four layers, each patching the one below it. `must_option` beats the look-ahead, the look-ahead only sees what the utility list ranks top 5, and a utility of `None` removes a play from every layer. |
| 2 | Distance to a win | **Flawed.** Nothing measures it. `combo_progress` is the squared share of a line's pieces held, with no mana, tutors or timing in it. A win scores 100 whenever it happens inside the horizon. |
| 3 | Horizon | **Flawed for combo.** This is a one-step rollout over a slow policy. It can value "tutor now, win next turn", but it can't value anything that pays off later, and it scores mana development at or below zero. |
| 4 | Holding | **Mostly sound at a table, wrong in a goldfish.** The hold value (2.4-3.25), the reserve penalty (1.7-2.3) and the hold bonus are all still active against seats that can't cast. They were measured as a small effect. |
| 5 | Utilities | **Patchwork.** A cast-order priority (0-100 divided by 10) from about 45 tag rules, 15 or more per-deck functions and about 220 per-card priority hooks, with hundreds of constants. It is not a value scale. |
| 6 | Mana and sequencing | **Flawed.** The AI makes one play at a time against the mana it has now. It has no turn plan, never counts a ritual in hand as mana, and never casts some rituals at all. |
| 7 | Other foundations | Mulligans: acceptable (rule-based). Land play and fetches: sound. Discard: bug (known). The 18-iteration and work caps: unknown, no counter for the first. Determinization: no leak, but it loses a card put on top. |
| 8 | Overall | **Not sound for combo decks.** The core needs a goal-directed planner for the catalogued lines, with the utility AI as the fallback. See section 8. |

## New defects found (ranked by the turns they likely cost)

1. **"Win now" and "win next turn" score the same.** `evaluate` returns 100 for any win (`ai/search.rs:209`).
   d10bbb05 caps scores at 100 and hands ties to the higher heuristic utility (`search.rs:385-392`). Example: turn 5,
   five mana, Oracle and Demonic Tutor in hand. Demonic Tutor then the line wins this turn. Casting Scion (8.5, above
   Demonic Tutor's 7.2: `ai/pool.rs:90-93`) leaves the win for next turn, which the heuristic finds in most playouts.
   Both lines score 100, the tie goes to Scion, and the kill slips a turn. In general, any higher-utility play whose
   line still wins next turn beats a multi-step win this turn. Likely cost: up to 1 turn, in the games where it happens.
2. **`must_option` spends mana before a ready line.** It runs before the look-ahead and before the combo option
   (`ai/brain.rs:1236-1247`). It never looks at the `combo:` option, except Sisay's (`ai/t6_table.rs:296`). Cheap
   rocks on turns 1-4 and dorks or three-mana rocks on turns 1-3 are forced first (`t6_table.rs:277-290`). Example:
   Oracle + Consultation ready with exactly 3 mana, and Arcane Signet (2 mana in, 1 out) or Birds of Paradise (summoning
   sick) in hand. The rock is cast, the line can no longer be paid for (`impls/combos.rs:680`), and the win waits a
   turn. A plan priority of 90 or more has the same problem for other decks: Thrasios at 93 is cast before a ready line.
   This hits exactly the fast games (turns 2-4).
3. **Some rituals are never cast, and the ones that are cast rank below "stop".** Cabal Ritual
   (`impls/t5.rs:976-978`), Rite of Flame (`impls/t6_mana.rs:583-585`) and Desperate Ritual (`t6_mana.rs:562-564`)
   have `prio = zero_prio`. `card_utility` then returns `None` because their priority is hand-written
   (`brain.rs:317-322`), so they appear only inside catalogued lines. Dark Ritual gets the DSL value, 3 × 0.5 = 1.5
   (`dsl/mod.rs:1904`, `brain.rs:321`). That is below "stop (hold mana)" whenever a counter is held (2.375). Nothing
   counts a ritual in hand as mana (section 6). Result: 2-land + ritual Oracle kills on turns 2-3 are mostly missed.
4. **The evaluation is anti-development.** A mana rock on the battlefield is worth 0 (`engine/values.rs:442-548`: no
   tag gives it value), and each card in hand is worth 0.9 (`search.rs:237`). So casting Sol Ring, a Signet, Mox
   Diamond or a ritual lowers the score unless the mana turns into board or pieces before the horizon.
   `t6_table::must_option` exists to override this (its own comments say "Sol Ring lost to stop (hold mana) at 100%"
   and "the mana pays off after its horizon"), but only on turns 1-4.
5. **Tutors are blind to mana and to the library top.**
   - `missing_pieces` skips a line that is complete in hand but can't be paid for (`combos.rs:720`). With Oracle +
     Consultation and 2 mana, Demonic Tutor fetches a piece of the next-closest line, never mana.
   - A card put on top by Imperial Seal, Vampiric or Mystical is still "missing", so a second tutor can fetch it again
     (`engine/tutors.rs:401-417`; nothing records a known top card).
   - Enlightened Tutor is tagged `tut=any`, so its priority is 72 for finding Oracle (`pool.rs:233-234`). But it
     resolves through its DSL, which searches artifact/enchantment to top (`engine/cast.rs:1024`; data/cards.json).
   Each of these wastes a tutor rather than a turn, except when the tutor was the turn's only route to the line.
6. **Known, for completeness:** cleanup discards the highest-CMC card, or the first land when it holds two or more,
   and ignores `ai::discard_keep` (`engine/turn.rs:796`).

## 1. The decision stack

`brain::main` (`ai/brain.rs:1229-1283`) loops up to 18 times. Each pass:

1. **List.** `main_options` (`brain.rs:1129-1220`) builds the option list.
   - For each card in hand or the command zone, it computes `card_utility` (`brain.rs:283-371`). Its base is
     `decks::deck_prio_f` (`ai/decks.rs:79`). That calls `t6_plans::prio` (the deck slot), else `deck_prio`, else
     `pool::generic_prio` (`ai/pool.rs:72-321`). `generic_prio` checks the plan config's key cards and prio function,
     then the card's registry prio hook, then about 45 tag rules, with the DSL value as a last resort.
   - Situational terms are then added (`brain.rs:324-370`), then the reserve penalty (`brain.rs:1164`), the ninjutsu
     reserve and the flash penalty (`brain.rs:1166-1179`).
   - Then the other options: removal, wipes, special, extra, Breach/GC, DSL abilities and hook options. The hook
     options include `combos::combo_options` at 14 - 6·risk, offered only if the line is ready and affordable now and
     `go_ok` (`combos.rs:653-693`).
   - Last comes "stop" at -3, or "stop (hold mana)" at the hold value, plus `t6_plans::hold_up` (`brain.rs:1211-1219`).
2. **Must-do (Tier 6).** `t6_table::must_option` (`t6_table.rs:261-307`) picks first, unless `hold_up` > 0:
   - a plan priority of 90 or more;
   - a card that completes a line from public pieces (`piece_gain` 1.0 or more);
   - cheap rocks on turns 1-4, Chrome Mox, and mana first on turns 1-3.
3. **Look-ahead.** `search::choose` (`ai/search.rs:296-399`):
   - the Thrasios/Kraum commander-first and sink rules;
   - the top 5 options by utility, plus stop;
   - 6 determinized rollouts each, played by the heuristic to the end of p's next turn;
   - `evaluate`, plus `ramp_bonus` (0) and `hold_bonus`, plus the Karn/Cloud bonuses;
   - the cap at 100, and the argmax (ties go to the higher utility).
4. **Heuristic.** A Gumbel order of the utilities at `temp` (1.0 for every Tier 6 deck, `ai/mod.rs:97`,
   `pool.rs:24`); the first play that works is made.

**Who overrides whom**, strongest first:

- **Utility `None`.** The play does not exist for any layer. Examples: a plan priority of 0, the zero-prio rituals,
  Oracle unless it wins alone (`combos.rs:1097`).
- **`go_ok` / `go_now`.** They gate whether the `combo:` option exists at all.
- **`must_option`.** It beats search and heuristic, and it is blind to a ready `combo:` option.
- **Search.** It picks only among the heuristic's top 5. Its ties go back to the heuristic.
- **Heuristic sampling.** It decides when search is off, capped, or its pick fails.
- **Deck code.** It is not a layer of its own. It enters as inputs: priority, wish list, keep, hold-up and go-now.

The heuristic is also the playout policy, and it runs `must_option` inside playouts too.

## 2. Distance to a win

`combo_progress` (`impls/combos.rs:808-842`) is the best line's share of piece groups held (hand, battlefield,
commander in the command zone, and lands for Tier 6), **squared**. It ignores mana, colours, tutors in hand and
sorcery timing. For a two-piece line it is 0 → 0.25 → 1.0, so with `combo_w` = 12 (`settings.rs:57`) the first piece
is worth +3 raw and the second +9. `strength` adds 0.25·life, board `pval`, 0.35·power, 0.9·hand, 0.6·lands and
0.4·treasures (`search.rs:228-241`). `evaluate` is 95·tanh(raw/100), and is 100 or -100 when the game is over
(`search.rs:207-224`). Neither function has a time term.

**Worked example (goldfish).** Turn 3, main 1, three untapped lands making U, U and B. In hand: Thassa's Oracle,
Demonic Tutor, Swan Song, and a land for next turn.

- **Heuristic.**
  - Demonic Tutor: tag `tut=any`, the wish list is Consultation/Pact, so `tutor_prio` = 72 and the utility is 7.2. The
    counter-risk factor is 1 against inert seats.
  - Oracle: not offered (`oracle_prio` 0, `combos.rs:1097`).
  - Stop: "stop (hold mana)" = max(1.0, 1.5 + 2.5 × 0.35 × 1) = 2.375 (`brain.rs:234-252`).
  - Reserve penalty on Demonic Tutor: 0, since Demonic Tutor + Swan Song costs 3, which is payable.
  - So Demonic Tutor is sampled first about 99% of the time.
- **Look-ahead, the Demonic Tutor line.** Demonic Tutor fetches Consultation, leaving 1 mana. The line costs UU + B =
  3, so it can't be paid this turn. Next turn has 4 lands, the `combo:` option at 14 is taken, and every playout wins:
  100.
- **Look-ahead, the stop line.** Main 1 → combat → main 2, where the heuristic casts Demonic Tutor anyway, and it wins
  next turn: 100. The hold bonus adds +2.375, which was 102.4 before d10bbb05 and is capped at 100 now. The tie goes to
  Demonic Tutor. The kill turn is right either way.
- **If no win fits in the horizon** (for example 2 lands and no land drop to come): Demonic Tutor now ends with
  progress 1.0 (+9 raw). Stop in main 2 lets next turn's heuristic cast Demonic Tutor first, which ends at the same
  state. The scores are equal, the hold bonus (+2.375) decides for stop, and the tutor waits a turn. That is harmless
  here, but the same arithmetic defers the **first** tutor of a two-tutor plan every turn: both lines reach +3 by the
  horizon, so only noise and the bonus decide.
- **The case that costs a turn** is defect 1: with 5 mana, a higher-utility play that still wins next turn ties
  Demonic Tutor at 100 and wins the tie.

Main 1's "stop" is not a pass. The playout goes to combat and then lets the heuristic play main 2
(`search.rs:439-446`). So main-1 comparisons are mostly between the same plays in a different order, plus noise.

## 3. The horizon

The look-ahead plays each of 6 root actions with the heuristic until the end of p's next turn (`settings.rs:54-56`,
`search.rs:162-203`). That is one-step policy improvement. It can beat its policy only on the root choice, and only
when the difference shows in `evaluate` by the horizon.

- **Tutor now, combo next turn:** visible, because the policy combos at utility 14 next turn.
- **Tutor now, tutor next turn, combo the turn after:** invisible beyond the +3 progress term, which the "later" line
  earns too.
- **Rituals and fast mana:** valued only if the extra mana is spent inside the horizon on something `evaluate` counts
  (board, pieces or a win). Otherwise a rock is -0.9 (a card from hand) + 0 (`pval`), and Mox Diamond is about -1.8 (it
  also discards a land). A ritual at 1.5 utility usually isn't even in the top 5 (`search.rs:331`), so the look-ahead
  never tries ritual → line.
- **Noise:** a tutor shuffles the library (`tutors.rs:59`), so the tutor line and the stop line draw different cards.
  Six rollouts can't separate differences of a few points, and the hold bonus (2.4-3.25) is that size.

## 4. Holding

| Source | Where | Size | Active in goldfish? |
| --- | --- | --- | --- |
| "stop" default | `brain.rs:1211` | -3.0 | yes (no held card) |
| `hold_value`/`hold_of` (paid counters, prot hi/phase/indes/blink; Ad Nauseam) | `brain.rs:210-252` | 1 + 3·caution·threat/20 + 2.5·near; outside decks max(·, 1.5 + 2.5·caution·n≤2); -2 on turns 1-2 | **yes**: Scion (caution 0.35) gets 2.375 with one counter and 3.25 with two; threat and near are 0 |
| `reserve_penalty` (a cast that can't be paid alongside the cheapest held card) | `brain.rs:254-266` | 0.7·hold_v (about 1.7-2.3); 8 or more when near and not ramp | **yes** (0.7·hold_v); the near branch is off |
| `t6_plans::hold_up` | `impls/t6_plans.rs:77`; plans | Kinnan -2 on turns 1-2, +1 with cover; Dack -2 on turns 1-3; Godo -2 early; Scion none | per deck |
| ninjutsu reserve | `brain.rs:1166-1172, 1215` | -6 on casts, stop at 4.5 or more | only decks with ninjutsu |
| flash creatures | `brain.rs:1173-1179` | -1.5 | **yes** |
| `gandalf_hold` | `brain.rs:1103-1126` | -3 | no (Gandalf only) |
| counter-risk scaling | `brain.rs:362-364` | ×(1 - 0.35·ctr_risk) | no (ctr_risk 0) |
| `stop_chance` / `go_risk` / `go_ok` | `t6_table.rs:48-66, 186-226` | gates the `combo:` option | no (0 against Wastes) |
| `hold_bonus` in search | `search.rs:275-282` | +hold_v per rollout to "stop (hold mana)" | **yes**, capped at 100 |
| 0.9 per card in hand in `strength` | `search.rs:237` | -0.9 per card cast | **yes**, implicitly |
| instant to-top tutors on own turn → `None` | `brain.rs:298-306` | wait for the end-of-turn window | yes (the right call) |
| Mox Diamond land rule | `pool.rs:117-127`, `settings.rs:121` | 0 unless lands in hand + battlefield ≥ 3 | yes |
| `tutor_prio` with a full hand | `pool.rs:60-62` | -1.0 | yes |

For comparison, typical cast utilities: Sol Ring and Mana Vault 8.8; other rocks 8.5 up to turn 5, then 3.8 less
0.35 per turn; Scion 7.5-8.5; tutors 7.2; Rhystic Study 6.6 early; engines 6.4; creatures 4.2+; draw 4.6-5.2; Dark
Ritual 1.5. The held value beats rituals and late creatures, and takes 1.7-2.3 off any play that spends the counter's
mana. With `SEARCH_HOLD_W=0` the goldfish median was unchanged (audit/scion/FINDINGS.md, 2026-10-27), so holding is a
secondary cost.

## 5. Utilities

`card_utility` = the deck's priority / 10, plus about a dozen situational terms (`brain.rs:283-371`). The priority is
a **cast order**, not a value. Rhystic Study 66, Demonic Tutor 72, Sol Ring 88 and Dark Ritual 15 are not comparable
amounts of anything. Cabal Ritual has no utility at all, and the `combo:` option is 14 on a scale where cards top out
near 9.

- **Tutors** score 72 whenever the wish list has a target, whether or not it can be cast this turn or next.
- **Rocks** score 85 by turn number, with no reference to what the mana enables.
- **The commander** outranks tutors (75-85).
- **At `temp` 1.0** these gaps (0.6-1.3) leave the best play only 50-80% likely. For example, Rhystic Study is cast
  before Demonic Tutor about 35% of the time (e^6.6 against e^7.2).

Rough counts in the main-phase path for a Tier 6 seat (string and float literals, comments excluded):

- **Numeric constants:** about 400 shape the choice. That is about 177 in `ai/brain.rs`, about 124 in `ai/pool.rs`
  (including about 44 integer priorities in `generic_prio`), about 64 in `ai/t6_table.rs`, 29 in `ai/search.rs`,
  and more in `plans.rs` and `decks.rs`.
- **Card-name literals:** about 540 across the eight Tier 6 plan files (28-116 each), and about 4,600 across
  `ai/` and `impls/`.
- **Per-card priority hooks:** about 220 `c.prio = Some(...)` registrations, and about 210 `fn *_prio` functions.

## 6. Mana use and sequencing

Each pass the AI makes one play, judged against `can_pay` for what is open now. Lands, rocks and dorks count, and so
do Spirit Guides from hand (`engine/mana.rs:239, 307`). There is no turn plan.

- **Potential mana is never counted.** A ritual in hand counts toward nothing: not the line's `affordable`
  (`combos.rs:417-421`), not `one_tutor_away` (`combos.rs:1428-1461`), not the reserve check.
- **One-shot mana gives no credit.** Mox Diamond, Lotus Petal and rituals add nothing to the play that needs them.
- **Mana-first plays can cost the win:** `must_option` casts mana before the line (defect 2).
- **"Go if the mana adds up"** exists only inside catalogued lines. `combo_options` offers a line that is ready and
  affordable right now, and Scion's Prismari/Breach lines do their own mana arithmetic, rituals included
  (`impls/t6_combos_scion.rs`).
- **Mox Diamond → Dark Ritual → Demonic Tutor → Oracle + Consultation** happens only by accident: the ritual must
  sample high, or land in the look-ahead's top 5 with the policy finishing the rest.

## 7. Other foundations

- **Mulligans.** Deterministic count rules, not probabilistic: `t6_plan_scion::keep` and `t6_keep::mana/settle`
  (`impls/t6_keep.rs:45-48, 67+`), with the free mulligan. They are sensible as heuristics, but they never ask "can
  this hand win by turn 3". Acceptable; marked provisional (D11).
- **Land play.** Automatic, before main 1, by colours and untapped (`engine/turn.rs:155, 118-223`). Mox Diamond's
  rule allows for this (`pool.rs:126`). Sound.
- **Fetchlands.** Cracked on play (`turn.rs:284, 316+`). That is fine here: the land drop comes after the draw, so
  it never shuffles away a card tutored to the top. Sound.
- **Hand-size discard.** A bug (defect 6).
- **The 18-iteration cap** (`brain.rs:1230`). Nothing counts how often it ends a phase. Lines run as one `Plan`
  action, so it should be rare. Unknown.
- **Work caps** (`settings.rs:68-71`, `search.rs:56-77`).
  - `game_search_work` (3M) or `game_decisions` (1,000) switches the look-ahead off silently for the rest of the game,
    leaving the heuristic, whose goldfish median is 11.
  - `playout_work` (2,000) cuts a playout and scores it mid-horizon.
  - `search::stats()` (capped, cut) is not in the goldfish report. Unknown.
- **Determinization** (`search.rs:100-120`).
  - No leak: opponents' hands are re-dealt from hand + library.
  - It drops public knowledge, such as cards revealed by tutors.
  - It shuffles `me`'s library, so a card put on top (Imperial Seal in main 1; Vampiric or Mystical before a
    decision) is lost in the playouts of every later decision until it is drawn. Those playouts can't see "combo next
    turn" and may value a second tutor for the same piece (see defect 5).
  - Low cost in the goldfish, because end-of-turn tutors are drawn before the next main phase.

## 8. Verdict, and what a sound core looks like

**One-play-at-a-time priority scoring with a one-turn heuristic rollout is not sound for combo decks.**

- The quantity a turbo pilot optimizes, the expected turn of the kill, appears nowhere: not in the utilities, not in
  `evaluate`, and not in the tie-breaks.
- The layers above the utilities mostly patch that gap: `must_option`, the mana-first and Chrome Mox rules, the
  hold-bonus cap, `ramp_bonus` and the per-deck priorities. The patches then collide (defects 1 and 2).
- The look-ahead can't fix a slow policy: it is one root step better than the policy, over a horizon that ends before
  mana and first tutors pay off.
- The evidence agrees: the adaptive AI (no search) has a median of 11, the look-ahead 10, and the line is ready at a
  median of turn 10 (`audit/scion/2026-10-27-win-cap/goldfish-lookahead.txt`).

**A sound core: a goal-directed line planner first, the utility AI second.**

1. **Goals.** The catalogued lines already exist: `combos::all()`, each with `groups`, `ready` and `mana_need`.
2. **A this-turn planner** (`plan_turn`). A depth-limited search (depth 6 or so) over own-turn actions, run on game
   clones (about 2 µs each, `search.rs:12`).
   - The actions are: cast or activate mana (rocks, rituals, Mox, Petal, Spirit Guides, dorks that can tap), tutor for
     a named card (the planner picks the target; this needs a forced-target hook in `tutor` / `tutor_to_top`), and
     draw spells.
   - Success: some line is `ready && affordable` and `go_ok` holds. The planner returns the action sequence. Branching
     stays small because only mana, tutors and pieces are expanded.
3. **A next-turn planner.** If nothing wins this turn, find the play this turn (including end-of-turn tutors and
   to-top tutors) that maximizes P(a line is ready and affordable next turn). Use expected untap mana plus the land
   drop, and remember the known top card. Rank by kill turn, then by cards left over.
4. **Integration.**
   - When a sequence wins this turn, it overrides everything except table safety (`go_ok`, `protect_before_go`).
   - The next-turn plan is a strong prior: its first action gets top utility, and it replaces `must_option`'s mana
     rules.
   - With no plan, the existing utility AI runs as now.
   - In `evaluate`, replace `combo_progress` with a kill-turn estimate, and score a win by when it happens (100 minus
     a few points per turn elapsed), which also fixes defect 1.
5. **Cost.** A new module of a few hundred lines, plus a tutor-target hook, a known-top-card field, and ritual and
   one-shot mana as actions.

**What it would break or change:**

- **The Tier 6 calibration.** It is already too combo-heavy (76% against 47.8%) and too slow (median round 9
  against 7). A faster combo AI shortens games but raises the combo share, so the table's interaction AI and the
  realism fit must be redone.
- **Lower tiers.** Gate it to Tier 6 and combo-tagged decks, or a strength knob, so Tiers 0-5 keep their pace.
- **Patches and tests.** `must_option`'s mana rules, `chrome_mox_now`, the hold-bonus cap and some plan priorities
  become redundant. Tests asserting specific picks will fail, and every RNG stream and baseline shifts.
- **Speed.** About one planner call per main-phase pass; bounded by depth, and cheaper than the 36 playouts the
  look-ahead runs now.

**Cheap fixes before the planner** (each measurable on its own in the goldfish):

- Make a win score depend on when it happens (defect 1).
- Make `must_option` yield to a ready, affordable line, and skip rocks that would make it unaffordable (defect 2).
- Give rituals a utility of "enables a line or tutor this turn" instead of 0 or 1.5 (defect 3).
- Value mana sources in `strength` (defect 4).
- `missing_pieces`: treat mana as the missing piece of a complete line, and exclude a known top card (defect 5).
- Use `discard_keep` at cleanup (defect 6).

## 9. Counters that would confirm the ranking

- Per kill: the first turn a this-turn win sequence existed (a dry run of `plan_turn`) against the actual kill turn.
  This is the direct measure of the gap.
- Tie-broken decisions at 100 where the chosen play's win came a turn later than an alternative's (defect 1).
- `must_option` picks made while a `combo:` option was in `opts` (defect 2).
- Ritual casts per game, by card; turns with a ritual in hand where a line was one ritual short (defect 3).
- `search::stats()` per goldfish run: `capped`, `cut`, `max_work`; and a counter for the 18-pass cap.
