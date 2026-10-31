# Dack Fayden (Tier 6): why the hand AI off wins 14 points more (2026-10-31)

Brief: `rust/agents/recal-dack-check.md`. Finding checked: `pairs-2026-10-31.md` section 4, Dack's share of decisive
games 38.4% with the hand AI off (generic policy + `data/plans/dack-fayden-white-stax.toml`), 24.5% with
`AI_HAND=dack-fayden-white-stax`, -13.9 [-17.4, -10.4].

All runs: branch `overnight-stack` (built from 13b76a04 before the fix, then from f35d1978 with it), `--pool t6 --games 2000 --seed
500000`, loose, `--jobs 22`, under the 24G cap, rc=0 on every one. Raw output:
`/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/dack-check/runs/`.

## Verdict: (c) the hand AI is a handicap, and (a) the generic policy plays the deck's real plan

The generic policy casts Dack as soon as it can pay for him, and that is the deck's plan (SPEC.md: "resolve Dack as
early as turn 2"; a resolved Dack wins the same turn in most piles). The hand AI holds Dack until the table can't stop
him: `t6_plan_dack::prio` returns 0 for Dack unless every opponent is Silenced, or `table_risk` (1 - the product of
each opponent's `stop_chance`) times 0.8 per Grace/Reprieve/Gambit held is at or below `dack_bar`. The bar is 0.2 in
rounds 1-2 (0.1 with all seven creatures in the library), 0.3, 0.4, 0.5 in rounds 3-5, and 0.8 from round 6. Against
a Tier 6 table of three blue decks with free counters, the estimated risk stays above that bar until round 6, so the
hand AI waits. Waiting doesn't help: the table counters Dack about as often either way, and every turn waited gives the
table a turn to win.

No rules or card-model bug explains the gap (b): the flip line, its table and the opponents' answers are the same code
in both arms, and a flip that goes off wins the same share of the time in both. One model bug turned up while checking
the flip line (below). It inflated both arms; it is fixed.

### Evidence (pre-fix binary, `--calibrate within` T6 report, same 2000 games)

| | hand AI off (plan) | hand AI on | hand AI on, Dack's cast left to the generic policy (diagnostic) |
|---|---|---|---|
| Dack win share (seated) | **38.4%** (516 of 1345) | **24.0%** (323) | **35.2%** (473) |
| wins by the first flip | 457 | 239 | 409 |
| went for the flip (games) | 484 (36% of games) | 253 (19%) | 446 (33%) |
| flip stopped once it went | 6% | 6% | 8% |
| median round: line ready / won | 4 / 5 | 6 / 7 | 5 / 5 |
| wins by damage | 46 | 69 | 47 |
| counters resolved on Dack's spells (all opponents) | 1542 | 1703 | - |
| mulligans | 53.0% | 40.9% | 40.9% |

- The extra wins are all flip wins (457 against 239). Once the flip goes off it converts the same way in both arms
  (94% won, 6% stopped), so the line isn't stronger off: Dack just resolves twice as often, two rounds earlier.
- Diagnostic arm: the hand AI with one change, Dack's own cast priority returns `None` so the generic policy decides
  (a scratch build with the env var `DIAG_DACK_GENERIC`; it was never committed). Everything else stays hand AI: keep
  rule, wish list, the other cards' priorities, hold-up. Dack goes from 24.0% to 35.2%, so **about 11 of the 14 points
  are the Dack-cast hold alone**. The other ~3 points (35.2 against 38.4, inside the noise: each arm's CI is +-2.5) are
  the rest of the slots. The biggest of those is the mulligan rule: the plan file ships 53% of hands against the hand
  AI's 41%, because it leaves out the "two mana and three cheap plays" keep.
- Same seed, same hand, game 0 (`--trace 0`; seats Sisay, Dack, Rograkh/Silas, Kinnan): Dack makes the same plays in
  rounds 1-3 in both arms (Paladin Class, Fellwar Stone, Voltaic Key, Thorn of Amethyst, Get Lost). On round 4 with 6
  mana, the generic policy casts Dack, flips Abdel + Preston + Werefox (pile 19) and wins. The hand AI holds Dack (the
  risk is over the round 4 bar of 0.4), attacks, and casts Razorgrass Ambush (countered). It waits until round 8,
  when Sisay's Pact of Negation counters Dack, and Sisay wins on round 11. Game 10: the generic arm (a mulligan to 5)
  casts Dack on round 3 off Cavern of Souls, City of Traitors, Sol Ring, Chrome Mox, Prismatic Lens and Grim
  Monolith, and wins. The hand AI arm (kept 7) doesn't cast Dack until round 7, gets no win from that flip, and loses.
  Over the 12 traced games (9 with Dack seated), the generic arm wins 3 and the hand AI arm wins 2 (one of them by the
  round cap).
- The cards the generic policy leans on, checked against `data/scryfall_cache.json` and the trace: Dack Fayden,
  Helping Hand (X = opponents, onto the battlefield, shuffle, goad, one to each opponent: `flip_and_donate` matches);
  Preston, Felidar Guardian, Radiant Solar, Fiend Hunter, All-Fates Stalker, Werefox Bodyguard and Abdel Adrian (the
  seven: the traces' rules use match the Oracle text; the deck has exactly these seven creatures plus Dack). The
  game 10 mana was legal (Cavern naming Human makes WW for Dack and makes him uncounterable). Nothing free or illegal
  turned up.
- The known gaps in the plan file (Chalice's `chalice_net`, Cursed Totem, Defense Grid, Bauble and Void Mirror, the
  wipes; `audit/phase4/hand-off-handoff.md`) are on the hand AI's side, and the hand AI still loses. Its stax count
  is higher (2182 against 1664) only because its games run longer.

## The bug fixed on the way: a first flip with a creature missing from the library (f35d1978)

All 35 pile traces (`audit/t6/dack-fayden-white-stax/flips-*.md`) assume the four creatures not in the pile are still
in the library, where the second flip takes them. `t6_dack::record_first_flip` recorded every three-of-the-seven flip.
When the library didn't hold the other four (one drawn, cast, in the graveyard or exiled), it recorded "no card left",
and `pile_wins` still won the turn with it: piles 1-5, 16, 17, 19, 20, 22 and 24 ("win whatever the second flip is"),
and 23 and 26-35 with a flicker spell. Example: pile 19 (Preston + Abdel + Werefox) wins by looping the Felidar card
through Preston with a Solar token. With Felidar in the graveyard there is no loop, but the shortcut still called the
turn won. The fix: record the flip only in the traced case (the other four all in the library). Otherwise the engine
has already played the flip, the game goes on, and the board combos (`Preston + Felidar + Radiant Solar ...`) still see
the real board. This is conservative: a missing creature that is already on Dack's side of the battlefield might
still allow a win, but no trace covers that case.

Test: `engine_t6_dacktest::a_pile_is_not_a_win_when_a_piece_its_trace_flips_next_is_not_in_the_library`. Pile 19 wins
with the full library and doesn't with Felidar or Solar in the graveyard (seeds 0-3). It failed before the fix (the
shortcut won) and passes after. Full `cargo test --release`: 3807 passed, 0 failed, no warnings.

## Re-run on this branch after the fix (same seeds)

`--realism --pool t6`, 2000 games, seeds 500000-501999, binary from f35d1978 (rc=0 both; `re-fix-base.txt`,
`re-fix-ha.txt`). Share of the decisive games each deck sat in:

| deck | hand AI off | hand AI on (Dack) | on - off (95% CI) |
|---|---|---|---|
| **Dack Fayden** | **27.3%** (was 38.4) | **14.9%** (was 24.5) | **-12.4 [-15.5, -9.3]** (was -13.9) |
| Kinnan | 22.3% | 25.7% | +3.4 |
| Kraum/Tymna | 28.8% | 32.6% | +3.8 |
| Rograkh/Silas | 19.5% | 21.2% | +1.7 |
| Rograkh/Thrasios | 21.3% | 21.2% | -0.1 |
| Sisay | 30.8% | 34.5% | +3.7 |
| deck win spread (target 0-8) | **11.3** (was 20.3) | 19.7 | |

The `--calibrate within` T6 report on the same games (`cw-fix-base.txt`, `cw-fix-ha.txt`): first-flip wins went from
457 to 271 with the hand AI off and from 239 to 100 with it on. So 41% and 58% of the old flip wins came from a
library the traces don't cover. The hand AI arm loses the larger share because it flips later, after drawing more of
the seven. Dack's win share (seated): 27.6% off, 15.5% on; median win round 5 and 9.

So the fix takes about 11 points off Dack in the default arm and brings Tier 6's spread from 20.3 to 11.3. Dack is no
longer the outlier: Sisay is at the top with 30.8% and Rograkh/Silas at the bottom with 19.5%. The hand AI gap
stays at -12.4: the hand AI's Dack hold is a handicap with or without the bug.

## What follows

- Leave the hand AI off for Dack, as the default already has it. No switch changed here. Its `dack_bar` /
  `table_risk` hold is the handicap. If the hand AI is ever wanted back, the bar needs recalibrating against the
  table's real counter rate first.
- The fix is conservative: a flip where the missing creature is already on Dack's side of the battlefield (Preston or
  Felidar cast from hand, SPEC.md's "deploy Preston first") no longer gets the shortcut. It may still win through the
  board combos. Tracing those cases (one of the seven already on the battlefield, or in hand with the mana to cast it)
  would show whether some should come back.
- The plan file's keep rule ships 53% of hands, against the hand AI's 41%. It leaves out the "two mana and three cheap
  plays" keep. That is worth about 3 points, inside the noise here. A plan-schema count of cheap cards would close it.
