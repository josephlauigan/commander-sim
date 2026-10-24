# Report: sheoldred-upgrade (10/11)
Cards (impls/sheoldred.rs, `register_upgrade`; tests at the end of tests/cards_sheoldred.rs, 11 new):
- Seizan: `upkeep` hook (at once) for every player's upkeep: 2 life loss ("other", not damage), then draw 2. Draws fire Sheoldred (both sides) and Dreams.
- Underworld Dreams: `draw` hook (at once): opponent drawers take 1 damage (kind "triggers", damage = true: prevention applies).
- Dark Deal: all alive players discard the hand (caster first), then draw count-1 (none for 0 or 1 cards).
- Painful Lesson: target (`ctx.face`) draws 2, then loses 2 life.
AI (key sheoldred only): Seizan prio 80 with Sheoldred out, 50 at life >= 30 when own hand + 4 <= biggest opposing hand, else 0. Dreams prio 90.
Dark Deal option (Sheoldred out, opposing hands >= 4, own hand after casting <= 2). Painful Lesson option aims at the lowest-life opponent with Sheoldred, else self (life > 6).
Helm goes on Seizan (flag bit 56 in the equip plan arg) once Darksteel Plate is on Sheoldred; otherwise on Sheoldred as before.
Data: the four cards were not in data/scryfall_cache.json: appended slim records (fetched from Scryfall), overrides added (Full), `cardgen export --offline`
(cards.json now holds them). Gotcha: an override `tags = ""` wipes the autotagged pow/tgh/leg; Seizan uses tags = "pow=6 tgh=5 leg".
Simplifications: Seizan/Dreams triggers resolve at once (no stack responses).
Checks: cargo test 1195 pass 0 fail; no warnings; all_decks adaptive 3 and lookahead 1 ok; audit sheoldred 0 not Full; 48-game smoke ok;
swap smoke (24 games, all four swaps): 16.7% -> 37.5%, within noise.
Shared files: data/scryfall_cache.json, data/card_overrides.toml, data/cards.json (export). No engine changes.
