# Tier 6 mulligans, 2026-10-19

Question (JD): are the AI's mulligans a reason the Tier 6 table is slow? **No.** Raw: `base-2000.txt` (old bottom rule),
`new-2000.txt` (new rule); 2,000 games, loose, seed 500000; same hands in both.

## Win % by mulligans (old bottom rule)

| Deck | none | one (free) | two (6 cards) | three (5 cards) | hands mulliganed |
| --- | --- | --- | --- | --- | --- |
| Dack Fayden | 26.8 | 29.8 | 26.8 | 18.8 | 41% |
| Kinnan | 25.0 | 30.2 | 24.8 | 17.2 | 48% |
| Kraum/Tymna | 34.6 | 34.1 | 29.9 | 18.3 | 52% |
| Rograkh/Silas | 25.3 | 22.9 | 25.4 | 27.3 | 42% |
| Rograkh/Thrasios | 16.3 | 15.9 | 18.3 | 10.6 | 38% |
| Sisay | 22.0 | 20.7 | 19.4 | 12.2 | 43% |

The free mulligan costs nothing, going to 6 costs little, and only going to 5 (3-7% of hands) hurts. The median win
round barely moves with mulligans (7-10, Rograkh/Thrasios and Sisay 11-16 as always).

## The bottom rule

Before, every deck bottomed its most expensive spell (or a land past four), which could be the combo piece or tutor
the hand was kept for. Now, Tier 6 only (`t6_keep::bottom_pick`): prefer a bottom that leaves a hand the deck's own
keep rule still takes; among those, bottom the card worth least to the plan (not a tutor target from the deck's wish
list, not a mana piece, a land only past the third, then the deck's lowest play priority).

Result: every deck's win % is unchanged within noise (largest move Dack 28.5 to 27.6), median win round still 9. Deep
mulligans do somewhat better (Kinnan at 5 cards 17.2 to 24.1%, Sisay at 5 cards 12.2 to 22.0%, Rograkh/Thrasios at 6
cards 18.3 to 21.7%), but those groups hold only 40-120 hands each. Kept: it costs nothing and bottoms what a player
would, and the practice-mode mulligan hint names this card for Tier 6 decks (Godo).
