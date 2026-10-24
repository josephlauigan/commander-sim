# Report t6-ai-must

## Mechanism
The look-ahead (`ai/search.rs::choose`) ranks only the top-k options by heuristic utility plus "stop", then picks by rollout
score; plan priorities enter only as the prior, so "stop (hold mana)" or a lesser spell outvoted them.
Fix: `ai/t6_table.rs::must_option`, called from `brain::main` (Tier 6 players only, `t6_plans::is_t6`) right after
`main_options` and BEFORE the search. It returns the play to make now:
1. a cast whose plan `prio` >= 90 (`MUST_PRIO`): commanders, fast mana, the protection pre-cast (highest prio first);
2. a card with `combos::piece_gain` >= 1.0 (it completes a line), score 89, unless the plan set its prio to 0;
3. Sisay's fetch (`Sisay: <card>` ability option), score 91;
4. a non-creature mana rock/fast mana with cmc <= 2 on turns <= 4 when the plan has no opinion (Sol Ring lost to "stop
   (hold mana)" at 100% in a Kinnan trace).
Skipped while the plan's `hold_up` is positive, so "stop (hold mana)" survives the turns a plan asks for it. If the
play fails to perform, the old path (search/heuristic) runs. Trace lines show `[must]`. No plan file was edited.
Tests: `crates/sim-core/tests/ai_t6_must.rs` (Kinnan cast ahead of Chord/Sol Ring; nothing forced otherwise).

## Results (300 games per profile, seed 777, conservative / loose; base = HEAD before, after = final)
Win % (cons/loose), median win round (cons/loose), combo tries per game (cons/loose):

| Deck | win base | win after | median rd base | after | tries base | after |
| --- | --- | --- | --- | --- | --- | --- |
| Dack | 32.6/25.3 | 38.9/34.2 | 7/7 | 6/6 | .28/.25 | .35/.34 |
| Kinnan | 31.5/33.3 | 31.9/33.8 | 7/8 | 8/8 | .34/.34 | .30/.31 |
| Kraum/Tymna | 22.5/21.5 | 26.5/25.5 | 8/9 | 9.5/9 | .28/.25 | .32/.27 |
| Rograkh/Silas | 20.4/26.3 | 16.7/22.6 | 6.5/8 | 6/7 | .36/.39 | .34/.30 |
| Rograkh/Thrasios | 13.9/13.0 | 13.9/12.5 | 13/12 | 12/11 | .10/.12 | .10/.07 |
| Sisay | 29.1/30.5 | 22.2/21.7 | 11/11.5 | 12/12 | .15/.10 | .06/.06 |

Avg rounds 9.7/9.8 -> 9.4/9.2; timeouts 1.0/2.3% -> 1.3/1.3%. Seat 1-4 share (cons/loose) base 31.6/25.9/21.5/20.9
and 24.9/28.0/23.9/23.2; after 28.4/25.0/20.6/26.0 and 29.4/29.1/18.6/23. Godo vs T6 (cons/loose): 44.3/45.0 -> 41.7/46.0
(first variant without rule 4: 45.0/49.3); median-of-wins turn 7.1 -> 6.5/7.1. Single-run CIs are about +-6 points: deck
swings under 6 points are noise.

## Findings
- The must rule works as designed (traces: Sisay cast on turn 2, Kinnan on turn 1; Dack's median win round fell 7 -> 6, tries up).
- The median win round of Sisay and Rograkh/Thrasios (11-13) did NOT move. Their delay is not the search voting: they
  sit on 3 lands with the commander or fetch (WUBRG for Sisay) uncastable/unaffordable; that is the `keeps`/mana task, and
  Sisay is also bounced/killed after the cast (combo tries stopped 58-91%). Sisay's win rate fell ~7 points (29 -> 22),
  because casting her early now exposes her to the removal she used to dodge by being late; Stage 6 should look at that.
- Rograkh/Silas down ~4 points, within noise; Dack up 6-9 points (was already high; Stage 6 number tuning).
- No "internal simulator error" in any run. t3 lock identical (diff empty). Full `cargo test --release` passes, no warnings.
