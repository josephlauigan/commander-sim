# Y'shtola (yshtola) — AI audit, phase 1
Strategy: Esper drain. Every noncreature spell of mana value 3+ makes Y'shtola deal 2 to each opponent and gain 2; her
end-step draw needs 4 life lost by one player in a turn; Sanguine Bond, Marauding Blight-Priest, Ill-Gotten
Inheritance and the X drains (Exsanguinate, Debt to the Deathless) turn life gain into the kill; locks, Propaganda and
wipes keep the table off her. Priority function: `yshtola_prio` / `yshtola_named_prio` (`impls/yshtola.rs:1085`),
wired in `ai/decks.rs:54,80`. Tutors: `yshtola::tutor_pick` (`:923`); Zur fetches: `fetch_value` (`:1021`);
protection/wipe responses: `yshtola_protect` / `yshtola_wipe_response` -> `impls/zur.rs:733,795`. Plan config:
"first drain from Y'shtola's cast trigger" (analysis output); no custom reanimation/target valuation.
Evidence base: `--analyze --pool t5 --ai adaptive --games 300` (conservative 33.7%, loose 31.0%; the card report lists
only the commander because no other card reaches the 100-games threshold at 300 games, so I also read 20 traces
`--trace 1..20`, loose, seeds 500001-500020, aggregated by hand) plus the code. The 10/18 band-calibration note in
`audit/yshtola/FINDINGS.md` says FoW is cast 76% of games drawn; Farewell/Teferi's are stuck 68-75%.

## Problems (ranked by likely impact on win rate)
1. **Teferi's Protection (and any `prot=phase` card) runs through Clever Concealment's code, not Teferi's** — in
   `zur_protect` (`impls/zur.rs:749-760`) and `zur_wipe_response` (`:806-818`) it only phases the one creature (and its
   Auras) or the creatures, goes to the graveyard via `pay_card` (`ai/decks.rs:1215-1230`; real card exiles itself),
   gives no life lock and no protection from everything, and never phases Sanguine Bond/Tithe/locks. The wipe response
   fires only if the *creature* loss is >= 6 (`:798`), so Farewell/Austere "enchantments" or "artifacts" modes at her
   engine are never answered. It is also spent on any single removal at a pval >= 4 creature (no `worth_whole_board`
   check, which the pool path has at `ai/pool.rs:1085`). The engine's full version exists (`zones::teferis_protection`,
   `last_chance` for lethal, used by pool decks and sauron/seph). Matches the "Exile ~" note in `--cards`. Fix: for
   yshtola call `teferis_protection(g, owner)` (and keep it for wipes/lethal) instead of the Concealment branch.
2. **Mystical Tutor never finds a finisher.** `tutor_pick` offers Exsanguinate/Debt to the Deathless only when
   `kind == "any"` (`:938-947`, `:984`); Mystical Tutor's kind is "is", so the named list is empty and the generic
   fallback (`decks.rs:828-866`) takes the best cast-priority instant/sorcery. Traces: it put Bribery on top
   (twice, turn 3), and Your Fate Ends Here on top 3 of 5 casts; never Debt/Exsanguinate, which the deck doc says is
   the point of the card. It was also cast at the end of turn 3 with Y'shtola not out. Fix: add the "is" kind to the
   finisher branch (Debt, Exsanguinate, then Dark Petition/Cast Away Doubt) and hold it until a finisher can follow.
3. **Her draw / second-trigger plan is not in the AI.** `lost_this_turn` is read only by her end-step draw
   (`:148`); `trigger_bonus` (`:1066`) is applied only to cards her rules name. The mana-value-3 instants (Anguished
   Unmaking, Generous Gift, Your Fate Ends Here), Austere Command, Farewell, Force of Will and Teferi's Protection are
   *unnamed* (pinned by `cards_yshtola.rs:679`) and fall to `generic_prio` = 0 ("held / cast by the response logic",
   `ai/pool.rs:151`), so removal is chosen only by target value (`brain.rs` removal options) and never gets +trigger
   value, and she never casts an MV3 removal at an opponent's end step to make the 4th life point. The deck doc calls
   this the easiest second trigger. Not a "never cast" bug: in traces the removal is cast (YFEH 5, Gift 3, Unmaking 2,
   Push 3, GftT 5, Path 4 in 20 games). Fix: add `trigger_bonus` to the utility of her MV>=3 removal/wipe options and
   offer them at the opponent's end step when `lost_this_turn(opponent)` is 2-3.
