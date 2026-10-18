# Game 6 (seed 500006)

Seats: 1 Rograkh/Thrasios, 2 Kraum/Tymna, 3 Sisay, 4 Kinnan. Winner: **Sisay, damage, round 12** (26 damage into Kinnan with
8 creatures). Eliminations: Kraum/Tymna R1 (own Pact), Thrasios R11, Kinnan R12. Hand contents are not logged; "had X" is
inferred from decide lines and hand sizes.

## Kraum/Tymna - dead on turn 1 (ODD, big)
R1 `Kraum/Tymna casts Pact of Negation at Rograkh, Son of Rohgahh`, then on its own turn: `Kraum/Tymna can't pay for Pact of
Negation and loses` / `*** Kraum/Tymna is eliminated (life 0) by ?`. Pact of Negation: "At the beginning of your next upkeep,
pay {3}{U}{U}. If you don't, you lose the game." The seat had 0 lands and cast the Pact against a free Rograkh. The attempt
cannot be paid on its own upkeep without 5 mana, and an unpayable Pact is a game loss. Tags: `CHOICE` (the counter
decision never checks that the upkeep cost can be paid), `ODD`. This removes a whole seat on turn 1 and, over many games,
inflates the others' win rate; it is the highest-value fix in this game.

## Rograkh/Thrasios (creature combo) - out R11, never close
R1: Command Tower, Rograkh countered by Pact of Negation, then Mox Diamond and Rograkh again (resolved). The hand sizes
(7, then 5 at R2 after Tower, Diamond, Rograkh) show Mox Diamond discarded the second land: **1 land R1-R6** (`turn 6: ...
5 cards in hand, 1 lands`), first new land R7 (Forest). Thrasios cast R2 (72%), Allosaurus Shepherd R3, Magus R3, Flesh
Duplicate R4, Phantasmal Image R5: a board but no Cradle, no mana for Thrasios' {4}. From R8 it passed with
`[... Thrasios, Triton Hero 43%, Rograkh, Son of Rohgahh 33%, stop (hold mana) 22%, Spellseeker 2% -> stop (hold mana)]` and
`stop (hold mana) 92%` R8-R10, holding Force of Negation/Flare of Denial and using them on Finale (R9), Marvin (R10), Ioreth (R6).
Earliest line: none (no Cradle, 2 lands). Tags: `MANA`, `MULL` (2-land 7 plus Mox Diamond that discards one of them),
`CHOICE` (Mox Diamond discarding the second land), `HOLD` (counters held instead of developing, which worked as defence only).

## Sisay (legends toolbox) - WON R12, spec targets turn 3-4 threat, win by loop
R2 Lotho (Mount Doom as the land), R3 Sisay herself (`Sisay, Weatherlight Captain 65% -> Sisay`, after one `stop (hold mana)`),
R4 Sisay fetched **Derevi**, R5 **Emiel**, R6 Magda, R9 **Gaea's Cradle**, R12 **Agatha's Soul Cauldron**. So Derevi + Emiel
(line 1/2) were in play on R5, Cradle R9, Cauldron R12. Earliest go: R12 for line 2 (Cradle + Derevi + Emiel +
Cauldron), but it was never used as a loop; the game ended by an 8-creature attack the same turn. Rounds R7 and R8 show
`[Sisay decides: Sisay: Gaea's Cradle 100%, Emiel the Blessed ability 0%, stop 0% -> stop [search]]` - the best option at
100% rejected with 6 lands, then taken R9 (`ODD`: same board, maybe colour availability; the log does not say). Each
activation costs WUBRG and the AI spent them on a different legend each turn (Magda, Ioreth, Shang-Chi, Kinnan, Teferi, Tyvar,
Cauldron, Ragavan) - value, not loop assembly. Tags: `SPEC` (line 1 needs Bloom Tender + Derevi + Emiel; Bloom Tender was cast
R4 but the loop was never offered, `NOLINE` not shown since no `ready` line is logged), `MANA`.

## Kinnan (mana combo, spec win turn 3-5) - out R12
R1 Llanowar Elves, R2 Endurance, **Kinnan not cast until R4** (R2: `Kinnan, Bonder Prodigy 65% ... Endurance 0% ->
Endurance`; R3: `... -> stop (hold mana)` then attacked with Elves). R4 Kinnan + Elvish Mystic, R5-R9 `stop (hold mana)`
five times with `Flesh Duplicate 77%` the top option. R9: Chrome Mox, Finale of Devastation, which Rograkh/Thrasios
countered with Force of Negation. Earliest go: no loop piece (Basalt Monolith, Hullbreaker, Mastermind) ever showed up;
Finale was cast as a spell at R9 with 5 lands. Tags: `HOLD` (hold mana R3-R8; Kinnan could have come R2 with Tropical
Island + Mana Confluence + Elves), `STOPPED` (Force of Negation on Finale), `SPEC`.

## Why the game took 12 rounds
Sisay was the only seat that developed every turn; Thrasios was mana-screwed, Kinnan held mana, Kraum died to its own Pact.
Sisay's loop (Derevi + Emiel + Cradle + Cauldron) was in place on R12 but the AI never needed it.
