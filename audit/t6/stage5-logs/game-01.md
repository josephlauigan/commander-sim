# Game 1 (seed 500001) - Sisay, Kraum/Tymna, Dack Fayden, Rograkh/Thrasios

**Seats:** Sisay, Kraum/Tymna, Dack Fayden, Rograkh/Thrasios. **Winner:** Dack Fayden, by damage (2 combat damage, Rograkh/Thrasios to 0), **round 19**. The slowest kind of Tier 6 game: nobody went off, the table ground each other down with stax, wipes and attacks.

## Sisay (eliminated R2, on her own upkeep)
R1: Volcanic Island, Mox Amber, then `Sisay casts Pact of Negation at Ragavan, Nimble Pilferer` / `Sisay counters Ragavan, Nimble Pilferer with Pact of Negation`. R2: `Sisay can't pay for Pact of Negation and loses` / `*** Sisay is eliminated (life 0) by ?`. One land and a Mox Amber cannot pay {3}{U}{U} next upkeep, and Pact's cost is certain. She spent her whole game, and 3/4 of a seat, to counter a one-drop (Ragavan).
Earliest line: none (no Sisay cast, 2 mana on T1); the loss itself is the finding.
Tags: `CHOICE` (the Pact cast was legal and lost the game: the AI must not cast Pact of Negation unless it can pay next upkeep; the spec itself says "Pact is the only counter and next upkeep cannot pay {3}{U}{U}" is a *wait*), `RULES` is not claimed (Pact's Oracle text: "At the beginning of your next upkeep, pay {3}{U}{U}. If you don't, you lose the game." - engine matches).

## Kraum/Tymna (eliminated R18)
Played a real long game: Floodcaller R3, Tymna R4, Kraum R5 (bounced by Snap), Mnemonic Betrayal R6 (it exiled Snap and Spellseeker from Rograkh/Thrasios's graveyard, which cut that deck's Witness + Snap line), Necropotence R8 (paid 5, then 4 life). Went for no win line until R11: `Kraum/Tymna casts Gamble` -> `tutors Tainted Pact` / `Gamble discards Thassa's Oracle` (the random discard hit the Oracle), and `Chrome Mox imprints Silence`.
Earliest: R11 would have been Oracle + Tainted Pact had Gamble not discarded Oracle; before that, from R8 (7 lands, Necropotence out) a tutor for Oracle was the line. At R10 `[Kraum/Tymna decides: Gamble 100%, stop 0% -> stop [search]]` (twice) then R12/R13 `Intuition 100% ... -> stop`: both tutors sat in hand 2-4 turns with 8-9 lands open. Life was 8 by R11 (Rograkh/Thrasios swung 11).
Tags: `TUTOR` (Gamble used as the tutor: 1/N odds of binning the Oracle; fetched Pact with no Oracle in hand), `HOLD`/`CHOICE` (tutors declined at 100% in R10, R12, R13 with mana up), `MULL`/`MANA` (Chrome Mox imprinted Silence, its only protection), `STOPPED` (life: raced to 8 then 2 by creature damage).

## Dack Fayden (winner, R19)
Land-light: 1 land through R9 (City of Traitors sacrificed R4), 2 lands R9-R10. R1 Mox Opal + Damping Sphere, R2 Manifold Key, R3 Fellwar Stone, R4 `Vanquish the Horde` (wiped 5 opposing creatures, all on its opponents' sides; no Dack-side creatures lost), R7 Eldrazi Confluence killed Thrasios and Kraum, R11 finally cast Dack with 5 mana ("Dack reveals 2 creature cards: Preston, the Vanisher, Werefox Bodyguard", both donated). From R12 it chip-attacked with Dack (unblockable via Manifold Key) and Abdel Adrian, won R19.
Earliest: R4 had Fellwar Stone, Manifold Key, Mox Opal and 1 land: ~4 mana, short of Dack's 6; only R11 had 6 mana with WW.
Tags: `MANA` (stuck on 1-2 lands for 9 rounds; also self-tax: `Damping Sphere` cast R1 on its own deck's turn-1 plays), `SPEC`/`MULL` (the keep was a 1-lander with free artifacts), `HOLD` (R5 `stop (hold mana)` with Reprieve when only 1 land).

## Rograkh/Thrasios (eliminated R19)
R1 Rograkh + Chrome Mox (imprinted Noxious Revival). R2 Talon Gates, Spellseeker. R3 Cradle then Thrasios. R4 Vanquish wiped Rograkh, Spellseeker, Thrasios. R5 Thrasios again, R6 Thrasios again (March of Otherworldly Light exiled it R6), R7 Eldrazi Confluence killed Thrasios again. Thrasios was cast, and removed, four times. From R9 Thrasios dug each turn ("Thrasios reveals Birds of Paradise: drawn") and R17 it had 11 cards in hand, with Finale of Devastation, Nature's Rhythm, Eternal Witness, a Mockingbird copy of Witness on board and ~6 lands plus Sol Ring and a Mana Vault; it cast Witness, Mockingbird (as Witness), Nature's Rhythm and passed (`... -> stop (hold mana)`). It did not play Finale at 2 life life for the opponent (Kraum/Tymna at 1) nor go for line B (Snap was exiled by Kraum's Betrayal).
Earliest: line B (Cradle 5 + Witness + Snap) was assembled enough on R3-R5 (Cradle in play R3, Snap shown cast R5) but Snap was spent on Kraum at R5 and Witness was never in play. Line A (Oboro) never appears.
Tags: `STOPPED` (Vanquish R4, March R6, Reprieve R6, Confluence R7 on Thrasios/Rograkh: fair stax removal), `CHOICE` (R5 Snap spent as a bounce on Kraum instead of keeping it for line B), `HOLD` (R14-R18 repeated `stop (hold mana) 69%`), `NOLINE`? (R17: Cradle + Witness + Mockingbird-Witness in play and no Snap: line B was missing Snap, not misrecognised), `ODD` (see below).

## ODD
- R14-R18 Rograkh/Thrasios `pays {4} to untap Mana Vault` every upkeep with 5 lands: a 4-mana payment for a 3-mana return to dodge 1 damage; it nets -1 mana and uses up the turn.
- R16 `Dack Fayden casts Paladin Class`, `Swan Song` counters it: fine. R12 `Dack Fayden attacks Kraum/Tymna ... life now 2` while Kraum/Tymna had 9 lands and 3 cards: fine.
- R9 `Rograkh/Thrasios sacrifices Talon Gates of Madara to copy Chain of Vapor`: a combo land spent as a copy cost.
