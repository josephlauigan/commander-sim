# C1: Tier 6 decks and "one tutor away" (branch c1-t6-lines, 2026-11-05)

Brief: `rust/agents/c1-t6-lines.md`. The agent returned this report as text (its write of this file was refused); the
coordinator saved it. No sim was run by the agent: evidence is from `commander-sim --calibrate within --pool t6 --ai
adaptive --games 4000 --seed 900000 --trace N`, N = 1..400 (each deck sat in about 270 of those games). Game numbers
below are N. The before-files are in this folder; the coordinator's scorecard results are at the end.

## The trace line
`t6stats::hold_diag` is trace-only: no decision reads it and it draws no random numbers. When a deck passes while a line
is one tutor away it prints `[X passes (<line> lacks [<group>]; tutors: [<tutor>->Hand|Top finds n/m, ...]); mana M,
creatures C; hand: ...]` (`n/m`: how many of the m library copies of the missing group the tutor may legally take).
When a line is ready it prints `[X passes (ready: <line> (plan go_now rule i -> verdict, go_ok, risk, each opponent's stop
chance, round)); ...]`. `plan_data::go_now_rule_idx` supports it.

## What "one tutor away" measured
`combos::one_tutor_away` counted any card with a tutor tag in hand, whether or not it could take the missing piece. Pass
decisions while "one tutor away", in 400 games:

| deck | pass decisions | a tutor could take the piece | none could |
| --- | --- | --- | --- |
| Kinnan | 23 | 8 | 15 |
| Kraum/Tymna | 77 | 47 | 30 |
| Rograkh/Silas | 44 | 42 | 2 |
| Rograkh/Thrasios | 76 | 39 | 37 |

- Green Sun's Zenith, Chord, Finale, Nature's Rhythm and Invasion take creatures; the missing piece was Basalt Monolith,
  Mirage Mirror or a rock (g114, g236, g260, g75, g165, g280, g392).
- Mystical Tutor and Enlightened Tutor cannot take Thassa's Oracle (g23, g109, g116).
- Fix (measurement only): `combos::tutor_finds` and `useful_tutors`; `one_tutor_away` and the `spah|` "tutors held" count
  now count only tutors that can take a missing piece (a card the planner does not model as a tutor still counts). The
  next calibrate's figures will be lower than the 2026-11-05 ones for this reason alone.

## Causes found and fixed
### 1. A sacrifice tutor held because its only fodder is the commander (Kraum/Tymna, Rograkh/Silas)
- Evidence: g109 (Kraum/Tymna): Diabolic Intent, Demonic Consultation, Ad Nauseam in hand, Oracle in the library, Tymna the
  only creature; passed at R8, R10, R11. g117, g20, g76 (Rograkh/Silas): Diabolic Intent held with no creature out and
  Rograkh in the command zone. The 2026-11-05 report lists Diabolic Intent held 249 (Kraum/Tymna) and 55 (Rograkh/Silas).
- Cause (AI): `brain::spare_creature` never offers a commander unless its mana value is 0 and refuses a creature worth more
  than 3, so `do_cast` refused Diabolic Intent and the plan never existed; nothing cast a creature to feed it.
- Fix: a tutor the planner chose (`Player::plan_tutor`) may sacrifice any creature, the commander last
  (`brain::planned_fodder`); with no creature out and a sacrifice tutor in hand, the planner may first cast the cheapest
  creature in hand or a commander (`planner::fodder_cast`); a sacrifice tutor with nothing to sacrifice is not tried
  (`planner::tutor_extra`).
