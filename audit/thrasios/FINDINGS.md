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

Next: agent `thrasios-ai` (branch `thrasios-ai`) closes the gaps and checks the AI's play; then both decks are rerun.
