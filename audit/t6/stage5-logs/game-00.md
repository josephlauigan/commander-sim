# Game 0 (seed 500000) - Sisay, Dack Fayden, Rograkh/Silas, Kinnan

Reading note: in `[X decides: A 80%, B 20% -> C [search]]` the percentages are the heuristic's softmax over option
utilities; `-> C [search]` is the look-ahead's own pick, which can disagree with the percentages (e.g. "stop 0% -> stop").

**Seats:** Sisay, Dack Fayden, Rograkh/Silas, Kinnan. **Winner:** Kinnan, by combo (Kinnan + Basalt Monolith), **round 5**
(Kinnan's 5th turn, last seat). All three opponents eliminated at once at R5.

## Kinnan (winner, R5)
R2 it had 2 lands and chose `Volatile Stormdrake 2%` over Kinnan: `[Kinnan decides: Kinnan, Bonder Prodigy 64%, Talisman of Curiosity 34%, Volatile Stormdrake 2%, stop (hold mana) 0% -> Volatile Stormdrake [search]]`. The Stormdrake stole Sisay's The Cabbage Merchant, whose Foods then paid for most later turns ("Kinnan taps 6 Foods for mana"). R3: `Rhystic Study` over Kinnan (`... Kinnan, Bonder Prodigy 29%, Rhystic Study 14% ... -> Rhystic Study`). R4: Snapback on the Kinnan that Sisay had fetched, recast Kinnan, then Trophy Mage (6 Foods). R5: Basalt Monolith, Flesh Duplicate (copy of Stormdrake stole Kinnan back), `Kinnan goes for Kinnan + Basalt Monolith`, win.
Earliest line per spec: Kinnan on T2 (2 lands, spec "turn 2 Kinnan") then Monolith + outlet on T3-T4. It had Kinnan castable on R2 and R3 and chose a steal spell and Rhystic Study. Kinnan only hit the table on R4.
Tags: `CHOICE` (R2/R3 look-ahead preferred Stormdrake and Rhystic Study to the commander; the steal did pay off, so mild), `SPEC` (Monolith came from Trophy Mage only on R4). The win was fast for this tier (R5) because the table did nothing to it.

## Sisay (seat 1, eliminated R5)
T1 fetched Taiga, Noble Hierarch. T2 Cabbage Merchant (Sisay 98% listed, but only 2 lands: Hierarch not mentioned again, so it had 2 mana). T3 with 2 lands + Hierarch (3 mana) it did NOT cast Sisay: `[Sisay decides: Sisay, Weatherlight Captain 97%, Chord of Calling 3%, stop (hold mana) 0% -> stop (hold mana) [search]]`. T4 (3 lands + Hierarch): `Sisay, Weatherlight Captain 65%, Selvala 33% ... -> Chord of Calling`, which fetched Kinnan (Snapback'd away). Sisay herself was never cast in 5 turns; its only attacker was the stolen Stormdrake.
Earliest: T3, cast Sisay (3 mana: 2 lands + Hierarch, per spec "Sisay turn 2-3").
Tags: `CHOICE` (look-ahead passes with Sisay 97% on the heuristic, then picks Chord of Calling over Sisay at T4), `HOLD` (stop (hold mana) with no counter shown), `MANA` (3 mana available T3 and T4 and Sisay never resolved).

## Dack Fayden (seat 2, eliminated R5)
T1 Lotus Petal, Arcane Signet, Expedition Map (Radiant Solar discarded to venture). T2 City of Traitors, Blind Obedience, Expedition Map. T3 Staff of Compleation. T4 Basalt Monolith, Vexing Bauble. T5 Pithing Needle on Sisay. Dack the commander ({4}{W}{W}) was never cast, though the log shows 4 mana by R2 (Petal, Signet, City of Traitors + land) and Monolith by R4. Because the Dack plan needs WW and 6 mana: R3 had Urza's Saga, 2 lands + Signet + Map ... hard to tell exact mana; by R4 it had 3 lands + Signet + Monolith (3) + Sol Ring in R5 = 6+ mana with W (Plains played R4).
Earliest: R4 or R5, Dack with 6+ mana. It cast Basalt Monolith/Bauble (R4) and Pithing Needle (R5, `Pithing Needle 93%`) instead; a Construct 6/6 attacked Sisay for 8.
Tags: `MANA` (R4: Monolith + Bauble cast with 6+ mana available; Bauble even taxes its own free spells), `HOLD`/`CHOICE` (never offers Dack; the stax pieces score higher: `Pithing Needle 93%`). The Dack flip needs odds (3/7 Felidar) so waiting is per spec, but no "waiting for protection" reason is visible (no Silence/Chant ever cast).

## Rograkh/Silas (seat 3, eliminated R5)
T1 fetch only. T2 Gemstone Caverns, Rograkh, Grim Monolith. T3 Dark Ritual then Timetwister, countered by Kinnan's Dispel (Foods paid for it). After that, `Mana Vault 100%, stop 0% -> stop [search]` twice: Mana Vault was in hand with mana and the search declined to cast it. T4 Silas Renn, Mana Vault. T5 Birgi. No Oracle, Breach, Ad Nauseam, Necropotence or tutor was ever cast. The log does not show the hand, so the only visible plays are rituals/rocks/Timetwister.
Earliest: T2/T3 a win attempt needs Oracle + Consultation or Ad Nauseam; none appears in the log (never cast a tutor), so a hand without a line is likely.
Tags: `SPEC` (probable: no combo piece shown), `MULL` (kept a hand that spent turns 4-5 on Silas, Birgi), `CHOICE` (T3 declined Mana Vault at 100%). Timetwister into 2 open mana is a bait that worked as a Dispel sink.

## ODD
- R4 Kinnan decide line: "Kinnan, Bonder Prodigy 3%" for Snapback -> Kinnan is a combined option, fine. 
- Kinnan had **9 cards in hand** at R4 and 8 at R5 (Rhystic Study + Cabbage Merchant + Foods fed cards): its hand was never the constraint.
- Sisay's R3 attack with the stolen Stormdrake: 4 damage on Kinnan, who owns it (it flipped sides every turn: Stormdrake changes control each upkeep).
