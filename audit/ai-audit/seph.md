# Seph/Atraxa Phyrexian Reanimator (seph) — AI audit, phase 1

Strategy (decklists/JD/sephiroth-phyrexian-reanimator.md): fill the graveyard (Stitcher's Supplier, Entomb, Buried Alive, Tortured Existence, Gifts), cheat a bomb (Atraxa/Sephiroth, Archon, Grave Titan, Hornet Queen, Sheoldred x2) out with five reanimation spells, protect it (Ephemerate, Resto, Dismissal, Abolisher, counters), win by four infinite loops (Mikaeus+Triskelion; Mikaeus/Melira+Finks+outlet; Deathmantle+Altar+Titan) or by drain/stax.
Priority function: `seph_prio` (rust/crates/sim-core/src/ai/seph.rs:99). Plan code: `seph::options` (reanimate / fill / tutor / hardcast / Yawgmoth's Will / Tortured Existence plans), `seph_bval` (target value, :43), `rean_targets` (ai/decks.rs:925), `seph_tutor_target` (:228), `seph_fill_resolve` (ai/mod.rs:336), `protect` / `wipe_response` (seph.rs:939/989), `mine::loop_*` (impls/mine.rs:4163+). The plan config is "first 6+ power bomb on the battlefield" (reported as `bomb_by`).

Run: `--deck seph --pool t3 --analyze --ai adaptive --games 300` (both profiles): 43.0% / 43.3% win. Bomb online by T4 18%, T5 35%, T6 50%, T8 73%, never 14-16%. Atraxa resolves median T7-8 (72-76% of games). Loses mostly to Tergrid combat (15%), Aurelia combat (12-14%), Marwyn/Atraxa combat (6% each). Atraxa is removed 0.6x per game, Archon 0.15-0.17x.
Limits of the evidence: the card report lists only the top/bottom 10 cards by win rate when cast and its "often stuck in hand" list was EMPTY, so there are no never-cast / stuck numbers; the rows below rest on code. (Environment note: `commander-sim` and `cardgen` refuse to start in the shared worktree while `decklists/JD/cloud-midgar-equipment.md` has no `## Import list` section (someone's uncommitted rewrite); I ran from a private copy with that file restored from HEAD. `cardgen audit --mine` says "seph: 90 unique cards, 0 not Full", so the model column below is judged from the Oracle text vs the code, not from that status.)

## Problems (ranked by likely impact on win rate)
1. **Hornet Queen: not a bomb anywhere** — `--cards` shows `Hornet Queen [manual] C 4GGG :: pow=2 fly dt tok=4 tokfly tokdt`, no `bomb=`. `seph_bval` (seph.rs:46-50) gives a bomb-less creature its power = 2.0, so: (a) `rean_targets` (decks.rs:938) needs bval >= 4.0, so no reanimation spell ever targets her; (b) `own_bomb_in_gy` needs >= 6.0, so a Hornet Queen in the yard does not count as "bomb in graveyard" and the plan keeps tutoring/filling for something else; (c) `seph_fill_resolve` (ai/mod.rs:337) and `fill_go` only look at creatures with bomb >= 5, so Entomb/Buried Alive never bin her; (d) `seph_tortured` (seph.rs:545) treats her as a "small" creature (bomb == 0) and returns her to HAND from the yard instead of keeping her as a target; Stitcher's Supplier / Gifts / Atraxa-bottom can bin her and she is wasted; (e) `bomb_on_bf` (>= 6) false, so with only her out the deck still thinks "no bomb": it keeps tutoring (`seph_tutor`) and holds Ephemerate/Resto only by value; `wipe_response` (seph.rs:1023) does not sacrifice her to an exile wipe (bomb != 0 test); (f) hardcast `bomb >= 4` option (seph.rs:352) skips her, she is only cast as a generic creature at prio 30 (7 mana). She is the one reanimation target that also answers Aurelia/Tergrid combat (4 deathtouch fliers; 12-15% of losses each) — evidence: loss table above. Fix: give her `bomb=7` (4 bodies, 6-7 power total) in `data` tags, plus a `Tag::Tok` row in `mine::ETB_VALUE` (see 2).
2. **Hornet Queen and token-makers get no blink value** — `mine::ETB_VALUE` (mine.rs:1937) lists Atraxa 8, Archon 6, Rsd 4, Titan 3, Witness, Wall, Wurm, Bowmasters, Skate only. Hornet Queen's four-Insect ETB is the `tok=4` tag (engine/zones.rs:1396), which neither `etb_value` nor `t2_blink_value` (no dsl, no registry etb hook) counts, so her blink worth is 0: Kitten, Teleportation Circle, Closet, Hound, Resto and Ephemerate never pick her, though the doc says "every blink makes four more". Same gap for Kitchen Finks (gain 2, dsl etb only if it has dsl: shows `ability: triggered etb -> gain_life`, so probably small but nonzero) and Elesh Norn Mother of Machines (no ETB, fine). Fix: add `(Tag::Tok, 3.0)`-style row (scaled by tok count and flyers/deathtouch).
3. **Entomb / Buried Alive / Gifts can't fetch anything but bomb >= 5 creatures** — `seph_fill_resolve` filter `bomb >= 5`; loop pieces (Kitchen Finks, Triskelion, Melira: no bomb tag; Mikaeus has bomb=5 so it is OK), Muldrotha (pow 6, no bomb) and Hornet Queen are never fetched, though the doc names Entomb/Buried Alive as "combo tutors" for a creature piece in the yard "for Reanimate, Animate Dead, Necromancy, or Dread Return". With Mikaeus + a free outlet on board, Entombing Finks/Triskelion is the line; the AI instead bins the biggest bomb. Also `fill_go` is gated on a library bomb >= 5 existing. Fix: let `seph_fill_resolve` use `loop_need` (the missing loop piece) first, as `rean_targets` already does for the yard.
4. **Mixed thresholds for "bomb"** — `bomb_on_bf` and `own_bomb_in_gy` use >= 6, `hardcast` >= 4, fill >= 5, `rean_targets` >= 4. Mikaeus (bomb=5) Entombed does not count as "bomb in graveyard" (5 < 6) so `options` (seph.rs:281, :307, :337) still thinks the yard is empty and may tutor/fill again even though Reanimate has a good target. Muldrotha (bval 6 from power) counts as a bomb in yard and on bf only via `bomb_on_bf` -> false (bomb=0): she is a "bomb" in one test and not in the other. Fix: one `is_bomb(g,p,c)` built from `seph_bval` used by all of them.
5. **Momentary Blink is not protection** — `protect()` (seph.rs:939) calls only `ephemerate_cast` and `resto_cast`. Momentary Blink ({1}{W} instant, flashback {3}{U}, same fizzle trick, doc: "a blink is available again after the first") has no `prot=` tag (`fb=3U` only) and priority 0, so it is only cast via the generic dsl `card_value` fallback in `card_utility` (brain.rs:268), never in response to removal and never as a Kitten-style ETB re-buy on purpose. Fix: tag `prot=blink` and route it in `protect` / `ephemerate_value` like Ephemerate.
6. **Grand Abolisher has no combo-turn logic** — doc: "Resolve Grand Abolisher first" before the combo turn. `seph_prio` gives it the generic 30 (human pow=2, bomb==0 branch); `mine::loop_*` does not mention it (grep: only `ai/plans.rs:223` lists it as an opposing prison piece, and `impls/common.rs:2968` models it "Approximate: activated abilities are not restricted"). The Approximate model also under-delivers the protection it is cast for. Evidence is code only. Fix: prio ~70 when a loop is one piece from going off or a bomb is on board; model Oracle fully (no spells AND no creature/artifact/enchantment abilities on your turn).
7. **Tireless Tracker never gets a land-drop Clue the turn it is cast** — the land drop is always played at the start of the main phase (`engine/turn.rs:933`) before any spell, and Tracker is a prio-30 generic creature. Doc: Tracker "turns every land drop into a Clue", Evolving Wilds / Fabled Passage / Elder extra drops. Fix: if Tracker is castable this turn, hold the land (or at least a fetch land) until after it, in `play_land`.
8. **Muldrotha is only a prio-30 creature** — tags `pow=6 tgh=6 leg`, no `bomb`, not in the hardcast plan (needs bomb >= 4). She is a reanimation target (bval 6), and cardcode has the graveyard-play rules, but the AI has no reason to cast her early/at all when a bomb is castable (`hardcast` 8.0/`bval-1` always wins the option list). Unknown how often she is cast: stuck-list empty. Fix: give her `bomb=6` or a `seph_prio` row (e.g. 60 once a reanimation spell or a loop piece is in the yard).
9. **Entomb target ranking** — `seph_fill_resolve` ranks by `seph_bval` which adds up to +1.5 for Summon: Bahamut (9 mana, bomb=8): Bahamut (9.5) outranks Archon of Cruelty (9) and Sheoldred WO (8) as the thing to bin; reasonable only if `mv` of permanents is already big, but at the Entomb stage (turn 1-3) it is ~0, so 8 < 9 and Archon wins; no real problem, noted for completeness.
10. **Tutor wish lists reference cards this deck does not own** — `seph_tutor_target` (seph.rs:228-267) mentions Unmarked Grave, Persist, Evil Reawakened, Heroic Intervention, Counterspell, Swiftfoot Boots, Dovin's Veto: none are in this list (shared AI for old seph versions). Harmless, but the real fallback is only Swan Song / Phyrexian Arena / Animate Dead: no Muldrotha, Aura Shards, or Austere Command wish ("Long game, combo answered -> Muldrotha", "Under pressure -> tutor Deluge/Austere" in the doc). Fix: add Muldrotha, Austere Command, Toxic Deluge, Aura Shards (with danger input) to the late `first(...)` list.

## All cards
Model column: judged against Oracle text (cardgen audit marks all 90 as Full). Valuation: deck / generic-ok / generic-WRONG.

| Card | Model | Valuation | Evidence / note |
|---|---|---|---|
| Sephiroth / Atraxa, Grand Unifier | Full (top-ten dig by type; ETB 8.0 in ETB_VALUE) | deck | hardcast plan 8.0 (seph.rs:340); resolves T7 median; removed 0.6/game. Protected by `protect` |
| Archon of Cruelty | Full | deck | bomb=9; bval 9; ETB 6 in ETB_VALUE |
| Grave Titan | Full | deck | bomb=7, `titan`; blink worth 3 |
| Hornet Queen | Full model of ETB (tok=4); no bomb rating | generic-WRONG | problems 1, 2 |
| Sheoldred, Whispering One | Full | deck | bomb=8; upkeep reanimate hook not checked (unknown) |
| Sheoldred, the Apocalypse | Full | deck | bomb=7, `sheoA`; `rsd` ETB 4 |
| Elesh Norn, Grand Cenobite | Full | deck | bomb=8, `normgc` +2 vs najeela; Triskelion break not modelled in loop code (unknown) |
| Elesh Norn, Mother of Machines | Full (doubling in zones.rs) | deck | bomb=7, +1 vs sauron |
| Summon: Bahamut | Full (Saga creature, restart, Mega Flare) | deck | bomb=8 + Mega Flare bonus in bval; Karn's Bastion proliferate wired (lands.rs:573) |
| Muldrotha, the Gravetide | Full (cardcode muld_*) | generic-WRONG (weak) | problem 8 |
| Mikaeus, the Unhallowed | Full | deck (loop) | bomb=5; loop_prio 85/55 else hardcast only (prio 0 outside loops) |
| Kitchen Finks | Full (persist, gain 2) | deck (loop) | loop_prio else generic 30 |
| Melira, Sylvok Outcast | Full | deck (loop) | same |
| Triskelion | Full | deck (loop) | same; Entomb cannot fetch it (problem 3) |
| Blood Artist, Zulaport Cutthroat | Full | deck | prio 45 (bartist/drain); win-rate-when-cast +15 (Zulaport) |
| Viscera Seer | Full | deck | `sac` 50; +12.7 |
| Ashnod's Altar, Altar of Dementia | Full (mill via `seph_sac`) | deck | `sac` 50, loop_prio; Altar +16.6 |
| Nim Deathmantle | Full | deck | prio 58 when bomb on bf / in yard else 35; +14.9 |
| Stitcher's Supplier | Full | deck | fill 75; +19.0 (second best); can mill Hornet Queen (problem 1) |
| Entomb, Buried Alive | Full | deck (plan "fill graveyard") | problem 3 |
| Tortured Existence | Full | deck | plan 6.0; discards bomb >= 6 only; returns HQ (problem 1d); +11.5 |
| Gifts Ungiven | Full (pile_tutor) | deck | 55 at end of turn; split policy not audited (unknown) |
| Demonic Tutor, Diabolic Tutor | Full | deck | `seph_tutor_target`; problem 10 |
| Reanimate, Animate Dead, Necromancy, Unburial Rites, Dread Return | Full (all legal on legends; Reanimate life gate >= 12 left) | deck | `rean_targets`; Necromancy at instant speed and Dread Return flashback sacrificing 3 handled by `fbsac` zone |
| Yawgmoth's Will | Full | deck | plan 7.5, only to recast a reanimation spell; the doc's "big turn" use is not modelled (no tutor/Altar chain) |
| Skullclamp | Full | deck | 50 + plan 4.0 on X/1 fodder; +8.5 |
| Phyrexian Arena | Full | deck | `eng` 60; +9.0 |
| Harmonize | Full | generic-ok | draw sorcery 35 |
| Tireless Tracker | Full (landfall Clue, counters) | generic-WRONG (minor) | problem 7 |
| Displacer Kitten | Full (kitten; picks best etb_value >= 2) | generic-ok | prio 30 creature; targets limited by ETB_VALUE (problem 2) |
| Teleportation Circle, Conjurer's Closet, Flickering Hound, Restoration Angel | Full | deck (`flicker_prio`) | worth >= 3 needed; HQ gives 0 (problem 2) |
| Ephemerate | Full (rebound, fizzle trick) | deck | `ephemerate_value`, `protect` v >= 5 |
| Momentary Blink | Partial: dsl blink only, no protect role | generic-WRONG | problem 5 |
| Galadriel's Dismissal | Full (phase, kicked) | deck | `protect` v >= 6, `wipe_response` |
| Grand Abolisher | Approximate | generic-WRONG | problem 6 |
| Arcane Denial, Swan Song | Full | generic-ok (counter logic) | `ctr` tag; `ctr_risk` discount |
| Swords to Plowshares, Path to Exile, Anguished Unmaking, Assassin's Trophy | Full | generic-ok | rem tags; Path +19.8 win-when-cast (34-38% cast rate), Swords +8.6 |
| Austere Command, Toxic Deluge | Full (modes via wipe_modes) | generic-ok | decks.rs:1172 wipe_modes, seph-specific guard at :1386 |
| Aura Shards | Full | deck | `plans::shards_prio` |
| Grave Pact | Full (dsl triggered dies -> sacrifice) | generic-ok | no tag; value via dsl::card_value fallback (brain.rs:268), brain.rs:931 sacs for edicts |
| Sol Ring, Arcane Signet, Talisman, Mind Stone, Chromatic Lantern | Full | deck | rock 80 early, 30 later |
| Birds of Paradise, Sakura-Tribe Elder | Full | deck | dork/lr 80 early |
| Bojuka Bog | Full (opponents only) | generic-ok | exiles the opp graveyard with most worth; tapped so low land score |
| Strip Mine | Full | generic-ok | only vs key lands (mine.rs:2219), no Muldrotha replay (doc says optional) |
| Karn's Bastion | Full | deck | proliferates Bahamut lore (lands.rs:573) |
| Evolving Wilds, Fabled Passage, Ash Barrens, Exotic Orchard, Command Tower, Treno Dark City, shock/pain/check lands, basics | Full | generic-ok | land_score; Treno has no code (fine, tapped Town) |
