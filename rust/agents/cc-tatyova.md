# Brief: combo and alternate-win lines for `tatyova-simic-landfall` (t1)

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/cc-tatyova` on branch `cc-tatyova`. It is your whole world: run every command from there.
Read `CLAUDE.md` first (memory cap, no Python, loose profile). Do not use `rust/agents/RULES.md`.

## Goal

On Playgroup.gg, 18.7% of this commander's logged wins are by combo or alternate win (mill included). In the sim this
deck almost never wins that way. Find out why and close the gap where the list really holds the lines: the deck should
win the way the real list can, no more.

1. **List the lines.** From the deck's list (`decklists/pool/<tier folder>/tatyova-simic-landfall.md`, the import list), write down
   every way it can win other than combat damage: combos of any size (loops, unbounded mana or damage, lock pieces
   that win), alternate-win cards (e.g. "you win the game"), mill-outs, and big drains. Name the cards each needs.
2. **Check each against the sim.**
   - Is it in the combo catalogue (`rust/crates/sim-core/src/impls/combos.rs` and the files it chains in `all()`),
     or handled by the cards' own code (an alternate-win card that calls `ai::win`)? Are the cards modelled
     (`rust/target/release/cardgen audit --deck tatyova-simic-landfall`)?
   - Does the AI go for it? Trace games: `rust/target/release/commander-sim --calibrate within --pool t1 --ai
     lookahead --trace N` (N = 0..199; the log says "<deck> goes for <combo>" and "*** <deck> wins by combo"). Count
     how often the pieces are drawn, assembled, attempted and stopped.
3. **Fix what is missing or broken**, in order of how often it would matter:
   - a line not in the catalogue: add it in the catalogue's style (a new file `impls/cc_tatyova.rs`
     with a `COMBOS` list, chained in `combos::all()`), with a test;
   - a line whose pieces are held but never put together: the AI's wish list for tutors and the cast priorities (see
     how other decks' `wish` / `prio_fn` work in `ai/plans.rs` and `ai/pool.rs`), with a test;
   - an alternate-win card or engine that doesn't work: fix its card code, with a test.
   Do not change the deck list. Tier 1 and 2 lists allow no two-card infinite combo: if you find one in the list,
   put it at the top of your report and leave the list as it is.
4. **Measure** before and after: this deck's wins by type from 200 traced games (`grep "^Winner"`), and the tier's
   scorecard `rust/target/release/commander-sim --realism --pool t1 --games 4000 --quiet`.

## Rules

- **Commands.** Every build, test and sim under the memory cap:
  `systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0 <command>` (four other agents share this
  machine; pass `-j 4` to cargo). Build from `rust/`. Run long commands in the background where your tools allow.
- **No Python**, not even inside a longer command.
- **Shared files** (`engine/*`, `ai/*`, `state.rs`, `hooks.rs`, `combos.rs`): smallest additive change, marked
  `// --- casual combos (tatyova)`, each listed in your report.
- **Formatting.** Only the files you write (`rustfmt --edition 2024 <file>`), never `cargo fmt`; lines up to 120.
- **Commits.** Commit often (`git add -A && git commit -m "..."`), no Co-Authored-By line; never push.
- **Before you finish:** `cargo test --release --no-fail-fast` passes, `cargo build --release --all-targets` has no
  warnings.
- **Report** in your final reply (no report file): a short summary first, then each line you found with its status
  before and after, what you changed (files), and the before/after numbers.
