# Report: cloud-wide (option #3, an Equipment deck with several threats)

Branch cloud-wide. Files: `audit/cloud/wide/list.txt` (100-card import list), `audit/cloud/wide/wide.swaps` (16 `OUT=>IN`
lines against today's list, PR #71). Design only; nothing measured except a 200-game smoke run (below).

## The list (16 swaps)
Same commander, mono-white, Bracket 3 (Game Changers: Smothering Tithe, Enlightened Tutor; none added). Kept: 33 lands,
all 6 rocks + Page/Solemn/Knight of the White Orchid/Loyal Warhound, all of Package D (Crawlspace, Archon of Absolution,
Norn's Annex, Windborn Muse, Baird, Wrath, Day of Judgment, Fumigate, Austere Command), the three non-Kaldra Swords,
Sword of Kaldra, Champion's Helm, Swiftfoot Boots, Mask of Memory, the free-equip bodies (Ardenn, Kor Outfitter, Armory
Automaton, Armored Skyhunter, Puresteel Paladin, Balan, Auriok Steelshaper, Helitrooper, Cid).
Creatures 28 -> 32 (+ tokens: Cats, Germs, Spirits, Warriors), Equipment 14 -> 11 (8 old + Skullclamp, Greaves, Batterskull,
Flayer Husk), rocks and lands unchanged.

| Out | In | Role of the In card |
|---|---|---|
| Unbreakable Formation (dead) | Brass Squire | free re-attach every turn, to any body |
| Ultimate Magic: Holy (dead) | Kor Skyfisher | cheap flier; bounces Cloud/Solemn/Outfitter to re-trigger the tutor |
| Helm of Kaldra | Skullclamp | equips any x/1 (tokens), draws; Cloud doubles it |
| Shield of Kaldra | Lightning Greaves | equip {0}; haste + shroud on whoever carries |
| Colossus Hammer (equip {8}) | Batterskull | brings its own 4/4 vigilance lifelink Germ |
| Hero's Blade (legend-only) | Flayer Husk | {1} living weapon: a free carrier |
| Wrecking Ball Arm (legend-only) | Kemba, Kha Regent | a Cat per Equipment on her each upkeep |
| Nim Deathmantle | Sigarda's Aid | flash Equipment attach, free |
| Bastion Protector | Hero of Bladehold | token maker that carries gear |
| Jocasta, Automaton Avenger | Mirran Crusader | double strike carrier, protection from B/G |
| Grand Abolisher | Sram, Senior Edificer | draws on every Equipment (and Aura/Vehicle) |
| Fate of the Sun-Cryst | Stonehewer Giant | {1}{W},T: an Equipment straight onto a creature |
| Thalia, the Survivor | Selfless Spirit | cheap flier; protects the team from wipes |
| Recommission | Secure the Wastes | instant X Warriors: carriers on demand |
| Veteran's Powerblade | Ornithopter | free evasive carrier / Squire food |
| Stand Up for Yourself | Triplicate Spirits | three 1/1 fliers (convoke) |

Kaldra triad cut to Sword of Kaldra only (Helm and Shield do nothing alone); Cloud-only cards (Bastion Protector, Jocasta,
Hero's Blade, Wrecking Ball Arm, Recommission) and the 2 dead cards are the main cuts. Spot removal kept except Fate and
Stand Up (the R package lost points by cutting bodies for removal; this cuts removal for bodies, which is untested).

## Ownership and buys
Owned (bulk-cards.txt, not in any JD/Avery deck): Triplicate Spirits, Ornithopter, Secure the Wastes. Everything else is a
buy (none of the other candidates are in bulk or binder; Adeline, Mentor of the Meek, Elspeth, Swords, Path are in Avery's
Galadriel, Skullclamp is in Kefka and Seph/Atraxa, Lightning Greaves in four decks: the Cloud copies are separate buys).
Prices: Scryfall `cards/collection` batch POST, 2026-10-15, cheapest printing.

| Card | Owned / price | Role |
|---|---|---|
| Brass Squire | $1.20 | free attach each turn |
| Kor Skyfisher | $0.63 | flier, re-triggers ETBs |
| Skullclamp | $5.04 | draw, token fodder |
| Lightning Greaves | $3.75 | equip {0}, haste |
| Batterskull | $5.41 | self-carrying threat |
| Flayer Husk | $0.22 | free carrier |
| Kemba, Kha Regent | $0.24 | token engine |
| Sigarda's Aid | $16.88 | free attach (the expensive one; cast 96% when tested) |
| Hero of Bladehold | $0.60 | tokens + battle cry |
| Mirran Crusader | $0.42 | double-strike carrier |
| Sram, Senior Edificer | $1.73 | Equipment draw |
| Stonehewer Giant | $8.43 | tutor-onto-creature |
| Selfless Spirit | $0.45 | flier, wipe protection |
| Triplicate Spirits, Ornithopter, Secure the Wastes | owned | tokens / carriers |

Total **$45.00** (13 cards); $28.12 without Sigarda's Aid ($16.88) and $19.69 more without Stonehewer Giant. Not bought, pricey:
Sword of Truth and Justice $29.97, Mother of Runes $12.30, Sword of Light and Shadow $12.75, Hammer of Nazahn $14.26,
Ranger-Captain of Eos $35.89. Cheap and unmodeled: Maul of the Skyclaves $0.35 (not in the Scryfall cache; `cardgen export`
without `--offline` must fetch it), Wedding Announcement $0.43, Squadron Hawk $0.22, Kaldra Compleat $9.54 (Partial).

## Model status
Pre-flight (`--swap`, all 16) prints no WARNING. 13 of the 16 In-cards are Full; Stonehewer Giant and Triplicate Spirits are
Full-auto (compiled from Oracle text, like Day of Judgment and Fumigate in today's list). Modeled here, with tests in
`rust/crates/sim-core/tests/cards_cloud_wide.rs` (7 tests): **living weapon** (Batterskull, Flayer Husk: a black 0/0 Germ
that dies when the Equipment leaves or moves; Batterskull's {3} return is an AI option only when it has lost its wearer and 8
mana is free), **Kemba** (upkeep Cats; the old regex guess made one 1/1). Code in `impls/cloud.rs`, verdicts in
`data/card_overrides.toml`, `data/cards.json` regenerated. Not modeled, strong: Kaldra Compleat (living weapon + exile on
damage, big), Hammer of Nazahn (auto-attach every Equipment, Partial), Maul of the Skyclaves (needs card data), Sword of
Truth and Justice (Full, $30). **Spectral Procession is mis-costed** ({2/W} hybrid shows as 6 generic): not used.

## AI check (10 traces, T1, loose, look-ahead, seeds 1-10, wide list)
Cloud still funnels Equipment onto himself, and the new cards are cast late.
- Of 84 equips in 10 games, 62 went onto Cloud; the rest were Token 6, Brass Squire 4, Puresteel Paladin 3, Baird 3, and
  mostly only while Cloud was dead or away. Where: `impls/cloud.rs` `holder_bonus` (+20 to Cloud), `equip_utility_bonus`
  (+4.5, +1.5 trigger/size), `holder_score` (+50 to Cloud), `wants_move` (any piece not on Cloud is pulled onto him) read
  by `dsl::equip_options` and `cloud_special`. `best_holder` (used by Outfitter, Armory Automaton, Beatrix, Puresteel,
  Scythe, Lightblade, Brass Squire's `squire_target`) is the same +50.
- **Kemba never wears anything** (cast in 3 games, 0 equips on her): nothing scores her or Hero of Bladehold; she only pays
  off with 2+ pieces. Fix: in `holder_bonus`/`equip_utility_bonus`, a bonus for Kemba (per Equipment already on her) and
  a rule "the second and later non-trigger pieces go to the best other body" (Cloud keeps the Swords whose triggers he doubles).
- **Living weapons get stripped.** Flayer Husk (game 2, R11) and Batterskull (game 6, R12) were moved off their Germs by
  Kor Outfitter / `wants_move` / Ardenn's combat move. The Germ then died (fixed here: a state-check sweep), so the
  Equipment's body was lost for a +2/+1. Fix: `wants_move`, `best_pull` and `ardenn` skip a piece whose wearer is its own Germ.
- **Cloud's tutor never aims at the new pieces**: `cloud_pick`'s ordered name lists don't mention Batterskull, Skullclamp,
  Greaves, Husk; they only come out of the last "cheapest cast+equip" fallback (it picked Greaves R14, Helm R8).
  Fix: add Batterskull (when mana >= 5), and "a piece for a body that has none" to the order.
- Cast timing is the bottleneck again: first casts in the traces were Sram R2-5, Skullclamp R1/R5, but Kemba R9-15,
  Hero R10-11, Stonehewer R9-14, Brass Squire R9-14, Batterskull R7. Hand clog, not the list, is the limit; a priority fix is in
  `cloud_prio` (it knows nothing of these cards).
- `keep_cloud_home` fired 12 times in 10 games ("nothing on him yet"); with several bodies it should let another body lead.
- Ardenn at beginning of combat moves *all* Equipment onto one permanent (seen onto Ardenn herself): should choose the
  attacker, not himself.
No AI code was changed.

## Smoke run
200 games, T1, loose, look-ahead, 4 jobs, paired against today's list: loads and plays, no panic, 253 s (so 2,000 paired games at 7 jobs is about 25 min). Result not claimed (noise is ±7 points).

## Test plan for the lead
All paired look-ahead, T1 then T2, loose profile, 2,000 games per tier, `--jobs 7`, `nice -n 19`, memory cap:
```
commander-sim --deck cloud --pool t1 --profile loose --games 2000 --ai lookahead --jobs 7 $(sed 's/.*/--swap "&"/' audit/cloud/wide/wide.swaps)
```
(build the `--swap` arguments one per line of the swaps file; same with `--pool t2`.)
1. **Run A, no AI change:** the list as is, T1 then T2 (the baseline for the AI work; the 20.6 / 18.2 is the paired baseline).
2. **Run B, with the AI fixes above** (Kemba/other-body holder bonus, living weapons stay, tutor order, Ardenn): same two tiers.
3. Cheaper cuts if A is flat: A without Sigarda's Aid and Stonehewer Giant (swap them back for the cut cards) to see whether
   the $25 matter.
Time (pairs play both lists; 1,000 games per tier is ~15 min at 7 threads from the Package D runs): 2,000 games per tier
~25-30 min, so A is ~1 h for T1+T2 and B another ~1 h. Run A first; only run B if the traces still show the funnel after A.
