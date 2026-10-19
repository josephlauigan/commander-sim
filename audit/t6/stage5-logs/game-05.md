# Game 5 (seed 500005)

Seats: 1 Rograkh/Silas, 2 Dack Fayden, 3 Kinnan, 4 Rograkh/Thrasios. Winner: **Rograkh/Thrasios, damage, round 18**
(Dack died to its own One Ring burden plus an attack). Eliminations: Silas R7, Kinnan R9, Dack R18. The log shows hand
sizes, not hand contents, so "had X" below is inferred from what was cast or offered in the `decides` lines.

## Rograkh/Silas (turbo, spec: win turns 1-3) - out R7, never went off
Turn 1: Gemstone Caverns, Mox Opal, no Rograkh. R2: City of Brass, then `[Rograkh/Silas decides: Talisman of Dominance
60%, Rograkh, Son of Rohgahh 23%, Demonic Counsel 17%, stop 0% -> stop [search]]` (twice) - passed with 2 lands and
Opal doing nothing (one artifact, no metalcraft). R3: cast **Timetwister** at 2 lands with no payoff (`... -> Timetwister`),
which refilled all four hands (Dack went from 5 to 7 cards). R3/R4: Rograkh (cost {0}) was the only option at 100% and was
passed: `[... Rograkh, Son of Rohgahh 100%, stop 0% -> stop [search]]`. Rograkh was first cast R5, Oracle came off Beseech
the Mirror R6 (free via bargain) but no Consultation/Pact followed, so it sat as a 1/3 and was exiled by Dack's flip.
- Earliest line: none visible. Oracle was in the deck by R6 with 5 lands and cards in hand; no Consultation/Pact
  appeared in any decide list, so the turn-6 Oracle was a bare 1/3. `MULL` (kept a hand that found one land by R2 and no
  fast mana except Opal; 6 cards in hand at R2 with nothing cast), `MANA` (Opal without artifacts, 2 lands on R2),
  `CHOICE` (Timetwister into a table with 4 hands; Rograkh declined four turns in a row, spec says "always cast on
  turn 1"), `STOPPED` (Mental Misstep on Vampiric Tutor R5; Dack's Touch the Spirit Realm exiled Silas Renn R4).
- `ODD`: casting the free commander is declined repeatedly while the spec treats it as the enabler for Mox Amber, Culling,
  Fierce Guardianship. It was likely held as "do not feed removal"; no rule in `t6_plan_rograkh_silas.rs` asks for it.

## Dack Fayden (white stax / Dack flip) - alive to R18
R1 Petal + Blind Obedience, R2 Workshop + Mind Stone, R3 Tezzeret (-3 fetched Sol Ring), R4 Apple of Eden, Touch the Spirit
Realm, Manifold Key. Dack himself first cast R5 behind Silence. Earliest go: **R4** - lands Plains, Plains, Ruins of
Trokair, Mind Stone and Sol Ring give 6 with WW, yet the R4 decide lines never list Dack (`Touch the Spirit Realm 59%,
Manifold Key 15%, Apple of Eden 15%, Enlightened Tutor 12%`); `HOLD` (protection not yet in hand: Silence appears R5).
The flip then gave the table its own creatures: R5 `Dack reveals 3 creature cards: Werefox Bodyguard, Felidar Guardian,
Radiant Solar`, donated one each; none were the Felidar+Preston+Solar trio (`SPEC`, spec says 2.9%), and Dack was bounced
(Otawara) and exiled. Radiant Solar later killed Dack's own Tezzeret R9. Dack flipped four more times (R6, R12-R14) with
`Dack reveals 0 creature cards` twice. Wins lost to: Thrasios deck's attacks.
- `ODD` (important): Dack taps The One Ring every turn (`R10 taps The One Ring: burden 3`, `R12 burden 4`, `burden 5`).
  Burden upkeep loss ran 1,2,2,3,4,5,5,5 and `R18 Dack loses 5 life ... (life 3)` then died. About 30 life self-inflicted.
  `CHOICE`: the look-ahead values the draw and not the growing upkeep cost.
- Tags: `HOLD`, `SPEC`, `CHOICE`.

## Kinnan (mana combo, spec: win turns 3-5) - out R9
R1 Foothills, Fyndhorn Elves, Mana Vault; Force of Will on Rograkh/Thrasios. Then **stuck on 2 lands R2-R8** (`Kinnan turn
8: life 5, 6 cards in hand, 2 lands`) with 4-7 cards in hand. Kinnan himself was never cast: R2 `[Kinnan decides: Kinnan,
Bonder Prodigy 62%, Sylvan Library 38%, stop 0% -> stop [search]]`, R3 `Kinnan, Bonder Prodigy 45% ... -> stop (hold mana)`,
R4 `... -> stop (hold mana)` then cast Thrasios + Chrome Mox instead. Earliest go: R2 (Elves G, Tropical Island U, Vault 3
mana: Kinnan costs {G}{U}). Its life fell from 40 to 5 through Mana Vault upkeep damage (life 38 at R2), creature attacks from Dack and Thrasios, and then Dack killed it R9; after
the Vault. Tags: `MANA` (2 lands for 8 turns, no more land plays), `HOLD`
(`stop (hold mana)` over casting the commander three turns), `MULL` (2-land keep with Vault).

## Rograkh/Thrasios (creature combo) - WON R18
R1 Rograkh countered by Force of Will. R2 **Gamble tutored Gaea's Cradle and then discarded it at random**
(`tutors Gaea's Cradle` / `Gamble discards Gaea's Cradle`): the line's key land went to the graveyard with no recursion
(`TUTOR`, Gamble's discard is by rule, but fetching Cradle with Gamble at 2 lands is a risk the AI took at 32/25/21/21%).
Thrasios (2 mana) was passed R4-R6: `[Rograkh/Thrasios decides: Thrasios, Triton Hero 97%, Enduring Vitality 3%, stop 0% -> stop
[search]]` (R4), 96% R5, 97% R6, cast first on R9 (spec: turn 3-4). `CHOICE`/`HOLD`. Earliest line: none (no Cradle, no Oboro,
no Eternal Witness shown). It won because the others killed each other and it kept 9 lands and Thrasios/Formidable
Speaker beats. Tags: `TUTOR`, `CHOICE`, `SPEC`.

## Why the game took 18 rounds
No deck assembled its line. The two turbo/combo seats made weak early choices (Silas Timetwister, Kinnan stuck at 2 lands
and never casting Kinnan, Thrasios not cast), and the stax deck's flip donated creatures instead of winning.
