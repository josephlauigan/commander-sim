# Handoff: practice-server

## Done (committed)
- `practice/src/catalog.rs`, `images.rs`, `server/{mod,http,hub,routes,sse,lan,api}.rs`, `tests/server_fake.rs`; deps httparse, ureq(json), libc.
- Design: `Hub` ports Python's Hub line by line and keeps the numbered history/pending/views itself, fed by a pump thread
  from the session's RAW events (`SessionApi::events_after(cursor, timeout) -> (cursor, Vec<Value>)`, JSON shaped like
  Python's Session.events; contract in `server/api.rs` header). `server/api.rs` is my local copy of the session shape
  (`SessionApi` + `SessionFactory`); when practice-session lands, adapt its `session/api.rs` (or impl my traits for Session)
  and change `server::start` to use the real factory instead of `NoSessions`.
- `lib.rs` got `pub mod catalog; images; server;` (additive; shell owns lib.rs).
- Tests with a fake session pass (routes, cookie seats, SSE + Last-Event-ID, 403, traversal, lobby/join, autosave).

## Next
1. Compare `/api/options` + `/api/saves` JSON with `python3 -m commander_sim.play` (see report).
2. Wire the real session once `practice/src/session/api.rs` exists; run practice-httptests' suite.
3. Final RULES checks (all_decks, difftest are untouched by this crate).

## Gotchas
- `Config` has no images field: images dir = `<data>/images`. `PRACTICE_NO_FETCH` skips the commander fetch in `fetch_commanders`
  (call it from main after `start`); `PRACTICE_SCRYFALL` overrides the Scryfall base URL.
- catalog needs `<data>/../decklists/` for brackets/names/win rates/tier labels; without it falls back (iPad).
