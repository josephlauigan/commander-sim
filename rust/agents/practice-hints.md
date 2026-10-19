# Task: practice-hints, a hint for every decision (mulligans first) and no empty hints

Read `rust/agents/RULES.md` (skip the `git merge rust-port` step: your worktree is on main), then this brief. Worktree:
`/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/practice-hints`, branch `practice-hints`.

JD plays practice mode (browser and iPad) and uses the **Hint** button to learn. He reports two problems:
1. **No hints or feedback on mulligans.** He wants to know whether to keep or mulligan, and why.
2. **Some hints are "null".** The page shows `The AI would: null`, or a box with nothing useful in it.

How hints work now: `rust/crates/practice/src/advisor.rs::hint` handles only `priority` (main phase, counters,
otherwise a generic "Pass") and `attack`. Every other request kind returns "No hint for this kind of decision yet."
The page (`rust/crates/practice/static/app.js`, `hint()`) prints `'The AI would: ' + r.data.text` and the
`detail` list. Requests come from `crates/sim-core/src/human/*.rs` (the mulligan is in `choices.rs`); the session
is `crates/practice/src/session/{mod,human}.rs`.

## CPU and memory (shared machine: hard limits)
At most **4 threads**: `-j 4` and `--test-threads=4`, under
`nice -n 19 systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0 ...`. No sims and no `all_decks`; the
RULES.md all_decks check is waived. Build and test `-p practice` (and `-p sim-core` only if you touch it).

## Files owned
`rust/crates/practice/src/advisor.rs` (split it into a module folder if it grows), `rust/crates/practice/static/app.js`
(the hint box only), tests under `rust/crates/practice/tests/`, `rust/agents/{handoff,report}-practice-hints.md`.
sim-core: **read-only, except** the smallest additive `pub` you need to reach the AI's existing decision code (for
example making a private mulligan or block function `pub`). Never change how the AI decides: sim results must not move.

## Do (in this order; commit after each)
1. **Inventory** (write it to the report early): every request `kind` a human seat can get (grep the `Request {`
   constructions in `crates/sim-core/src/human/` and their callers), what the prompt is, and what `hint` returns for it
   now. Then find **every way a hint can come out null or empty**: `text` missing or null (so the page shows
   "null"), Err paths, `choice` labels, detail that is empty, panics in the copy. Reproduce each one in a test.
2. **Test first: a hint at every decision.** Write a practice-crate test that plays several seeded games with a human
   seat, the human's answers coming from the AI (or the first legal choice), asks `hint` at **every** decision, and
   asserts the hint's `text` is a non-empty string and not "null", that `detail` is an array, and that nothing panics.
   Use the session or advisor API the existing tests use (`tests/` has examples). Keep it fast: a few games, the
   adaptive AI where look-ahead isn't needed. It fails now; it must pass at the end.
3. **Mulligan hint.** Ask the AI's own mulligan logic (the code the AI seats use to keep or mulligan; find it from
   `engine/turn.rs`) on a copy, and answer **Keep** or **Mulligan** with the reasons, in plain words: the land count,
   the mana sources (rocks, dorks), which colours the hand can make against what its spells need, what it can cast on
   turns 1-3, whether the commander comes down on curve, and key cards (tutors, ramp, wincons) if the AI's rule looks
   at them. Show the threshold the AI uses ("the AI keeps 7 cards with 2-5 lands and ...") so JD learns the rule.
   If a mulligan leads to putting cards on the bottom (London mulligan), hint which cards the AI would bottom and why.
4. **Every other kind**: where the AI has a decision function for it (blocks, targets, choose-one modes, discards,
   sacrifices, scry or surveil, tutors, payments ...), give the AI's pick on a copy with a one-line reason. Where it
   honestly has none, say what the decision is and what to weigh, in a full sentence, never "No hint ... yet" and
   never null. List in the report which kinds have a real AI pick and which a general tip.
5. **Mulligan feedback after the game**: check whether the end-of-game review (`shadow.rs`, `review`) compares your
   mulligan decisions with the AI's. If it doesn't, add them (did you keep when the AI would have mulliganed, and
   the other way round).
6. **The page**: never print "null"; if `text` is missing, show "No hint for this decision." Keep the box's look.

## Checks
`cargo test --release -j 4 -p practice` passes (the new every-decision test included, nothing ignored);
`cargo build --release -j 4 -p practice` has no warnings; if you touched sim-core, `cargo test --release -j 4 -p sim-core`
passes too. `commander-practice --no-browser` starts.

## Working
Write `rust/agents/handoff-practice-hints.md` after your first concrete step and keep it current. Commit often
(standalone `git add -A && git commit -m "..."`, no Co-Authored-By line; don't push). Report in at most 15 lines; details
(the inventory table, which kinds get a real pick, the null causes found) in `rust/agents/report-practice-hints.md`.
