# Report: practice-partners (Partner decks playable by a person)

## What changed
- `practice/src/catalog.rs`: `my_deck_files` no longer drops decks with `commander2`; each catalog deck now has `commander`,
  `commander2` (or null), `commanders` (list), `ident`; `display_name` reads `**Commanders (2).** A, B, C` as "A and B" (short
  name of the second: "Tymna the Weaver and Kraum"); `commanders()` (image list) includes both.
- `practice/src/view.rs`: partner seats add `commander2`, `commander2_in_zone`, `tax2`, `commander_damage_by`
  (`{who: [by first, by second]}`); `commander_damage` is now `cmd_dmg_from` (the most one commander dealt, rule 903.10a).
- `sim-core/src/human/human.rs` (shared): `{"do":"cast","zone":"cmd","cmd":i|"name"}` picks the commander (default: first
  in the zone, as before). `commander_of` is the only new code. check_cast already refuses one not in the zone.
- `practice/src/shadow.rs`, `advisor.rs`: both commanders in the AI comparison / hint (`cmd2` field; label resolves the
  chosen commander; advisor lists every commander in the zone).
- `static/app.js`, `table.js`, `styles.css`: two overlapping portraits on the deck buttons (setup and join), a
  command-zone box with both commanders (each with its tax, click casts that one, the list button too), opponents' chip per
  commander, per-commander damage tooltip.
- **Stale deck list fixed (needs your eye):** `MY_DECKS` in `catalog.rs` and `session/setup.rs` was still the old 6 decks, so
  `Session::new` / `/api/new` refused kefka, titania, sheoldred, cloud, kraum and sephfs ("unknown deck") although the
  catalog listed them. Both are now all 12 keys of `my_decks.toml` (its order) and a test checks them against decks.json.
  The error text in `server_fake.rs` changed with it.

## Not changed / notes
- Moving a dead or exiled commander to the command zone is automatic in the engine (`zones::to_zone_card`, no prompt, for
  both commanders), for AI and human alike; there is no "choice" prompt to extend. A test shows both return after dying.
- Bots/opponents: the engine, `Engine::seat` and shadow/advisor already handle `commander2`; a kraum opponent in the games was
  already playing (checked in the sessions above).
- Kraum has no row in pool-results.md, so the setup screen says "No simulation results yet" and 0 Game Changers.
- No `ios/` changes. The iPad app ships the same static files, so it gets the UI change when rebuilt.

## Tests (all in the worktree, `cargo test --release -j 8`: 1961 passed, 0 failed; `build --all-targets`: 0 warnings)
- `sim-core/tests/play_partners.rs` (2): cast by index / by name / default, separate tax, refusal of one already out, both die and
  return to the zone, recast costs (Tymna 1+2, Kraum 3+2), the other's tax untouched.
- `practice/tests/session_partners.rs` (4): catalog lists kraum with both commanders; deck-file line; a person plays kraum
  through a Session for seeds 1..24 (land, tap, cast either commander by `cmd`): per commander tax = 2 x its casts, Tymna recast
  seen, Kraum cast seen; MY_DECKS = decks.json.
- Updated: `http_catalog.rs`, `http_server.rs` (kraum in the deck lists), `server_fake.rs` (12 decks, deck error text).

## Real server check
`rust/target/release/commander-practice --port 18765 --no-browser` (under the memory cap), then
`rust/agents/practice-partners-check.sh` (curl + jq; rerun with `/api/new` first: see the header of the script).
- `/api/options`: kraum with `commanders: ["Tymna the Weaver","Kraum, Ludevic's Opus"]`, ident WUBR.
- `POST /api/new {"deck":"kraum","tier":"t3","seed":7,"ai":"adaptive"}` ok; the state view shows both commanders in the zone.
- Seed 7: Tymna cast over HTTP in rounds 5, 7, 10 (`{"do":"cast","zone":"cmd","cmd":0}` -> `{"ok":true}`), tax 0, 2, 4, then 6 at the end;
  Kraum untouched (tax2 0).
- Seed 19: round 12 `cmd:1` cast Kraum with {W}{W}{W}{U}{R} -> ok; end view tax2 2, tax 0, both back in the zone.
- Not tried: browser clicks (no browser available to me); the JS was only read and syntax-checked by the server serving it.
