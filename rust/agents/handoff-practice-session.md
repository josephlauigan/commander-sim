# Handoff: practice-session

## Done (committed)
- sim-core additive hooks: `ai/tape.rs` (Tape, Taped, TapeSlot, taped(), LAST_SCORES), taping in `search::choose/choose_attack/choose_counter`
  (`playout_main/attack` now pub), `Game.link` (LogSink + abort flag, `control.rs`), `Game.search_tape`, `Game::log_secret`, `glog!` feeds the sink,
  `Game::tick` returns `Stop::Cancelled` when the abort flag is set.
- practice crate: `view.rs` (+ `view::seam` placeholders), `advisor.rs`, `shadow.rs` (worker threads), `session/{mod,human,setup,api_impl}.rs`.
  `SessionHandle` implements the server's `SessionApi` (server/api.rs, NOT a separate session/api.rs); `RealSessions{engine}` is the `SessionFactory`.
  Wire it with `server::start_with(cfg, Arc::new(RealSessions{engine}))` (shell task).

## Status
All committed; tests pass (session_basic 8, session_shadow 3); all_decks adaptive 3 / lookahead 1 fine; difftest AI 0, build 0.
Report: rust/agents/report-practice-session.md

## Next
1. Tests `practice/tests/session_*.rs` (undo, save, hint, review, shadow, replay determinism, no thread left after close) -- need ScriptHuman-free
   driving: answer via `Session::answer` on `events_after` requests, `ai=adaptive`.
2. `view::seam` functions (mana pool text/sources, graveyard_playable, hand_special, land ability, pool_total) wait on practice-core mana.rs/legal.rs and
   practice-cards abilities.rs/cards.rs; wire when merged. Also `view` omits `face` for two-faced land cards (needs scryfall cache).
3. The `log_secret` call sites (tutors.rs, dsl/mod.rs, yshtola.rs, marked PORT(phase 9)) still use glog!: switch them to `g.log_secret(p, public, line)`.
4. Final RULES checks (cargo test, all_decks, difftest) after merging practice-core.

## Gotchas
- `Human::ask` gets `&Game`; the session clones it as the Hint snapshot at every live request.
- Marks use `tape.pos`, not len (Python's len was wrong during replays).
- Engine `run_rounds`/`mulligan` signatures may change in practice-core (mulligan -> Res); setup.rs ignores mulligan's result with `let _ =`.
