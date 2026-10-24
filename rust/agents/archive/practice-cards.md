# Task: practice-cards, the person's card and ability choices (phase 9, part 2)

Read `rust/agents/RULES.md`, then `documents/rust-practice-design.md` (sections 1, 2, 6, 10).
**Waits for practice-core milestone A** (see `rust/agents/handoff-practice-core.md`): `human_of`, the `choices.rs` signatures, `ScriptHuman`,
`Stop::Cancelled`. While you wait, read `play/cards.py` and `play/abilities.py` and group the work by deck. Merge `rust-port` again when the
milestone lands.

## Files owned
`sim-core/src/human/{cards,abilities}.rs`; every `HUMAN(phase 9)` / `PORT(phase 9)` site in `sim-core/src/impls/*.rs` (about 80, `grep -rn 'HUMAN(phase 9)' rust/crates/sim-core/src/impls`);
`sim-core/tests/play_{cards,abilities,marchesa,sauron,seph,veyran}.rs`. Not yours: `engine/*`, `ai/*`, `cardcode.rs` (practice-core). If a
site there needs a `cards.rs` function that core stubbed (`PORT(practice-cards)`), replace the stub body only.

## Port
`play/cards.py` (1327 lines: the per-card choices of the human decks: targets, modes, tutors, pay-or-not, ordering) and `play/abilities.py`
(activated abilities by the rules). Each site: ask through the `choices.rs` helper with Python's kind/prompt/choices, then apply the result with the same
engine functions as the AI branch. Prompts and `data` equal Python's. Inert when `g.humans` is empty (difftest 0 differences).

## Python tests to port
`tests/test_play_{cards,abilities,marchesa,sauron,seph,veyran}.py` (79 tests), same names and assertions, in `sim-core/tests/play_*.rs`.
Report every Python bug found (ported as is).

## Checks
RULES.md checks and difftest (0 AI, 0 build differences). Also grep: no `HUMAN(phase 9)` left in `impls/`.

Commit message: "Rust port: the person's card and ability choices (cards.py, abilities.py, the HUMAN sites in impls)"
