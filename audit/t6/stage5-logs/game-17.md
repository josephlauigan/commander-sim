# Game 17 (seed 500017) - Tier 6 within-tier, conservative profile, look-ahead AI

Seats: Dack Fayden, Kinnan, Sisay, Kraum/Tymna. **Winner: Kinnan, round 6, by combo** (`Kinnan goes for Kinnan + Basalt Monolith`, all three opponents eliminated in one step). Fastest of my five games, still one to two turns slower than the spec's turn 3-5. Log: `/tmp/claude-1000/t6logs/g17.txt`.

## Kinnan (winner, R6)
R1 Breeding Pool (2 life), Kinnan, Mox Opal. R3 Kinnan bounced at end of turn by Kraum's Snapback (`Kraum/Tymna casts Snapback -> Kinnan, Bonder Prodigy (Kinnan)`). R4 3 lands: `Kinnan 56%, Mirrormade 35% ... -> Mirrormade` (copy of Dack's Basalt Monolith), activated it, then The Cabbage Merchant. R5 4 lands: `Kinnan, Bonder Prodigy 79%, Chord of Calling 21% ... -> Chord of Calling`, countered by Sisay's Fierce Guardianship. R6 5 lands: Kinnan recast, `combo: Kinnan + Basalt Monolith 100%` -> win.
Earliest line: **R5**. On R5 it already had a Monolith (the Mirrormade copy from R4) and 4 lands; casting Kinnan (2 mana) was the top-scored play and enabled the same combo line that won on R6. The look-ahead chose Chord of Calling instead and lost a turn to a Fierce Guardianship. Even on R4 (3 lands) Kinnan + one more mana was available, but Mirrormade (the Monolith) was also needed, so R5 is the realistic floor after the R3 bounce. Without Snapback the R2 line would have been Kinnan T1, Monolith T3/T4.
Tags: `CHOICE` (R5 Chord of Calling ahead of Kinnan, decide line above), `STOPPED` (Snapback R3, Fierce Guardianship R5). Not `HOLD`: once Kinnan was in play the combo option showed at once at 100%.

## Kraum/Tymna (R6 eliminated)
R1 Mystic Remora (never paid for; `sacrifices Mystic Remora (cumulative upkeep 4)` R5), R2 Ancient Tomb, R3 Snapback on Kinnan, Enlightened Tutor for Lion's Eye Diamond, R4 Demonic Tutor for Thassa's Oracle (`Kraum/Tymna turn 4: ... 11 cards in hand`), R5 Tataru Taru, Wishclaw Talisman, tutored Tainted Pact (`Sisay gains control of Wishclaw Talisman`).
Earliest line: **R6** (Oracle + Tainted Pact both in hand after the R5 Wishclaw; 4 mana needed). It never got to untap. At R5 it had ~6 mana (5 lands incl. Ancient Tomb) and spent it on Tataru Taru (a 0/3 draw-punisher) plus Wishclaw plus its activation, so it could not cast Oracle + Pact the same turn; and the Talisman went to Sisay (who could also tutor).
Tags: `TUTOR` (R3 Enlightened Tutor took LED with no Breach/Brain Freeze in sight; the three tutors fetched LED, Oracle, Pact one per turn: no piece was fetched to go with the previous one), `MANA` (Tataru Taru before Wishclaw used mana a combo turn needed), `CHOICE` (spec: "Wishclaw Talisman only on the win turn since an opponent gets it" is a Rograkh/Silas rule, but the same logic applies: it tutored on a turn when it could not win and handed the Talisman over), `STOPPED` none. Also `MULL`/`ODD`: 11 cards in hand R4 (no discard).

## Sisay (R6 eliminated)
R1 Mox Amber, R2 Magda, R3 Loki's Scepter, Marvin (countered by Dack's Bilbo's Gambit), R4 Sol Ring, Marvin, Tyvar (+1), R5 Sisay herself, activated for Mount Doom, Tyvar -2, Elves of Deep Shadow; won Wishclaw Talisman from Kraum but never used it.
Earliest line: R4 had Sol Ring + Mox Amber + 3 lands (6 mana) and `Sisay, Weatherlight Captain 93%/95%` led the scores twice, yet it cast Marvin and then `-> Tyvar, Jubilant Brawler`, so Sisay came a turn late (R5). First activation R5 fetched Mount Doom (a finisher without the untap pieces). No Derevi, Emiel, Ioreth or Bloom Tender appeared; the Marvin + Ioreth + legend untap loop (line 3) needed Ioreth and a mana legend.
Tags: `CHOICE` (Tyvar over Sisay R4, decide line `Sisay, Weatherlight Captain 95%, Tyvar 5% ... -> Tyvar, Jubilant Brawler`), `SPEC` (no engine piece found), `STOPPED` (Marvin countered by Bilbo's Gambit R3).

## Dack Fayden (R6 eliminated)
City of Traitors on turn 1 and **no second land until R6** (`Dack Fayden turn 5: ... 1 lands`). R3 Chrome Mox (imprinted Orim's Chant, the protection the spec wants held for the Dack turn), R4 Basalt Monolith + Manifold Key, R5 `Paladin Class 97% ... -> stop (hold mana)`, R6 Get Lost on Magda, Paladin Class. Dack never cast.
Earliest line: none; spec needs 6 mana with WW (Monolith, Mox, Key give about 5, but untapped lands < Dack's {4}{W}{W} white pips).
Tags: `MULL` (a one-land seven with 7 cards kept; spec mulligan keeps), `MANA` (stuck on one land; tapped Basalt Monolith three times at Sisay's end step `Dack Fayden activates Basalt Monolith` x3 with nothing to spend it on), `CHOICE` (Get Lost at end of Sisay's turn went at Magda, a 1-drop, with `Get Lost -> Sisay, Weatherlight Captain 49%` available, and Chrome Mox imprinted Orim's Chant, the Dack-turn protection).

## ODD
- Printed scores vs choice again: Sisay R4 (`Sisay 95% ... -> Tyvar`) and Kinnan R5 (`Kinnan 79% ... -> Chord of Calling`) choose a 5%/21% option. If that is the intended look-ahead override it should be logged as such; if not, the printed priors are misleading.
- Kraum/Tymna R4 at 11 cards in hand and Dack at 7 for five turns: hand size is not trimmed (see game 15).
- Kinnan R1: `pays 2 life for an untapped Breeding Pool` for tempo is fine.
