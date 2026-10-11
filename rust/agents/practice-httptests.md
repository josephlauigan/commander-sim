# Task: practice-httptests, the HTTP-level test suite that runs on both servers

Read `rust/agents/RULES.md`, then `documents/rust-practice-design.md` (section 10). Starts at once; no Rust server is needed to begin.

## Files owned
`rust/crates/practice/tests/http_*.rs`, `rust/crates/practice/tests/common/mod.rs` (a small blocking HTTP client over `TcpStream`, cookie jar, SSE
reader; no new dependencies). Nothing else.

## Port
`tests/test_play_{server,lan,catalog,images}.py` (36 tests) and the HTTP-reachable parts of `test_play_{undo,save,hint,review,shadow}.py`, same test
names. The suite reads `PRACTICE_URL` (and `PRACTICE_URL_HOST2` style second client for LAN, using a different cookie jar). When `PRACTICE_URL` is
unset it starts the Rust server through `practice::server::start(Config{port: 0, lan: true, saves: tempdir, ai default: adaptive})`; until that exists
those tests are `#[ignore = "needs the Rust server"]`.
1. Write and validate against Python: `python3 -m commander_sim.play --no-browser --port 8799 --lan` (cap memory per RULES.md),
   `PRACTICE_URL=http://127.0.0.1:8799 cargo test --release -p practice --test 'http_*' -- --test-threads=1`. All must pass on Python.
2. Assertions on shuffle-dependent content are limited to structure (keys, types, kinds, increasing event ids, status codes, error text).
3. Images tests use a fake Scryfall (a local listener) via `PRACTICE_SCRYFALL`; for the Python server, skip those with a clear message.
4. Once practice-server lands (merge `rust-port`), un-ignore and make the same tests pass on Rust; list any difference between the two servers
   in `rust/agents/report-practice-httptests.md` (a difference is a Rust-server bug unless Python's behaviour is plainly an accident).

## Checks
RULES.md checks; both runs green (Python server: all but images; Rust server: all).

Commit message: "Rust port: HTTP-level practice tests that run on the Python and the Rust server"
