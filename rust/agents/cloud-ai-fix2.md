# Task: cloud-ai-fix2 — fix the problems found by the C4 audit

Read `rust/agents/LASTDECK-RULES.md` for the hard rules (no Python, memory cap, worktree, commit often, handoff,
short report). Changes for this task:
- Your branch `cloud-ai-fix2` starts from `cloud-midgar`. Never edit the Cloud deck list (it is JD's real list).
- **No batch sims** (the lead measures afterwards): no `--games`, `--analyze` or `all_decks` beyond the LASTDECK
  smoke checks below. Single-game traces (`--trace N`) and unit tests only. Build and test under `nice -n 19`, `-j 4`.
- **One commit per fix below**, each with its unit test(s), so the lead can measure or revert them one at a time.
- Read `rust/agents/report-cloud-audit-c4.md` (the diagnosis: evidence, file:function pointers, proposed fixes) and its
  probes `tests/cards_cloud_audit_c4.rs` first; also `report-cloud-ai-fix.md` (the previous round).

## Fixes, in this order
1. **Engine-aware tutor** (`impls/cloud.rs` cloud_pick and its `total()`): count the equip cost a piece would really
   have given the board and hand: Puresteel Paladin with metalcraft (3+ artifacts) makes equips {0}; Cid, Auriok
   Steelshaper discount; Ardenn re-attaches every combat for free; Kor Outfitter in hand attaches on entering; Armored
   Skyhunter. When equips are (nearly) free, fetch the biggest payoff the mana can cast (Sword/Shield/Helm of Kaldra,
   Colossus Hammer, Wrecking Ball Arm); otherwise the cheap order: Sword of Feast and Famine, Sword of Hearth and Home,
   Sword of Fire and Ice, with Champion's Helm right after the first Sword when Cloud is out and unprotected. Skip pieces
   already in hand or on the battlefield. **Enlightened Tutor** must use the same pick (today it uses cloud_prio).
2. **Don't strand equipment**: don't cast an Equipment when there is no creature to carry it and no mana/free equip to
   attach it this turn or next, unless it is a protection piece for a creature already out or its cast is the only use
   of the mana. Prefer holding it.
3. **Protection**:
   - Model **Plaza of Heroes**' "{3}, {T}, Exile Plaza of Heroes: target legendary creature gains hexproof and
     indestructible until end of turn" (today only its mana is modeled, though the override says Full), and add it to
     `ai::pool::PROTECTORS` (or Cloud's protect path) so it answers removal aimed at Cloud.
   - Cast **Unbreakable Formation** and **Rootborn Defenses** in response to a board wipe that would kill Cloud or
     equipped creatures (find how other decks answer wipes in response).
   - **Giver of Runes**: check it activates in response to targeted removal on Cloud (the audit saw it never activate).
4. **Casting Cloud into open removal mana** (put this behind a named constant or setting, default ON, so the lead can
   A/B it): prefer casting Giver of Runes / Swiftfoot Boots / Champion's Helm first when that delays Cloud by at most one
   turn; after Cloud has died once, recast him in a turn where protection can follow (mana for Boots/Helm equip, Giver
   untapped, Grand Abolisher out, or Plaza ready).
5. **Mask of Memory ping-pong** (regression from the earlier `dsl::equip_options` "exclude current holder" change): only
   offer moving an Equipment to another creature if the new holder is clearly better (holder_score margin), so no deck
   moves a piece back and forth. Keep Sauron's, Sheoldred's and Y'shtola's equip behaviour the same (one trace each).
6. **Cloud's mulligan** (`engine/turn.rs` or a deck hook): keep 2-4 lands plus at least one rock or two-drop, or 3+
   lands; mulligan 0-1 landers and 6-7 landers. Follow how other decks have deck-specific keeps, if any exist.
7. **Logging gap**: board wipes and edicts (Pernicious Deed, Grave Pact, Toxic Deluge, `zones::edict`) remove creatures
   with no trace line. Add a log line naming each creature that dies or is exiled that way.

## Done when
`cargo test --release -j 4` passes (the practice `http_lan` save test is a known intermittent failure: note it), the
`--all-targets` build has no warnings, `all_decks adaptive 3` and `all_decks lookahead 1` finish without a panic, and
10 single-game traces at t1 and t5 show the tutor fetching big pieces when Puresteel/Ardenn are out, no stranded
equipment, Plaza used, and no ping-pong. Report in at most 15 lines (line 1 DONE or HANDOFF, line 2 your model ID,
then one line per fix with its commit); full report in `rust/agents/report-cloud-ai-fix2.md`, committed.
