# Karn, Legacy Reforged — findings

The deck's living list of strengths, weaknesses and what was tested. Deck doc: `decklists/JD/karn-colorless-artifacts.md`.

## 2026-10-20: first tier run (loose, look-ahead, 1,000 games per tier, `--any-tier`)

| Tier | Win rate | 95% CI | Avg win turn | Karn online |
|---|---|---|---|---|
| t0 | 22.6% | 20.1-25.3 | 12.0 | 79% |
| t1 | 30.5% | 27.7-33.4 | 13.0 | 78% |
| t2 | 33.0% | 30.2-36.0 | 14.3 | 82% |
| t3 | 30.2% | 27.4-33.1 | 13.2 | 81% |
| t4 | 23.8% | 21.3-26.6 | 12.6 | 74% |
| t5 | 29.0% | 26.3-31.9 | 11.6 | 74% |
| t6 | 34.0% | 31.1-37.0 | 12.4 | 73% |

Not monotonic in opponent strength: best against cEDH, worst against precons. `--analyze` at t0 and t6 (1,000 games,
seed 77) explains it.

## Strengths
- **Karn arrives on time:** median turn 4 (66% by turn 4 at t0, 54% at t6).
- **The big artifacts win when they land:** Blightsteel, Portal to Phyrexia, All Is Dust, Triplicate Titan, Spine,
  Darksteel Forge, both Ugins: +20 to +40 points when cast (correlation: they come late, in games already going well).
- **Against decks that don't attack (t6 cEDH):** when the cEDH decks fail to combo, nothing stops a big colourless
  board: 40.6 combat damage per game and 0.99 opponents eliminated per game at t6, vs 28.9 and 0.58 at t0.

## Weaknesses
- **Creature beatdown.** At t0 it is eliminated in 77% of games (round 9.8), almost all by combat (Giada's angels,
  Captain America, Anikthea, Zurgo). The deck has few early blockers, and artifacts that aren't creatures don't block.
- **Karn dies a lot:** removed 0.87 times per game at t0 (0.53 at t6), so he adds mana in only 74-79% of games and
  his win rate when cast is no better than average (+0.7 at t0).
- **The finishers come late:** each is cast in only 10-16% of games; wins average turn 12-14.
- **The combo rarely happens:** 1-5% of wins (three specific cards, few tutors for them).
- **Against cEDH it simply loses the race:** eliminated in 64.5% of games at t6, round 7.9, almost all to opposing
  combos; it has no interaction for the stack.

## Open issues
- **Everflowing Chalice is never cast** (untagged in the sim): a modeling gap to fix.
- **Oblivion Stone and Nevinyrral's Disk are stuck in hand** (cast in 27-47% of draws): the AI's "only when behind"
  rule may be too strict.
- **Win turn 12-14 for every deck in the sim:** possibly a simulator-wide pace issue (see JD's question, 10/20), not
  specific to Karn. Being investigated.
- **Tiers not set:** `data/my_decks.toml` has placeholder t3-t4.
