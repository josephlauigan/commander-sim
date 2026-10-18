# Cloud, Midgar Mercenary (cloud) — AI audit, phase 1
Mono-white Equipment voltron: Cloud (WW) tutors an Equipment on entering and doubles triggers of himself and of Equipment
on him; Swords and big Equipment (Wrecking Ball Arm, Kaldra, Colossus Hammer) make a 2/1 lethal, backed by hexproof/
protection, a sweeper suite and attack taxes. Priority function: `cloud_prio`, `impls/cloud.rs` (called from
`ai/decks.rs::deck_prio`). Tutors: `cloud_pick` (Cloud's own search), `cloud::tutor_pick` (kinds "ae"/"art" only, from
`decks.rs::tutor_pick_named`). Plan config: milestone `cloud` = first attack by an equipped Cloud (`plan_milestone`);
look-ahead bonus `search_bonus`; also `cloud_protect`, `cloud_wipe_response`, `keep_cloud_home`, `strands_equipment`.

Method: `--deck cloud --pool t1 --analyze --ai adaptive --games 300 --jobs 2` (23.0% conservative / 21.0% loose, Cloud
online by T4 32%, T5 57%, never by T12 ~15%; Cloud removed 0.60 per game), plus 60 play-by-play traces (`--trace 0..59`,
conservative, t1) read for casts, searches, equips and wipes. The card report only lists cards cast in 100+ games and its
"stuck in hand" list came out empty, so the traces carry most of the evidence. `cardgen audit --deck cloud` printed nothing
(see the warning at the end); model status below comes from `--cards` plus a grep of the code.

## Problems (ranked by likely impact on win rate)
1. Cloud's own search defaults to Sword of Feast and Famine on turn 2: `cloud_pick` sets `clogged = hand.len() >= 4` and
   then puts Feast and Famine first. On turn 2 the hand is 5-6 cards, so "clogged" is nearly always true. Evidence: 44 of
   the first-search reveals in 60 traces (R2) were Feast and Famine; Fire and Ice (the doc's first choice) was revealed
   on R2 once. Card report: Feast and Famine cast in 87% of games (win rate when cast +3.5), Fire and Ice 74% (+6.7),
   Hearth and Home 51% (+14.7, correlation only). Fix: make the Feast-and-Famine-first rule require a truly clogged hand
   (e.g. 6+ after the land drop) or only when a Sword is already held; default order Fire and Ice, Hearth and Home, Feast.
2. Open the Armory fetches non-Equipment: its search is the generic `tut=any`, and the pick goes through the normal
   tutor ranking (`tutor_pick_named` only applies `cloud::tutor_pick` to kinds "ae"/"art"). Evidence: 8 casts in 60
   traces fetched Wrecking Ball Arm x3, Sword of Kaldra, Sword of Feast and Famine, but also Sol Ring, Coalition Relic and
   Cid, Freeflier Pilot (Oracle: an Aura or Equipment card). Model bug plus valuation (illegal ramp/body fetches). Fix:
   give Open the Armory an Aura/Equipment filter and route it through `cloud_pick`.
3. Own wipes are not weighed against the plan: Austere Command picked modes `["ench","ge4"]` and destroyed the AI's own
   Smothering Tithe (2.txt R12), and `["le3","ge4"]` (all creatures) in 22.txt R6, killing Forum Familiar. Wrath of God
   in 12.txt R5 killed Cloud himself while facing 11 attackers, which is defensible (command zone). Only 1 clear own
   enchantment loss in 4 Austere casts, so the sample is thin: treat as a watch item. Fix: Austere mode choice should
   drop "destroy enchantments" when it owns Tithe/Court/Sphere and the table has no relevant enchantment.
4. Equip timing/online rate: first equipped attack by T4 is only 32% (T3 4%) against the doc's "equipped Sword by turn
   four". Cloud resolves median T2.0, so the lag is in cast-plus-equip mana (Sword 3 + equip 2 = 5 mana vs 4 lands on
   T4) and in `cloud_pick` choosing a 3+2 Sword instead of Champion's Helm (3+1) or a rock. Unknown how much is AI vs
   mana; worth an A/B once problem 1 is fixed.
