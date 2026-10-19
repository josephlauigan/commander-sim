# Stage 5 log review, slice d (games 15-19)

Results: g15 Kraum/Tymna R9 combo; g16 Dack Fayden R11 damage; g17 Kinnan R6 combo; g18 Rograkh/Silas R9 combo; g19 Rograkh/Silas R7 combo. Median win round 9 against a spec target of 3-5. Counts are deck-games (20 deck appearances over 5 games; a deck can carry several tags). `MULL` was not judged (the log shows no opening hands); `ODD` is listed separately.

| Tag | Count | Where |
| --- | --- | --- |
| MULL | 1 | g17 Dack kept a one-land seven (stuck on 1 land until R6) |
| MANA | 12 | g15 Kinnan, Silas; g16 Sisay, Kinnan; g17 Kraum, Dack; g18 all four; g19 Silas, Dack |
| TUTOR | 6 | g15 Kraum, Silas; g17 Kraum; g18 Silas, Kraum; g19 Silas |
| HOLD | 9 | g16 Dack/Sisay/Kinnan/Thrasios; g18 Silas/Kraum/Dack; g15 Silas; g19 Sisay |
| NOLINE | 1 (possible) | g18 Kraum: Oracle + Tainted Pact fetched R2, combo option never printed R3-R8 |
| CHOICE | 15 | every game; 4 each in g17 and g19 |
| STOPPED | 14 | counters and bounce on key pieces: Fierce Guardianship, Snap, Pyroblast, Eldrazi Confluence, Ertai, Reprieve |
| RULES | 1 (suspect) | hand size above seven never trimmed |
| SPEC | 8 | missing Oboro/Witness, no coloured legend for Sisay, flip without Preston/Solar |

## The 5 problems that matter

1. **No discard to hand size (RULES, suspect).** `R9 Kinnan turn 9: ... 15 cards in hand` (g15), `Rograkh/Silas turn 6: ... 12 cards in hand` (g15), `Kraum/Tymna turn 4: 11 cards in hand` (g17), Kinnan `11 cards in hand` (g18), Silas `11 cards in hand` (g19). The only "discards" line in any log is Gamble. Rule 514.1 sends everyone down to seven in the cleanup step. Fix: check the cleanup step in the engine (rust/crates/sim-core); if it exists, it is not firing for the trace. It also hides the real cost of hoarding (Kinnan sat on 9-15 cards with nothing cast).
2. **The look-ahead overrides its own top score with a worse play, and the printed scores do not say why.** g17 R5 `Kinnan, Bonder Prodigy 79%, Chord of Calling 21% ... -> Chord of Calling` (countered by Fierce Guardianship; Kinnan + the Monolith copy already in play won the next turn, so this cost Kinnan the R5 win); g17 R4 `Sisay 95%, Tyvar 5% ... -> Tyvar`; g16 R1 `Kinnan 98%, Borne Upon a Wind 2% -> Borne Upon a Wind` (Kinnan cast a draw spell on T1 instead of himself); g16 R1/g19 R1 `Rograkh 100% ... -> stop` before the "stop" is overturned. Fix: print the final search value next to the prior, or log "search overrode prior" so a reviewer can tell a good override from an evaluation error. Candidate slots: the card-draw/tutor bonus in the evaluator that favours Chord/Borne over a mana-cost-2 commander.
3. **Kraum/Tymna never offers the Oracle + Tainted Pact line it already holds (g18), and mis-sequences tutors (g17).** g18: Vampiric Tutor and Demonic Tutor on R2 (`tutors Tainted Pact`), 4 mana from R5 (3 lands + Sol Ring), yet no `combo: Thassa's Oracle + ...` line in R5-R8 and it cast Tymna, Esper Sentinel and Enlightened Tutor for LED instead. g17: `Kraum/Tymna tutors Tainted Pact` via Wishclaw R5, `Sisay gains control of Wishclaw Talisman`, six mana already spent on Tataru Taru + Wishclaw, so no Oracle + Pact the same turn. Fix: add a trace line when `ready`/`go_ok` rejects the combo (it is `HOLD` or `NOLINE`, the log cannot say); Wishclaw only when the pair can be cast the same turn (t6_plan_kraum). One more data point is g15 R9, where the combo appeared only once Silence was in hand.
4. **Silas (Rograkh/Silas) has no way to find Oracle and often cannot use its mana.** g18: Consultation on top R1, no Oracle until R8 (`Rograkh/Silas tutors Thassa's Oracle` from a stolen Demonic Tutor), 3 lands R4-R8 with 5-7 cards. g19: `Chrome Mox imprints Underworld Breach`, and R6 Beseech the Mirror cast Oracle free with no Consultation in hand (Snap by the opponent accidentally returned it). g15: 12 cards in hand with 2 lands. Fix: imprint choice (never imprint a win-line piece: Breach, LED, Brain Freeze, Oracle, Consultation, Pact) and a bargain-cast rule for Beseech (do not cast Oracle without Consultation/Pact in hand) in t6_plan_silas; a land-count/mulligan look at the list given MANA appears in every Silas game.
5. **Dack Fayden is not cast when it can be, and when it is, it is cast into held mana.** g18: Dack never appears in any decide list although Gleaming Splendor made at least six Treasures and he had 5 lands by R8 (`Dack Fayden decides: Sunscour 49% ... -> Sunscour`). g16 R6: Dack cast into Sisay's five untapped lands (`Sisay counters Dack Fayden, Helping Hand with Ertai Resurrected`); R10 skipped the Mind Stone harness that starts the Preston + Felidar + Solar loop (`Pearl Medallion 69%, Felidar Guardian 31%`), and won only on R11 by attack. g19: Dack cast R7. Fix: log why Dack is filtered (mana count of Treasures, WW requirement) and add the line-B (Preston + Felidar + Solar, one flicker) as a plan step (t6_plan_dack).

## Also noted
- g19 R7 Thrasios `Snap -> Thassa's Oracle`: bouncing an already-resolved Oracle hands the opponent the win. Rule for the look-ahead / Thrasios plan: do not bounce an Oracle on the battlefield.
- g16: Sisay cast R7 and was never able to activate to a useful target; g17/g19 Sisay's first fetch was Mount Doom at power 2 (no coloured legend). The Sisay list may need Ragavan-type legends found, or the spec's "first activation" tutor to chain.
- Three of the four combo wins were Oracle + Consultation/Pact lines, one was Kinnan + Monolith; g16 had no combo attempt at all by anyone (Dack won by attacking in R11).
