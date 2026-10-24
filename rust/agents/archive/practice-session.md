# Task: practice-session, sessions, views, Undo, saves, Hint, the AI comparison (phase 10)

Read `rust/agents/RULES.md`, then `documents/rust-practice-design.md` (sections 3-6, 10).
**Waits for practice-core milestone A** (`ScriptHuman`, `Stop::Cancelled`, `human_of`; see `rust/agents/handoff-practice-core.md`). Until
then read `play/session.py`, `view.py`, `advisor.py`, `shadow.py` and the `Hub` parts of `server.py` (answer, undo, save, load, review, hint).

## Files owned
`practice/src/{session/*,view,advisor,shadow}.rs`; `Tape` in `sim-core/src/ai/search.rs` and `Game.search_tape` in `state.rs` (additive;
clones set it to `None`); `practice/tests/session_*.rs`. Not yours: `server/*` (practice-server; you define the `SessionApi` it calls, below).

## Port
- `Session` (new, settings, seat choice `_exact_seats`/`_choose_seats`, start, the game thread, `SessionHuman`, EventLog, autopass, answer, close,
  join), Undo/`tryit`/`rewind` by replay of seed + answers + tape, `saved()`/`load()`, `view.py`, `advisor.py`, `shadow.py`.
- Hint and the comparison on `Game::clone` copies and worker threads (design section 5), not on the game thread.
- **`SessionApi`** (in `session/api.rs`, fixed early so practice-server can code against it; commit it first): `new(opts) -> Result<Arc<Session>, String>`,
  `state(seat) -> Value`, `events_after(since, timeout) -> Vec<Event>`, `answer(seat, id, Value) -> Result<(), String>`, `undo`, `tryit`,
  `set_autopass`, `hint(seat) -> Result<Value, String>`, `review(seat)`, `saved() -> Value`, `load(Value)`, `close`. Python's messages for the Err strings.

## Python tests to port
`tests/test_play_{undo,save,hint,review,shadow}.py` (12 tests) as `practice/tests/session_*.rs` (against `Session` directly, `ai=adaptive`), plus the
new replay-determinism tests (design section 10, kind 3): same view JSON after Undo, save+load, and a second load, with and without the tape.

## Checks
RULES.md checks; no thread left running after `close()` (test it); difftest 0 AI/0 build differences (the `Tape` hook must not change simulations).

Commit message: "Rust port: practice sessions (game thread, Undo and saves by replay, views, Hint, the AI comparison)"