5. Colossus Hammer is held until Equipment is already out (36 vs 52) and its equip {8} is only reached with Puresteel
   Paladin or Ardenn: cast 14 times in 60 games, equipped to Cloud 10 times. Plausible, no evidence it is wrong.

No card is "never cast": every nonland card was cast at least once in the 60 traces (rarest: Windborn Muse 1, Akroma's
Will 2, Exalted Angel 2, Thalia 2, Sphere of Safety 3, Elspeth 3, Citizen's Arrest 3, Winds of Abandon 3).

## All cards
Legend: Model = F full, A approximate, P partial, U unmodeled (judged from `--cards` tags, DSL rows and code hooks).
Valuation: deck (named in `cloud_prio`/cloud code), generic-ok, generic-WRONG.
Casts = games (of 60 traces) in which the card appeared at least once as "casts X" counts (raw cast counts, recasts
included for Cloud).

| Card | Model | Valuation | Evidence / note |
|---|---|---|---|
| Cloud, Midgar Mercenary | F | deck | prio 130 minus 3 per tax (min 70) from turn 2; cast 153 times in 60 games (recast after removal 0.60/game); Cloud search see problem 1 |
| Adelbert Steiner | A (self_scaling DSL) | generic-ok | 14 casts; backup carrier, no deck entry |
| Akroma's Will | P (no tags) | deck (`will_resolve`) | 2 casts; choice of modes in code, rarely cast |
| Arcane Signet | F | generic-ok (rock 80/40) | 9 casts |
| Archangel of Tithes | A | generic-ok | 4 casts; tax on attackers modeled in common.rs |
| Archon of Absolution | A | generic-ok | 5 casts; attack tax in common.rs |
| Ardenn, Intrepid Archaeologist | F (hook) | deck (72 with Equipment) | 8 casts |
| Armored Skyhunter | F (hook) | deck (backup carrier 52/60) | 6 casts |
| Armory Automaton | F (hooks `armory_etb/attack`) | deck (64/50) | 7 casts; `--cards` still says "not modeled" but hooks exist |
| Ash Barrens | F | generic-ok | land |
| Auriok Steelshaper | F | deck (66/46) | 7 casts |
| Austere Command | F (modes austere2) | generic-WRONG? | 4 casts; mode choice can kill own Tithe, see problem 3 |
| Baird, Steward of Argive | A | generic-ok | 4 casts |
| Balan, Wandering Knight | F | deck (58, +8 as backup) | 10 casts |
| Bastion Protector | A (anthem + keyword DSL) | deck (74 with Cloud out) | 6 casts |
| Champion's Helm | F (tag helm) | deck (62/52) | cast 41-44% of games, win rate when cast +13.6/+15.6; fetched as Enlightened Tutor target 3/3 on turn 1 |
| Cid, Freeflier Pilot | F (hooks) | deck (82 with Equipment in hand) | 10 casts; wrongly fetchable by Open the Armory |
| Citizen's Arrest | A (etb exile DSL) | generic-ok | 3 casts |
| Coalition Relic | F | generic-ok | 6 casts; wrongly fetched by Open the Armory |
| Colossus Hammer | F (equip_bonus DSL) | deck (52/36) | 14 casts, 10 equips to Cloud |
| Court of Ardenvale | A | deck (64 at 6+ lands, else 50) | 7 casts; return-each-upkeep modeled in cloud.rs/legends.rs |
| Day of Judgment | F | generic-ok | 7 casts |
| Disenchant | F | generic-ok | 3 casts (targets Shadowspear, Pearl Medallion, Esper Sentinel) |
| Elspeth, Sun's Champion | P (no tags) | generic-ok | 3 casts; removed 0.05/game |
| Enlightened Tutor | F (kind "ae") | deck (`cloud::tutor_pick`) | 6 casts; fetched Helm x3 (R1), Hearth and Home x2, Fire and Ice, Wrecking Ball Arm, Mask of Memory |
| Evolving Wilds | F | generic-ok | land |
| Exalted Angel | F (morph, bulk_white.rs) | generic-ok | 2 casts (one face down) |
| Fate of the Sun-Cryst | F | generic-ok | 2 casts |
| Fellwar Stone | F | generic-ok | 12 casts |
| Fireshrieker | F (equip_keyword DSL) | generic-ok | 8 casts, 9 equips to Cloud |
| Forum Familiar | F | generic-ok | 12 casts; the doc's "return Cloud to hand" use is not in the prio (no deck entry) |
| Fumigate | F | generic-ok | 4 casts |
| Giver of Runes | A (`cloud_protect` uses it; `--cards` says unmodeled) | deck (64/40) | 8 casts |
| Grand Abolisher | A | deck (`cast.rs`, cloud.rs mention) | 10 casts |
| Helitrooper | F (hook) | deck (56) | 7 casts |
| Hero's Blade | F (hook `blade_etb`) | deck (84 if Cloud waiting and 4 mana) | 14 casts, 3 equips by trigger |
| Jocasta, Automaton Avenger | F (hooks) | deck (52) | 5 casts; carried Equipment 4+3+2+1 times |
| Knight of the White Orchid | F | deck (62/46) | 11 casts |
| Kor Outfitter | F (hook) | deck (66/46) | 10 casts |
| Lay Down Arms | F | generic-ok | 5 casts (targets Light-Paws, Teysa, Isshin, Laelia and more) |
| Loyal Warhound | F | deck (60/36) | 7 casts |
| Mask of Memory | F | deck (72) | 13 casts, 18 equips to Cloud; Cloud-doubled loot modeled |
| Mind Stone | F | generic-ok | 6 casts |
| Norn's Annex | A | generic-ok | 9 casts (attack tax modeled) |
| Open the Armory | U/Partial (tut=any, no Aura/Equipment filter) | generic-WRONG | problem 2 |
| Page, Loose Leaf | A (dork, Grandeur unmodeled) | generic-ok | 5 casts; Grandeur needs a second Page, irrelevant |
| Puresteel Paladin | F (hook) | deck (81 with Equipment in hand) | 12 casts; metalcraft equip {0} in `equip_cost` |
| Silverblade Paladin | A (soulbond, bulk_white.rs) | generic-ok | 5 casts; double-strike pairing with Cloud is in the doc, no deck entry |
| Skyclave Apparition | F | generic-ok | 10 casts |
| Smothering Tithe | F (tag tithe) | generic-ok | 5 casts; removed 0.03/game |
| Sol Ring | F | generic-ok | 12 casts |
| Solemn Simulacrum | F | deck (68/56) | 5 casts; blinked 4x by Hearth and Home in one trace |
| Sphere of Safety | A | generic-ok | 3 casts |
| Sram, Senior Edificer | A (t1.rs) | generic-ok | 6 casts |
| Stand Up for Yourself | F | generic-ok | 6 casts |
| Swiftfoot Boots | F | deck (74 with Cloud out) | 9 casts, 10 equips |
| Sword of Feast and Famine | F | deck (78) | 54 casts, 76 equips; over-tutored, problem 1 |
| Sword of Fire and Ice | F | deck (80) | 36 casts, 44 equips; removed 0.24/game, the most-removed Equipment |
| Sword of Hearth and Home | F | deck (76) | 19 casts, 23 equips; blink + land modeled |
| Sword of Kaldra | A | deck (76) | 12 casts, 9 equips |
| Thalia, the Survivor | A (static cost DSL) | generic-ok | 2 casts |
| Thought Vessel | F | generic-ok | 5 casts |
| Wall of Omens | F | generic-ok | 6 casts |
| Windborn Muse | A | generic-ok | 1 cast |
| Winds of Abandon | A (rules2.rs; overload) | generic-ok | 3 casts |
| Wrath of God | F | generic-ok | 7 casts |
| Wrecking Ball Arm | F (base 7/7, evasion in code) | deck (78) | 22 casts, 20 equips; cast 33-36%, win rate when cast +7.7/+14.4 |
| Zack Fair | F (hook) | deck (66/54) | 6 casts |
| Plains / Plaza of Heroes / Path of Ancestry / Terminal Moraine | F | `plaza_protect` for Plaza | lands; Plaza protection logic exists |

## Tooling warning
`cardgen audit --deck <jdkey>` (and `--md`, `--mine`) OVERWRITES `decklists/JD/<deck>.md` with a model-status table. It clobbered the Cloud doc and then the Y'shtola doc during this survey (both restored with `git checkout`, `git status` is clean except `audit/ai-audit/` and the brief). Do not run it on JD decks. Also: commander-sim refuses every deck while any JD doc lacks an "## Import list".
