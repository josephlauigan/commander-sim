# Report: cand2-rakdos (The Lord of Pain, Smaug the Impenetrable)

Model: claude-sonnet-5-5. DONE. All brief cards Full in `cardgen audit --mine` (remaining non-Full cards in both decks are the other agents': Apex Witchstalker, Necrosquito, Salvaged Manaworker, Key to the Side-Door, Dread Drone).

**Process slip:** I ran `python3 --version` once (forbidden by the rules). It only printed the version; no Python code ran.

## Cards (impls/lordofpain.rs holds the shared ones; impls/smaug.rs; tests/cards_rakdos.rs, 19 tests)
- The Lord of Pain: `cast` hook; first spell of each player each turn (via `turn_casts`), damage = mana value ("triggers" damage from the Lord's controller). Target: opponent of the Lord's controller who is not the caster, lowest life after the damage then highest threat; the controller only if nobody else is legal. No-lifegain static was already compiled.
- Smaug: `smaug::dealt_damage` makes that many Treasures; called from `apply_removal` (all "dmgN" removal/burn, even though Smaug is indestructible) and the dsl's damage-to-creatures loop (sweepers). Combat damage never calls it.
- Charforger: dies hook (creatures/artifacts incl. tokens) + sacrifice hook (Treasure/Food/Clue tokens) add oil counters; option to remove three and exile the top card (playable this turn, via `impulse`). ETB Goblin was already compiled.
- Kalain: etb hook adds +1/+1 counters equal to Treasure mana paid for the creature's cast.
- Lagomos: dsl cleared (it wrongly tutored freely and made the token at upkeep). combat_start hook makes the 2/1 red trample haste Elemental (marked `DataKey::EotSac`); option for the tutor only with 5+ creature deaths this turn.
- Hero's Heirloom: `grant_kw` trample/haste to a legendary equipped creature.
- Ultimate Magic: Meteor: `resolve` runs the compiled 7 damage, then if cast from foretell also destroys one artifact (pval>=2) else best land per opponent. Foretell is an AI option ({2}, kept in hand marked in `impulse_long` like Irascible Wolverine's plot); the foretold cast is another option, taken when it kills more of theirs than mine.
- Not modeled and noted: human (practice) players' Treasure payments are not counted for Kalain; Meteor foretold card can be discarded like any hand card.

## AI plans
- lordofpain (`lordofpain_prio`): with the Lord out, on my turn before any spell, the biggest affordable spell (3+) gets +12; spells 2+ cheaper (not lands/ramp/commander) get -30.
- smaug (`smaug_prio`): Smaug 97 once 7 mana is available, early Treasure/mana makers at least 70. Pings: Viashino Fangtail option, Orcish Cannonade option (Action::Removal on Smaug, life > 15), and `bolt_something` plus the dsl any-target fallback aim at Smaug instead of face when nothing worth killing and no lethal (covers Goblin Medics, Arsonist, Bomber Corps).

## Shared-file changes (all small)
- state.rs: `Game::{died_count, treasure_paid, cast_treasure}`, `DataKey::EotSac`.
- engine/mana.rs: `pay` resets `treasure_paid`, `spend_unit` counts Treasure. engine/cast.rs: `cast_card` records `cast_treasure`; `resolve` clears it; `bolt_something` Smaug fallback. engine/zones.rs: `die` counts creature deaths. engine/turn.rs: `end_step` sacrifices `EotSac` permanents. engine/removal.rs: `apply_removal` calls `smaug::dealt_damage`. dsl/mod.rs: damage loop calls it; any-target fallback for the smaug deck.
- data/card_overrides.toml (7 status/dsl entries appended), data/cards.json + decks.json re-exported.

## Checks
cargo test --release: all pass. Build --all-targets: no warnings. all_decks adaptive 3 and lookahead 1: no panic (under 8G cap). Smoke runs 48 games lookahead for lordofpain and smaug: finish clean.
