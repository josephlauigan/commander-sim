# Tier 6 annotated games: why the table is slow (2026-10-18)

Twenty Tier 6 games (`commander-sim --calibrate within --pool t6 --trace N`, N = 0-19, conservative profile, t6-cedh
at 31c3da6), annotated deck by deck in `game-00.md` … `game-19.md`; the four slice summaries are `summary-a.md` …
`summary-d.md`. The log prints plays and the AI's choices but not hands, so "could have won on turn X" is judged from
the visible mana, tutors and choice lines.

**Winners:** Kinnan 5, Sisay 4, Dack 4, Rograkh/Silas 3, Kraum/Tymna 2, Rograkh/Thrasios 2. Median win round 9 (the
specs aim for turns 3-5). Sisay's four and two of Dack's were combat wins at rounds 11-19.

## Why decks didn't win sooner (cause tags, counted per deck per game)

| Cause | Count | What it means |
| --- | --- | --- |
| CHOICE | ~60 | the look-ahead picked a worse play than its own top-scored one or the spec's |
| MANA | ~49 | stuck on 1-3 lands, mana unused, fast mana held |
| STOPPED | ~42 | a fair answer from an opponent (the table interacting, as it should) |
| HOLD | ~37 | the line or the commander was available and the AI waited |
| SPEC | ~34 | the deck didn't have its pieces this game (variance) |
| MULL | ~19 | kept a hand the spec would ship (mostly 1-landers) |
| TUTOR | ~18 | wrong tutor target, or the tutored card thrown away |
| NOLINE | ~5 | the line was assembled but never offered |
| RULES | 3 found | see below; fixed |

## The problems, most important first

1. **The look-ahead overrides the deck plans (CHOICE + HOLD, ~100 tags).** The plan slots only set a prior; the
   search then picks something else, and often the "stop (hold mana)" option.
   - Sisay is cast late or never, although its plan gives it 92 (g0, g2, g3, g13, g17). In g3 R10-R12 Sisay had both
     lines and Mount Doom on the table, and `Sisay: Mount Doom 100%` lost to `stop (hold mana)`.
   - Kinnan: `Kinnan 79%, Chord 21% -> Chord` (g17 R5), Sol Ring and a transmute over Basalt Monolith at 100% (g8
     R4), Borne over Kinnan on turn 1 (g16).
   - Dack: `Dack 70% … -> Swords to Plowshares` and then nothing cast for two turns with 6-8 mana (g13); never scored
     with 5 lands and 6 Treasures (g18).
   - Rograkh/Silas skips the free Rograkh on turns 1-3 (g5).
   Fix direction: for Tier 6, let a plan mark a play as "do it now" (commander out, the line's piece) so the search
   cannot outvote it, or weight the plan prior far more heavily in Tier 6 searches.
2. **Pact of Negation cast without the upkeep mana (3 losses: g1 Sisay, g6 Kraum, g7 Thrasios).** Pact's {0} fell
   through to the plain "can it be paid" check after the upkeep guard. **Fixed** (Tier 6).
3. **Two Consecrated Sphinxes drew both players to 58-66 cards (g9)**, then one decked. **Fixed** (Tier 6: stop taking
   the draws at 10 cards in hand).
4. **Ranger-Captain of Eos fetched Thassa's Oracle (g11, g12).** It finds a creature with mana value 1 or less; the
   parsed ability had lost the limit. **Fixed** for every tier (Aurelia t3, Heliod and Chulane t4, Winota t5 play it).
5. **Land-light keeps (MULL + MANA).** Dack, Kraum, Sisay and both Rograkh decks keep 1-landers and stay on 1-3 lands
   for 6-9 rounds; Thrasios' Mox Diamond discarded a land on a 2-lander (g6). The keep rules count fast mana as
   sources too generously.
6. **Tutors and combo pieces thrown away (TUTOR).** Gamble's random discard binned the Oracle just fetched (g1, g4);
   Beseech cast Oracle free with no Consultation or Pact in hand (g2, g12, g19); Chrome Mox imprinted Demonic
   Consultation (g11) and Underworld Breach (g19); Invasion of Ikoria X=14 fetched Faerie Mastermind over Oboro (g2);
   Wishclaw Talisman, cast with no win that turn, handed the opponents a tutor (g7, g17).
7. **Lines held or not offered (NOLINE).** Kraum held Oracle + Tainted Pact from R2 and never offered it (g18, also
   g7, g4); Sisay had both lines and was never offered either (g3).
8. **Self-damage not priced.** Dack tapped The One Ring until it died at 3 life (g5); Kinnan paid 4 life per Sylvan
   Library card (g9).

Smaller notes: the Dack flip kill prints no steps (it is a pile lookup), so a log can't show it; hands of 9-15 at the
start of a turn come from draws on opponents' turns (no Tier 6 deck has a no-maximum-hand-size card) and are trimmed at
the owner's cleanup, which isn't logged; seat 1 wins less than the target (slow games wash out the first-seat edge).
