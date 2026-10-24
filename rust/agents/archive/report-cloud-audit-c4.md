# Report: cloud-audit-c4 (diagnosis only; model claude-sonnet-5-5)

Evidence: 30 single-game traces (`--deck cloud --pool t1..t5 --profile loose --ai lookahead --trace 1..6`, 6 per tier;
6 Cloud wins) read through filtered views, plus probes in `rust/crates/sim-core/tests/cards_cloud_audit_c4.rs` (9 pass,
assert nothing). No batch sims, no AI/engine code changed. Not covered: the trace has no opening hands, no mulligan lines
and no block lines, so keeps and blocks were not audited from traces (mulligan code read instead). Batch outputs
(analyze-t2..t5) were not yet present; only analyze-t1 was used.

## Tutor policy (lead's addendum) - answers
1. `cloud::cloud_pick` (cloud.rs ~289). Fetches in this order, taking the first not already held with cast+equip <= lands+1+rocks+2:
   Kaldra third piece, Shield (if threatened), SoFI, SoHH, SoFF, Helm (first only if `removal_risk>=0.3` and a piece is held, else
   after SoFF), then Mask/SoK/Arm/Blade, then Hammer/Armor/Shield/Helm of Kaldra. The `total()` closure explicitly ignores Cid and
   Puresteel ("Cid and Puresteel Paladin are ignored"); Ardenn, Kor Outfitter, Steelshaper (equip -1), Beatrix, Armored Skyhunter
   and Balan are not considered either. Reveals in 30 traces: SoFI 27, SoHH 23, SoFF 14, Helm 8, Mask 6, Shield 4, never a Kaldra
   Sword/Helm, Hammer, Arm. So the first two picks are almost always SoFI then SoHH; SoFF/Helm (best win rate when cast) come
   third/last, and Helm only when "exposed". Right for turns 2-4 (cheap, equip 2). Wrong once free-equip engines exist.
2. Puresteel metalcraft equip {0} IS used when Paladin is on board with 3 artifacts: t3 trace 5 R8 equipped Hammer, SoFI, SoHH
   and Mask to Paladin for 0 in one turn (`equip_cost` -> 0). But nothing steers toward it: the tutor ignores it, and Paladin
   (cast 3) is held behind Swords (t1 trace 4 R5-R7 etc.). Ardenn (t2 traces 3/6) moved Colossus Hammer onto Cloud for free
   in combat: engine works, again unplanned.
3. Enlightened Tutor does not follow cloud_pick: it uses `cloud_prio` ("picks by cloud_prio", fix report), so it takes the
   highest-priority card (a Sword again), has no mana check and no engine check, and puts it on top of the library.
