# Game 18 (seed 500018) - Tier 6 within-tier, conservative profile, look-ahead AI

Seats: Rograkh/Silas, Kraum/Tymna, Dack Fayden, Kinnan. **Winner: Rograkh/Silas, round 9, by combo** (Thassa's Oracle + Demonic Consultation; Kinnan's Flusterstorm was Mental Missteped). Log: `/tmp/claude-1000/t6logs/g18.txt`.

## Rograkh/Silas (winner, R9)
R1 Underground Sea, Mystical Tutor (put Demonic Consultation on top), Gitaxian Probe. R2 Otawara, Rograkh. R3 Borne Upon a Wind at end of turn, Grim Monolith, Talisman of Dominance. R4 Silas Renn; Dack's Eldrazi Confluence then killed Rograkh, Silas and Kinnan. R5-R7 `stop (hold mana)` on 3 lands (`Grinding Station 58% ... -> stop (hold mana)`), Red Elemental Blast on Kinnan's Thrasios at end of R6, Rograkh recast R7. R8 Mnemonic Betrayal, exiling 23 cards from three graveyards, then Demonic Tutor from Kraum's graveyard for Thassa's Oracle. R9 `Rograkh/Silas goes for Thassa's Oracle + Demonic Consultation / Tainted Pact`.
Earliest line: **R9**. Consultation was in hand from R2 (Mystical Tutor on R1, drawn). Oracle came only on R8, because the deck had no tutor in hand until Betrayal gave it Kraum's Demonic Tutor; at R8 the mana (3 lands, Grim Monolith, Talisman) was used up by Betrayal + Tutor, so it could not cast the pair the same turn. Before that it sat on 4-7 cards.
Tags: `MANA` (3 lands from R4 to R8, no land drops with 5-7 cards in hand), `TUTOR` (R1 Mystical Tutor for Consultation is right, but the deck then held Consultation for seven turns with no way to find Oracle; Gitaxian Probe never became a plan), `HOLD` (R5-R7 held mana for REB/Misstep, spec "wait a turn when you expect interaction", fair but its pieces were incomplete anyway), `STOPPED` (Eldrazi Confluence killed Rograkh and Silas R4).

## Kraum/Tymna (R9 eliminated, 29 life)
R2 Vampiric Tutor at end of the Silas turn (Thassa's Oracle on top), then Demonic Tutor for Tainted Pact on its own turn 2 (`Kraum/Tymna tutors Tainted Pact`). R3 Mockingbird copying Kinnan's Thrasios. R4 held (`Tymna the Weaver 90% ... -> stop (hold mana)`), R5 Sol Ring and Force of Negation on Dack's One Ring, R6 Smothering Tithe, R7 Intuition, Tymna, Esper Sentinel, Enlightened Tutor for Lion's Eye Diamond.
Earliest line: **R5**, from the log: Oracle (Vampiric, R2) and Tainted Pact (R2) are both fetched by the end of its second turn; on R5 it had 3 lands + Sol Ring (about 4 mana left after the Sol Ring: {U}{U} + {1}{B}). The decide lists R5-R8 never show `combo: Thassa's Oracle + Demonic Consultation / Tainted Pact` (it prints for Silas on R9 and Kraum in game 15 only when the AI is going). So either the combo was `ready` and go_ok refused with no log line, or the pieces were gone. I cannot see its hand; the graveyard exile on R8 lists no Oracle or Pact, so they were probably still in hand.
Tags: `HOLD` or `NOLINE` (cannot be told apart from this log: ask for a "combo ready / go_ok false" line in the trace), `MANA` (stuck on 2-3 lands R3-R5, spec turn 3-4 plan), `CHOICE` (R7 spent its mana on Tymna, Sentinel and a third tutor, Enlightened for LED, instead of the pair it already held), `TUTOR` (R7 LED is a Breach piece with no Breach).

## Dack Fayden (R9 eliminated)
R1 Lotus Petal, Gleaming Splendor (feeds Treasures from opponents' second draws; it made at least six by R8). R2 Void Mirror, R3 Monumental Henge, R4 `Eldrazi Confluence`: killed the table's three creatures (good, kills Kinnan, Silas, Rograkh), R5 The One Ring (Force of Negation), Cloudshift with nothing to flicker, R6 Disruptor Flute, R8 Sunscour, Pithing Needle on Silas Renn. Mox Opal sat at `Mox Opal 100% ... -> stop` for six turns.
Earliest line: **R7/R8**. With 5 lands and 4-6 Treasures it had well over 6 mana with WW on R8, yet Dack Fayden himself never appears in any decide list this game (he does in game 16 and 19), so the AI never even scored the cast. Spec: "Go when 6 mana with WW ... or opponents tapped out". Silas's 3 lands and Kinnan's 6 lands + Flusterstorm were visible.
Tags: `MANA`/`ODD` (Treasures not counted, or Dack filtered before scoring), `HOLD` (if the filter is go_ok, it holds forever against open blue mana), `CHOICE` (T1 Gleaming Splendor and R5 Cloudshift into nothing, Sunscour into its own empty board R8), `STOPPED` (Force of Negation on The One Ring).

## Kinnan (R9 eliminated)
R1 Lotus Petal and Thrasios (`Kinnan 98%, Thrasios 2% -> Thrasios`), R2 Kinnan, R3 Dizzy Spell transmute. R4 Dack's Confluence killed Kinnan; R4 Mystic Remora. R5-R8 hand 9 to 11 cards: `Mox Amber 98% ... -> stop (hold mana)` (Mox Amber needs a legendary permanent and Kinnan was in the command zone), Cabbage Merchant, Mockingbird, Halfling, Elvish Mystic, Volatile Stormdrake. Kinnan was never recast (he shows only at 19% R7 and 48% R8).
Earliest line: none. Line A needs Kinnan + Basalt Monolith + an outlet: Kinnan died R4, Basalt Monolith never appeared.
Tags: `STOPPED` (Eldrazi Confluence), `CHOICE` (never recast the commander for {2}{G}{U}+tax on 4-6 lands R5-R8; the dork pile was not enough alone), `MANA` (cards stuck: 11 cards in hand at R8).

## ODD
- Hand sizes 9-11 for Kinnan and Kraum with no discard line (see game 15).
- Silas's `Mnemonic Betrayal` R8 exiled all three graveyards, including Kinnan's Hullbreaker Horror and Thrasios; the Silas deck is the only one of the four that cast cards out of the exiled pile and it found its win piece that way.
- Printed scores vs choice: Dack R3 `Mox Opal 100% ... -> stop`, Kinnan R1 `Kinnan 98% ... -> Thrasios`.
