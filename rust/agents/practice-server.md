# Task: practice-server, the HTTP server, the catalog and card images (phase 11)

Read `rust/agents/RULES.md`, then `documents/rust-practice-design.md` (sections 7-9). Starts at once; final wiring waits for practice-session
(its `SessionApi` trait in `practice/src/session/api.rs`). Until then code against a local stub of that trait (the signatures are in
`rust/agents/practice-session.md`) and a fake session in your tests.

## Files owned
`practice/src/server/{mod,http,hub,routes,sse,lan}.rs`, `practice/src/{catalog,images}.rs`, `practice/Cargo.toml` (add `httparse`, `ureq`).
Not yours: `lib.rs`/`main.rs` (practice-shell: expose `server::start(Config) -> Result<ServerHandle, String>` and `Config{data, saves, static_dir, lan, app, port}`).

## Port
- `server.py`: every route, status code, validation message and JSON shape as in design section 7; `Hub` (new_game, lobby/join/`--lan`, seat_of by cookie,
  request_rewind/approve, autosave, saves, load, quit), `HOST_ONLY` 403, SSE `/api/events` with `Last-Event-ID`, `lan_addresses`, static files with the
  traversal check, `/images/*`. Hand-written HTTP on `std::net` + `httparse`, thread per connection, 1 MB body cap, no async runtime.
- `catalog.py`: decks, tiers, brackets, the results table; same JSON keys. `images.py`: Scryfall lookup and cache with the same `index.json` format;
  base URL overridable by `PRACTICE_SCRYFALL` (tests run offline).

## Python tests to port
`tests/test_play_{server,lan,catalog,images}.py` (36 tests) are ported by practice-httptests (same suite runs on both servers); you make them
pass once the session is wired. Your own tests: parser/router unit tests, SSE keep-alive and reconnect, path traversal, cookie seats, 403 for
non-loopback (unit-level).

## Checks
RULES.md checks; open the Rust server with `python3`-free `curl`: `/api/options`, `/api/state`, an SSE stream; the unchanged `play/static/` page loads.
Verify against `python3 -m commander_sim.play` that the JSON of `/api/options` and `/api/saves` is identical.

Commit message: "Rust port: the practice server (routes, SSE, seats, LAN lobby), catalog and card images"
