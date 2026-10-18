# Game 9 (seed 500009)

Seats: 1 Kraum/Tymna, 2 Sisay, 3 Rograkh/Thrasios, 4 Kinnan. Winner: **Sisay, damage, round 13** (the Thrasios seat is
"eliminated (life 18) by Sisay" with no damage line; see ODD). Eliminations: Kraum/Tymna R9, Kinnan R10 (both by the
Thrasios deck's attackers), Thrasios R13. Hand contents are not logged; "had X" is inferred.

## Kraum/Tymna - played nothing for nine rounds
R1: Gemstone Caverns, Mana Vault (`Kinnan counters Mana Vault with Mental Misstep`), then it pitched into Force of Will on
Thrasios (4 cards in hand R2). From R2 to R9 the log has no cast at all: `Kraum/Tymna turn 2: ... 1 lands`, R4 `6 cards in
hand, 1 lands`, R5 Cavern of Souls and Simian Spirit Guide only, R9 `7 cards in hand, 2 lands`, dead R9 at 4 life. The deck
had mana only through the countered Vault and 1-2 lands. Earliest go: none. Tags: `MULL` (kept a hand that needed Mana
Vault and one land; the spec says keep fast-mana hands, but a Misstep-able Vault is the only mana), `MANA`, `STOPPED`
(Mental Misstep on the Vault), `ODD` (the engine/search printed no decide lines for this seat for 8 turns, so I cannot tell
whether it held cards on purpose; the log would show a `stop (hold mana)` line if it had considered a cast).

## Rograkh/Thrasios (creature combo) - nearly won on R7, lost by decking or attrition R13
R1 Petal + Thrasios (Force of Will'd by Kraum), R2 Earthcraft + Rograkh (Expedition Map Mindbreak-Trapped), R3 Borne Upon a Wind +
Springleaf Drum, R4 Springheart Nantuko, **R5 Sowing Mycospawn fetched Gaea's Cradle**, Cloud of Faeries, Thrasios resolved.
R6: Snap on Kinnan (Pact of Negation'd), Flesh Duplicate. **R7: `combo: Oboro Breezecaller + Gaea's Cradle + Talon Gates of Madara
99%` then `Rograkh/Thrasios goes for ...`** - Oboro cast, `Kinnan counters Oboro Breezecaller with Force of Will`,
`...stops at step 1 (cast Oboro Breezecaller)`. Recognition worked. Hand was 66 cards (see ODD) with Cradle in play. After the
counter it kept casting spells (Chatterstorm, Moxen, Allosaurus Shepherd) and attacked with 9 creatures. R8: harmonize Nature's Rhythm
X=11, Shifting Woodland as Oboro, 17-creature attack for 20 (Kraum to 4), R9 and R10 kills. No second Oboro attempt: R11-R12
`Rograkh, Son of Rohgahh 97% ... -> stop (hold mana)` with 7 cards in hand and 6 lands, and it never tried line B (Witness +
Snap + Cradle) or finishers.
- Earliest go: **R5-R6** (Cradle R5, Thrasios + 3 creatures, 5 lands R6) if Oboro was in hand; the combo is first offered R7.
  Tags: `STOPPED` (Force of Will on Oboro), `HOLD` (R10-R12 passes with a full hand and no follow-up), `CHOICE` (it never
  recast Oboro after the counter; R12 `Shifting Woodland becomes Oboro Breezecaller 0%`), `SPEC`.

## Sisay (legends toolbox) - WON R13, but only because it was last standing
Stuck on 1-2 lands R2-R9 with 6-7 cards (`Sisay turn 7: ... 7 cards in hand, 2 lands`), holding Redirect Lightning
(`Redirect Lightning 97% -> stop`, R3-R6). R5 Shang-Chi, R7 Tataru Taru, R8 Enduring Vitality + Bloom Tender, R9 Magda, **Sisay
herself cast R10**, Emiel activated R10, Derevi cast R10. From R11 `Sisay: Mount Doom 100% ... -> stop (hold mana)` every
turn: the best option is the Sisay activation fetching Mount Doom, but it is not taken R11-R13 (`ODD`: probably colour or power). Earliest
go: none; Derevi + Emiel arrived R10 but Bloom Tender + Derevi + Emiel loop (line 1) was never offered. Tags: `MULL`/`MANA`
(1-2 lands for nine rounds), `HOLD` (holding Redirect Lightning mana rather than developing), `SPEC`.

## Kinnan (mana combo) - out R10
R1 Mystic, R2 Elves, Kinnan R3 (on 1 land + Elves), Sylvan Library; R4 Mystic Remora (Fierce Guardianship'd); R5 Copy Artifact
on Springleaf Drum and Consecrated Sphinx; R6 Finale of Devastation (log then shows Endurance bottoming Sisay's graveyard),
Mox Diamond, Fellwar Stone. R7 Flash Photography (copy of Sphinx, twice) and `Kinnan casts Force of Will` on Oboro. No
Monolith, Hullbreaker, or Mastermind line appeared in any decide list. The Library cost life each turn: `Kinnan pays 4 life for
a Sylvan Library card` R4 (x2), R5, R6; life 37 to 20 by R7. Earliest go: none (no loop pieces; `Trophy Mage 57%` at R9 is the
route to Monolith but came late). Tags: `SPEC`, `CHOICE` (Sylvan Library paid 4 life repeatedly against a table that attacks
for 17+), `STOPPED` (Rograkh/Thrasios chain and counters).

## ODD (engine)
- R7 `Rograkh/Thrasios turn 7: life 37, 66 cards in hand` and `Kinnan turn 7: ... 58 cards in hand`. No draw effect in the
  log explains it (Thrasios' draw was not used that turn); both are back to normal at R8 (6 cards), so either a display bug
  or a mass-draw bug hit two seats between R6 and R7. If real, it is a `RULES` problem worth a trace.
- R13 `*** Rograkh/Thrasios is eliminated (life 18) by Sisay` right after `Urza's Saga: ... makes a Construct (7/7)` with no
  damage or loss line. Cause unknown (decking is my guess given the R7 hand counts, not checked).

## Why the game took 13 rounds
Three of four seats were on one or two lands for most of the game; the one deck with a board (Thrasios) lost its only
go-for-it turn to Force of Will and then dropped into attack-and-hold.
