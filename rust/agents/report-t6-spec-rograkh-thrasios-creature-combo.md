# Report: T6 spec, rograkh-thrasios-creature-combo (2026-10-17)

Spec: audit/t6/rograkh-thrasios-creature-combo/SPEC.md (not committed).

Sources opened: EDHREC "Building RogThras" (2026-05-05, full text), EDHMeta "Competitive Temur: Rog and Thras" (2026-07-11, WebFetch summary),
Commander's Herald Sam Black 2024-04-18 (WebFetch summary; different Curio build), Hipsters of the Coast Sam Black 2023-08-18 (full text),
cedh-decklist-database entry (one paragraph; Moxfield 403), EDHTop16 decklist page. Oracle text via Scryfall /cards/collection (one call, 75 cards).
Scryfall single-card calls were rate-limited (429) by parallel agents, so the batch endpoint was used.

Findings:
- Three real win lines in our list: Cradle + Oboro Breezecaller + Talon Gates (6 creatures), Cradle + Eternal Witness + Snap (Cradle 5+ and an Island), and
  infinite mana -> Thrasios draws library -> Finale X>=10 or Faerie Mastermind + Green Sun's Zenith. Both loops net only +1 mana per cycle.
- Primer lines missing from our list: Valley Floodcaller loop (no Retraction Helix / Banishing Knack), Brain Freeze, Underworld Breach, Cloudstone Curio, Dockside.
- No source covers mulligan/turn order/when-to-go for this list; those sections are mostly [inference].
- Against Godo: Louisoix's Sacrifice counters the ETB trigger or equip; Boseiju kills Helm; Force of Will / Pact / Flare of Denial / Mindbreak Trap hit Godo or Dualcaster.
