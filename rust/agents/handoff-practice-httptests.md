# Handoff: practice-httptests
Python server for validation: `systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0 python3 -m commander_sim.play --no-browser --port 8799 --lan`
(from the worktree root). Run: `PRACTICE_URL=http://127.0.0.1:8799 cargo test --release -p practice --test 'http_*' -- --test-threads=1` (from rust/).
Saves/images folders default to the repo's data/saves, data/images (env PRACTICE_SAVES / PRACTICE_IMAGES override). Never commit data/.
Without PRACTICE_URL tests print SKIPPED and pass (`common::start_rust` is the stub to fill when practice::server exists).

## Done
- common/mod.rs (client, SSE, bot, play helper); http_server.rs (19 tests, green on Python).
## Next, in order
1. http_lan.rs (port test_play_lan.py TwoPlayerServer; remote client = LAN ip, `server.remote_client()`; no X-Test-Remote header)
2. http_catalog.rs (options deck fields / tiers / win_rates shape), http_images.rs (fake Scryfall, PRACTICE_SCRYFALL, Rust only: skip if !server.rust)
3. http_undo.rs, http_save/hint/review/shadow (play bot over HTTP, compare sequences with/without hints / compare tool)
4. report-practice-httptests.md; RULES checks; when practice-server lands: merge rust-port, fill start_rust, run on Rust.
