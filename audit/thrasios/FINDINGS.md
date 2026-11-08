# Tymna/Thrasios (key `thrasios`): findings

The deck's living list of strengths and weaknesses and what was tested.

**List:** `decklists/JD/tymna-thrasios-doomsday.md`, the Doomsday rebuild of 2026-11-05 (cEDH; the Thanos turbo core under
Tymna and Thrasios, Doomsday and three cantrips in). **Tiers:** t6 (`data/my_decks.toml`). First baseline 2026-11-06 below. Doomsday has card code, a pile builder and pile play (2026-11-05, `impls/doomsday.rs`, tested in `tests/doomsday.rs`), but
its first sim is the baseline below. The findings of the previous list (t5, Basalt
Monolith + Rings + Ballista deck with attack taxes) are in `FINDINGS-old-list.md`, and the retired Thanos turbo list's
(t6) in `audit/thanos/FINDINGS-old-list.md`.

## 2026-11-07: more black mana for Doomsday (swap tests; main 7d4d5b35; look-ahead, loose, 5,000 games per list, seed 500000, t6)
Why: Doomsday in hand in the main phase but not cast was "no {B}{B}{B}" 42% of the time (2026-11-06 below), and the
only U/B land was Underground Sea, while three duals make no black (Savannah, Breeding Pool, Hallowed Fountain). All
cards checked: Commander-legal, inside WUBG, the cuts in the current list.
- **S1, Savannah => Watery Grave:** 34.7% -> 35.6%, **+1.0** (noise ±1.5): no clear effect. Mana on turn 4 unchanged
  (4.25 -> 4.22); plan online by turn 4 60.4% -> 61.0%.
- **S2, S1 + Breeding Pool => Swamp:** 34.7% -> 36.2%, **+1.6** (noise ±1.6): at the edge of the noise, not clear.
  Plan online by turn 4 60.4% -> 63.0% (+2.6); mana on turn 8 6.46 -> 6.55; eliminated 65.3% -> 63.7% of games.
  Lands on the battlefield unchanged (3.33 at turn 4), so the gain, if real, is colour (black on time), not quantity.
Why so small: the Consultation / Pact line wins first (77% of wins) and needs only {B} or {U}{B}; Doomsday was chosen 153
times in 10,000 games (baseline below), so even fixing every missing {B}{B}{B} moves little. The swap report doesn't count Doomsday's
reasons, so whether "no {B}{B}{B}" fell isn't measured here (an `--analyze` run of the S2 list would show it).
Not adopted: JD's call. Raw: `audit/thrasios/2026-11-07-black-mana/`.

