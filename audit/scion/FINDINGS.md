# Scion of the Ur-Dragon (key scion): findings

The deck's living list of strengths, weaknesses and what was tested. Scion plays Tier 6 (cEDH) only. List v1 (the
EDHTop16 consensus list, `rust/agents/scion-v1-list.txt`) is the baseline every later version is measured against.

## 2026-10-27: the look-ahead passed up sure wins in the goldfish (fixed); the corrected baseline

Raw output: `2026-10-27-win-cap/` (goldfish look-ahead, adaptive, and look-ahead with `SEARCH_HOLD_W=0`; scion vs
Tier 6; `--realism` t0-t4 and t6, 4,000 games each, `before/` and `after/` the fix).

**The bug.** In the v1 baseline below, a traced goldfish game (seed 500003) had Thassa's Oracle + Demonic Consultation
ready on turns 16 and 17 (heuristic 100%) and the look-ahead chose "stop (hold mana)" both times. Both lines won
every playout (100: the heuristic combos next turn and the inert seats can't stop it), and the hold bonus
(`search::hold_bonus`) was added on top (103.25), so waiting beat winning. Fix (`ai/search.rs`, `choose_inner`): the
rollout scores, bonuses included, are capped at a sure win, and a tie goes to the play the heuristic rates higher. Test:
`search_hold::a_hold_bonus_never_beats_a_sure_win` (fails without the cap: 100.0 vs 102.375).

**Goldfish after the fix** (look-ahead, 2,000 games, seed 500000; own turn of the kill):

| deck | median | mean (kills) | by T2 | by T3 | no kill by R20 | protection held on the kill |
| --- | --- | --- | --- | --- | --- | --- |
| **scion** | 10.0 (was 13.0) | 10.2 | 0.3% | 2.9% | 3.9% | 80.5% |
| dack-fayden-white-stax | 10.5 (11.0) | 9.9 | 0.8% | 6.9% | 10.4% | 47.3% |
| kinnan-bonder-mana-combo | 8.0 (9.0) | 8.7 | 0.1% | 2.1% | 1.0% | 86.1% |
| kraum-tymna-blue-breach-midrange | 9.0 (11.0) | 9.5 | 0.3% | 2.0% | 7.2% | 90.5% |
| rograkh-silas-turbo-breach | 10.0 (14.0) | 9.2 | 0.6% | 4.8% | 21.1% | 77.3% |
| rograkh-thrasios-creature-combo | 9.0 (10.0) | 9.3 | 0.0% | 0.1% | 0.5% | 85.0% |
| sisay-legends-toolbox | 7.0 (7.0) | 7.6 | 0.0% | 0.1% | 0.2% | 73.0% |

Scion's kills: the Oracle line 73.0% (was 63.2%), damage 18.8%, commander damage 6.4%, the Prismari line 0.9%. The
adaptive AI doesn't use the look-ahead and is unchanged (median 11.0).

**At the table the fix is neutral.** `--realism` t0-t4 and t6: no row changes pass/fail, and only one moves by 0.8
points or more (t3 combo wins 15.8% -> 16.7%, inside the noise); Tier 6 deck shares move by 0.3 points at most. Scion
vs Tier 6: 26.5% (was 26.6%). t5 was not run: it hits the memory cap on the Miirym Treasure blow-up that is fixed on
the `phase2-doubles` branch, not on main.

**What is left is the Tier 6 heuristic, not the hold bonus.** With the hold bonus off (`SEARCH_HOLD_W=0`) the goldfish
is the same (scion median 10.0, Kinnan 8, Sisay 7), and the adaptive AI, with no look-ahead at all, is at 11. A median
kill by turn 8-10 against seats that never act is still the shared Tier 6 AI's pace (tutors held, rocks held, few land
drops in the traced games), not the deck's; D1 (a median kill by turn 3) can't be judged until that AI plays faster.

## 2026-10-26: v1 baseline (goldfish and Tier 6; 2,000 games each, seed 500000, loose)

*Superseded by the 2026-10-27 section above: these goldfish numbers carry the look-ahead bug fixed there.*

Raw output: `2026-10-26-baseline/goldfish-lookahead.txt` (scion and the six Tier 6 pool decks),
`goldfish-adaptive.txt` (scion), `pool-t6-lookahead.txt` (scion vs Tier 6).

### Goldfish (three inert seats; own turn of the kill; look-ahead AI)

