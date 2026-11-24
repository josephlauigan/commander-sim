> Not built. Opus's design for ticket T-TP (phase a protected player's lands and token counters). JD decided on 2026-11-24 to label Teferi's Protection Approximate with a waiver and not build this; the card note in data/card_overrides.toml points here. Written 2026-11-24 against main 2079d835.

# Teferi's Protection: phasing lands and token counters (design consult)

Tree read: /home/joseph/projects/commander-sim-wt-gatebase at 2079d835 (its CLAUDE.md matches origin/main). Read-only: no
builds, tests or sims were run. All paths below are under `rust/crates/sim-core/src/` unless written otherwise.

## Summary

1. Today `engine/zones.rs:711-732` phases out only `Player::perms`. Lands stay listed and are tapped. The Treasure, Clue, Food and Blood counters stay live and can be spent. The floating mana pool is emptied, which the rules do not do. Opponents' Auras and Equipment on its permanents do not phase out. The protection ends at upkeep, after `cardcode::turn_start`. The card is marked `Full` (`data/card_overrides.toml:4990-4992`, "lands too"), but `audit/GAME_CHANGERS.md:55` lists "taps lands instead of phasing them" as a known approximation. The label is wrong today.
2. Inventory. There are about 120 cross-player `lands` reads in sim-core, about 30 more in `human/`, about 160 token-counter reads and about 20 token-sacrifice cost sites. In current pods the reachable gaps are:
   - all four TP decks: the protected player spends Treasure, Clue and Blood counters during the window;
   - Arzakon at t5: lands tapped by TP stay stranded under Urza's Winter Orb, Static Orb and Back to Basics;
   - opponents' Exotic Orchard and Fellwar Stone still read the phased player's land colours;
   - Edgar's phased Urborg still turns every land into a Swamp;
   - Windgrace's Strip Mine can destroy Kaalia's phased lands;
   - Yoshimaru's Flickerwisp targets (and Brago's) can reach phased lands;
   - Miirym's Astral Dragon can copy Arzakon's phased lands.
3. Recommendation, option B': set `phased` on each land and also move it out of `Player::lands` into a stash on `Game`. Move the four token counters into a stash on `Player`. `phase_in_for` restores both before the untap locks run. Every list reader then becomes correct without being edited, and about 11 files change. I reject option A: it needs an edit at every reader and wrongly hides objects that enter after TP resolves. I reject option C: it changes every game that has a Treasure in it.
4. AI. Nothing about when the AI casts TP changes. The forecast readers (look-ahead `strength_seen`, `next_turn_mana`, `pact_affordable`, `tithe_prio`, the card-worth "fit" term) add the stash back, so the AI stays decision-neutral. Three real exploits close:
   - Treasure mana while phased out;
   - Strip Mine on a phased land;
   - untap locks stranding lands that were untapped when TP resolved.
5. Status. TP becomes truly Full only if JD approves D1-D4 below. No card drops below Full. Dockside's `life_locked && ring_prot` workaround is deleted; only its note text changes.
6. Effort is about 8 worker hours, plus L2/L3 runs by the coordinator. Proof: every non-SAME L2 record must seat Arzakon, Kaalia of the Vast, Aurelia or Edgar Markov, and every other record must be byte-identical. JD's Sheoldred t3/t4, Yoshimaru t4 and Arzakon t5/t6 baselines will move.

## Inventory

How it works today, for reference:
- `zones.rs:711-732`: `life_locked`, `ring_prot`, `pl.mana.clear()` (715), `phased = true` on `pl.perms` (717-719), `tapped = true` on `pl.lands` (720-722), `end_links` (723-725).
- `turn.rs:1242-1249`: `step_start` runs `phase_in_for` (perms only, `turn.rs:1178-1203`), then `untap_lock`, then the land untap, then `tapped_treasures = 0`.
- `turn.rs:711-716`: upkeep runs `cardcode::turn_start` (delayed Bauble draws, land upkeep hooks, Pact payments, sagas, rebound: `cardcode.rs:929-980`) before it clears `ring_prot` and `life_locked`.
- `engine/values.rs:196-200`: `untargetable` already treats any `phased` permanent as untargetable, and so does `vondam_elspeth.rs:165-168` (`may_target`). Flagging lands as phased covers every targeting path that goes through these two functions.

