# Stage 5 log review, slice b (games 5-9)

Winners: G5 Rograkh/Thrasios (R18), G6 Sisay (R12), G7 Kinnan combo (R7), G8 Kinnan combo (R7), G9 Sisay (R13). Decks seen:
Rograkh/Silas x3 (5,7,8), Rograkh/Thrasios x4 (5,6,7,9), Kinnan x5, Kraum/Tymna x3 (6,7,9), Sisay x3 (6,8,9), Dack x2 (5,8).
Hand contents are not in the log, so "could have won on turn N" comes from decide lines and hand sizes.

## Cause tags per deck (count of games where the tag applies)

| Deck (games) | MULL | MANA | TUTOR | HOLD | NOLINE | CHOICE | STOPPED | RULES | SPEC | ODD |
|---|---|---|---|---|---|---|---|---|---|---|
| Rograkh/Silas (5,7,8) | 2 | 1 | 1 | 0 | 0 | 3 | 2 | 0 | 1 | 1 |
| Rograkh/Thrasios (5,6,7,9) | 1 | 1 | 1 | 3 | 0 | 4 | 1 | 0 | 2 | 2 |
| Kinnan (5,6,7,8,9) | 1 | 1 | 1 | 4 | 0 | 3 | 3 | 0 | 3 | 0 |
| Kraum/Tymna (6,7,9) | 1 | 1 | 1 | 0 | 1 | 2 | 1 | 0 | 0 | 2 |
| Sisay (6,8,9) | 2 | 3 | 0 | 1 | 0 | 0 | 1 | 0 | 2 | 1 |
| Dack (5,8) | 0 | 1 | 0 | 2 | 0 | 2 | 0 | 0 | 1 | 1 |

(Counts are games per deck where I applied the tag; per-game files give the evidence.)

## Top problems

1. **Pact of Negation kills its owner on the next upkeep.** G6 R1: `Kraum/Tymna can't pay for Pact of Negation and loses`;
   G7 R2: `Rograkh/Thrasios can't pay for Pact of Negation and loses`. Both seats cast the Pact with 0-1 lands and could not pay
   {3}{U}{U}. Two of 15 seats in five games removed themselves before turn 3 (G8: Silas cast one on R6 and would have paid on R7
   with 3 lands + moxen at best). Fix: in the free-counter code (Pact of Negation) allow it only if the caster will have
   {3}{U}{U} at the next upkeep (lands + untapped mana sources + fast mana), or the counter saves a game-deciding spell
   (a win attempt). Slot: shared counter/hold logic, not a deck plan file.
2. **Kinnan delays its commander and the Monolith combo.** G8 R4: `Basalt Monolith 100% ... -> Dizzy Spell: transmute` after
   `Sol Ring 54%` (Monolith was castable; won R7 instead of R4); G8 R6 `Basalt Monolith 83% ... -> Chord of Calling`; G6 R2-R3
   and G7 R2-R5 `Kinnan, Bonder Prodigy 65% ... -> stop (hold mana)` (commander held 2-4 turns). Fix: in
   `t6_plan_kinnan.rs`, a `prio` boost so Basalt Monolith (with Kinnan in play) and Kinnan himself outrank transmute/Chord/
   Sylvan Library lines, and a `go_now` yes when Kinnan + Monolith (or two rocks + Horror) is castable this turn.
3. **Self-damaging engines are not priced.** G5: Dack taps The One Ring every turn (`burden 3, 4, 5`), losing about 30 life, and
   dies at life 3 on R18 (`Dack Fayden loses 5 life to The One Ring's burden (life 3)`). G9: Kinnan `pays 4 life for a Sylvan
   Library card` four times in four turns, 37 to 20 life. Fix: look-ahead value for The One Ring's tap and Sylvan Library
   payments should subtract the next-upkeep burden and cap life paid when opponents hold attackers; in `t6_plan_dack.rs`
   `hold_up`/`prio` stop activating Ring above burden 2-3.
4. **Oracle/Wishclaw tutors fetched with no second half, and given away.** G7 R5: Kraum/Tymna cast Wishclaw Talisman, tutored
   Oracle, and `Kinnan gains control of Wishclaw Talisman`; Kinnan then tutored Basalt Monolith with it and won R7. G7 R6:
   `Rograkh/Silas tutors Thassa's Oracle` (Beseech) with no Consultation or Pact in sight. Also in G7 Kraum held Oracle +
   Tainted Pact + 4 lands on R7 (line 1 of its spec, 4 mana) and the decide line is `Flash Photography 42%, stop (hold mana)
   30%, Smothering Tithe 28% -> Smothering Tithe`. Fix: `t6_plan_kraum.rs` should mark Oracle + Consultation/Pact in hand with
   enough mana as a `go` line (NOLINE check), and Wishclaw should be held until the win turn per the spec.
5. **Mulligan and land screw for Rograkh/Thrasios, Sisay, Kraum.** G6 Thrasios 1 land for six turns after Mox Diamond
   discarded the second land (5 cards in hand); G8 Sisay 1 land through R6; G9 Sisay 1-2 lands R2-R9, Kraum 1-2 lands R1-R9 with no
   casts after the countered Mana Vault. Fix: the `keep` functions (e.g. `t6_plan_rograkh_silas.rs::keep`, the equivalents for
   Thrasios, Sisay and Kraum) should ship 1-land hands unless they hold 3+ real fast mana, and Mox Diamond should not be
   cast when it discards the second land of a 2-land hand.

## Smaller findings
- Rograkh/Silas does not cast the free commander on turns 1-3 (G5 R1-R4, G7 R2-R3): `Rograkh, Son of Rohgahh 100% -> stop`.
  The spec says always cast it turn 1 (it enables Mox Amber, Fierce Guardianship, Culling the Weak).
- G5 Thrasios deck held Thrasios R4-R6 at 97% `-> stop`, cast it R9; G9 it was countered on the single go-for-it turn (R7) and never retried.
- G9 `66 cards in hand` (Thrasios) and `58 cards in hand` (Kinnan) at R7 with no logged cause, plus an elimination at life 18 by Sisay
  with no damage line: possible engine bug (`RULES`), needs a trace.
- Dack's Dack flip fails by design in this list (G5: no Felidar + Preston + Solar trio), and G8 R6 it cast The One Ring over the
  commander ranked 89%.
