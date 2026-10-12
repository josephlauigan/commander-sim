# Report: cloud-plan

Measurements: `--deck cloud --pool tN --profile loose --ai lookahead --jobs 8`, same default seeds (paired). 240 games
unless noted. Noise is large (about +-2 points per cell), so compare averages.

| change | t1 | t3 | t5 | avg |
|---|---|---|---|---|
| baseline | 4.2 | 2.9 | 6.7 | 4.6 |
| search bonus +3 on casting Cloud | 5.0 | 11.2 | 8.3 | 8.2 |
| bonus +8 | 11.2 | 6.2 | 7.1 | 8.2 |
| bonus +20 | 11.2 | 6.2 | 10.8 | 9.4 |
| bonus +10 (kept) | 6.2 | 12.1 | 9.6 | 9.3 |
| bonus +10, 480 games | 8.8 | 7.7 | 9.0 | 8.5 |
| + equip-onto-Cloud search bonus +4 (480) | 8.1 | 7.3 | 9.6 | 8.3 (no gain, dropped) |
| + commander stays in hand when bounced (480, final) | 9.8 | 7.3 | 9.6 | 8.9 |

Adaptive AI, t3, 480 games: 2.3% (before). Plan online (--analyze t3): 50% of games, T4 12%, T5 23%, T8 40%.

## What was found
1. The search did value Cloud's cast low: the position score (`search::strength`) can't see the tutor, so Tome of Legends
   etc. scored higher. Fix: `cloud::search_bonus` (+10 on the Cloud cast option, only while he is out of play), added
   to the rollout totals in `search::choose_inner` (gated to key cloud). He now comes down on turn 2 in nearly every game.
   Waiting for protection mana was not tried as a rule: the bonus runs show casting early beats not casting (4.6 -> ~9).
2. Equip: the equip options already out-rank casts by utility; a search bonus for equipping onto Cloud changed nothing
   measurable, so it was dropped. The real limit is mana (3 lands most of turns 3-6) and Cloud being killed or wiped
   (Toxic Deluge, Beast Within) with commander tax 2 afterwards. Not solved.
3. Commander bounce (903.9b): for cloud a commander headed to hand stays in hand (no tax): `cloud::cmd_stays_in_hand`,
   used in `zones::to_zone_card`. Effect in the table is within noise.
4. Mulligan (engine/turn.rs) keeps 2-5 lands; left alone (35 lands in 100 gives few 3-land misses by itself).
5. Milestone `cloud` (first attack by an equipped Cloud): `cloud::plan_milestone` called from `combat::attack_triggers_once`,
   `report.rs` plan_text and plan_turn. Shows in the report.

## Remaining weakness
Most games end with the cloud player eliminated on rounds 6-9 (Aurelia, Marwyn, Korvold swing for 15-18): the deck
doesn't defend. That is the AI's blocking, outside this brief.

## Shared-file edits (all small, gated to key cloud)
`ai/search.rs` (bonus in choose_inner), `engine/zones.rs` (to_zone_card, commander to hand), `engine/combat.rs` (milestone
call), `sim-cli/src/report.rs` (plan text and turn). Tests: 3 new in `tests/cards_cloud.rs`.
