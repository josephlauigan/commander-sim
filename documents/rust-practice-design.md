# Rust port: practice mode (phases 9-13)

Design record for phases 9-13, kept for the reasons behind the choices. The code that came out of it is described in `practice-mode.md`; the Python it mentions (`commander_sim/play/`) has been deleted and is in git history. Sources: `documents/rust-rewrite-scope.md` (sections 5, 6 part 2, 7 items 5-6), `documents/rust-design.md`
section 6 (`Humans`), `commander_sim/play/*.py` (frozen reference), `crates/sim-core/src/control.rs`, the 124
`HUMAN(phase 9)` / `PORT(phase 9)` markers in `crates/sim-core`, `crates/practice` (the iOS proof) and `ios-shell/`.
The browser page (now `rust/crates/practice/static/`) is not touched. Python was removed at the end (phase 13), so the Rust
tests carry everything the Python tests checked.

## Contents
1. Where code lives
2. The human seat: Controller, Request, Answer
3. How a game pauses
4. Sessions, Undo, saves, the tape
5. Hint, the AI comparison, review
6. Module mapping
7. The HTTP server
8. Images and the catalog
9. The iPad shell
10. Tests
11. Tasks and order
12. Decisions for the user

---

## 1. Where code lives

The engine calls the person's code at its decision sites, and `sim-core` cannot depend on `practice`. So:

- **`sim-core/src/human/`** (new): everything the engine calls, or that decides the legal moves of a seat:
  `human.rs`, `legal.rs`, `mana.rs`, `combat.rs`, `choices.rs`, `cards.rs`, `abilities.rs`. It is inert in
  simulations: every entry point first asks `g.humans.get(p)`, and a copied game has no humans (`Humans::clone` is
  empty), so look-ahead and the sim paths are unchanged and `difftest` (0 AI differences) must stay at 0.
- **`practice` crate** (rlib + staticlib, binary `commander-practice`): sessions, views, Undo/saves, Hint, the AI
  comparison, the HTTP server, catalog, images, the text client, and the C entry point for the iPad.

`Humans`, `Human`, `Request`, `Answer` already exist in `control.rs`; `human/` builds on them.

## 2. The human seat: Controller, Request, Answer

Python has one `HumanController.ask(Request) -> answer` plus `tell`, and per-site helpers in `choices.py` (22:
`choose`, `yes_no`, `pick_cards`, `scry`, `discard`, `pay_tax`, `pick_basic`, `target_opponent`, `deal_damage` ...).
Rust keeps that split:

- `Human` (in `control.rs`): `ask(&self, g, seat, Request) -> Answer` and `tell(seat, kind, text)`. `Request {kind,
  prompt, choices: Vec<String>, data: Value}`; `Answer::{Pick, Picks, Yes, No, Cancel, Action(Value)}`. Kinds,
  prompts and `data` keys equal Python's, because the page (`app.js`) reads them; the HTTP tests compare them with
  the Python server's output (section 10).
- `human/choices.rs`: the 22 helpers as free functions, e.g. `fn choose(g: &mut Game, p: PlayerId, ..) ->
  Res<Option<usize>>`, built on `human_of(g, p)?.ask(..)`. A helper returns `Err(Stop::Cancelled)` when the answer is
  `Answer::Cancel` (the session was closed or rewound). **Add `Stop::Cancelled` to `flow.rs`** (additive); it unwinds
  the whole game through the existing `?`, as `OutOfWork` does. This is Python's `Cancelled` exception.
- Each `HUMAN(phase 9)` site becomes `if let Some(h) = human_of(g, p) { return human::choices::xxx(g, p, ..); }` in
  front of the AI code, in the same order as Python's `hc = human_choice(g, p); if hc: ...`. The comment names the
  helper (`hc.pay_tax`, `hc.pick_cards` ...); `grep 'HUMAN(phase 9)'` lists the sites and the porting agent deletes
  each marker as it fills the site. Owners: `engine/*`, `ai/*`, `cardcode.rs` (task core); `impls/*` (task cards).
- Main phase, priority, attacks and blocks (`human.rs`, `combat.rs`, `legal.rs`, `mana.rs`) are the long interactive
  loops: `human_main` asks `Request{kind:"main"}` and gets `Answer::Action(json)` (tap, land, cast, use, pass),
  applies it through the same engine functions the AI uses, tells the reason when it is illegal (`legal.rs` returns
  `Result<(), String>` with Python's wording), and asks again. `mana.rs` is the human pool (tapping, paying from the
  pool, emptying between steps) that the engine's cast path calls when the payer is a person.
