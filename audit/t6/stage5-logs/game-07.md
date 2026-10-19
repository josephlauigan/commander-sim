# Game 7 (seed 500007)

Seats: 1 Rograkh/Silas, 2 Kraum/Tymna, 3 Rograkh/Thrasios, 4 Kinnan. Winner: **Kinnan, combo (Kinnan + Basalt Monolith),
round 7**. Eliminations: Thrasios R2 (own Pact), then Silas and Kraum on the combo in R7. Hand contents are not logged;
"had X" is inferred.

## Rograkh/Thrasios - dead turn 2 (ODD, repeat of game 6)
R1: Island, Rograkh cast, Silas Force of Will on it, then `Rograkh/Thrasios casts Pact of Negation at Force of Will` (1 land,
Island). R2: `Rograkh/Thrasios can't pay for Pact of Negation and loses` / `*** Rograkh/Thrasios is eliminated (life 0) by ?`.
Oracle: "At the beginning of your next upkeep, pay {3}{U}{U}. If you don't, you lose the game." Paying needs 5 mana on turn 2.
Tags: `CHOICE`, `ODD`. Second game in this slice where a deck kills itself with Pact of Negation; the AI saves a Rograkh that
costs {0} and could be recast anyway.

## Rograkh/Silas (turbo) - out R7, no line
R1 fetch land only (cracked Mire; Force of Will cast on Rograkh/Thrasios' Rograkh, countered by that Pact). R2 Grinding
Station; Rograkh again passed: `[Rograkh/Silas decides: Rograkh, Son of Rohgahh 100%, stop 0% -> stop [search]]` (R2, R3; cast first
R4 after Silas Renn). R3 Birgi. R4 Defense Grid (countered by Force of Negation). R6 **Beseech the Mirror tutored Thassa's
Oracle** (`Rograkh/Silas tutors Thassa's Oracle`) but not bargained/cast, and no Consultation, Pact or other half of the combo
appears in any decide list. Earliest go: none: it held Oracle without Consultation/Pact/Breach. Tags: `MULL` (kept a hand
with 1 land and no fast mana on turn 1, which cast nothing R1; 5 cards R2), `TUTOR` (Oracle fetched with no way to use it),
`CHOICE` (Rograkh passed three times), `STOPPED` (Force of Negation on Defense Grid; Endurance R5 put 9 graveyard cards
on the bottom, removing Breach/Brain Freeze targets).

## Kraum/Tymna (blue breach midrange) - out R7, had the win and gave Kinnan the tutor
R2 hold (`Borne Upon a Wind 100% -> stop (hold mana)`), R3 Rhystic Study, R4 Tidecaller + Tymna, R5 Ragavan and **Wishclaw Talisman**:
`Kraum/Tymna tutors Thassa's Oracle` then `Kinnan gains control of Wishclaw Talisman`. R7: Vampiric Tutor put **Tainted Pact**
on top, drawn on R7; with Oracle in hand that is line 1 (Oracle + Pact = {U}{U}{1}{B}, 4 mana) and Kraum had 4 lands (Tundra,
Starting Town, Otawara, Badlands). The R7 decide line was `[Kraum/Tymna decides: Flash Photography 42%, stop (hold mana) 30%,
Smothering Tithe 28% -> Smothering Tithe]`, no combo offered. Earliest go: **R7**. Hand: 8 cards. Tags: `NOLINE` (or
`CHOICE`; Oracle + Pact never appears as an option, inferred since the log has no `ready` line), `TUTOR` (Wishclaw cast
on R5 with no way to finish; spec: "Wishclaw only on the win turn since an opponent gets it"). Wishclaw then let Kinnan
tutor Basalt Monolith R7, which won the game. This is a self-inflicted assist: `CHOICE`.

## Kinnan (mana combo) - WON R7 (spec turn 3-5)
R1 Elves, R2 Gene Pollinator + Birds, R3 passed with `Kinnan 68% ... -> stop`, R4 Trickster + Remora at instant speed (EOT), R4 Flesh
Duplicate, Snapback on Silas Renn, R5 Endurance, **Kinnan cast R6**, R7 used the Wishclaw Kraum had cast to tutor Basalt
Monolith, cast it and went: `[Kinnan decides: combo: Kinnan + Basalt Monolith 100% ... -> combo: Kinnan + Basalt Monolith]`
then `Kinnan goes for Kinnan + Basalt Monolith`. Recognition worked. Earliest: R2-R3: Kinnan could have been cast R2 (Caverns,
Elves + Pollinator + Birds gave mana) but was held; Kinnan R6 was `Kinnan, Bonder Prodigy 67% ... -> stop` then cast
anyway. Tags: `HOLD` (Kinnan commander held until R6), `TUTOR` (Monolith only arrived with the opponent's Wishclaw).

## Why it took 7 rounds
Table was thin: one seat died to a Pact on R2, the Kinnan deck waited for its commander and its Monolith tutor, and the
two blue seats fetched Oracle with no second half.