4. **Spot removal is spent on low-value targets.** Traces: Path to Exile on Ornithopter (gives Winota a land) and
   Mother of Runes, Fatal Push on Esper Sentinel twice, Prison Sentence on Fyndhorn Elves, Go for the Throat on Orc
   Army / Quirion Ranger / Priest of Forgotten Gods, while Kinnan/Urza combos were the cause of 35% of her losses
   ("by Kinnan via combo 22%, Urza via combo 12.7%", analysis). `cast_removal` samples a target from `pval/1.5` with
   `temp`, and the deck has no "is it the engine piece" ranking. Likely generic-WRONG for a deck that wins slowly
   and loses to combo; unknown how much of it is the sampling temperature. Fix: for yshtola require target pval above
   a floor unless the spell is MV>=3 and she is out (then the trigger is the value).
5. **Farewell / Austere Command have no "Teferi's first" line and no Y'shtola trigger credit.** `wipe_options`
   (`brain.rs:733`) values the wipe by board swing only (`swing/2.2 - 2.5 + 2*danger`); the deck's key line (Teferi's,
   then Farewell) is not planned, and Farewell/Austere cast with her out earn no `trigger_bonus`. One Farewell cast in
   20 traces, correct modes ("art","cre") in that game. Low confidence on impact.
6. **Esper Sentinel / Mystic Remora cast priority.** Remora 66 early, Sentinel 62 early (`:1118-1124`). Existing
   findings: Remora has the lowest win rate when cast (39%) and Sentinel/Curiosity/Greaves are in the bottom group
   (`audit/yshtola/FINDINGS.md`). Remora was cast in 4/20 traces and sacrificed to its upkeep. The doc says to use it
   for a turn or two at most; the AI pays 4 never (it sacrifices). Unknown whether the cast is a net loss; list it as
   a candidate for a lower priority, not proven.
7. **Tooling/doc mismatch: `--cards` shows "not modeled" for Venat and Static Net, but the code has them**
   (`legends.rs:479` draw on legendary cast, `:497` Hero's Sundering; `common.rs:3250` Static Net lifegain + Powerstone).
   Venat's Hydaelyn side (counter + indestructible each turn) is unverified. `cardgen audit --md` cannot read
   `decklists/JD/yshtola-esper-drain.md` (it looks for `## Import list`, the doc uses `## Import list (100)`), so the
   per-card model status below comes from the code and `--cards`, not the audit tool.

## Cards the plan needs that the AI handles well (no change)
Sanguine Bond (76 + trigger bonus; first tutor target), Smothering Tithe, Consecrated Sphinx, Necropotence (named
priorities), the X drains' own option (`x_drain_option`, "Exsanguinate (X = 6)" in traces), Force of Will (cast 2/20
traces, countered The One Ring and Trinisphere; pitch/counter logic is generic and fine), the lock Auras with Zur.

