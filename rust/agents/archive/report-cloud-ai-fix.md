# Report: cloud-ai-fix (model claude-sonnet-5-5)

Branch cloud-ai-fix (from cloud-ai-equip). No batch sims; unit tests and traces only. C3 deck-doc commit untouched.

## Changes (all in impls/cloud.rs unless noted)
1. Equip for Champion's Helm (equip {1}) and Sword of Feast and Famine (equip {2}): `cloud_special` offers
   "equip X to <best holder>" (Plan `equip_plan`), gated to key cloud, so Sauron/Sheoldred/Y'shtola paths are unchanged
   (no double options). Magitek Scythe already had an equip option: the diagnosis probe was wrong (the Scythe attaches itself
   on entering, so it was not loose); probe fixed. Stats of Helm/SoFF already live in values.rs tags.
2. `cloud_pick`: mana-aware. Order: SoFI, SoHH, SoFF, Helm (first when exposed and a piece is already held), then
   Arm/Blade/Sword of Kaldra/Mask, Hammer/Armor/Shield/Helm of Kaldra; takes the first not in hand/battlefield with
   cast + equip <= lands + 1 + rocks + 2; else the cheapest piece. Kaldra triple and Shield-when-threatened rules kept.
3. Search value: why `stop` beat `equip Shield of Kaldra`: the playout after "stop" goes on to cast in main 2, and the position
   score sees only +N/+N, so a no-stat equip scores below it. `search_bonus` now adds 8 (Shield/Helm of Kaldra, Mask), 6
   (SoFI, SoHH), 4 (Scope, Shuriken, Kitesail, Boots), 7 (Champion's Helm, SoFF) for equipping onto Cloud. Probe: equip Shield now 28.6 vs stop 26.1.
4. `cloud_prio`: SoFI 80, SoFF 78, SoHH 76 (SoFF had prio 0 and no compiled value: was never cast), Helm 62/52,
   Solemn 68 (turn <= 5 with Equipment in hand) else 56, Court 64 at 6+ lands, Knight of the White Orchid 62 when behind on lands.
   Others (Tithe 44, rocks 85-88, Warhound, Grand Abolisher 46, Skyclave 35, Enlightened Tutor 49) left as they were (judged sensible;
   Enlightened Tutor picks by cloud_prio, so a cheap Sword now).
   C4 (lead's addition): Ardenn 72 with Equipment waiting else 54; Steelshaper and Kor Outfitter 66 with Equipment out or in hand;
   Swiftfoot Boots 74 once Cloud is out, 70 when removal is likely, else 50; Benevolent Bodyguard 64 with Cloud out else 40;
   Balan 58, Kor Duelist 54, Armored Skyhunter 52 (+8 with Equipment waiting and Cloud away); `holder_score` +3 for Balan and
   Kor Duelist (double strike), so they carry gear when Cloud is out of play. Cloud himself stays 130, so these cannot go ahead of his recast.
5. `attack_filter`: "equipped" = power >= 4 or Arm/Boots/Armor/Helm of Kaldra (`strong_gear`), not any Equipment. The look-ahead's
   "all" plan skipped the filter, so `all_out_trim` applies the Cloud rule there too.
6. Recast: `search_bonus` for casting Cloud +4 in main 1 when Equipment is out, +8 with a hasty piece (Helm of Kaldra, Swiftfoot Boots).
7. Found in traces: `dsl::equip_options` re-offered "equip X to Y" with Y = the current holder (SoFI moved to Helitrooper 5 times in one turn); now excludes the current holder.

## Shared files touched (one line each)
`engine/combat.rs` (the "all" plan calls `cloud::all_out_trim` for key cloud), `dsl/mod.rs` (equip_options excludes the current holder).
`tests/cards_cloud.rs`: two tutor tests updated to the new pick policy; `tests/cards_cloud_ai_diag.rs`: ignored demo now passes.

## Tests
New `tests/cards_cloud_ai_fix.rs` (8): Helm/SoFF equip + cost + effect + no repeat, Scythe option, tutor choice by mana, tutor skips cards in hand,
Shield equip beats stop, priorities (incl. C4 ones that exist in this branch), attack filter, precombat recast bonus.
`cargo test --release -j 4 --no-fail-fast`: all pass (practice http_lan failed once on the first run, passed on the next: flaky/known).
Build with --all-targets: no warnings.

## Traces (t1, --trace 1..10, cloud)
SoFI/SoHH/SoFF/Helm get cast and equipped (game 2: Helm R11, SoFF R12 each equipped the turn they were cast). Pieces left unattached
in the traces came from lands stuck at 3 (games 1, 3) or Cloud absent. Remaining "0 damage" lines are other creatures (Helitrooper,
Bastion Protector, ...) attacking into blockers by the generic pool filter, not Cloud; I did not change that. Sauron, Sheoldred and
Y'shtola: their tests pass and traces show their usual equips (Sheoldred: Helm/Plate/Boots, Sauron: Greaves/Boots).

## For the lead to test after the merge (C4 cards not in this branch)
Ardenn, Kor Duelist prio entries (name-keyed, untested here: not in this branch's card data); Balan/Kor Duelist `holder_score` bonus with the
real cards; check their exact names match ("Ardenn, Intrepid Archaeologist", "Kor Duelist").
