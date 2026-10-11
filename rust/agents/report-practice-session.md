# Report: practice-session

Commits on this branch: sim-core hooks, session/view/advisor/shadow, tests, seams.
Merged: rust-port, 3563cb4 (milestone A) and then eade84a (practice-core's later human.rs/legal.rs/mana.rs, needed for the pool,
sources and graveyard plays in views) -- when merging, those are practice-core's commits.

## Shared-file changes (all additive)
- sim-core: `control.rs` (LogSink, Link, Game::log_secret), `engine/mod.rs` (glog! feeds the sink), `state.rs` (fields `link`, `search_tape`;
  `tick()` returns Stop::Cancelled when the abort flag is set), `ai/tape.rs` (new), `ai/mod.rs`, `ai/search.rs` (taping wrappers around
  choose/choose_attack/choose_counter; playout_main/attack pub; LAST_SCORES).
- practice: `lib.rs` (+4 `pub mod`). `SessionApi`/`SessionFactory` are the server's (server/api.rs); `session/api_impl.rs` implements them
  (`SessionHandle`, `RealSessions{engine}`). No separate session/api.rs.

## Design notes / deviations
- The comparison runs on worker threads (design section 5), so there is no hint_lock; the Hint uses a copy of the game stored with the pending request.
- Tape marks use `tape.pos` (Python used len(entries), wrong during a replay).
- `view` omits Python's `face` key for two-faced land cards (needs the scryfall cache); `hand_special` / land `ability` are false until practice-cards.
- Seat drawing uses sim-core's Rng (not Python's `random.sample`): same seed seats the same table in Rust only.
- Python quirk kept: the game-over `how` is `""` when no win type.

## Not done / for later
- The `log_secret` call sites (tutors.rs, dsl/mod.rs, yshtola.rs) still use glog!; switch to `g.log_secret(p, public, line)`.
- Full-turn tests (Hint/comparison through a person's real main phase, Try it) wait on practice-core filling the HUMAN sites; the session tests run
  with only the mulligan being asked. Shadow/advisor are tested on a game held at main phase.
- `server_fake` test `a_game_through_the_routes` failed once in a full-workspace run (autosave race), passes alone 3/3 (practice-server's).
