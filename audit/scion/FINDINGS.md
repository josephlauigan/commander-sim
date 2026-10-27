# Scion of the Ur-Dragon (key scion): findings

The deck's living list of strengths, weaknesses and what was tested. Scion plays Tier 6 (cEDH) only. List v1 (the
EDHTop16 consensus list, `rust/agents/scion-v1-list.txt`) is the baseline every later version is measured against.

## 2026-10-26: v1 baseline (goldfish and Tier 6; 2,000 games each, seed 500000, loose)

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
