# Stage 5 log review, slice c (games 10-14)

Winners: G10 Kinnan (combo, R5), G11 Sisay (damage, R14), G12 Sisay (damage, R13), G13 Dack (combo, R10), G14 Rograkh/Silas (combo, R6). Three of five games were won by combat damage or a very late combo; the decks' own lines almost never started.

## Cause tags (counts of deck-appearances over 20 deck-games; one deck can carry several tags)

| Deck (appearances) | MULL | MANA | TUTOR | HOLD | NOLINE | CHOICE | STOPPED | RULES | SPEC |
|---|---|---|---|---|---|---|---|---|---|
| Dack (4) | 2 | 3 | 0 | 2 | 0 | 2 | 1 | 0 | 0 |
| Sisay (4) | 0 | 2 | 0 | 2 | 0 | 4 | 3 | 0 | 3 |
| Kraum/Tymna (2) | 1 | 2 | 1 | 1 | 1 | 1 | 2 | 1 | 0 |
| Kinnan (5) | 0 | 4 | 0 | 2 | 0 | 4 | 4 | 0 | 3 |
| Rograkh/Silas (3) | 2 | 3 | 1 | 2 | 1 | 2 | 0 | 0 | 0 |
| Rograkh/Thrasios (2) | 1 | 2 | 0 | 0 | 0 | 2 | 1 | 0 | 2 |

Totals: MANA 16, CHOICE 15, STOPPED 11, SPEC 8, HOLD 9, MULL 6, TUTOR 2, NOLINE 2, RULES 1. The dominant story is land-light keeps (1-2 lands for many turns on ten deck-games) plus the AI preferring value plays to the commander or the line.

## Top problems

1. **Dack held back four turns with the line open** (G13). R6: `[Dack Fayden decides: Dack Fayden, Helping Hand 70%, Swords to Plowshares -> Kinnan, Bonder Prodigy 28% ... -> Swords to Plowshares -> Kinnan, Bonder Prodigy [search]]` with 5 lands + Sol Ring + Mana Vault; R7 and R8 it cast nothing with 6-8 mana; Dack arrived R10. Fix: the Dack plan (`t6_plan_dack.rs`) go-rule should take the flip when mana (6 with WW) is up and protection exists or opponents have less than two open blue mana, and should not let removal/value spells outrank the commander at 70%. Look at why the look-ahead preferred Swords (the `[search]` override) and why R7-R8 had no cast at all (go_ok refused?).
2. **Ranger-Captain of Eos fetches Thassa's Oracle** (G11 R10, G12 R8: `Kraum/Tymna searches for Thassa's Oracle`). Oracle text: search for a creature card with mana value 1 or less; Oracle is MV 2 and the Kraum spec says it cannot. Fix: the Ranger-Captain ETB search filter (tutor code, `engine/tutors.rs`) or the card's search in the Kraum plan is wrong. This gives Kraum a free Oracle tutor it should not have.
3. **Kraum/Tymna loses its own win pieces and never casts Ad Nauseam** (G11). R8 Intuition -> Ad Nauseam, then never cast through R14 with 6+ mana, life 25-36 and a 7-card hand; R12 `Chrome Mox imprints Demonic Consultation` destroyed half the Oracle line while Oracle was already fetched. Fix: Chrome Mox imprint choice must exclude cards the plan lists as combo pieces (Consultation, Pact, Oracle, Brain Freeze, LED, Breach); the Kraum plan's Ad Nauseam `go_ok` (life > 25, mana >= 5) should have passed.
4. **Sisay's best option is rated 100% and then not taken** (G11 R12-R14 `Sisay: Mount Doom 100%`, G12 R10-R13 `Sisay: Emiel the Blessed 100% ... -> stop (hold mana)`, G13 never activated at all; the commander itself went uncast until R6-R8 in G10/G12/G13). Fix: check the look-ahead's reason for rejecting the fetch (5-colour mana unavailable? the move is legal but the search disfavours it) and give the plan a rule to cast Sisay early and tutor Derevi/Emiel/Marvin on end steps. Also Greaves equips (G13 four times) outranked Eladamri's Call (64% -> equip 13%).
5. **Land-light keeps and wasted fast mana** (G10 Dack 1 land R2-R4, G11 Dack 2 lands R2-R9, G12 Kraum 2 lands R2-R5, G14 Dack/Rograkh/Silas 1 land). Both Rograkh decks also spent turn 1-2 on Defense Grid, Paradise Mantle, Vexing Bauble (G14), Chrome Mox imprinting Windfall (G12) rather than the line. Fix: the mulligan scoring for 1-land hands without cheap fast mana per spec "Mulligan"; and the cast priority in `t6_plan_rograkh_silas.rs` so Rograkh ({0}) is on the table turn 1 (G10 and G12 showed Rograkh coming only R2).
6. Minor: Thrasios the commander uncast for 8 turns in G13 and G14 (`Thrasios, Triton Hero 100%` ignored in favour of `untap a permanent (Formidable Speaker)`); Rograkh/Silas cast Thassa's Oracle (G12 R12 via Beseech the Mirror) with no Consultation/Pact; Kinnan's own One Ring burden helped kill it at R7 in G11; Rograkh seats carrying 9-11 cards in hand with no discards logged (G14).
