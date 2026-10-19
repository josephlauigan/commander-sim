# Stage 5 log review - slice a (games 0-4, seeds 500000-500004)

Winners and rounds: g0 Kinnan R5 (combo), g1 Dack R19 (damage), g2 Rograkh/Thrasios R9 (combo), g3 Sisay R13 (damage), g4 Dack R8 (combo).
Median win round 9, and two of five games ended by combat after 13-19 rounds. Only Kinnan (g0) and Dack (g4) won on a spec line.

## Cause tags by deck (count = deck-games in which the tag applies; games played in brackets)

| Deck (games) | MULL | MANA | TUTOR | HOLD | NOLINE | CHOICE | STOPPED | RULES | SPEC |
|---|---|---|---|---|---|---|---|---|---|
| Sisay (5) | 1 | 4 | 0 | 1 | 1 | 5 | 1 | 0 | 0 |
| Dack Fayden (5) | 1 | 4 | 0 | 4 | 0 | 2 | 2 | 0 | 4 |
| Kraum/Tymna (3) | 1 | 2 | 3 | 2 | 0 | 2 | 1 | 0 | 2 |
| Rograkh/Silas (3) | 2 | 1 | 2 | 0 | 0 | 3 | 2 | 0 | 1 |
| Kinnan (2) | 0 | 1 | 0 | 1 | 0 | 2 | 1 | 0 | 2 |
| Rograkh/Thrasios (2) | 0 | 1 | 1 | 1 | 0 | 2 | 2 | 0 | 0 |
| **Total** | 5 | 13 | 6 | 9 | 1 | 16 | 9 | 0 | 9 |

No `RULES` tag: I found no Oracle violation. The Pact upkeep loss (problem 1) matches Pact's text; the fault is the AI choosing to cast it. Counts are my reading of the logs; the hands are not printed, so a few `MULL`/`SPEC` calls are inferred from what was (not) cast.

## The 5 problems that matter most

1. **Pact of Negation cast with no way to pay the upkeep: a player eliminated on turn 2.** Game 1, R1 `Sisay casts Pact of Negation at Ragavan, Nimble Pilferer` (Sisay had 1 land + Mox Amber), R2 `Sisay can't pay for Pact of Negation and loses`. `pact_affordable` (engine/stack.rs ~line 712, requires 5 lands per pact and 2 blue lands in Tier 6) guards only the `t.has(Tag::Pact)` branch at line ~753. Pact is MV 0, so when that branch is skipped the loop falls through to the final `can_pay(g, q, cg, &cp, false)` at ~line 783 and picks it anyway (cost {0}, always payable). Fix: in `pick_counter`, `continue` past any `Tag::Pact` card that fails `pact_affordable`, before the generic fallbacks (read from code, not run; add a test: 1 land + Pact in hand vs a 1-drop must not counter).
2. **Sisay never goes off, and rarely casts Sisay.** g0: Sisay not cast in 5 rounds (`Sisay, Weatherlight Captain 97% ... -> stop (hold mana) [search]` at R3, then `-> Chord of Calling` R4); g2: not cast at all (3 lands at R8, chose Lavinia); g3: Tithe at R4 over Sisay (`Sisay, Weatherlight Captain 68% ... -> Smothering Tithe`), then at R10-R12 she had Marvin, Ioreth, Selvala, Derevi, Emiel, Mount Doom and Bloom Tender on the table and the log has no `goes for ...` line, only a one-tutor-per-turn chain and `stop (hold mana)`; she won by 21 damage of combat on R12-R13. Fix: `t6_plan_sisay.rs` should rank "Sisay on curve" above Tithe/Chord/Lavinia when 3 mana is open, and `combos.rs` needs recognisers for line 1 (Bloom Tender + Derevi + Emiel), line 3 (Marvin + Ioreth + a mana legend) and a Mount Doom finish; check why `ready` stays false. This is the clearest `NOLINE`.
3. **Tutor sequencing burns the win pieces.** Gamble's random discard hit the card just fetched: g1 R11 Kraum `tutors Tainted Pact` / `Gamble discards Thassa's Oracle`; g4 R2 Silas `tutors Demonic Consultation` / `Gamble discards Thassa's Oracle` (Oracle had been fetched by Wishclaw one action earlier); g3 R1 Gamble binned Rite of Flame (harmless) but left the Oracle unsupported. Beseech the Mirror fetched and cast Oracle free with no Consultation/Pact in hand (g2 R6, Kraum), and fetched Remora with Oracle already gone (g4 R7). Wishclaw Talisman hands the table a free tutor each use (g2 R7-R8, g4 R2/R4). Fix in the Kraum and Rograkh/Silas plan files: Gamble only when no combo half is in hand (or when the hand is 1 card), Beseech/Oracle only with the pair in hand or on the stack, drop Wishclaw below the plan unless the win is on that turn.
4. **Rograkh/Thrasios fetches the wrong piece for its line.** g2 R8: `combo: Oboro Breezecaller + Gaea's Cradle + Talon Gates of Madara 100%` then `Invasion of Ikoria // Zilortha ... puts Faerie Mastermind onto the battlefield (Invasion of Ikoria, X = 14)`; the line needs Oboro (non-Human, so Invasion can put it in). The win came a turn later (R9) from Oboro cast from hand. Also R1 `Chrome Mox imprints Crop Rotation` (the Cradle tutor). Fix: target choice for Invasion/Chord/Nature's Rhythm in `t6_plan_rograkh_thrasios.rs` should pick the piece the current line is missing, and Chrome Mox imprint should avoid Crop Rotation/Spellseeker-type pieces.
5. **Mana and keeps: three decks stuck on 1-2 lands for 6-9 rounds, and the hand is never shown.** g1 Dack 1 land until R9 (then won only at R19), g2 Sisay 2 lands to R8, g3 Kinnan 3 lands for 9 turns, g4 Kraum mulligan to 5 and one land through R7 while holding Oracle + Tainted Pact (colourless Mana Vault only). The tier is stuck waiting for land drops because the keep logic (and fast-mana-only hands) accepts 1-landers; also Dack stacks self-taxes (`Damping Sphere` R1 g1, Vexing Bauble g0/g1/g2/g4, Trinisphere g3). Fix: the T6 mulligan slot in each plan file; prefer keeping 2+ lands that fix for the spec's colours; hold Bauble/Sphere/Trinisphere until the Dack line is cast or dead.

## Also noted
- Dack's kill is a table lookup (`pile_wins`, `t6_combos_t6.rs`): the log shows `goes for Dack Fayden: the first flip wins the turn (35 piles)` and then three eliminations with no triggers or ventures printed (g4). Add trace lines so the Stage 5 review can check it.
- `pays {4} to untap Mana Vault` every upkeep (g1 Rograkh/Thrasios R14-R18, g2 Kraum R8): net -1 mana to avoid 1 damage.
- Search overrides the heuristic constantly (e.g. `Intuition 100%, Kraum 0%, stop 0% -> stop [search]`, g1 R12-R13; `Mana Vault 100% -> stop`, g0 R3): in these games every `HOLD` and several `CHOICE` come from `[search]` disagreeing with 99-100% heuristic picks, with no counterspell or protection held to justify holding mana. Worth checking what the look-ahead's evaluation rewards for passing.
