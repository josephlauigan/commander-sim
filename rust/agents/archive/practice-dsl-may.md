# Task: practice-dsl-may, "you may" effects of DSL-compiled cards ask the person

Read `rust/agents/practice-fixes-common.md` first. Worktree:
`/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/practice-dsl-may`, branch `practice-dsl-may`.

## The problem
Cards whose behaviour is compiled from the card DSL (the `dsl` field in `data/cards.json`, built by `cardgen` from
`data/card_overrides.toml` and the Oracle text; run-time code in `crates/sim-core/src/dsl/`) have an `optional` flag
on effects (`dsl/model.rs:85`), but nothing sets it, and `dsl/mod.rs:441` reads it only for draws. So every "you may"
in a DSL card always happens for the person. Examples in JD's decks: Solemn Simulacrum ("you may search ... basic
land"), Sun Titan ("you may return"), Restoration Angel ("you may exile target non-Angel creature"), Recruiter of the
Guard, Reya Dawnbringer, Farhaven Elf, Planar Genesis, Cartographer's cards. (The hand-written cards were done in
branch practice-may with `engine::stack::may_window`; this is the DSL half.)

## Do
1. **Read first:** how cardgen builds a card's `dsl` (where the effects come from: overrides vs Oracle parsing), how
   `dsl/mod.rs` runs a triggered/ETB effect, and where `optional` is read today (line 441: what it does for the AI,
   e.g. skipping a draw when the library is small). Write what you find in the handoff file before changing anything.
2. **cardgen:** set `optional: true` on the effect when the Oracle sentence for it says "you may" (only that effect,
   not the rest of the ability). Prefer a precise rule (the sentence that produced the effect) over a whole-card
   "contains 'you may'" test; list in the report every card that gets the flag (JD's cards: `audit/practice-human/
   jd-cards.tsv` in the practice-human-audit worktree, path in the common rules) and any you were unsure of.
3. **Run time:** in `dsl/mod.rs`, when an effect is optional **and its controller is a person**, ask yes/no
   (`choices::yes_no`, "Solemn Simulacrum: search your library for a basic land?") and skip the effect on no. **For the
   AI nothing changes**: the AI must ignore the new flags exactly as today (careful: line 441's draw rule now reads
   `optional` for the AI; with the flag newly set on more draws, that rule would start firing for the AI. Keep the AI
   behaviour by gating the new reading to the person, or by keeping the old flag for the AI's rule, so the sims stay
   identical).
4. Rebuild the data: `cargo build --release -j 4 -p cardgen && ./target/release/cardgen export` from `rust/`, commit
   the new `data/cards.json`.
5. Tests: a scripted person says yes and no to two or three representative DSL cards (an ETB search, a "may return",
   a "may exile"); the AI path unchanged.

## Checks
The common rules' checks, and the before/after sims must be **identical** with the new `data/cards.json` (that's
the proof the AI ignores the flags). `cargo test --release -j 4` passes (cardgen's own tests too: `cardgen` has
equivalence and workflow tests that may need the fixtures regenerated; do that only through cardgen's own commands).

## Files owned
cardgen's DSL building code, `dsl/mod.rs` / `dsl/model.rs` (the person-only branch), `data/cards.json` (regenerated),
a new `tests/play_dsl_may.rs`, `rust/agents/{handoff,report}-practice-dsl-may.md`.

Commit message for the last commit: "Practice mode: 'you may' effects of DSL cards ask the person (N cards flagged)".