Proposed tutor policy: (a) engine-aware `total()`: if Paladin on board (or in hand and metalcraft reachable), Ardenn/Beatrix on
board, or Steelshaper out, equip cost counts 0 (or -1), so Hammer (cast 1), Sword/Helm of Kaldra, Arm become "ready" and are
fetched first when cast alone fits; (b) when an engine is in HAND and mana is >= engine cost, fetch the expensive piece
that is the best stat bonus (Sword of Kaldra +5/+5, Arm 7/7 base) rather than a third cheap Sword; (c) put SoFF above SoHH
(+15.2 win rate, protection from B/G) and Helm right after the first Sword whenever Cloud is on board (hexproof; equip {1});
(d) keep SoFI first. (e) Enlightened Tutor: call `cloud_pick`'s logic, not cloud_prio; fetch Cloud's missing engine
(Paladin/Ardenn are creatures, not Equipment: so at least fetch the piece that fits next turn's mana). Cards "stuck in hand"
(Shield/Sword of Kaldra cast 4 + equip 4) are the cast+equip>mana pieces; with (a) they become fetchable and castable.

## Problems, ranked (AI mistakes first)
1. **Tempo / late first real attack.** First hit of >=4 damage by Cloud: median R6; no 4+ hit at all in 12 of 30 games (none in 5
   of 6 combo/aggro losses). Games lost by R3-R9 to combo/aggro: t4 g1,g4; t5 g2,g3,g5; t1 g1,g3,g4. Mostly deck-building
   (Cloud 2/1, equip costs), partly #2/#3.
2. **Tutor ignores free-equip engines and mana order** (above). Code: cloud.rs `cloud_pick`/`total`. Fix as proposed.
3. **Equipment cast with nobody to carry it, into removal.** t3 trace 1 R3 (SoFI cast, Cloud exiled), R8 (SoFF cast and exiled the
   next turn), t2 trace 1 R3/R4 (SoFI destroyed/exiled before use), t1 trace 3 R5 (SoFI cast, no mana left to equip), t1 trace 5
   R1 Hammer (equip 8). Rule to add in `cloud_prio`/`search_bonus`: cast a loose piece only if a carrier is on the board or cast
   + equip fits this turn's mana; else prefer rocks/bodies. Probe 4 shows the AI does cast+equip when it fits, so the failures
   are the "no carrier / no mana" cases.
4. **Protection is almost never used.** In 30 traces (21 Cloud removals logged, plus wipes): Giver cast once, never activated;
   Unbreakable Formation / Rootborn Defenses never cast; Plaza of Heroes played as a land 1x, never used; Boots/Helm only
   when drawn. **Plaza of Heroes' ability ({3},{T}, exile: legendary creature gains hexproof+indestructible) is not modeled
   at all** (mine.rs/legends.rs: colours only; absent from `ai::pool::PROTECTORS`; probe 6: no option offered). Card-modeling gap.
   Fix: add the activated ability + a protector entry (Where::BfLand, free to respond).
5. **Cloud is cast into open mana on turn 2** and removed in 8/30 games at once (Swords/Path/Journey/Heartless Act/Beast Within):
   t1 g1,g2; t2 g1,g3,g4,g6; t3 g1; t4 g3. Each exile = +2 tax; recasts cost 4, 6, 8 and eat the whole turn (t1 trace 6 R5/R8/R16).
   The ETB tutor is still gained, so this is acceptable once, but later recasts at tax >= 4 are tempo-negative when the hand holds
   a carrier + equip; consider casting a cheap carrier first (Kor Outfitter etc.) when tax >= 4.
6. **Mask of Memory ping-pong (regression from the "exclude current holder" fix).** t3 trace 3 R16-R19: the Mask is re-equipped
   between Bronze Guardian / Kor Duelist / Knight 3-13 times a turn (equip {1}), no gain. Cause: `dsl::equip_options` now always
   offers the other creature and the search scores it >= stop. Fix: only offer a move if the new holder's `holder_score`
   exceeds the current one's by a margin (or the piece has no per-holder value). Cheap bug; low win-rate impact (late game).
7. **"stop" with a castable cheap play** (look-ahead prefers stop slightly): t2 trace 1 R12-R13 (Helm castable, 11 lands, two
   stops), t4 trace 3 R9/R11 (Sol Ring), t2 trace 5 R9. Probes 1/2/4 on rebuilt boards pick the right plays (24 seeds), so
   the cause is the real opponent board in the rollouts, not scores of Cloud's options. Low impact; the anti-stop penalty
   suggested in report-cloud-ai-equip.md #3 is the lever.
8. **Mulligan is generic**: `engine/turn.rs::mulligan` keeps any hand with 2-5 lands for every deck (no rock, no castable-Cloud
   check). Lands at start of own turn: on curve in 21/28 games at R4, only 12/25 at R6. A Cloud-specific keep (2 lands needs a
   rock or a <=2-mana play) would help; no trace evidence of the actual hands.
9. t5 trace 2 R2: search casts Hero's Blade over Cloud with WW up (probe 3 on a rebuilt board picks Cloud 24/24 once the
   probe sets `turns` >= 2; so this is board/rollout dependent). Low.

## Card modeling errors / gaps
- Plaza of Heroes protection ability missing (#4). Page, Loose Leaf's Grandeur is irrelevant (single copy).
- Pernicious Deed, Grave Pact edicts and wipes remove Cloud with NO trace line (t2 trace 3 R6 Deed X=4 killed Cloud and
  Ardenn; t1 trace 6 R5 Toxic Deluge, R8 silent loss). Probe 7: `zones::edict` kills the only creature (Cloud). Trace logging
  gap, not an engine bug; but `edict` could prefer a token/Cloud-less sacrifice only if one exists (it does: min `sac_worth`).
- Other cards read against Oracle (Cloud's doubling, Ardenn, Balan, Kor Duelist, Beatrix, Cid, Thalia, Court of Ardenvale,
  Puresteel) match; `--cards` shows "not modeled" lines for Beatrix/Hero's Blade/Armory/Helitrooper/Jocasta/Paladin/Arm, but
  cloud.rs has hooks for each (blade_etb, beatrix_combat, armory_*, helitrooper_attack, jocasta_*, puresteel_etb, base_pt).
  I did not verify every hook with a test.

## Deck-building limits (no AI fix)
- Cloud is a 2/1 that dies to any ping or Swords; 10 of 21 logged removals were exile/destroy at instant speed.
- 6 of 30 games end by R3-R8 from opposing combos (Kinnan R3/R4, Urza R8, Krenko R6, Prosper R7): the deck cannot interact.
- Life collapses 40 -> <10 within 3 turns in T1/T3 aggro games (Isshin, Aurelia, Marwyn): no blockers/lifegain early; Austere
  Command was never cast in 30 games (6 mana, hand-held).
- Opponent wipes/Deed/Rift in ~12 of 30 games reset a one-piece-per-turn board.

## Files
Only the probe file and this report. Run: `cargo test --release -p sim-core --test cards_cloud_audit_c4 -- --nocapture --test-threads 1`.
