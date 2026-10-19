# Game 19 (seed 500019) - Tier 6 within-tier, conservative profile, look-ahead AI

Seats: Rograkh/Thrasios, Dack Fayden, Rograkh/Silas, Sisay. **Winner: Rograkh/Silas, round 7, by combo** (Thassa's Oracle + Demonic Consultation, no protection spell shown). Log: `/tmp/claude-1000/t6logs/g19.txt`.

## Rograkh/Silas (winner, R7)
R1 Misty Rainforest, then Mana Vault, Sol Ring, Grim Monolith from one land. R2 Rograkh, Mystic Remora. R3 Arcane Signet, Silas Renn (`stop (hold mana) 99%, Silas Renn 1% -> Silas Renn`), Mental Misstep on Sisay's Worldly Tutor. R4 Paradise Mantle, Necropotence (paid 3 life; Dack's Reprieve bounced it R5). R5 11 cards in hand, 2-3 lands: Mox Diamond, Hexing Squelcher, `Chrome Mox imprints Underworld Breach`, Borne Upon a Wind, Chain of Vapor on Dack's One Ring. R6 Gamble (Swan Song), Beseech the Mirror, bargained: `Thassa's Oracle is cast free`, Imperial Seal put Demonic Consultation on top. R7 Snap by Thrasios bounced Oracle back; then `Rograkh/Silas goes for Thassa's Oracle + Demonic Consultation / Tainted Pact`.
Earliest line: **R7**, which is also when it went. Consultation reached the top only after Imperial Seal on R6 (drawn R7) and there was no Tainted Pact. R4 was the real chance: Necropotence on R4 with Vault, Sol Ring and Grim Monolith on the table; the AI paid 3 life and did nothing else (`Rograkh/Silas pays 3 life to Necropotence`), so the Necro cards came too late and the Reprieve bounce on R5 wasted the setup.
Tags: `TUTOR` (R5 the 11-card hand held a Beseech/Imperial Seal/Gamble trio that first appears in the decide list on R6; R6 Beseech cast Oracle free with no Consultation in hand: Oracle on the table early was useless, a `CHOICE` that Thrasios' Snap accidentally fixed), `MANA` (2-3 lands R3-R6), `STOPPED` (Reprieve on Necropotence R5, Swan Song on Gamble R6), `CHOICE` (R5 `Chrome Mox imprints Underworld Breach`: the spec's preferred line B piece).

## Dack Fayden (R7 eliminated)
R1 Plains and Cloudshift (a flicker spell, cast with nothing to flicker; spec line B needs it). R2 Kozilek's Command exiled Rograkh (Thrasios's commander). R3 The One Ring, R4 Staff of Domination and Scroll Rack, R5 Reprieve (bounced Necropotence), Cursed Totem, Jeweled Amulet; R6 Grim Monolith, Basalt Monolith, Pearl Medallion; R7 `Dack Fayden, Helping Hand 95% -> Dack Fayden, Helping Hand`. Dack resolved with 5 lands plus Medallion/Amulet/Monoliths, flipped Werefox Bodyguard, Felidar Guardian, Fiend Hunter, and cast Calamity's Wake afterwards (exiled 25 cards from graveyards).
Earliest line: **R6-R7**. Dack only shows in the decide list on R7, after Medallion R6. R4-R6 mana went to Staff, Scroll Rack, Cursed Totem and Monoliths (artifacts that entered tapped), all fine engine pieces but none a win line; line A needs 6 mana with WW, which it had at R7. Dack's flip put none of Preston/Solar/Felidar-engine in place, so there was no loop to start, and it died to Oracle before its next turn.
Tags: `CHOICE` (R1 Cloudshift; R3-R6 engines first), `MANA` (Grim and Basalt Monolith entering tapped), `SPEC` (flip hit Felidar but no Preston/Solar), `STOPPED` none.

## Rograkh/Thrasios (R7 eliminated)
R1 Rograkh (exiled R2 by Kozilek's Command), R2 Earthcraft, R3 Biomancer's Familiar, Gene Pollinator, R4 Thrasios and Rograkh, R5 Sowing Mycospawn (fetched Gaea's Cradle), Candelabra of Tawnos, R6 Formidable Speaker, Spellseeker, R7 `Rograkh/Thrasios casts Snap -> Thassa's Oracle (Rograkh/Silas)`.
Earliest line: no Oboro Breezecaller or Eternal Witness in view; Cradle came R5 (Mycospawn), 6-7 lands by R6-R7. The pieces for lines A/B (Oboro, Talon Gates, Witness) never showed.
Tags: `SPEC`, `CHOICE` (**the Snap on Oracle was the wrong answer**: Oracle was on the battlefield with its trigger already spent; bouncing it let Silas recast it with Consultation on the next turn and win. Leaving it on the table kept Silas's only Oracle out of play), `STOPPED` (Kozilek's Command on Rograkh R2, Fiend Hunter on Thrasios R7), `MANA` fine.

## Sisay (R7 eliminated)
R1 Birds, R2 Ignoble Hierarch, R3 Worldly Tutor countered by Mental Misstep, Gaea's Cradle, Mana Vault, **Sisay on turn 3**, R4 `Sisay: Mount Doom 100%` (first activation, power 2). R5 `stop (hold mana) 70%` for Cyclonic Rift (cast R6 on Dack's Staff of Domination), R6 Esper Sentinel.
Earliest line: Sisay on T3 is on the spec's schedule. The first activation fetched Mount Doom because with only colorless legends in play her power was 2 and she can only fetch MV 0-1 (Cradle, Doom, Mox Amber): spec "the only coloured legend Sisay can reach naked is Ragavan". No coloured legend and no Derevi/Emiel/Bloom Tender came, so lines 1-3 and 7 had no pieces; she then held mana instead of activating again (no target at power 2).
Tags: `SPEC` (no coloured legend to raise power), `HOLD` (R5/R6: holding mana for Rift rather than a second activation; fair), `CHOICE` (Cyclonic Rift on Staff of Domination, which was not threatening anyone, while Silas was one draw from winning).

## ODD
- Dack: `Dack Fayden casts Dack ...; Rograkh/Silas gains control of Felidar Guardian`, and one flipped creature each; the flip gave Silas a creature that later exiled nothing relevant. Worth confirming Fiend Hunter and Werefox Bodyguard "removed: exile" lines are the donated creatures' ETBs, not Dack's.
- `Rograkh/Silas has 11 cards in hand, R5`, 9 in R6, no discard (see game 15).
- Silas: `Rograkh/Silas decides: Mystic Remora 99%, stop (hold mana) 1% -> stop (hold mana)` followed by the Remora cast: same two-pass "stop then do it" pattern as elsewhere.
