# Report: retire-jodah

Removed the Jodah deck (key `jodah`); card code kept, module renamed `impls/jodah.rs` -> `impls/legends.rs`.

## Removed
- decklists/JD/jodah-wubrg-legends.md, jodah-illustrators.md, `[jodah]` in data/my_decks.toml, fixture entry,
  `cardgen export` rerun (data/decks.json -108 lines; cards.json: Karplusan Forest loses `pain` since no deck plays it now;
  the seven `prot=jodah` tags renamed to what each card does: swat/ind/hex/hexind, no code reads those values).
- Key from sim-cli decks.rs (FILES, MINE 11 -> 10), args.rs, report.rs, README(s), ai/brain.rs (hold block, special_options
  arm, GENERIC_PLAYS), ai/decks.rs (prio, tutor, protect, wipe_response), ai/mod.rs (MAIN_STYLES), ai/plans.rs
  (`response_only`, and its use in brain.rs), engine/cast.rs, engine/zones.rs, impls/rules.rs (CTHRESH), settings.rs,
  practice catalog.rs / session/setup.rs (MY_DECKS 7 -> 6), and the counts in practice and sim-core tests (game.rs 66 -> 60).
  `grep -rn '"jodah"' rust/crates` finds nothing.
- Deck-only AI in legends.rs: jodah_prio, jodah_tutor, the whole protection machinery (PROTECT table, cast_protection,
  rune_save, plaza_save, key_legend, jodah_protect, jodah_wipe_response, jodah_hold, response_only), jodah_options
  (equipment moves) and its equip helpers, runes_home, the A/B switches (JODAH_AI, JODAH_HOLD), the `jodah` milestone.
  Sisay's pick no longer asks the tutor list (plain dearest-legend rule).
- Notes in data/card_overrides.toml that described Jodah's AI.

## Kept
All card models in legends.rs (Jodah, Kaldra, Garland, Tymna, walkers, Profane Tutor/suspend, Memory Jar, Plaza/Territory
colours, Mithril Coat, Command Beacon, Nexus, Sisay, Shalai, ...). Cloud's/pool protection (Giver/Mother of Runes) lives in
ai/pool.rs and ai/plans.rs and was never routed through jodah.rs, so it is unchanged.

## Tests
tests/cards_jodah.rs -> cards_legends.rs: seats Jodah as commander on an `alela`-keyed table (`jodah_table`); deleted the
deck-AI tests (priority, tutor picks, protection section, equipment moves, hold); kept a Mithril Coat -> Jodah test;
Unclaimed Territory's Human check uses the Winota pool deck. `jodah` seats in ownership, play_marchesa, land_entry,
cards_seph, practice session_shadow/play_bot_games, cardgen workflow (now uses kefka) switched to other decks; the
Mother-of-Runes-stays-home test in cards_rules.rs was deleted (that AI is gone).

## Results
- `cargo test --release --no-fail-fast`: all pass (one run showed `http_play::test_review_and_try_it` fail; the next full run
  passed it, same intermittent http_play flake as in report-retire-elrond).
- `cargo build --release --all-targets`: no warnings. `all_decks adaptive 3` (180 games) and `lookahead 1` (60): no panic.
- `cardgen audit --mine`: no jodah; only the 3 non-Full cards of other decks that were there before (Venat, Fatehold Charm,
  Initiates of the Ebon Hand).
- kefka, cloud and sauron, t3 loose lookahead 240 games seed 500000: output identical to the base commit (progress and timing
  lines aside; base sauron run used the base tree via COMMANDER_SIM_ROOT because the deck doc is deleted).

## Notes for the lead
- Not touched: bulk-cards.txt, binder-cards.txt (inventory is yours), docs in rust/agents/ that mention Jodah.
- Comments in partials.rs and tests that retell the 10-12 Jodah recursion bug are left as history.
