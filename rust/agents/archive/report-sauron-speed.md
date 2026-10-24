# sauron-speed report

Done. Sauron's plan milestone is now `army_hit` (first combat damage to a player by an Army), commander timing is
reported for every deck, and Sauron's Army power is snapshotted on turns 6, 8, 10.

Shared-file changes (all additive, one line or one call each): `engine/combat.rs` (calls `mine::army_hit_milestone`
where an Army's combat damage tempts), `engine/cast.rs` (milestone `cmd_cast` when the commander resolves as a
permanent), `engine/turn.rs` (`step_start` calls `mine::army_power_snapshot`), `impls/mine.rs` (the two functions).
Milestones are report-only; no game decision reads them. Also `sim-cli/src/report.rs` (plan_turn, plan_text, metric
`cmd_turn` in the normal report's Speed axis, `Analysis.cmd/army`, `print_speed`). Test: `tests/cards_sauron_speed.rs`.

Equivalence: `--deck sauron|kefka --pool t3 --games 96 --seed 500000 --profile loose --ai adaptive --brief` output is
identical before and after, except Sauron's "Plan online" line (4.2% combo -> 45.8% Army hit).

Smoke (`--ai lookahead --games 96 --analyze`, sauron):
```
Plan: first combat damage to a player by an Army
Online by turn:  T2 0%  T3 1%  T4 1%  T5 3%  T6 9%  T7 12%  T8 18%  T9 22%  T10 26%  T11 34%  T12 39%
Online at all: 45.8% of games
Commander resolves on turn: median 7.0, mean 6.8 (over the 84.4% of games it resolves)
Commander resolved by turn:  T4 8%  T5 25%  T6 40%  T7 56%  T8 68%
Army power at the start of turn  6: mean 0.9, median 0.0, 10+ in 1.0% of games
Army power at the start of turn  8: mean 1.9, median 0.0, 10+ in 6.2% of games
Army power at the start of turn 10: mean 1.0, median 0.0, 10+ in 3.1% of games
Combo attempted in 1.0% of games
```
Notes: Army power counts 0 for no Army, a dead player, or a game that ended before that turn (as briefed), so the
turn-10 mean is lower than turn 8. Power is the effective power (`values::epow`) at the start of the turn.
Checks: cargo test --release -j 6 all pass; all-targets build has no warnings; all_decks adaptive 3 and lookahead 1
finish. `cardgen audit --mine`: no cards in this brief.