- `ScriptHuman` (`human/script.rs`, `pub`, for tests and the text client): answers from a `Vec` or a closure of the
  request, records `asked` and `told`, and returns `Cancel` after 20000 requests (Python's `ScriptController.LIMIT`).

## 3. How a game pauses

**Decision: the game runs on its own thread and `Human::ask` blocks on a channel.** A resumable step machine would
turn the engine's deep recursion and its 97+ `?`-carrying sites into explicit state; threads leave the engine as is
and match Python line for line.

```
HTTP threads                    Session (Arc; Mutex<Inner> + Condvar)          game thread
POST /api/answer  -> answers.send(value) ---------------------------------->  ask() returns value
GET  /api/events  <- EventLog (Vec<Event> with ids), Condvar notify <--------  ask(): push Event::Request, recv()
```

- `SessionHuman { session: Arc<Session>, seat, rx: Mutex<Receiver<Answer>> }` implements `Human`. `ask` pushes a
  `request` event, then `rx.recv()`. There is no `idle()` hook: Python used it to squeeze the AI comparison into the
  waiting engine thread; Rust runs it on workers (section 5).
- `close()` sends `Answer::Cancel`, sets a flag and joins the thread; `ask` returns `Cancel` at once if the flag is
  set.
- Two people (`--lan`): one `SessionHuman` and queue per person; each request carries its seat, and the other
  person's page shows "waiting for X" from their view.
- A panic on the game thread is caught (`catch_unwind`) and becomes an `error` event; the process stays up. The game
  thread gets a large stack (64 MB, the engine recurses).

## 4. Sessions, Undo, saves, the tape

A game is `(deck, tier, seed, seat, opponents, profile, ai, step, answers)`. Rust must only be deterministic with
itself, not with Python.

- **Replay.** `Session` keeps `answers: Vec<Answer>` and `marks: Vec<usize>` (how far the tape had got at each
  decision), as `session.py` does. Undo = stop the thread, restart from the seed, feed `answers[..to]` back without
  asking (the `replaying` flag silences `tell` and marks events `replay: true`, so the page shows them at once), then
  ask live. `tryit` is "undo, then a different answer to decision n". Autopass changes apply from answer n+1
  (`autopass_of`); two-player Undo goes through `request_rewind` / `approve` as in `Hub`.
- **The search tape** (`Tape` in `ai/search.rs`, additive; the placeholder is in its module comment): the real game's
  look-ahead results as `Vec<(kind, result)>`, recorded when a search runs and played back in a replay so Undo does
  not search again (the speed reason; the Rust look-ahead is deterministic from the seed). A mismatch ("search tape
  out of step") panics loudly, as Python's `RuntimeError`. `Game` gets `search_tape: Option<Arc<Mutex<Tape>>>`;
  clones set it to `None`. `Tape::cut(n)` on a rewind.
- **Saved games**: JSON `{version, settings, seed, answers, tape, shadow}` in a `saves/` folder (a path the server is
  given; on iOS the app's writable folder). Same names and the autosave rule as `Hub._autosave`. Python's saves
  cannot be loaded (decision 1).
- **Views** (`view.rs`): `build_view(g, me) -> serde_json::Value`, the same JSON as `view.py`, for one seat's eyes
  only (other hands hidden, libraries as counts), built on the game thread at each request or event and stored with it.
- **EventLog**: each log line becomes an event (`is_private` lines only for their seat; `secret(key, public, line)`),
  with increasing ids for SSE `Last-Event-ID`; `catch_up` condenses a reconnecting client's backlog.

## 5. Hint, the AI comparison, review

All three run on `Game::clone` copies (AI seats only) on worker threads:

- `advisor.rs` (Hint): `hint(g, p, req)` runs the look-ahead for main phase / attack / counter on a copy and returns
  Python's JSON. The HTTP thread gets the game from a snapshot (`Arc<Game>`) the game thread stores with the pending
  request; the game is paused, so it is consistent.
- `shadow.rs` (comparison): `Shadow::record(g, p, req, ans, n)` stores a snapshot at each human decision of the kinds
  Python compares (main phase, attack); workers (`min(4, cpus-1)`, 2 on iOS) pop entries and fill `Entry.ai_answer`;
  `finish(progress)` and `review()` return the table. Entries are saved (`saved`/`from_saved`) and dropped by
  `forget_from(n)` on a rewind; a flag cancels the workers when the session closes.

## 6. Module mapping

| Python (`commander_sim/play/`) | Rust | Task |
|---|---|---|
| `controller.py` | `sim-core/src/control.rs` (exists), `human/script.rs`, `practice/src/session/human.rs` (`SessionHuman`) | core, session |
| `human.py` | `sim-core/src/human/human.rs` | core |
| `legal.py` | `human/legal.rs` | core |
| `mana.py` | `human/mana.rs` | core |
| `combat.py` | `human/combat.rs` | core |
| `choices.py` | `human/choices.rs` | core |
| `text.py` | `practice/src/text.rs` (`commander-practice --text`) | core |
| `cards.py` (1327 lines, per-card choices) | `human/cards.rs` and the `HUMAN` sites in `impls/*.rs` | cards |
| `abilities.py` | `human/abilities.rs` | cards |
| `session.py` | `practice/src/session/{mod,events,replay,human}.rs` | session |
| `view.py` | `practice/src/view.rs` | session |
| `advisor.py` | `practice/src/advisor.rs` | session |
| `shadow.py` | `practice/src/shadow.rs` | session |
| `server.py` (Hub, Handler) | `practice/src/server/{mod,http,hub,routes,sse,lan}.rs` | server |
| `catalog.py` | `practice/src/catalog.rs` | server |
| `images.py` | `practice/src/images.rs` | server |
| `__main__.py` | `practice/src/main.rs` (`--text --port --no-browser --lan --deck --tier --seed --seat --opponents --profile --ai --step`) | shell |
| `static/` | unchanged; served from a folder (`--static DIR`, default next to the card data) | shell |

## 7. The HTTP server

**Crate choice: no framework; a thin layer on `std::net::TcpListener` + `httparse`**, a thread per connection like
Python's `ThreadingHTTPServer`. The page needs an SSE stream (`/api/events`: a long chunked response with keep-alive
comments) and cookies; both are easy by hand and awkward in `tiny_http` (buffered writes); `axum`/`tokio` would put a
big runtime in the iOS build for a one-table server; `crates/practice` already serves by hand. About 250 lines:
request parse (headers, `Content-Length` body, 1 MB cap), a `Router` (method + path -> fn), `Response`, an SSE
writer, static files with a path-traversal check (normalise, then prefix check, as `_static`). `serde_json` for
bodies; `ureq` (rustls, already in the lock through cardgen) for images.

Routes, status codes and JSON (including the `{"error": ...}` texts) equal `server.py`:

- GET `/`, `/index.html` (sets the token cookie), `/static/*`, `/images/*`, `/api/state`, `/api/options`
  (`options()` plus `lan`, `host`, `app`), `/api/saves`, `/api/review`, `/api/events?since=N` (SSE, also
  `Last-Event-ID`).
- POST `/api/new` (validation messages verbatim; `two` needs `--lan`, else 409), `/api/join`, `/api/answer`,
  `/api/undo`, `/api/tryit`, `/api/approve`, `/api/save`, `/api/load`, `/api/autopass`, `/api/hint`, `/api/quit`.
- `HOST_ONLY` (`/api/new /api/load /api/quit /api/save /api/saves`) answers 403 to non-loopback clients.
- Seats by cookie token (`Hub.seat_of`), lobby and join codes (`lobby_info`, `join`), `lan_addresses()` (the
  UDP-connect guess of routable addresses). `--lan` binds `0.0.0.0`, else `127.0.0.1`.
- `Hub` (the current session, lobby, autosave, rewind approval) is an `Arc<Hub>` with a `Mutex`; handlers never hold
  the lock across a wait (SSE waits on the session `Condvar` with a 15 s timeout and sends `: keep-alive`).

## 8. Images and the catalog

- `catalog.rs`: decks, tiers, brackets and the results table for the setup screen, from the same data as the sim
  (`decks.json`, pool tables, `pool-results.md` if `catalog.py` reads it); same JSON keys.
- `images.rs`: Scryfall lookup (75 names per request, pacing and User-Agent as `cards/scryfall.py`); cache under
  `data/images/` with the same `index.json` (`name -> {id, files, urls, tokens}`, `token:<id>`), so images already
  downloaded keep working. A failed fetch never stops a game. A background thread started by `new_game`; the page polls.

## 9. The iPad shell

`crates/practice` already builds as a `staticlib` and exports `practice_start(data_dir, port) -> u16`. Final shape:

- `practice_start` starts the real server (`server::start(Config{data, saves, static_dir, lan: false, app: true})`)
  on `127.0.0.1` and returns the port. `app: true` is what `/api/options` reports. Add `practice_stop()` and
  `practice_set_lan(bool)` (the hidden two-player option) to `include/practice.h`.
- Swift changes little: the web view opens the port; *Continue* after iOS closes the app is the autosave
  (`/api/saves` -> `autosave`); `build-rust.sh` copies `static/` along with the card data into the app's writable
  folder on first launch; saves live there too, so they survive updates. The true-black palette is in the page.
- The proof page (*Play a game*) is deleted once the real page works. Memory: one game thread, 2 shadow workers.

## 10. Tests

213 Python tests (`tests/test_play_*.py`, 21 files) become three kinds of Rust test:

1. **Rules tests** (about 150: hand, mana, combat, choices, respond, turn, cards, abilities, marchesa, sauron, seph,
   veyran) become `#[test]`s in `sim-core/tests/play_<name>.rs`, driven by `ScriptHuman`. Same file names, so the
   mapping is obvious; each scenario and assertion is copied from the Python test.
2. **HTTP-level tests** (server, lan, catalog, images, save, undo, hint, review, shadow; about 60) are one test
   suite in `crates/practice/tests/http_*.rs` with a small client over `TcpStream` (no new dependency) that reads
   `PRACTICE_URL`: when set, it talks to that server; otherwise it starts the Rust server on port 0 with a temp saves
   dir and `ai=adaptive`. **Both servers run the same file.** It is written first against
   `python3 -m commander_sim.play --no-browser` (the oracle for status codes, JSON keys and error texts), then run on
   the Rust server. Assertions that depend on Python's shuffles (exact cards, seed-dependent log lines) are limited
   to structure (keys, types, kinds, increasing ids). Images tests use a local fake Scryfall (`PRACTICE_SCRYFALL`
   overrides the base URL), so they run offline.
3. **Replay determinism** (new, Rust only): a script of answers replays to the same view JSON after Undo, after save
   and load, and after a second load; with the tape and without it.

The manual checklist (scope doc section 7 item 6) is the user's; phase 13 lists it in the PR.

## 11. Tasks and order

All work is on `rust-port`; every agent follows `rust/agents/RULES.md`. Checks include `difftest` with 0 AI
differences (the new code must be inert in simulations).

| # | Brief | Owns | Waits for |
|---|---|---|---|
| 1 | `practice-core.md` | `sim-core/src/human/{mod,human,legal,mana,combat,choices,script}.rs`; `HUMAN` sites in `engine/*`, `ai/*`, `cardcode.rs`; `flow.rs` (`Stop::Cancelled`); `practice/src/text.rs`; `sim-core/tests/play_{hand,mana,combat,choices,respond,turn}.rs` | nothing |
| 2 | `practice-cards.md` | `human/{cards,abilities}.rs`; `HUMAN` sites in `impls/*`; `sim-core/tests/play_{cards,abilities,marchesa,sauron,seph,veyran}.rs` | task 1 milestone A (`choices.rs` signatures, `human_of`, `ScriptHuman`, `Stop::Cancelled` committed, named in its handoff) |
| 3 | `practice-session.md` | `practice/src/{session,view,advisor,shadow}`; `Tape` in `ai/search.rs`, `Game.search_tape`; `practice/tests/session_*.rs` | task 1 milestone A |
| 4 | `practice-server.md` | `practice/src/{server,catalog,images}`; `practice/Cargo.toml` deps | nothing to start (stub `SessionApi` trait in its brief); final wiring needs task 3 |
| 5 | `practice-httptests.md` | `practice/tests/http_*.rs`, `practice/tests/common/` | nothing (validated on the Python server first); then run on task 4's server |
| 6 | `practice-shell.md` | `practice/src/{lib,main}.rs`, `ios-shell/*`, `include/practice.h`, `documents/practice-mode.md`, READMEs | tasks 3 and 4 (and 2 for the full decks); last |

Parallel from the start: 1, 4, 5. Then 2 and 3 once task 1 reaches milestone A. Task 6 last. Shared files: `flow.rs`
(task 1), `ai/search.rs` and `state.rs` (task 3, additive), `sim-core/src/lib.rs` (`pub mod human`, task 1),
`practice/Cargo.toml` (task 4 only).

## 12. Decisions for the user

1. **Old saves**: games saved by the Python version cannot be loaded (different RNG and tape). Accept? (default: yes)
2. **HTTP layer**: hand-written over `std::net` + `httparse` (recommended), `tiny_http`, or `axum`/`tokio`.
3. **`static/`**: a folder next to the data (recommended, as the Briefcase app) or embedded in the binary.
4. Python-only quirks found while porting are kept and listed in each report; some may need a ruling.