Frequency rank (1 = every TP resolution, 4 = unreachable with today's lists). "Reached by" names the pods that seat both the reader's deck and a TP deck. The TP decks are arzakon (t5, t6), kaalia (t2), aurelia (t3) and edgar (t4). Cost is given for the recommended B' design and for A/B (per-reader checks).

### Rank 1: every resolution or every priority window inside a TP window

| Reader | What is wrong today | Cost with B' | Cost with A/B |
|---|---|---|---|
| `zones.rs:711-732` `teferis_protection` | Taps lands instead of phasing them; token counters stay live; floating mana is emptied; no indirect phasing | the core edit | the core edit |
| `turn.rs:1178-1203` `phase_in_for`, `turn.rs:1242-1249` | Nothing to restore today | core edit (restore) | core edit |
| `impls/t45_urza_yawgmoth.rs:167-190` (`untap_lock`), `:227-275` (`apply_lock`) | Lands that TP tapped but that were untapped when TP resolved stay tapped under Winter Orb (only 1 untaps), Back to Basics (nonbasics stay tapped) and Static Orb (2 things). The rules phase them in untapped. Arzakon (about 25 nonbasic lands) against urza-mono-blue-artifacts at t5. | free: lands come back in their own state before `untap_lock` | needs "remember untapped" data |
| `engine/mana.rs:384` (Treasure units in `units`), `state.rs:852-860` (`live_treasures`, `sac_treasure`), `mana.rs:909`, `impls/t3_korvold.rs:135-160` (`sac_artifact`), `ai/act.rs:66` (Clue), `impls/common.rs:1844` (Food), `impls/sheoldred.rs:678` (Blood), plus the `human/` copies (`human/choices.rs:793`, `human/tokens.rs:91-93`, `human/triggers_t1.rs:115-116`) | The protected player pays with Treasures and cracks Clues or Blood during the window. Reached on every `can_pay` and `mana_units` call it makes, which means every counter window and every end-of-turn instant decision. All four TP decks make counters: arzakon (Dockside, Ragavan), kaalia (Ancient Copper Dragon), aurelia (Smothering Tithe, Battle Angels of Tyr), edgar (Black Market Connections, Voldaren Estate Blood). | free | about 20 sites; hard |
| `engine/mana.rs:378-382` Rain of Filth | Tapped lands can still be sacrificed for mana while "phased". No TP deck plays it, so it is unreachable today. | free | 1 line |
| `ai/search.rs:418-432` `strength_seen` (`0.6 * lands`, `0.4 * treasures`) | A forecast reader (see AI behaviour). Hit in every look-ahead evaluation that ends while someone is under TP. | add the stash back (1 line) | unchanged |

### Rank 2: opponents' effects that today's pods reach

| Reader | What is wrong today | Reached by | B' | A/B |
|---|---|---|---|---|
| `impls/lands.rs:1202-1215` Exotic Orchard `orchard_cols` | Reads the phased player's land colours | korvold t3 vs aurelia; chulane/prosper/yuriko t4 vs edgar; kinnan-simic/miirym/zur t5 and kraum/sisay t6 vs arzakon. Hit on every mana computation by those seats. | free | 1 line |
| `engine/mana.rs:329-335` Fellwar Stone | Same | prosper t4 vs edgar; kinnan-simic/zur t5 and dack/kinnan-bonder/kraum t6 vs arzakon | free | 1 line |
| `engine/chars.rs:588-593` `lands_are` (Urborg/Yavimaya), read through `mana.rs:164-171` `granted_cols` | A phased Urborg still makes every land a Swamp and gives {B} | Edgar's Urborg affects all t4 pods, including JD's Sheoldred and Yoshimaru. Hit on every mana computation. | free | 1 line |
| `impls/mine.rs:1922-1944` `strip`, `:1946-1960` `strip_go` | Strip Mine can destroy a phased key land | lord-windgrace t2 vs kaalia (arzakon never faces a TP deck) | free (`strip_go`'s `contains` re-check also fizzles it) | 2 lines |
| `impls/yoshimaru_p7.rs:44-58` `targets_from` (Flickerwisp, part-7b triggers) | Lands are not flagged phased, so `may_target` lets them through | JD's yoshimaru t4 vs edgar | free | free once the flag is set |
| `impls/t12_brago_meren.rs:413-420` `wisp_pick` | Same | brago t2 vs kaalia | free | 1 line |
| `impls/t45_miirym.rs:81-92` `astral_targets` (`any`) | Can copy a phased land | miirym t5 vs arzakon | free | free once the flag is set (`live`/`untargetable`) |
| `impls/rules.rs:933-940` `stuff_of` (Liliana of the Veil -6 piles; Liliana, Dreadhorde General -9 at `:1134-1140`) | Filters phased permanents, but lands are not flagged and the token counters are always included | atraxa t3 vs aurelia; yawgmoth t5 vs arzakon. Rare (ultimates). | free | 2 lines |
| `impls/dockside.rs:58-80` | Already worked around with `life_locked && ring_prot` (`:61`). It never fires in today's pods, because arzakon is the only Dockside deck and never faces a TP deck. Flare of Fortitude plus The One Ring would give a false 0. | none | delete the workaround | keep it |

### Rank 3: correct in principle but unreachable with today's lists

No deck that reads these is seated against a TP deck, or no current list plays the card.
- Land Tax, Gift of Estates and friends: `impls/bulk_white.rs:1167-1170`, `impls/t1.rs:1303-1306`, `impls/rules2.rs:336-345`. Aurelia's Land Tax only counts its own opponents.
- Land targeting: `impls/lands.rs:465-491` (Ghost Quarter, Tectonic Edge: `best_key_land`, `ld_land_options`), `impls/fixes.rs:49-60` (`best_opp_land`), `impls/fixes.rs:365-370` (Decimate: urdragon, never seated with arzakon).
- `impls/t3_korvold.rs:638-662`: Pox is implemented but no list plays it.
- Retired or other-tier cards: `impls/mine.rs:944-1005` (Kefka's ruin, a retired deck), `engine/turn.rs:720-724` (Tabernacle: only dack has it, and dack has no TP), `impls/t45_chulane_kinnan.rs:219`, `impls/t6_legends2.rs:283`, `impls/t6_redirects.rs:203`, `impls/partials.rs:1559-1607`, `impls/t45_winota.rs:646-650`, `impls/t6_lands.rs:412`, `impls/t3_aurelia.rs:351`.
- Landwalk and evasion checks against a defender: `impls/bulk_blue.rs:57`, `impls/t6_counters.rs:62`, `impls/t45_krenko.rs:240`, `impls/cloud.rs:243`, `impls/tymna_cards.rs:1273`, `engine/combat.rs:142-148`. Combat damage to a TP player is prevented anyway.

All of these are free with B'; with A/B each needs a 1-line check.

### Rank 4: AI forecast readers

These estimate a later turn, when the stashed objects are back. Detail is in AI behaviour.
- `ai/search.rs:429-430`
- `engine/mana.rs:1430-1452` `next_turn_mana`
- `engine/stack.rs:1359-1366` `pact_affordable` (paid at the next upkeep)
- `cardcode.rs:1118-1124` `tithe_prio`
- `ai/values.rs:637-651` "fit" (own lands)
- `impls/combos.rs:1151` (t6 land combo pieces; it already ignores phased permanents, so it stays consistent if it ignores stashed lands too)
- `ai/planner.rs:885-910` `window_of`: today it untaps the TP player's tapped lands in its model, so the planner believes a phased player holds counter mana. With B' that belief disappears, which is correct, so this one is not compensated.

These are hard because grep cannot prove the list is complete.

### Structural by-id sites (84 `lands.contains(` sites, 16 `lands.push/retain` sites)

Most give the correct answer for a stashed land ("not a land you control now"):
- `impls/fixes.rs:32` (`destroy_land`), `impls/fixes.rs:392`
- `impls/mine.rs:1956`
- `impls/lands.rs` `has_land`
- `impls/rules.rs:949,1012`
- `human/protect.rs:26`

These need care:
- `engine/control.rs:52-61` (`gain_control`), `:111` (`end_links`), `:152-157` (`return_borrowed`): a land the TP player controls through a link would be pushed into the new controller's `perms` but not its `lands`.
- `engine/turn.rs:497-512` `remove_land` must also drop the land from the stash.
- `impls/alela.rs:1740-1746` (Mirage Mirror copying a land: kinnan decks only) and `impls/t6_green2.rs:564-566` (Shifting Woodland: t6 only). Unreachable today.
- `impls/yoshimaru_p7.rs:98-111` (`in_lands` and `leave_battlefield`) would send a phased land down the non-land `zones::leave` path, but only if something reached it by id. Its own `targets_from` cannot.
- `engine/life.rs:454-491` `eliminate`: hygiene only. An eliminated player's lands already stay listed today (pre-existing).
- `engine/life.rs:360-442` (legend rule and moon saga on lands) is correct with B': a phased legend is ignored.

### Practice paths

None needs a change with B':
- `human/cards.rs:1104-1148,3080` (a person's land-destruction targets), `human/walkers.rs:142`, `human/karn.rs:1180,1254,1267`, `human/specs.rs:157`, `human/gaps_c.rs:466`, `human/marvin_hooks.rs:80`, `human/proliferate.rs:115`.
- `practice/src/view.rs:83-95`: lands simply drop out of the view and the Treasure count shows 0. That matches how phased permanents are already hidden (`practice/static/table.js:188` filters `phased_out`).
- The person casts TP through `engine/cast.rs:1126-1130`, which calls the same `teferis_protection`.

## Design options and recommendation

**A. A player predicate `player_phased(g, p)` consulted by every reader.**
- Every rank 1-3 reader needs an edit (about 150 sites plus about 20 cost sites), and every future reader has to remember it.
- It cannot tell objects that entered after TP resolved from those that phased out. A land or Treasure the protected player gets during the window is a new object and must be visible; A hides it.
- Keying on `life_locked && ring_prot` (Dockside) is unsound: Flare of Fortitude plus The One Ring gives the same pair.
- Rejected.

**B. A `phased` flag on land permanents, plus a phased state for tokens; lands stay in `lands`.**
- Targeting through `untargetable`/`may_target` becomes correct for free.
- Every list reader (461 `lands` reads, including own-mana code once lands are no longer tapped) must add `!phased`. A missed reader silently keeps today's behaviour.
- Rejected on cost, but its flag is kept inside B'.

**B' (recommended). Flag the lands and stash them; stash the token counters.**
- `Game::phased_lands: Vec<PermId>` (global, so control changes work the same way as for perms). Each stashed land has `phased = true` and is no longer in any `Player::lands`. An animated land keeps its `perms` entry, flagged.
- `Player::phased_tokens: { treasures, tapped_treasures, clues, foods, bloods }`, plus `Player::all_phased: bool` (TP is in force until this player's next turn).
- Every list reader sees nothing, which is CR 702.26 "treated as though they don't exist".
- By-id paths see `phased`.
- New objects that enter during the window are live.

How B' handles each concern the brief raised:
- **Mana payment.** Lands, Treasures and phased rocks give nothing, because `units` skips phased perms (`mana.rs:286-289`). Mana from hand (Spirit Guides) and floating mana still work. Mox, Lotus Petal and Chrome Mox are perms and are already skipped. Treasures can no longer be sacrificed because they are not there.
- **Untap and phase-in ordering.** The restore happens inside `phase_in_for`, which `step_start` already calls before `untap_lock` (`turn.rs:1242-1243`). Lands come back in the state they phased out in. Lands tapped to pay for TP come back tapped and untap normally, subject to the Orbs; this is correct. Tapped Treasures are restored before `untap_lock` reads `tapped_treasures` (`t45_urza_yawgmoth.rs:189`), so Static Orb is handled.
- **Land drops.** Unaffected. TP ends before the player's main phase. Lands put onto its battlefield during the window stay live.
- **`control_links`.** `end_links` (`control.rs:101-137`) already ends links whose anchor is phased. `gain_control` and `return_borrowed` must recognise a stashed land: `land = lands.contains(&m) || g.phased_lands.contains(&m)`, and a stashed land is not pushed into the new controller's `lands`. `note_phase_holder` then keys its phase-in exactly as for perms.
- **Fizzles.** Land destruction already re-checks list membership when it resolves (`mine.rs:1956`, `lands.rs` `has_land`, `fixes.rs:32`), so with the stash it fizzles with no edit. DSL and targeted effects re-check through `untargetable`, which sees the flag.
- **Determinism.** No RNG is drawn. The stash is a `Vec` in phase-out order. Restore uses a stable `retain`, and restored lands go ahead of any lands that entered during the window. `search::clone` copies the new fields (derived `Clone`), and `determinize` does not touch them.
- **L2.** With no stash, every new path is a no-op. Pods that seat none of the four TP decks must be byte-identical (proof under Implementation plan).

**C. Token counters as real permanents.**
- This would also fix the separate artifact-wipe gap T1. But it rewrites about 160 counter sites and every mana-payment path, and it changes every game that contains a Treasure (all tiers), so L2 could not isolate the effect.
- Rejected for this ticket. It belongs to T1 if JD wants it.

### Decisions JD must make (each is part of "Full")

- **D1. Stop emptying the mana pool** (`zones.rs:715`). The rules do not empty it on phasing, and a person's `pool` is not emptied today either. Recommended.
- **D2. End TP at the start of the turn.** "Until your next turn" ends as that turn begins; this is my reading of the Gatherer ruling for the card (verify). The fix: when `all_phased` is set, `phase_in_for` clears `life_locked`, `ring_prot` and `all_phased`. Today the lock still holds through `cardcode::turn_start` (`turn.rs:711-716`), so for example a delayed Bauble draw at that upkeep would ignore an opponent's Sheoldred. Recommended. The One Ring keeps its upkeep clear; that is ticket T3.
- **D3. Indirect phasing** (CR 702.26 on attached permanents). Any player's Aura or Equipment attached to a phased permanent phases out with it, with `PhaseInWith = p` when another player controls it. `human/cards.rs:1755-1768` already does this for Clever Concealment. Recommended.
- **D4. Keep the look-ahead decision-neutral.** The forecast readers count the stash (see AI behaviour). Recommended.
- **D5. Flare guard.** `turn.rs:1223-1225` clears `life_locked` at any turn start when `life_lock_eot` is set, which would cut TP's lock short if Flare of Fortitude and TP were both used. Skip that clear while `all_phased` is set. This is one line and unreachable today (winota is not in the pool). Recommended.

### Separate findings (not this ticket; JD must accept or ticket them)

- **T1.** Artifact wipes never destroy Treasure, Clue, Food or Blood counters, nor artifact lands: `engine/removal.rs:482-547` iterates `perms` only. This covers Vandalblast (aurelia, krenko, prosper), Farewell and Austere Command.
- **T2.** AI player-targeting ignores protection from everything in at least Liliana of the Veil -6 (`impls/rules.rs:1045-1047` picks the opponent by board value).
- **T3.** The One Ring's "until your next turn" ends at upkeep, not at turn start (`turn.rs:712-715`). Changing it moves every One Ring pod.
- **T4.** The look-ahead values phased permanents at 0 although they come back (`ai/search.rs:422-424`, `engine/values.rs:469-472`), so an opponent who cast TP looks boardless.

## AI behaviour

When the AI casts TP (none of this reads the protected player's lands or counters, so the fix leaves it unchanged):
- Lethal combat: `engine/combat.rs:912-933` calls `zones::last_chance` (`zones.rs:762-771`).
- A drain or combo win: `ai::win` (`ai/mod.rs:200-215`).
- Targeted removal: PROTECTORS row `ai/pool.rs:1094`. With the heuristic's limit it is used only for the commander or 40% of the board (`worth_whole_board`, `ai/pool.rs:1343-1349`, `:1374`), plus the plan rows' `protects`: `data/plans/arzakon.toml:136-138` (commander, The One Ring) and `data/plans/aurelia-boros-extra-combats.toml:36-38`. The hold rows act only through `protects`: TP is a NO_DSL_HOLD (`ai/plan_data.rs:1392`, `ai/brain.rs:366-370`).
- Wipes: `wipe_with`, `ai/pool.rs:1576-1584`.
- The phase4 protect/wipe searches.

The only decision inputs are the caster's own mana before TP resolves, its hand, and perms values.

What changes after TP resolves ("effects now visible or invisible"):
- The protected player can no longer cast or activate with Treasure, Clue or Blood counters during the window.
- Opponents' Orchard and Fellwar colours, Urborg's grant, Strip Mine targets and blink targets no longer include the phased lands.
- `planner::window_of` stops imagining counter mana on a phased player.

Without D4, the look-ahead would also change for evaluation reasons: `strength_seen` would drop 0.6 per stashed land and 0.4 per stashed Treasure. With D4 it is identical in TP states. Inside look-ahead playouts, futures still differ because the phased player has no mana. That is an effect becoming invisible, not an evaluation change.

Exploits that exist today:
1. Strip Mine destroys a phased player's key land (`mine.rs:1922-1960`; Windgrace vs Kaalia). It is a real destroy of a land that should be immune.
2. The protected player's own Treasure mana during the window.
3. Untap locks strand the protected player's lands (Arzakon vs Urza). This is a penalty, not an exploit, but it is the largest single effect on JD's numbers I can see.
4. Opponents gain colours from a phased player's lands.

No mass land destruction (Armageddon and the like) is in any current list. The Pox code exists but no list plays it. "Opponent has N lands" counters (Land Tax family) are never seated against a TP deck today. With B' all of these close without per-reader edits.

## Status

- **Teferi's Protection.** Full after B' with D1-D3 and D5. Without D1 or D2 it stays below Full, with those named as limits for JD. The current `Full` plus "lands too" note is inaccurate. Arzakon's recorded numbers were measured with the approximation, and the audit-mine gate passed only because the label overstated the model. Update the note in `data/card_overrides.toml:4990-4992` (lands and the four token kinds phase out and back in, untapped lands come back untapped, floating mana stays, attached permanents phase with their host, ends as the next turn begins), then run `cardgen export` (a data change for JD's plan). Remove the `audit/GAME_CHANGERS.md:55` approximation line.
- **Dockside Extortionist.** Stays Full. Only its note text changes (`data/card_overrides.toml:1244`, "keyed on `life_locked` with `ring_prot`" becomes "phased lands and tokens are not listed").
- **Cards that become more correct, with no status change:** Strip Mine, Exotic Orchard, Fellwar Stone, Urborg, Tomb of Yawgmoth, Astral Dragon, Flickerwisp and the other `targets_from` users, both Lilianas' ultimates, Winter Orb, Static Orb and Back to Basics.
- **No card drops below Full or becomes Approximate.** T1-T4 are pre-existing gaps that this inventory surfaced. They do not come from the change. Each needs JD's decision (ticket or named limit) and must not be accepted silently.

## Implementation plan and tests

One Sonnet worker, in this order:

1. **`state.rs`.** Add `Game::phased_lands: Vec<PermId>` and initialise it in `Game::new`. Add a `TokenStash` struct, plus `Player::phased_tokens: TokenStash` and `Player::all_phased: bool` in `Player::new`. Add a helper `phased_lands_of(g, q)`.
2. **`engine/zones.rs:711-732` `teferis_protection`.**
   1. Set `all_phased`.
   2. Do not clear the mana pool (D1).
   3. Phase out the perms, then everything attached to them (D3).
   4. For each land in `take(pl.lands)`: set `phased = true`, push it to `g.phased_lands` in order, and leave `tapped` as it is.
   5. Move the five counter fields into `phased_tokens`.
   6. Then `end_links`, then `bf_ver += 1`.
   7. Keep the log line text.
3. **`engine/control.rs:44-90, 101-137, 144-171`.** Make the land test stash-aware, keep stashed lands stashed, and in the `end_links` dead-owner branch also `retain` the stash.
4. **`engine/turn.rs`.**
   - `phase_in_for` (`:1178-1203`): restore stashed lands with the same predicate as perms (`PhaseInWith == p`, or no key and `owner == p`, or the key's player is dead and `owner == p`). Drop entries with `!on_bf`. Clear `phased` and `PhaseInWith`. Rebuild `lands` as restored lands followed by any that entered meanwhile. Restore the token counters. If `all_phased`, clear it together with `life_locked` and `ring_prot` (D2). Bump `bf_ver` when anything was restored.
   - `remove_land` (`:497-512`): also retain the stash.
   - `:1223-1225`: the D5 guard.
5. **`engine/life.rs:454-491` `eliminate`.** No behaviour change needed. Add a test that a dead player's stash leaks into no reader.
6. **`impls/dockside.rs:55-63`.** Delete the `life_locked && ring_prot` skip and fix its doc comment.
7. **Forecast readers (D4).** Add the stash back in:
   - `ai/search.rs:429-430` (stashed lands × 0.6, stashed Treasures × 0.4)
   - `engine/mana.rs:1430-1452` (stashed lands' amounts)
   - `engine/stack.rs:1359-1366` (stashed lands, blue ones too)
   - `cardcode.rs:1118-1124`
   - `ai/values.rs:637-651`

   Leave `ai/planner.rs:885-910` as it is.
8. **Tests.** A new file `tests/cards_teferis_protection.rs`. Update `tests/rules.rs:224`: after the change that assertion passes vacuously (an empty `lands`), so it must assert the stash instead.
9. **Data and docs.** The two override notes, `cardgen export`, and `audit/GAME_CHANGERS.md:55`. Format only the changed files with rustfmt.

Red-green tests. Each fails today unless marked "coverage"; prove them with `scripts/red-green.sh`.

1. After TP, `lands` is empty, every former land is `phased`, and lands that were untapped are still untapped.
2. Back to Basics: a TP player with 4 untapped nonbasic lands has them untapped after `step_start` of its next turn.
3. Winter Orb: same as test 2.
4. Static Orb with 2 Treasures, 1 of them tapped: the restored tapped count enters the lock.
5. Two Treasures before TP: `can_pay(g, p, 1, "", false)` is false during the window, and `mana_units` has no Treasure units.
6. All five counters (including `tapped_treasures`) are 0 during the window and restored at `phase_in_for`.
7. A Treasure created for the TP player during the window is live (coverage, A's failure mode).
8. Strip Mine `strip` offers nothing against a phased opponent whose only lands are key lands.
9. Strip Mine already activated, TP resolves before `strip_go`: the land survives (fizzle).
10. Ghost Quarter: `best_key_land` skips phased lands.
11. Exotic Orchard: no colour from a phased opponent's lands.
12. Fellwar Stone: same as test 11.
13. A phased Urborg grants nothing: `lands_are(g, 'B')` is false.
14. `targets_from` excludes phased lands.
15. `astral_targets(any)` excludes phased lands.
16. `stuff_of` of a phased player is empty (no lands, no tokens).
17. Dockside with `life_locked` and `ring_prot` but no TP (Flare plus Ring) counts the board, 4.
18. Floating mana survives TP (D1).
19. `life_locked` is false right after `phase_in_for`, before `upkeep` (D2).
20. An opponent's Aura on the TP player's creature phases out and back in with it (D3).
21. A Flare lock and TP together: the next opponent's turn start does not end TP's lock (D5).
22. A land held through a WhileControls link whose anchor phases out returns to its owner's `lands` (never `perms` alone) at the holder's untap (coverage).
23. Invariant property test (coverage): random sequences of TP, `gain_control`, `remove_land`, `destroy_land`, `eliminate` and `phase_in_for`. Every land with `on_bf` must be in exactly one of an alive or dead player's `lands` or `g.phased_lands`; animated lands may also be in `perms`.
24. `strength(g, q)`, `next_turn_mana` and `pact_affordable` are equal before and after TP (coverage, D4).
25. A person's TP (practice `cast.rs:1126-1130`): their phased lands are absent from `human_sources` and from the land-destruction target list `human/cards.rs:1104` (red today).

Checks:
- Check disk space first: `df -BG --output=avail /` must show at least 150 GB.
- Focused: `cargo test --release -p sim-core --test cards_teferis_protection --test cards_dockside --test rules --test cards_urdragon_jd --test cards_kyodai --test ai`, memory-capped as `scripts/GATES.md:6-8` prescribes.
- Then `scripts/stage-gate.sh` with the full suite (`suite`, `audit-mine`, `export`, `fmt` and the rest).

L2 and L3, coordinator only:
1. Record base and branch with `trace-record.sh --mode l2`.
2. Run `trace-compare.py --allow arzakon`. Every other non-SAME record is a failure unless its `seat_order` (in `records.jsonl`) contains "Kaalia of the Vast", "Aurelia" or "Edgar Markov". Write a 10-line check over both `records.jsonl` files.
3. Expected non-SAME records: arzakon/t5 and arzakon/t6; within/t2, t3 and t4 and sheoldred/t3 and t4 and yoshimaru/t4 only where those seats are present.
4. Expected byte-identical (SAME): yshtola/t5, urdragon/t5, yoshimaru/t5, within/t0, t1, t5 and t6, and every pod without a TP seat. Any other non-SAME record is a bug.
5. First-difference check: under the look-ahead, a difference can appear before the first real TP cast. `determinize` (`ai/search.rs:123-150`) deals TP into a TP deck's hand inside playouts. So for a mechanical proof, replay the non-SAME seeds with `--ai adaptive`. There, the first differing line must be at or after the first base-log line containing "Teferi's Protection". Verify that adaptive also disables the phase4 protect/wipe searches; if it does not, the same playout caveat applies.
6. L3 with `--changed-cards "arzakon:Teferi's Protection"`: no new panics or timeouts, and the card is still cast.
7. Goldfish arzakon: expect identical (TP is only cast in response).

Effort: about 8 worker hours (core 2, control/remove_land/eliminate 1, forecast readers 1, 25 tests 3, notes/export/format 1), plus suite wall time. The coordinator's L2/L3 runs are extra.

What an independent adversarial reviewer should attack:
- the land-list invariant (test 23), using extra turns, TP cast on its own turn, a second phasing effect in the window, an animated land, and a land stolen while phased;
- `phase_in_for` order against `untap_lock`, and `tapped_treasures` under Static Orb;
- whether any reader of `Player::lands` for an opponent expects a phased land to be there (search for forecast readers grep missed: `lands.len()` used as a strength or mana proxy);
- determinism: the same seed played twice, and `search::clone` copying the stash;
- the practice human TP path and Undo/tape snapshots;
- that pods without a TP seat are byte-identical;
- that removing the Dockside workaround cannot zero or double-count anything.

## Risks

- **Forecast readers grep missed.** Any AI heuristic that reads `lands.len()` or `treasures` of a phased player as "what they will have" silently changes in TP pods. I found 7; I cannot prove there are no more. The drift is limited to TP windows in TP pods.
- **By-id code that reaches a stashed land.** It would treat the land as a non-land: control changes, Mirage Mirror, Shifting Woodland, `yoshimaru_p7::leave_battlefield`. I found no reachable path in today's pods. The invariant test is the guard.
- **JD's baselines move.** Arzakon t5 against Urza (Orb and B2B no longer strand its nonbasic lands; no Treasure mana in the window) could move noticeably. Sheoldred t3 (Aurelia), Sheoldred and Yoshimaru t4 (Edgar: Urborg colours, Flickerwisp targets, Edgar's Blood and Treasure use in the window) and the within t2-t4 pods move too. Re-baselining needs sims, which need JD's approval.
- **The L2 proof under the look-ahead is weaker for TP pods.** Divergence can precede the first real TP cast through playouts. Only non-TP pods can be proven identical by hash. TP pods need the adaptive replay.
- **D1 (floating mana kept)** could expose AI code that assumed TP empties the pool. It is rare, because the AI seldom floats mana across a response.
- **D2 changes when TP's life lock ends.** The protected player can now lose or gain life during its own untap and early upkeep (delayed draws, land upkeep hooks). This is rules-correct but differs from today.
- **The current Full label is wrong.** Past Arzakon findings were measured with the approximation, and JD may want that noted in `audit/arzakon/FINDINGS.md`.

## What I could not determine

- How often TP is cast per game in each pod, and how often the protected player spent counters during the window (no sims allowed). An existing L2 log grep for "Teferi's Protection" followed by Treasure payments before that player's next "turn" line would measure it.
- Whether `--ai adaptive` turns off the phase4 protect/wipe searches (`ai/ismcts.rs`), which the mechanical first-difference proof needs.
- The exact CR or Gatherer citation for "until your next turn ends as that turn begins" (D2). My reading is from the card's rulings; verify it before citing it in a note.
- Whether more forecast readers exist beyond the 7 listed, and whether other AI player-targeting sites ignore `ring_prot` (T2: only Liliana of the Veil -6 was checked).
- The practice front end's treatment of lands missing mid-game. It hides phased permanents, so lands vanishing should be consistent, but I did not run it.
- Whether JD wants T1 (Treasure tokens and artifact lands vs artifact wipes) folded into a token-permanent redesign (option C) later. That decision changes every Treasure pod and is out of this ticket's scope.
