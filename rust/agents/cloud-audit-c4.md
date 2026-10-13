# Task: cloud-audit-c4 — deep audit of how the AI plays the Cloud deck (diagnosis first)

Read `rust/agents/LASTDECK-RULES.md` for the hard rules (no Python, memory cap, worktree, commit, handoff, short
report). Changes for this task:
- Your branch `cloud-audit-c4` starts from `cloud-midgar`. The Cloud list in `decklists/JD/cloud-midgar-equipment.md`
  is JD's real list (the C4 list): never edit it.
- **No batch sims.** Single-game traces only (`commander-sim --deck cloud --pool t1..t5 --profile loose --ai
  lookahead --trace N`) plus unit tests and probe tests. The lead runs the batch analysis; its outputs will be in
  `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/cloud-c4-snap/audit/cloud/2026-10-13/c4/analyze-t*.txt`
  (read them as they appear; they cover how the deck wins and loses, plan timing, cards stuck in hand, cards removed).
- **Diagnose only.** Don't change AI or engine code. Probe tests that print option scores are welcome (commit them,
  `#[ignore]` any that fail on purpose).
- Build and test under `nice -n 19` with `-j 4`.

## Context
The deck: mono-white equipment voltron around Cloud, Midgar Mercenary ({W}{W} 2/1; enters: tutor an Equipment;
equipped: triggers of Cloud and his Equipment trigger twice). It now has ramp (6 rocks, Smothering Tithe, Solemn,
Knight of the White Orchid, Loyal Warhound), three combat Swords, Champion's Helm, the Kaldra set, free equips (Ardenn,
Kor Outfitter, Auriok Steelshaper, Puresteel Paladin), backup carriers (Balan, Kor Duelist, Armored Skyhunter, Bronze
Guardian, Exalted Angel) and protection (Swiftfoot Boots, Giver of Runes, Grand Abolisher, Plaza of Heroes, Bastion
Protector). It wins 10.9% over Tiers 1-5 (T1 14.0, T2 9.8, T3 7.4, T4 9.4, T5 13.8); JD's other decks win 25-35%.
JD thinks it should be stronger. Earlier fixes are in `rust/agents/report-cloud-ai-equip.md` (diagnosis) and
`report-cloud-ai-fix.md`; read both first so you don't re-report fixed problems.

## Audit every decision the deck makes, with trace evidence
Read at least 20 full traces across all five tiers (wins and losses) and check, turn by turn:
1. **Mulligans and keeps** (land count, rock count, Cloud-castable hands).
2. **Mana use**: unspent mana, the order rocks/ramp/Cloud/equipment are cast, Tithe and Solemn timing, land drops
   missed while lands were in hand.
3. **Cloud's tutor picks** and whether the picked piece gets cast and equipped soon.
4. **Equipping**: who wears what, Ardenn's combat attach, Balan's gather, Kor Duelist/Skyhunter as carriers when Cloud is
   dead, whether the best stack ends on the attacker, wasted equip activations.
5. **Attacking**: target choice (the weakest opponent? the one that can't block? commander-damage focus on one
   player?), attacking into bad blocks, not attacking when lethal or near-lethal was there, holding back blockers,
   the sword triggers (doubled) and what they target.
6. **Defense**: blocking decisions, chump blocks, life totals vs the table's clock; sweepers (Austere Command, Lay
   Down Arms, Secure the Scene) and when they are cast.
7. **Protection**: Giver of Runes, Swiftfoot Boots, Champion's Helm, Plaza of Heroes, Grand Abolisher: are they used
   before Cloud is exposed or in response to removal? How Cloud dies in each game (removal, wipe, combat).
8. **Recasting Cloud** after he dies (tax, timing, before combat).
9. Anything in the engine that models a card wrongly (compare with Oracle text in `data/scryfall_cache.json`).

## Deliverable
`rust/agents/report-cloud-audit-c4.md`: a ranked list of problems (expected impact first), each with the evidence (tier,
trace N, round, what happened, what should have happened), the responsible code (file:function), and a concrete fix.
Separate "AI mistakes" from "card modeling errors" from "deck-building limits" (things no AI fix can solve). Commit it.
Final reply: at most 15 lines (line 1 DONE or HANDOFF, line 2 your model ID, then the top problems).
