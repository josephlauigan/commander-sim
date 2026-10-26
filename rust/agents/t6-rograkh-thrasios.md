# Brief: missing win lines for `rograkh-thrasios-creature-combo` (Tier 6)

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/t6-rograkh-thrasios`
on branch `t6-rograkh-thrasios`. It is your whole world: run every command from there. Read `CLAUDE.md` first (memory
cap, no Python, loose profile). Do not use `rust/agents/RULES.md`.

## Goal

In the Tier 6 scorecard (`commander-sim --realism --pool t6`), Rograkh/Thrasios wins about 16.4% of the decisive
games it sits in. That is below the 18-32% band every other Tier 6 deck reaches, and it gets ready late (round 9, from
`audit/realism/t6-seat-edge-2026-10-24.md`). `rust/agents/handoff-realism.md` ("Next, in order", item 2) names lines
the real list holds that the sim doesn't model, for example This Town Ain't Big Enough and the improvised Thrasios wins.
Find every real win line the list holds and make the sim play them, no more.

1. **List the lines.** From `decklists/pool/t6-cedh/rograkh-thrasios-creature-combo.md` (the import list) and
   `audit/t6/rograkh-thrasios-creature-combo/SPEC.md`, write down every way the deck can win. Name the cards each one
   needs and check each line against Oracle text (`data/scryfall_cache.json`).
   - Combos of any size: unbounded mana or loops, and what turns unbounded mana into a win (Thrasios draws the
     library, then what kills).
   - Win-cons and finishers.
2. **Check each against the sim.**
   - Is it in the combo catalogue (`rust/crates/sim-core/src/impls/combos.rs`, the files `all()` chains in, and the
     T6 plan `impls/t6_plan_rograkh_thrasios.rs`, `t6_combos_t6.rs`)?
   - Are the cards modelled (`rust/target/release/cardgen audit --deck rograkh-thrasios-creature-combo`)?
   - Does the AI go for it? Trace games with `rust/target/release/commander-sim --calibrate within --pool t6 --ai
     lookahead --jobs 16 --trace N` (N = 0..199). The log says "<deck> goes for <combo>" and "*** <deck> wins by
     combo". The calibrate report's per-deck block shows ready, one-tutor-away and go rounds, and what it chose while
     ready.
3. **Fix what is missing or broken**, most frequent first:
   - Add missing lines to the catalogue or the deck's T6 plan, in the existing style, with a test.
   - Fix tutor wishes and cast priorities when the pieces are held but never assembled, with a test.
   - Fix card code that's wrong, with a test.
   - Don't change the deck list.
4. **Measure** before and after:
   - the deck's wins by type in 200 traced games;
   - the Tier 6 scorecard: `commander-sim --realism --pool t6 --games 4000 --jobs 16 --quiet`, the same seeds both
     times;
   - the calibrate per-deck ready round.

## Rules

- **Commands.** Run every build, test and sim from `rust/`, under the memory cap:
  `systemd-run --user --scope -q -p MemoryMax=16G -p MemorySwapMax=0 <command>`.
- **Thread limits.** Pass `--jobs 16` to every `commander-sim` run, without exception, and `-j 16` to cargo. Run only
  one sim or cargo command at a time, and run long commands in the background.
- **No Python**, not even inside a longer command.
- **Deck seats.** Don't seat the decks `omnath` or `godo` in new tests: another session is retiring them.
- **Shared files** (`engine/*`, `ai/*`, `state.rs`, `hooks.rs`, `combos.rs`): make the smallest additive change, mark
  it `// --- t6 rograkh-thrasios`, and list each one in your report.
- **Formatting.** Format only the files you write (`rustfmt --edition 2024 <file>`), never `cargo fmt`. Lines go up to
  120 characters.
- **Commits.** Commit often, with no Co-Authored-By line, and never push.
- **Before you finish:** `cargo test --release --no-fail-fast` passes, and `cargo build --release --all-targets` has
  no warnings.
- **Scratch files.** Put them in `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/t6-rograkh/`
  only.
- **Report** in your final reply, 15 lines or fewer, plus details in `rust/agents/report-t6-rograkh-thrasios.md`:
  - each line found, with its status before and after;
  - the files you changed;
  - the before and after numbers.
