# Tier 6 speed-up, 2026-10-19

Goal (JD): the Tier 6 table wins at a median round of about 7 (TARGETS-t6.md), with each deck still in 18-32%.
Result: **not reached.** The table's median win round is 9 (8-9 for four decks, 12 for Sisay and Rograkh/Thrasios),
the same as before. The work found no mana-model errors and no deck that waits once its line is ready (except Kraum/Tymna,
fixed). The decks are draw- and mana-limited. Raw: the .txt files here (loose profile).

## Final calibration (2,000 games, loose)

| Deck | win % s500000 | win % s900000 | median win round | ready (median round, share of games) |
| --- | --- | --- | --- | --- |
| Kraum/Tymna | 32.8 | 32.7 | 9 | 7 (49%) |
| Dack Fayden | 28.5 | 27.6 | 8 / 7 | 7 (22%) |
| Kinnan | 26.1 | 25.8 | 8 | 7 (26%) |
| Rograkh/Silas | 24.7 | 24.4 | 8 | 7 (42%) |
| Sisay | 21.5 | 25.3 | 12 | 8 (10%) |
| Rograkh/Thrasios | 16.5 | 14.3 (LOW) | 12 | 10 (13%) |

Table: average game 10.0 / 10.1 rounds, median win round 9 on every seat. Band check: Rograkh/Thrasios is below 18 on
both seeds (it was 13.5-17.5 at Stage 6), Kraum/Tymna is 0.7-0.8 above 32, and the gap is 16-18 points (target 8).
Seat edge: seat 1 27%, seat 4 22-24%; real 34% / 17%, so the edge is still flat.

## What was tried

| Change | Result | Kept |
| --- | --- | --- |
| Measure: ready / one tutor away / went for a line rounds per deck | Every deck goes the round its line is ready, except Kraum/Tymna (waited for cover) | stats |
| Kraum/Tymna goes with one cover, or when the table is tapped low | Median win 9 to 9, win % within noise | yes |
| Mana: early mana-first rule (a mana card before other plays, own turn, turns 1-3) and keeping one spare card early | Turn-2 mana about 2.5-2.8 before and after; no mana-model errors found | yes |
| Sisay: fetch in the end-of-turn window; running lines ignore a tapped Bloom Tender | Fired in 1 of 60 traced games: leftover mana is rarely WUBRG. 20.7 to 21.5%, median 12 | yes (Tier 6 only, no cost) |
| Rograkh/Thrasios: Thrasios keeps a land on top while mana is short | 600 games 14.0 to 18.1 was noise: 2,000 games 16.4 to 15.6 | dropped |

## Why the table is slow

- Lines are ready in only 10-49% of games, and at median round 7-10 when they are. The table goes as soon as a line is
  ready, so speed is set by how fast the pieces arrive, not by the AI waiting.
- Rograkh/Thrasios: its pieces (Gaea's Cradle, Oboro, Talon Gates) arrive late and its commanders get removed. The
  Thrasios dig is mana-limited (about 3.2 digs per game, 2.2 of them at end of turn), so more dig windows do not help.
- Sisay: 65-70% of its wins are damage, not a combo; 56 of 115 stopped tries name no card (Unicorn 38, DDR 18), cause
  not found (one DDR step 3 stop marks Ragavan unusable).

## Open

To reach a median of about 7, the next levers are the deck lists and their plans (more tutors, faster pieces), not the
engine: Rograkh/Thrasios and Sisay first. Godo vs Tier 6 is unchanged at about 49-50% (audit/godo/FINDINGS.md).