| deck | median | mean (kills) | by T2 | by T3 | no kill by R20 | protection held on the kill |
| --- | --- | --- | --- | --- | --- | --- |
| **scion** | 13.0 | 12.1 | 0.3% | 2.3% | 3.9% | 78.6% |
| dack-fayden-white-stax | 11.0 | 9.9 | 0.8% | 6.9% | 10.4% | 46.9% |
| kinnan-bonder-mana-combo | 9.0 | 9.4 | 0.1% | 1.9% | 1.0% | 85.6% |
| kraum-tymna-blue-breach-midrange | 11.0 | 11.0 | 0.3% | 1.6% | 7.3% | 89.6% |
| rograkh-silas-turbo-breach | 14.0 | 12.0 | 0.5% | 3.0% | 21.1% | 74.9% |
| rograkh-thrasios-creature-combo | 10.0 | 10.6 | 0.0% | 0.1% | 0.6% | 82.3% |
| sisay-legends-toolbox | 7.0 | 7.5 | 0.0% | 0.1% | 0.2% | 73.0% |

Adaptive AI (scion, for later quick reads): median 11.0, mean 11.1, by T2 0.1%, by T3 2.1%, no kill 5.0%,
protection 81.6%.

**The clock is the simulator's, not the deck's.** D1 asks for a median kill by turn 3 (about 30% by turn 2, 70% by
turn 3). Every Tier 6 deck, pool decks included, kills at a median own turn of 7-14 in the goldfish: the shared Tier 6
AI assembles its lines far more slowly than real cEDH players (the known slow-table problem, audit/t6/ and
documents/realism.md). So the goldfish can't yet say whether v1 meets D1; it can rank lists against each other and
against the six pool decks under the same AI. Against those, scion is in the slow half (median 13 against 7-14),
close to Rograkh/Silas, the other Oracle-turbo list.

Where the time goes (from the Tier 6 counters of the pool run): with a line one tutor away the deck chose
"stop (hold mana)" 290 times and "stop" 220 times, holding Diabolic Intent (172 times), Mystical Tutor (171),
Enlightened Tutor (119) and Demonic Tutor (82); Mox Diamond was held on turn 1 in 70 card-turns (the deck's rule).
In the goldfish the hold-up rule still spends mana on counters nobody will need (the inert seats never cast).

- **How it won (goldfish, look-ahead):** Thassa's Oracle + Demonic Consultation / Tainted Pact 63.2%, damage 25.2%,
  commander damage 10.2% (Scion as Copper Dragon attacking), Scion as Prismari + Breach + Manamorphose + Brain
  Freeze 0.6%, Breach + LED + Brain Freeze 0.5%. Went for the Oracle line in 60.9% of games (99.9% converted), the
  Prismari line in 0.6% (91.7% converted), the Breach + LED lines in 0.8%.
- **Mulligans:** kept 7 cards 88.3% (59.3% no mulligan, 29.1% the free one), 6 cards 9.3%, 5 or fewer 2.3%
  (the provisional Rograkh/Silas-style keep rule, D11 open).
- **Ready turn:** a line first ready at median own turn 11 (mean 8.8), in 69.5% of games.
- **Protection on the kill turn:** 78.6% of kills (D1 asks about 60%+). Counted: An Offer You Can't Refuse,
  Flusterstorm, Force of Negation, Force of Will, Mental Misstep, Mindbreak Trap, Orim's Chant, Pact of Negation,
  Silence, Swan Song (tags ctr / prot, plus Silence, Orim's Chant and Grand Abolisher; Veil of Summer, Misdirection
  and Redirect Lightning carry neither tag, so they don't count).
- **Scion:** cast in 67.4% of goldfish games (median own turn 5). Activated into Ancient Copper Dragon in 52.0% of
  games (1,050 activations), Korvold in 39.0% (783), Prismari in 0.5% (9, all as the storm line's first step).

### Against Tier 6 (comparative only)

**26.6% (95% CI 24.7-28.6%)**, even share 25%. Wins by combo 83%, damage 16%; average own turn of wins 8.6, median win
round 8. **Read this comparatively, as audit/godo/FINDINGS.md does:** the Tier 6 table wins at a median round of 9
against about 7 in real games, so it flatters fast decks. It loses most to Sisay (16.7% of games), Dack (12.9%),
Kraum/Tymna (12.5%); least to Rograkh/Silas (8.7%). Its Oracle tries are stopped by Pact of Negation, Mindbreak Trap,
Mental Misstep, Swan Song and Force of Negation.

### Model status (cardgen audit --mine): 9 of 100 cards not Full

Partial: Gemstone Caverns, Orim's Chant (no kicker), Redirect Lightning, Shifting Woodland, Tinder Wall (the blocking
ability). Approximate: Crop Rotation, Hexing Squelcher, Lotho, Corrupt Shirriff (regex tags), Scion of the Ur-Dragon
(the copy is modeled; the AI uses only Copper, Korvold and the Prismari line). Prismari is Full since 2026-10-26.