## Baseline 2026-11-07 (main 7c622f5f; look-ahead, loose, 10,000 games, seed 500000; list unchanged)
**35.1%** (34.2-36.0) at t6, against 33.9% (31.9-36.0) in Baseline B (2,000 games): +1.2, inside the noise. The list didn't
change; this checks that the 11-06/07 engine work (#292 counters wait for the combo, the Ringsight fix) didn't move it.
Plan online 80.6%; wins by combo 98%. Combo attempts 0.43 per game: Thassa's Oracle + Consultation / Tainted Pact 3,593,
Power Artifact / Rings 606, Doomsday + Oracle 84. Doomsday is still rarely chosen (153 times in 10,000 games, asked in
22.9% of games). Raw: `audit/baseline-2026-11-07/thrasios.txt`.

**Analyze (2,000 games, t6).** Eliminated in 66.2% of games (round 7.1), almost all by combo: Sisay 12.9%, Kraum/Tymna 11.3%,
Rograkh/Silas 9.9%, Dack Fayden 9.4%, Rograkh/Thrasios 6.7%, Kinnan 4.3%. Stuck in hand: Elvish Spirit Guide shows 0.0% "cast", which is expected: its mana is exiled from hand, not cast
(`partials::HAND_MANA`), so the card report never counts it. Then Orim's Chant
2.3%, Ad Nauseam 5.0%, Veil of Summer 7.3%, Doomsday 7.9%, Silence 13.3%. Lowest win rates when cast: Noxious Revival -10.3,
Force of Negation -9.4, Grim Monolith -8.0 (the reactive cards are cast in the games already going badly). Raw:
`audit/baseline-2026-11-07/thrasios-analyze.txt`.

## 2026-11-06: Cabal Ritual counted for Doomsday's {B}{B}{B} (branch doomsday-cabal; t6, look-ahead, 2,000 games, seed 500000)
The cost check counted Dark Ritual but not Cabal Ritual ({1}{B}: add {B}{B}{B}); now both (Dark Ritual first). Effect:
main-phase decisions declined for "no {B}{B}{B}" 3,142 -> 3,083, Doomsday chosen 31 -> 33, the line attempted 19 -> 20,
win rate 33.9% -> 33.8% (noise). Why so little: one Cabal Ritual is rarely the missing card. The missing {B}{B}{B} is the
mana base: black comes from five duals (Underground Sea, Godless Shrine, Scrubland, Bayou, Overgrown Tomb), Command Tower,
Mana Confluence, the fetches, the five-colour rocks and Birds, and two rituals; Cavern of Souls' coloured mana is for
creature spells only, and Gemstone Caverns makes colour only with its luck counter. Three black sources at once is hard
in four colours on turns 2-4. A list question for JD. Raw: `audit/thrasios/2026-11-06-doomsday-cabal.txt`.

## 2026-11-06: why Doomsday is rarely cast (trace-only counters, branch doomsday-why; t6, look-ahead, 2,000 games, seed 500000)
The run is identical to Baseline B apart from the new lines (the counters change nothing). Doomsday was in hand at a
decision in **471 games (23.6%)**, offered 37 times, chosen 31. What declined it (share of decisions; end step 2,133 /
main phase 7,423): **Demonic Consultation or Tainted Pact in hand 47% / 47%** (the guide: Line 1 is cheaper); **no flash
enabler 38%** (end step only); **no {B}{B}{B} 10% / 42%**; no winning pile 0% / 6%; an opponent's lock 3% / 2%; the rest
under 2% each. Tutors: on 3,685 own turns with a tutor in hand and Doomsday in the library, Doomsday is on the wish list
98% of the time, but a tutor would fetch Thassa's Oracle 71%, Demonic Consultation 12%, Doomsday 5%. Why so few: the
Consultation / Pact line is the deck's first line and wins first; Doomsday is the backup, and its main-phase cast often
lacks {B}{B}{B}. Raw: `audit/thrasios/2026-11-06-doomsday-why.txt`.


## Baseline B 2026-11-06 (after #278-#284; main 63d783b5; look-ahead, loose, 2,000 games per tier, seed 500000)
**33.9%** (31.9-36.0) at t6, against 34.4% in the morning baseline: -0.5, noise. Plan online 80.6% of games. Wins: Thassa's Oracle +
Consultation / Pact 77%, Power Artifact / Rings 16%. **Doomsday is still rarely cast:** the line was attempted 19 times in 2,000
games, the same as the morning. So neither the life rule (#278: no 16-life minimum, no board check) nor Lion's Eye Diamond as
Doomsday mana (#283) was what held it back. The likely blockers are the "never with Consultation or Pact in hand" rule, the narrow
tutor wish for Doomsday and the table-risk check; none is measured yet (proposed: a trace-only counter of which check blocks each cast).
Also new for this deck since the morning: Veil of Summer as its Oracle text, and spent only for a combo piece, the commander or an
important spell (#280); Tier 6 "close to winning" and the removal hold read the archenemy alone (#281, #282). Raw:
`audit/baseline-2026-11-06-b/thrasios.txt`.

## Baseline 2026-11-06 (after the rewire, Doomsday and Phase C engine work)
Tymna/Thrasios, **first number for the Doomsday list**, t6 (cEDH), 2,000 games, seed 500000, loose, look-ahead, integration build
daa0bfdb (main + #271-#276 with the overnight fixes): **34.4%** (95% CI 32.3-36.5) over t6; even share 25%: +9.4 points. No old-build
comparison (the old list played t5: 16.1% on 11-05). Wins by combo 97%, damage 2%; average own turn of wins 6.8. Plan online 80.5%.
How it wins: Thassa's Oracle + Consultation/Pact 76% of wins, Power Artifact / Rings + Monolith 18%. It went for **Doomsday only 19
times in 2,000 games** (the Oracle line 715 times, Monolith 130). Why it declines so often was not measured: its declines (Line 1
in hand, life < 16, lethal on board, no pile in 50,000 search nodes, #272/#275) are the candidates to trace next. Ready in 42% (round 6), one tutor
away in 61% (round 5); combos stopped 23% (Mindbreak Trap, Consultation in response...). Deaths: Sisay 14.8%, Kraum/Tymna 13.8%,
Dack 12.2%. The plan file is PROVISIONAL (written from the pilot guide). The first run on the earlier integration build hung on
one look-ahead game (the Doomsday pile builder ran at every playout decision; fixed on #275). Raw: `audit/baseline-2026-11-06/thrasios.txt`.

## Doomsday in the simulator (2026-11-05, before any sim)

What the AI does with Doomsday (`impls/doomsday.rs`; all of it untested by games, only by `tests/doomsday.rs`):

- **Pile.** An exact search over the guide's Piles A, B, C and the devotion-3 variants (Probe, Brainstorm, Preordain,
  Ponder, Borne Upon a Wind, Silence, Thassa's Oracle) with the mana and life it will really have; the pile with the lock
  (Silence) first and the fewest spells wins. Fillers: the highest `card_worth` cards left.
- **Timing.** The end step of the turn before its own with an instant-speed enabler (Teferi +1, Valley Floodcaller, or a
  Borne Upon a Wind cast first); in its main phase (Pile C) only when no enabler is around. Declines with Demonic
  Consultation or Tainted Pact in hand, at 1 life (life is otherwise no bar since 2026-11-06, JD: all or nothing; no board check), and into
  two or more opponents who can stop a spell with no lock in the pile and no counter held.
- **Pile extras (doomsday-b, 2026-11-06).** The search also uses Thrasios's {4} dig as a draw source (scry 1, then a land
  goes onto the battlefield and anything else is drawn: either way the library is one card shorter); a devotion
  permanent in hand (Mockingbird, Mystic Remora, Rhystic Study, Valley Floodcaller) as one more devotion; Veil of Summer
  held with {G} open as the lock when Silence cannot be (it answers a blue or black counter on the Oracle, not Mindbreak
  Trap or a red blast, so it is only used when no opponent's deck holds those); and a Noxious Revival in hand (or drawn
  by the line) as a backup that lets a pile without the lock be accepted into two stoppers. Piles are checked against a
  forced draw (our Esper Sentinel when an opponent casts a spell) and the draw step's extra cards (Howling Mine, Font of
  Mythos). Noxious Revival is aimed at the countered Oracle, not the best-valued card of the graveyard.
- **Lion's Eye Diamond (doomsday-led, 2026-11-06; tested in `tests/doomsday_led.rs`, not yet simulated).** Before, the
  diamond was dead in this deck (no wheel, no Underworld Breach): the sim never cracked it. Now it is Doomsday mana. LED is
  a mana ability, so it is cracked while the spell is on the stack and the spell is not discarded. (1) **Pays Doomsday's
  {B}{B}{B}** when that wins where the plain cast does not (no black source, or the pile's mana is short) or gives a better
  pile (the lock, a shorter line): the rest of the hand goes to the graveyard and Doomsday, which searches the graveyard
  too, can take those cards into the pile; the lands stay free for the pile. The plain cast wins a tie. Under Necropotence
  the discards are exiled, so the pile cannot take them (the guide's mistake 6). The table check for the cast is made with
  the hand gone (no Silence / Chant / counter / Veil held as cover). (2) **Inside the pile** the search may crack it for a
  spell or Thrasios's dig the lands cannot pay, as late as possible (the hand is lost with it), in {U}, {W} or {G}; it never
  does when the hand holds a card the line still needs (the Oracle in hand is lost unless it is the spell being cast; a held
  Veil / Revival goes too). (3) The plan file casts it from hand (value 90) only with Doomsday or a tutor that can fetch it
  in hand (`led_fuel`), else holds it; the held-mana statistic counts a Doomsday it pays for as a use. Not modelled: the
  diamond's mana being used for anything else in the Doomsday turn (a Necropotence, a tutor), and a crack in response.
- **The cast and the table.** The Doomsday cast now makes the same `go_risk` check as the other Tier 6 lines
  (`t6_table::go_risk_covered` against `go_threshold`): it waits when the table can stop it and there is no cover (Silence
  or Chant in hand, a free counter of ours, the lock in the pile, Veil held).
- **Keep rule.** `hand-castable:action=0` with an action card in hand sends the hand back: the hand's lands, a fetch
  (the colours of the library's lands) and the rocks and dorks it can cast must make the colours its action cards need.
  It replaces the Doomsday-only `hand-b-lands` term.
- **Still not modelled / to watch in the first sim:** Opposition Agent is declined, not played around; an opponent's Esper
  Sentinel taxes our first noncreature spell ({1} or they draw: the Doomsday cast in the main phase); Thrasios cast from
  the command zone as a devotion and dig source mid-pile; Mockingbird / Snapback / Chain of Vapor to redo an Oracle trigger
  that was countered (Stifle); the cast is not held for a better turn once it is legal.
