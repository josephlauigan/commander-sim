# Practice-mode fixes: shared rules (read with your task brief)

JD plays his own decks in practice mode (`rust/crates/practice`, the engine in `rust/crates/sim-core`). An audit
found many places where the AI decides for the person. Read its report first:
`/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/practice-human-audit/audit/practice-human/REPORT.md`
(read-only; per-group details in `findings-g1.md` ... `findings-g5.md` next to it; JD's cards with their decks in
`jd-cards.tsv`).

Then read `rust/agents/RULES.md`, but skip its `git merge rust-port` step: your worktree is already on main.

## The rule that matters most: the AI must not change
Every change is for a **person's** seat only (`crate::human::is_human(g, p)` / `human_of`). The AI's code paths,
choices and order stay exactly as they are, so simulation results don't move. **Prove it:** before your first change,
build `commander-sim` and run
`./target/release/commander-sim --deck sheoldred --pool t3 --games 200 --ai adaptive --profile loose --jobs 4 --no-preflight > /tmp/<task>-before.txt`
plus the same for `--deck cloud --pool t1` and `--deck kefka --pool t3`. Run them again at the end. The win lines
and the breakdown must be **identical** (diff them). Put the diff result in your report.

## Asking the person
Use the helpers in `crates/sim-core/src/human/choices.rs` (`choose`, `choose_one`, `yes_no`, `pick_cards`,
`sacrifice_creature`, `discard`, `target_opponent`, ...) so the page and the text client both work. Prompts start
with the card's name ("Bowmasters: ..."), in plain words, and use the card's own words where they fit. Every
"you may" gets a way to say no.

## Tests
Each fix gets a test with a scripted person (`crates/sim-core/tests/play_common/mod.rs`: `seat`, `pick`, `yes`,
`no`, `ScriptHuman::from_fn`, `h.asked()`; examples in `tests/play_sephfs_human.rs`, `tests/play_ardenn_human.rs`).
Check what the person was asked and that the answer was applied. A `ScriptHuman` with no answers left returns
`Cancel` (an error): use `from_fn` when a test may be asked things you don't care about.

## Limits (shared machine)
At most **4 build threads**: every cargo command gets `-j 4`, tests `--test-threads=4`, all under
`nice -n 19 systemd-run --user --scope -q -p MemoryMax=10G -p MemorySwapMax=0 ...`. The before/after sims above are
the only sims. No Python. Edit only what your brief owns; for shared files (`engine/*`, `human/*`, `hooks.rs`) make
the smallest additive change and list it in your report.

## Working
Write `rust/agents/handoff-<task>.md` after your first concrete step and keep it current. Commit after each piece
(standalone `git add -A && git commit -m "..."`, no Co-Authored-By line; don't push). When your context gets long
(~60+ tool calls), stop at a clean point with everything committed and the handoff current, and end with a report
whose first line is HANDOFF. Final report in at most 15 lines; details in `rust/agents/report-<task>.md`.
Checks before you finish: `cargo test --release -j 4` (whole workspace) passes, `cargo build --release -j 4` has no
warnings, the before/after sims are identical.
