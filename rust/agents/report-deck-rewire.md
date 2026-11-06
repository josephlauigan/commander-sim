# Report: deck-rewire (JD's two new decks wired in; karn and urdragon retired), 2026-11-05

Brief: `rust/agents/deck-rewire.md`. The agent returned this report as text; the coordinator saved it and added the
coordinator's fixes at the end.

## Tier 6 decision (JD, applied)
- `impls/t6_plans.rs` T6_KEYS and the `t6stats.rs` copy: "thanos" replaced by "thrasios". thrasios removed from
  `HAND_DELETED` (now 7 entries) and from `jd_deck`, like the other T6 keys. The Tier 6 plan slots (prio, wish, keep,
  hold_up, go_now) go to thrasios.
- thanos plays the ordinary pool-style rules from `thanos.toml` (`[style]`, `[commander]`, `[key_cards]`, `[[line]]`,
  `[[wish]]`, `[[prio]]`, `[[hold]]`, `[mulligan]` with `fast_mana = true`, read through `plan_data::keep_for`).

## 1. `data/my_decks.toml`
thrasios -> `JD/tymna-thrasios-doomsday.md`, tiers ["t6"] (name, commanders, ident WUBG unchanged and match). thanos ->
`JD/thanos-artifacts-and-dragons.md`, tiers ["t5"]. karn and urdragon removed. `data/decks.json` and `data/cards.json`
re-exported (Doomsday was fetched into `data/scryfall_cache.json`; `cardgen export --offline` is clean after).
`tests/fixtures/my_decks_parsed.json` produced with jq from decks.json in my_decks.toml order; the deck-guard test passes.

## 2. Old lists
The four old lists `git mv`'d to `decklists/pool/retired/` (karn-colorless-artifacts, ur-dragon-dragon-copies,
tymna-thrasios-partners, thanos-five-colour-turbo), notes and Simulator rows fixed. `decklists/CHANGELOG.md`,
`decklists/pool/README.md` (retirement line), `documents/testing-decks.md`, `README.md` updated; the new lists'
Simulator rows updated.

## 3. Printings
The two new files `git mv`'d over `data/printings/thrasios.txt` and `thanos.txt`; `karn.txt`, `urdragon.txt` removed.
Practice reads `data/printings/<key>.txt`; its tests pass.

## 4. Plans and Spellbook
- Removed `plans/karn.toml`, `plans/urdragon.toml`, `spellbook/karn.json`, `spellbook/urdragon.json`.
- `thrasios.toml` (non-Doomsday only): rules for Bastion Protector, Faeburrow Elder, Talisman of Curiosity removed; Tier 6
  sections added (Oracle + Consultation/Pact line, `[style]`, `[sets]`, `[mulligan]` with fast mana, from the old Thanos
  turbo plan cut to the new list). No `[[wish]]`, `[[hold_up]]`, `[[go_now]]` yet.
- `thanos.toml` rewritten for the new list (every card checked): Basalt Monolith + Rings + a sink (Ballista, Mystic Forge
  or Urza); Astral Dragon + Cursed Mirror; the same with Terror of the Peaks; wipe then a big threat; `[[wish]]` from the
  pilot guide; `[[prio]]`; a Teferi's Protection hold; Thanos cast priority 88 from turn 2; the keep rule. Style,
  priorities and keep rule are PROVISIONAL. The power-up parity / Rings-copy choice, wiping vs winning and the Yawgmoth
  rule are not expressible in a plan file (card code); the file says so.
- Spellbook refetched (`scripts/spellbook-fetch.sh`): thanos 2 combos, thrasios 4.

## 5. Audit
`audit/karn`, `audit/urdragon` -> `audit/retired/` with a "Retired 2026-11-05" line. thrasios and thanos: old FINDINGS.md
-> `FINDINGS-old-list.md`; new FINDINGS.md (list, tiers, "no sims yet"). References in CHANGELOG, `audit/kraum`,
`audit/sauron` fixed.

## 6. Code removed (deck-key code only; every card model kept)
- sim-cli: `decks.rs` FILES/MINE, `report.rs` labels and milestone, `poolmode.rs` expected tiers (thrasios t6, thanos t5),
  `args.rs` DECK_KEYS.
