# Task: practice-core, the human seat's rules in Rust (phase 9, part 1)

Read `rust/agents/RULES.md`, then `documents/rust-practice-design.md` (sections 1-3, 6, 10) and `rust/crates/sim-core/src/control.rs`.
Runs first; tasks practice-cards and practice-session wait for your **milestone A**.

**Milestone A (do this first, commit, and say so at the top of `rust/agents/handoff-practice-core.md`):**
`pub mod human` in `sim-core/src/lib.rs`; `Stop::Cancelled` in `flow.rs`; `human/mod.rs` with `human_of(g, p)`;
`human/choices.rs` with the final signatures of all 22 helpers (bodies may follow); `human/script.rs` (`ScriptHuman`).

## Files owned
`sim-core/src/human/{mod,human,legal,mana,combat,choices,script}.rs`; `sim-core/tests/play_{hand,mana,combat,choices,respond,turn}.rs`;
`practice/src/text.rs` (+ `commander-practice --text` in `main.rs`, only that flag); the `HUMAN(phase 9)` / `PORT(phase 9)` sites in
`engine/*`, `ai/*`, `cardcode.rs`, `dsl/mod.rs` (smallest change: an `if let Some(h) = human_of(g, p)` in front of the AI code, in Python's order;
delete the marker). Not yours: `impls/*` sites and `human/{cards,abilities}.rs` (practice-cards).

## Port
`play/human.py`, `legal.py`, `mana.py`, `combat.py`, `choices.py`, `text.py`, the `ScriptController` of `controller.py`. Request kinds,
prompts, `data` keys and illegal-move texts must equal Python's (the page reads them; HTTP tests compare). Main phase: `Request{kind:"main"}` +
`Answer::Action(json)`. Cancel unwinds with `Stop::Cancelled`. Everything inert when `g.humans` is empty: difftest must stay at 0 differences.
Where a site needs a per-card choice from `cards.py` (marked in the code), call `human::cards::xxx` and leave a stub returning the AI's choice with
`PORT(practice-cards)`.

## Python tests to port
`tests/test_play_{hand,mana,combat,choices,respond,turn}.py` (71 tests), same names, same assertions, into `sim-core/tests/play_*.rs` using `ScriptHuman`.
Done also means: the text client plays a full game against three AI seats (`--ai adaptive`, any deck) without a panic.

## Checks
RULES.md checks, plus `systemd-run ... cargo run --release -p practice --bin commander-practice -- --text --deck sauron --tier t1 --ai adaptive` with
scripted input to the end of a game, and difftest with 0 AI and 0 build differences.

Commit message: "Rust port: the human seat's rules (priority, mana, combat, choices) and the text client"