- Tests: `a_sacrifice_tutor_eats_the_commander_when_it_wins_the_turn`, `a_creature_is_cast_to_feed_a_sacrifice_tutor`.
- Expect: a small gain for Kraum/Tymna and Rograkh/Silas. In g109 the plan appears at R11; at R8-R10 the line was still
  unpayable (one blue source for the Oracle's {U}{U}: the list).

### 2. Transmute Artifact was not a planner tutor (Kinnan)
- Evidence: g165, g280, g160: Transmute Artifact in hand, Kinnan out, the line lacking Basalt Monolith (or Mirage Mirror).
- Cause (AI): `planner::tutor_rule` knew only Beseech the Mirror and Worldly Tutor among hand-written tutors; `transmute`
  chose by the wish list, not the planner's target.
- Fix: Transmute Artifact is a planner tutor for artifacts; it takes `plan_tutor` when set and, for a planned target,
  sacrifices the cheap artifact (worth under 3) with the highest mana value (`rules2::transmute_fodder`); the planner adds
  the difference as extra mana and skips it with no artifact to sacrifice.
- Test: `transmute_artifact_fetches_the_monolith_for_kinnan`. Expect: small (one copy).

### 3. Sisay's fetch ignored ability locks (Sisay, the strongest deck)
- Evidence (code): the option path is filtered by `ability_locked` in `brain::hook_options`, but the Tier 6 combo steps call
  `legends::sisay_fetch_card` and `sisay_go` directly, and `t6_combos_t6::get_piece` never asked: Cursed Totem, a Pithing
  Needle on Sisay, an Arrest never stopped the DDR and Flash Unicorn lines. Dack casts Cursed Totem 406 and Pithing Needle
  389 times in 2633 seated games; Sisay's and Rograkh/Thrasios's lists play a Totem too.
- Fix (rules): `sisay_go` returns false under `ability_locked`; `get_piece` offers no fetch while Sisay is locked; a
  locked Sisay's lines are not "ready".
- Tests: `sisay_cannot_fetch_under_cursed_totem`, control `sisay_fetches_without_a_lock`.
- Expect: Sisay lower in games with Dack, a Needle or a Totem (a rules fix). Nothing else found making its lines
  unanswerable (Snap, Red Elemental Blast, Mindbreak Trap do stop them).

### 4. Kinnan + Basalt Monolith ignored ability locks (Kinnan)
- `combos::kinnan_basalt` never asked whether Monolith's untap and Kinnan's ability can be activated (Collector Ouphe in
  the Kraum/Tymna and Rograkh/Silas lists, Cursed Totem, Needle). Now not ready while either is locked.
- Test: `kinnan_and_monolith_are_stopped_by_collector_ouphe`. Expect: Kinnan a little lower (correct rules, though it
  widens the spread).

### 5. Stops reported "by no card" when a counter hit (report only)
- Evidence: g40 (Green Sun's Zenith countered by Mental Misstep), g239, g260 (Snap countered by Force of Negation), g156
  (Snap countered by An Offer You Can't Refuse): Rograkh/Thrasios's "no card" stops at steps 1-2 were counters.
- Cause: `cast_card` set `g.last_counter = by` with `by` None although the stack window had named the card. Fix:
  `g.last_counter = by.or(g.last_counter)`. No game changes. A step stopped by Vexing Bauble, Chalice or Void Mirror as the
  spell is cast still names no card (they are permanents).

## Looked at, not changed (for JD)
- **Kraum/Tymna waiting while ready**: in 272 games it held a ready Oracle line in 24 (8.8%), 71 pass decisions: 59 under
  go-now rule 0 (Esper Sentinel out with an opponent holding a card and open mana, or Kraum out with an opponent who can
  cast a second spell), 12 under rule 2 (an opponent can stop it or has blue up, no cover). Waits: 1 round (g148, g318),
  3 rounds (g156, lost to another deck's combo), until death at R8 (g297). Rule 0 is a real risk (after Consultation /
  Pact the library is empty and a forced draw kills the pilot), but the sim's opponents never cast a spell just to
  trigger that draw, so the rule waits for a danger that exists only against answers, which `go_risk` already prices.
  Options: (a) keep; (b) drop only the `opp-second-spell` clause (Kraum); (c) drop it when the pilot holds cover (a free
  counter or Silence). A plan-file edit in `data/plans/kraum-tymna-blue-breach-midrange.toml`; can be A/B'd without code.
- Rograkh/Silas waiting while ready: 4 decisions in 400 games, all from the shared `go_ok` (risk 0.73-0.83 against a
  threshold of 0.6-0.75 before round 6): a deliberate shared rule.
- **Not found (open)**: Rograkh/Thrasios holds Finale of Devastation + Snap with 8-11 mana when Eternal Witness is the
  missing piece (seed 900000, `--trace 139`, R6-R7); the planner builds no Finale-for-Witness plan there. Next: a planner
  trace of the Finale step. The biggest open item for that deck.
- **Not examined (out of budget)**: Lion's Eye Diamond and Moxen held at the end of turns 1-3; Whir of Invention is not a
  planner tutor (Kinnan's second Monolith tutor).
- The list, not the engine (JD decides lists): Kraum/Tymna g109 (one blue source for {U}{U}); Kinnan's creature tutors
  reach only the Hullbreaker line's Horror or Thrasios, not Monolith, Mirage Mirror or rocks (Monolith comes from Transmute
  Artifact, Whir and draws); Rograkh/Thrasios's Finale and GSZ are held as finishers and can't take the non-creature pieces
  (Talon Gates of Madara is a land).
- Dack and Sisay: one pass decision each with a line ready or one tutor away in 400 games.

## Files changed
- `sim-core/src/t6stats.rs`: `hold_diag` (trace), `spah|` counts useful tutors.
- `sim-core/src/ai/plan_data.rs`: `go_now_rule_idx` (trace helper).
- `sim-core/src/ai/brain.rs`: `planned_fodder`.
- `sim-core/src/ai/planner.rs`: Transmute Artifact in `tutor_rule`, `tutor_extra`, `fodder_cast`.
- `sim-core/src/impls/rules2.rs`: `transmute_fodder`, planned target in `transmute`.
- `sim-core/src/impls/legends.rs`, `t6_combos_t6.rs`, `combos.rs`: ability locks; `tutor_finds`, `useful_tutors`,
  `one_tutor_away`.
- `sim-core/src/engine/cast.rs`: `last_counter` kept.
- `sim-core/tests/c1_t6_lines.rs`: six tests (five fail on the old code, one control).
