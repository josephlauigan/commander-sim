# Report: sauron-threats
Branch sauron-threats. Code: rust/crates/sim-core/src/impls/threats.rs; tests: tests/cards_sauron_threats.rs (15, all pass).
Done (all Full in card_overrides.toml, each with a test): Lazotep Reaver, Grim Initiate (compiled abilities; amass puts counters on any Army, tested), Torment of Hailfire (opponents take the cheapest way out each iteration; Sauron option when it kills or late), Grishnakh (amass Orcs 2 + steal until end of turn; the wrong compiled haste-to-all was removed with dsl = []), Dreadhorde Invasion (lifelink clause; the Army counts as a Zombie while Invasion is out), Dualcaster Mage (flashes in over a worthwhile spell, keeps X; NO combo, per lead), Exsanguinate / Talisman / Rakdos Signet / Fellwar Stone checked working in Sauron.
Dropped by instruction: Twinflame + Dualcaster combo (code removed; Twinflame untouched).
AI: sauron_extra_prio (early amass cards/Invasion above creatures, below rocks at 80; Grishnakh high when a victim exists; Dualcaster held until turn 6).
Shared-file edits: ai/decks.rs (3 lines in sauron_prio), impls/mod.rs (module + register), impls/yshtola.rs (x_option gate also allows "sauron"), data/ (cards.json, scryfall_cache.json: 3 cards added by hand from Scryfall since export only fetches deck cards).
Not tested: an opponent actually countering Dualcaster (AI declined in a trial).
Smoke (96 games, t3 loose lookahead): backup-win swaps win 19.8%; speed swaps win 34.4%. 96 games is below the card report's 100-game threshold, so per-card cast rates are not available; rerun with more games for them.
Checks: cargo test --release all pass; build --all-targets no warnings; all_decks adaptive 3 and lookahead 1 finish.
