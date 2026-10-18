# Brief: Tier 6 Stage 3 — model one batch of cards (cards first, engine later)

Read `rust/agents/RULES.md` (memory cap, no Python, commits, short reports; ignore the Python-port parts and the
`git merge rust-port` step), then `rust/PORTING_CARDS.md` (the hook-slot pattern; its Python notes are history).
Your batch name and your worktree are in the prompt that started you. Work only in your worktree.

## The rule for this stage: cards only, no shared-engine changes

JD's call (2026-10-17): model every card first, then change the engine ONCE for everything the cards need.

- **Allowed:** your batch's card code in a NEW file `crates/sim-core/src/impls/t6_<batch>.rs` (follow
  `impls/t1.rs`), its two registration lines in `impls/mod.rs`, tests in `crates/sim-core/tests/cards_t6_<batch>.rs`
  (built with `testkit`), and your cards' entries in `data/card_overrides.toml` / `data/audit.toml` `[card_notes]`.
- **Not allowed:** editing `engine/*`, `ai/*`, `state.rs`, `hooks.rs`, `cardcode.rs`, `flow.rs` or other shared code.
  If a card needs an engine feature that doesn't exist (e.g. "players can't cast spells", split second, countering
  an ability, the dungeon, giving a creature to an opponent, phasing, more than 4 combats a turn, an empty-library
  draw losing the game, enters-tapped for opponents), model everything else on the card, leave its status Partial,
  and add an entry to `rust/agents/engine-needs-<batch>.md`:
  ```
  ## <Card name>
  Needs: <the engine feature, one line, general — not card-specific if it can be general>
  Where it would hook: <engine file/function you found, if you know>
  Card side done: <what your card code already does>
  Test to add once the engine has it: <one line>
  ```
  Reuse an existing hook or helper if one fits (grep `impls/common.rs`, `cardcode.rs`, `engine/`), and say so.
- **Status:** a card you model completely and test against its Oracle text gets `status = "Full"` with a one-line
  note in `data/card_overrides.toml` (see existing entries for the format). Partial cards get `status = "Partial"`
  and a note naming the missing part. Check with `cardgen audit --deck <pool deck key>` (or `--mine` for Godo's
  cards); never pass `--md` (it overwrites a file).

## Oracle text

Get the text of all your cards with ONE Scryfall call, at the start:
`jq -R '{name:.}' names.txt | jq -s '{identifiers:.}' | curl -s -m 30 -A commander-sim/1.0 -H 'Content-Type:
application/json' -d @- https://api.scryfall.com/cards/collection` (max 75 names; save the result to your scratch
file and read from it). Every network call has `-m 30`. Model the card as its Oracle text says, not from memory.
How the deck uses a card is in `audit/t6/<deck>/SPEC.md` (read the parts that name your cards; specs say which
deck plays what).

## Progress and size

- Create `rust/agents/handoff-t6-<batch>.md` FIRST, then update it after each card (done / next / gotchas). A lead
  watches commits and this file and checks on an agent silent for 10 minutes. Update the handoff BEFORE a long build.
- **One card at a time:** model it, test it, commit (`git add -A rust/ data/ && git commit -m "t6 <batch>: <card>"`,
  standalone, no Co-Authored-By). Don't batch several cards into one commit.
- Shared machine: every cargo command is `CARGO_BUILD_JOBS=2 nice -n 19 systemd-run --user --scope -q
  -p MemoryMax=12G -p MemorySwapMax=0 cargo ...`, run from `rust/`. Use `cargo test --release -p sim-core --test
  cards_t6_<batch>` while working; the full `cargo test --release` once at the end. Never run a sim.
- If your context gets long (about 60 tool calls), stop at a clean point (committed, handoff updated) and end with
  a report whose first line is `HANDOFF`.

## Done

Full `cargo test --release` passes, `cargo build --release --all-targets` has no warnings. Report in
`rust/agents/report-t6-<batch>.md`: each card with status (Full / Partial + what's missing), the engine needs, and
anything shared you had to touch (should be nothing beyond `impls/mod.rs` and the data files). Final message at most
12 lines.