- practice: catalog and setup MY_DECKS.
- sim-core: `ai/mod.rs` (MAIN_STYLES, jd_deck, HAND_DELETED), `settings.rs` cthresh, `engine/cast.rs` commander
  importance, `impls/rules.rs` CTHRESH, `impls/combos.rs` (two `key != "karn"` gates), Karn's `search_bonus` and its
  three uses in `ai/search.rs` (`deck_extras` kept as a zero stub: `adapt_rollouts` and `pick_with` take a slice),
  `milestone("karn")` in `impls/karn.rs`, `t45_urdragon.rs` `scion_options` gate now `"scion"` only.
- `data/card_overrides.toml`: Ephemerate's tag `ephemerate` dropped (no such engine tag; the override began applying
  once no JD deck hand-tags Ephemerate); its cards.json entry is unchanged except the source field.
- Tests removed with the code: the `milestone("karn")` assertion in `cards_karn::karn_adds_one_colourless_per_artifact_at_upkeep`;
  `cards_scion::jds_urdragon_deck_uses_scions_copy_ability_through_the_card_hook`; `cards_urdragon_jd::the_urdragon_seat_plays_from_its_plan_file`
  (replaced by `the_thanos_seat_plays_from_its_plan_file`); practice `play_bot_games` `karn_t4`, `thanos_t6` removed,
  `urdragon_t5` -> `thanos_t5`, `thrasios_t5` -> `thrasios_t6`.

## 7. Tests re-seated
- `ai_planner` thanos -> thrasios (T6, has the Oracle and tutors); `ai_fix_bugs`, `play_thanos_human` thanos -> the
  retired scion seat (they need Underworld Breach and Brain Freeze).
- 11 -> 9 deck sets: `hand_off`, `thrasios_ai` (reads the T6 plan slot via `deck_prio_f`), `practice_rings_probe`,
  `practice_trigger_probe`, practice `http_catalog`, `app_entry`, `http_server`, `server_fake`, sim-cli `decks.rs`,
  `realism.rs` (`--deck karn` -> `thanos`).
- practice `play_bot_games` Bot answers "mode" requests (The Mightstone and Weakstone) with Pick(0) (fixture limit).
- `testkit::retired_decks` gained "karn" and "urdragon" seats built from the real retired lists (`listed(...)`, as
  omnath, godo, scion); a dead shadowed "thrasios" commander-plus-basics entry dropped. The other karn/urdragon tests
  are unchanged (NEEDS JD 1).

## 8. `binder-cards.txt`
177 distinct cards added from the swap guide's binder lists (basics skipped; no set codes; `Nx` counts, some merged).
317 lines. Removed 1x Chromatic Lantern, 1x Dark Ritual, 1x Eladamri's Call (now in the new lists; NEEDS JD 3).
Command Tower, Lightning Greaves, Worldly Tutor stay (the guide adds a spare copy of each).

## 9. CLAUDE.md
Tier line updated (thanos t5, thrasios t6, karn and urdragon retired 11/05). AGENTS.md has no tier list: unchanged.

## NEEDS JD (from the agent)
1. karn/urdragon tests kept on retired test seats, not re-seated on the new thanos: re-seating would drop coverage of
   cards not in the new list (Karn, Legacy Reforged; Dragonlord Dromoka; Korvold ...). No test weakened.
2. The thrasios Tier 6 plan is provisional and untested by sims; Doomsday lines are PR 2.
3. Binder: restore the 3 removed lines if they were genuine spare copies.

## Coordinator's checks and fixes
- Build without warnings; `cardgen export --offline` clean; `cardgen audit --mine`: only Doomsday (thrasios) below Full.
- Full suite: 4246 passed, 2 failed, 5 ignored. Both failures fixed by the coordinator:
  - `game.rs` `every_deck_finishes_games_at_every_tier` expected 11 owned decks (77 games): now 9 (63).
  - `practice_choice_probe`: 6 entries of retired decks removed; two real gaps listed in `TO_FIX`: "thrasios: Doomsday"
    (no card code: PR 2) and "thrasios: Ponder" (a person isn't asked to order the top three or to shuffle; PR 2 fixes it).
  - Both binaries re-run: pass.
