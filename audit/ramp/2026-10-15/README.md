# Look-ahead ramp bonus, 2026-10-15

**Question:** the look-ahead holds mana rocks and ramp (rust/agents/report-cloud-sequencing.md on branch
cloud-aifix3: Cloud's rocks waited 2+ turns in about half of the games). Does a shared fix that makes it cast early
ramp help the decks win?

**Answer: no.** The fix makes the AI cast ramp early, but no deck won more. It is merged **off** (`ramp_w` 0, the
old search), so a run can switch it on (`SEARCH_RAMP_W=1`).

## The fix
`search::ramp_bonus` (rust/crates/sim-core/src/ai/search.rs): the look-ahead plays only to the end of its own next
turn, and on the line that holds a rock the heuristic casts it next turn anyway, so both lines end on the same board
and a creature that attacks a turn sooner wins. The bonus adds `ramp_w` x mana made (rock amount, dork 1, land-ramp
count) x own turns left before turn 8 to casting a rock, dork or land-ramp spell (turn 2 Mind Stone +6, turn 1 Sol
Ring +14). A first try, a "future mana" term in the position score, did not change the choice: both lines end with
the rock on the battlefield. Test: rust/crates/sim-core/tests/search_ramp.rs (old search: "stop" over Mind Stone on
turn 2; with the bonus: Mind Stone).

## The check
Same build, same seed (1015), loose profile, look-ahead, 1,000 games per cell, `SEARCH_RAMP_W` 0 vs 1 (`run.sh`,
`cells.txt`, outputs `<deck>-<tier>-w0|w1.txt`). Noise of one difference: about +-2 points.

| Deck vs tier | Old | Ramp bonus | Change | Plan online at all | Commander resolves (mean turn) |
|---|---|---|---|---|---|
| Cloud vs T1 | 22.4% | 22.5% | +0.1 | 74.7% -> 78.7% | 2.1 -> 2.1 |
| Titania vs T3 | 29.5% | 28.2% | -1.3 | 58.7% -> 56.4% | 6.6 -> 6.7 |
| Sheoldred vs T4 | 30.1% | 28.2% | -1.9 | 79.7% -> 80.3% | 6.0 -> 6.0 |
| **Average** | | | **-1.0 +- 1.2** | | |

## Why it didn't help
- Ramp cast sooner doesn't reach the payoffs sooner: commander timing and plan timing are unchanged in all three
  decks. Cloud's plan comes online more often (+4 points), but the win rate doesn't follow.
- Games here end around rounds 9-12, so the look-ahead's preference for a body that attacks or blocks a turn sooner
  is not a mistake in this model.
- Sheoldred casts her late payoffs less (Howling Mine 50% -> 43% of the games drawn, Peer into the Abyss 31% -> 26%,
  Apex Witchstalker 31% -> 25%): the rock takes the mana first. Within noise, but it points the same way as the
  win rate.

Not tried: a gentler weight (0.5, turns <= 5) or a bigger sample; JD chose to merge it off.
