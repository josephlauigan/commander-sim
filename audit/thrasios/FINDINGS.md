# Tymna/Thrasios (key `thrasios`): findings

The deck's living list of strengths, weaknesses and what was tested. Built 2026-10-19 from the Tymna/Kraum list: Kraum
and the 23 red cards out, green cards from Omnath and bulk in, plus the Basalt Monolith + Rings of Brighthearth +
Walking Ballista combo (Thrasios is a second outlet). Matched tier: t5 (Kraum's).

## 2026-10-19: first run vs Tymna/Kraum (loose, look-ahead, 2,000 games each, same seeds, T5)
Raw: `2026-10-19-vs-kraum/<deck>-t5.txt` (`--analyze`).
| | Tymna/Kraum | Tymna/Thrasios |
|---|---|---|
| Win rate | **29.9%** | **24.6%** (-5.3, noise about ±2.8) |
| Damage dealt per game | 34.2 | 26.5 |
| Opponents eliminated per game | 0.78 | 0.63 |
| Eliminated, % of games | 70.0 | 75.2 |
| Commanders by turn 4 | 74% | 88% |
| Wins by combo | 0% | 10.6% of wins |

Why:
- **Less pressure.** The red dragons were Kraum's best finishers (Terror of the Peaks wins 68% when cast); their green
  replacements are mana and utility, so damage falls 7.7 per game.
- **The combo rarely assembles:** online in 2.7% of games; Walking Ballista cast in 21% of its draws.
- **Thrasios is mana-bound:** 0.52 activations per game though he is out in 86% of games (as on 10/13: 0.4).
- **Model gaps:** Blossoming Defense never cast (untagged for this deck); Dark Endurance cast in 5% of its draws.
- Strength: the commanders land sooner (Tymna turn 3.7, Thrasios 4.3; Kraum turn 6.1).


## 2026-10-19: AI fixes (agent thrasios-ai, branch thrasios-ai)
First run (2,000 games, T5): 24.6% against Tymna/Kraum's 29.9% on the same seeds. Win rate is not re-measured here: the
200-game checks (24.0–26.0%, noise about ±6) only show that the cards now play.

### What the AI got wrong, and the fix
- **Blossoming Defense was never cast** (no tag, so the generic priority gave it 0, and only the Omnath deck had a
  protection path). Now cast in response to targeted removal on a commander or a creature worth 4+, and as a combat
  trick on a blocked commander that would die. Cast in 9% of games, 40% of the games it is drawn.
- **Walking Ballista held to 8 mana** (the generic rule keeps it for the Rings combo while Rings is anywhere in the
  deck). Now held only when Basalt and Rings are both in hand or out; otherwise cast with 4+ mana.
- **Thrasios's {4}**: the look-ahead's score doesn't count the land or card, so it chose "stop". Now made whenever the
  search's best play among the others is "stop". The count stays near 0.5-0.7 a game: the deck rarely has {4} spare
  (7 cards in hand at most turns, and the colour-hungry ones — WW, UUBB — stay stuck; spare mana is real only late).
- **Dark Endurance** was cast in 5% of the games it was drawn: the AI taps out on its own turn, and it only answered
  destroy/damage removal (opponents mostly exile, bounce or wipe). Now also a combat trick, and cast on a commander
  against a destroy sweeper.
- **Combo pieces**: Basalt Monolith always ramp; Rings of Brighthearth once Basalt is there; the tutor fetches the
  missing third piece (Basalt, Rings, outlet). Plan label: first Tymna draw, or the combo going off.

### Tried and dropped
- Holding mana for Blossoming Defense / Dark Endurance through `hold_value`: the look-ahead's hold bonus then beat
  casting two creatures with ten lands in play (63 "stop (hold mana)" in 12 games, was 2). Removed; the mana is kept
  only when the spare goes to Thrasios.

## Weaknesses (by evidence, 10/19 traces)
- Board wipes (Cyclonic Rift overload, -X/-X) take Tymna and Thrasios together (7 of 14 traced games).
- Hands clog on colour: WW/BB/UU cards sit with 7-10 cards in hand and 8+ lands.

## 2026-10-19: rerun after the AI fixes (loose, look-ahead, 2,000 games each, same seeds, T5)
Raw: `2026-10-19-rerun/<deck>-t5.txt`. **Tymna/Thrasios 26.6%** (was 24.6%), **Tymna/Kraum 30.0%** (was 29.9%; it
shares the Dark Endurance fix): the gap is **-3.4** (noise about ±2.8). JD committed to Thrasios anyway (10/19).
- The fixes are worth about +2: combo wins 10.6% -> 16.2% of wins, plan online 76.5% of games, Blossoming Defense
  and Ballista now cast in about 41-43% of their draws (Dark Endurance 31%).
- What's left of the gap is pressure: 26.9 damage per game vs Kraum's 34.2 (the red dragons).
- Thrasios still activates 0.54 times a game: the deck rarely has {4} spare.

## 2026-10-19: Urza, Lord High Artificer from Sauron (paired, 2,000 games, T5; `2026-10-19-urza/`)
| Swap | Result | Why |
|---|---|---|
| Coiling Oracle => Urza | -1.5 (26.6% -> 25.1%, ±2.2) | Noise, leaning worse. Urza is stuck in hand 53% of the games he's drawn and first cast in round 8.6: {U}{U} in a four-colour deck with 6 Islands. Mana on turns 4 and 6 unchanged, so his artifact mana doesn't arrive in time to feed Thrasios. Wins 50% when cast (correlation). |
**Reading it:** not adopted. Urza stays in Sauron (Sauron's side: `audit/sauron/FINDINGS.md`, Spell Pierce back +0.5/+1.2, noise).
