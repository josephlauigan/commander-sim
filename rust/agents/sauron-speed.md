# Task: sauron-speed — measure how fast Sauron gets going

Read `rust/agents/SAURON-RULES.md` first. JD's complaint: Sauron "fully gets going on turn 12". That figure is the
`--analyze` "Game plan timing" line, whose Sauron milestone is the Sword of Feast and Famine + Aggravated Assault
combo attempt (`sim-cli/src/report.rs` `plan_text`, milestone `combo` in `impls/mine.rs`), which happens in about 1%
of games. Every other deck uses its first engine trigger. We need real speed numbers before testing speed packages.

## What to add
1. **Sauron's plan milestone** becomes "first combat damage to a player by an Army Sauron controls" (a new milestone
   key, set once per game, the turn number like the other decks). The combo attempt stays reported on its own line
   ("Combo attempted in N% of games"), which already exists.
2. **Commander timing, every deck** (in `--analyze`, and as a metric in the normal report if that is easy): the turn
   the deck's commander first resolves (median and mean over the games it resolves; plus "by turn 4/5/6/7/8" shares
   like the "Online by turn" line).
3. **Army growth, Sauron only** (in `--analyze`): Army power on Sauron's turns 6, 8 and 10 (mean and median, counting
   0 when there is no Army or the player is dead, and the share of games with power 10+ on that turn), and the turn of
   the first Army combat damage (it is the milestone of 1).
Keep these cheap (a per-game counter or snapshot at the start of the player's turn, recorded in the stats the report
already gathers, e.g. `pl.stat` / `milestone`). Don't change any game decision: with a fixed seed, every deck's win
rate must be identical before and after your change (check `--deck sauron --pool t3 --games 96 --seed 500000
--profile loose --ai adaptive --brief` and the same for `kefka` before and after).

## Files
`sim-cli/src/report.rs` (and the analysis printer it uses), `state.rs` (only if a new stat field is truly needed),
`impls/mine.rs` / `engine/combat.rs` for the milestone hook (smallest additive change).

## Tests and smoke run
A test that the milestone is set on the turn an Army first deals combat damage to a player and not before; a test for
the Army-power snapshot. Smoke: `./target/release/commander-sim --deck sauron --pool t3 --profile loose --games 96
--ai lookahead --jobs 6 --analyze` shows the new lines; paste them into your report.