## All cards
Model status: Full unless stated (from `--cards` tags and the code). Valuation: deck / generic-ok / generic-WRONG.
| Card | Model | Valuation | Evidence / note |
|---|---|---|---|
| Y'shtola, Night's Blessed | Full | deck | `ysh_cast`, `ysh_end`; prio 90 from turn 2 (60 on turn 1); cast in 74% of games, 45.5% win when cast |
| Anguished Unmaking | Full | generic-WRONG | unnamed, no trigger bonus, no 2nd-trigger plan (problem 3); 2 traced casts, both at Kinnan |
| Arcane Signet | Full | deck | rock 82 early / 30+b later |
| Arrest | Full | deck | lock via Zur helpers, 42+7*worth |
| Austere Command | Full | generic-ok | wipe_options; no trigger credit (problem 5) |
| Bound in Silence | Full | deck | lock |
| Bribery | Full (galadriel.rs:1209) | deck | 56+b; also what Mystical Tutor fetched (problem 2) |
| Cast Away Doubt | Full (draw, damage) | deck | `Draw` tag 46+b; the doc says it alone is 4 each: not special-cased; 3 casts/20 |
| Champion's Helm | Full | deck | 46 with her, equip option; 3 casts |
| Chromatic Lantern | Full | deck | rock |
| Coalition Relic | Full | deck | rock |
| Consecrated Sphinx | Full | deck | `sphinx_prio` |
| Curiosity | Full (curiosity_draws) | deck | 50 with her; Zur fetch value 3+1.5*opps; 3 casts/20 |
| Dark Petition | Full (spell mastery) | deck | tutor 52+b; kind "any" order OK (Bond first) |
| Debt to the Deathless | Full | deck | own X option; tutor target only at mana>=7 and lethal-ish |
| Deep Analysis | Full (flashback) | deck | `Draw` 46+b; Notion Thief pairing not planned (unknown) |
| Diabolic Tutor | Full | deck | found Bond/Priest/Debt in traces |
| Dovin's Veto | Full | generic-ok | counter response; MV2, no trigger |
| Encrust | Full | deck | lock |
| Esper Sentinel | Full | deck | 62 early / 30 (problem 6) |
| Exsanguinate | Full | deck | own X option |
| Farewell | Full | generic-ok | cast once in 20 traces with sound modes; problem 5 |
| Fatal Push | Full | generic-WRONG | target quality (problem 4), MV1 no trigger |
| Force of Will | Full | generic-ok | cast 2/20; FINDINGS 10/18: cast 76% of games drawn; MV5 trigger on a counter not valued |
| Generous Gift | Full | generic-WRONG | unnamed (problem 3); 3 casts |
| Glacial Fortress etc. (duals) | Full | generic-ok | lands |
| Go for the Throat | Full | generic-WRONG | target quality (problem 4); 5 casts, 3 on small creatures |
| Gonti, Lord of Luxury | Full | deck | 50; `gonti`, `stolen_worth` |
| Grand Arbiter Augustin IV | Full (common.rs:2936) | generic-ok | creature 40; 1 cast |
| Hostage Taker | Full | deck | 45 + target pval; 3 casts |
| Idyllic Tutor | Full | deck | found Bond each time |
| Ill-Gotten Inheritance | Full | deck | 54+b, sac option |
| Lightning Greaves | Full | deck | 58 with her; 2 casts; lowest-win-rate group (FINDINGS) |
| Luminous Bonds | Full | deck | lock |
| Marauding Blight-Priest | Full | deck | 62 |
| Massacre Wurm | Full | deck | 46+6k, up to 80 |
| Memory Trap | Full (ETB exile) | deck | 5 casts/20 |
| Mystic Remora | Full | deck | problem 6 |
| Mystical Tutor | Full | generic-WRONG | problem 2 |
| Necropotence | Full | deck | `necro_prio` |
| Night's Whisper | Full | deck | draw tag |
| Notion Thief | Full (steal_draw, t5.rs:974) | deck | 50 |
| Path to Exile | Full | generic-WRONG | target quality (problem 4) |
| Polluted Bonds | Full | deck | 56 early +b |
| Prayer of Binding | Full | deck | exile best permanent, 4 casts |
| Prison Sentence | Full | deck | lock |
| Propaganda | Full | deck | 44-72 with threat |
| Read the Bones | Full | deck | draw 46+b |
| Restoration Angel | Full | deck | 25 (held for flash, used by `zur_protect`); 2 casts |
| Sanguine Bond | Full | deck | 76+b; tutor target #1 |
| Skyclave Apparition | Full | deck | 45+target |
| Smothering Tithe | Full | deck | `tithe_prio` |
| Sol Ring | Full | deck | 82 |
| Static Net | Full (common.rs:3250); `--cards` says partial | deck | exile best permanent; 6 casts/20 |
| Talisman of Progress | Full | deck | rock |
| Teferi's Protection | Approximate (effect incomplete in the Y'shtola path; `--cards`: "Exile ~") | generic-WRONG | problem 1 |
| Tezzeret's Gambit | Full | deck | draw tag |
| The Eternal Wanderer | Full (zur.rs:335) | deck | 58+b |
| Thief of Sanity | Full | deck | 52 |
| Urborg Syphon-Mage | Full | deck | 46 + syphon option; 5 casts |
| Venat, Heart of Hydaelyn | Full per code; `--cards` says not modeled; Hydaelyn side unknown | generic-ok | creature 40; 1 cast |
| Windborn Muse | Full | deck | pillowfort |
| Your Fate Ends Here | Full (destroy + surveil) | generic-WRONG | unnamed (problem 3); 5 casts, one "Mystical Tutor" target |
| Zur the Enchanter | Full | deck | 54 + `fetch_value` |
| Lands with abilities: Exotic Orchard, Spire of Industry, Evolving Wilds, Terramorphic Expanse, Fabled Passage, Command Tower | Full (lands.rs) | generic-ok | no evidence of problems |
