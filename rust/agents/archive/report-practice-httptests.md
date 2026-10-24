# Report: practice-httptests

Files: `rust/crates/practice/tests/{common/mod.rs, http_server.rs, http_lan.rs, http_catalog.rs, http_images.rs, http_play.rs}`.
60 tests (server 19, lan 9, catalog 3, images 4, play 9 + the shared ones). No new dependencies.

## Running
- Python: `python3 -m commander_sim.play --no-browser --port 8799 --lan` (memory-capped), then from `rust/`:
  `PRACTICE_URL=http://127.0.0.1:8799 cargo test --release -p practice --test 'http_*' -- --test-threads=1` -> all pass
  (images tests print SKIPPED: they need the Rust server and a fake Scryfall; the two "needs --lan"/"without --lan"
  tests skip depending on how the server was started).
- Env: `PRACTICE_URL`, `PRACTICE_URL_HOST2` (address of the "other computer", default this machine's LAN ip + same port,
  so the server sees a non-loopback client; replaces Python's test-only `X-Test-Remote` hook), `PRACTICE_SAVES`,
  `PRACTICE_IMAGES` (default the repo's `data/saves`, `data/images`, where the Python server works).
- Without `PRACTICE_URL` every test prints `SKIPPED` and passes. TODO when practice-server lands: fill
  `common::start_rust()` (start `practice::server::start(Config{port:0, lan:true, saves: tempdir, ai: adaptive})` after
  `set_var("PRACTICE_SCRYFALL", fake::start())`, return host:port, set `Server.saves/images` to its folders), then run
  on Rust and list differences below. The images tests were written blind (the fake serves `POST /cards/collection`
  and `GET /img/<id>.jpg`; adjust to the Rust client's exact requests).

## Not ported (not HTTP-reachable; belongs in Rust unit tests of the owning task)
Python `catch_up`/`for_seat`/`log_secret` unit tests (catch_up is covered through the stream), `Session`-level calls
(`undo_point`, `answer_seats`, `shadow.entries`, `max_rounds`), catalog `win_rates` parser on a temp file, images
`prepare` internals (offline, failed download, two-faced cards), `Session.load({'version': 0})` (covered as a bad save
file in `test_save_and_load_errors`), two-player separate reviews/Try it (needs a game that ends; `max_rounds` is not
an API option).

## Differences Rust vs Python
None yet (Rust server not available).
