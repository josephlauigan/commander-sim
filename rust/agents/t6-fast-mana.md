# Brief: Tier 6 fast mana held when it should be played

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/t6-fast-mana` on branch `t6-fast-mana`, which is based on
`t6-quiet-progress`. Run every command from there. Read `CLAUDE.md` first: memory cap, no Python, loose profile.
Do not use `rust/agents/RULES.md`.

## Goal

The Tier 6 gate is seat 1's share of decisive games. Real is 31.8%; the sim is about 28.4%. The lever is speed: real
cEDH decks deploy fast mana on turns 1-3 and reach their lines sooner. Two earlier agents found that Chrome Mox and
Mox Diamond are often held on turns 1-3 with the means to cast them. The calibrate report's per-deck block counts
"fast mana held" (`--calibrate within --pool t6`). Background is in `audit/realism/t6-seat-edge-2026-10-24.md`,
`audit/realism/t6-rograkh-thrasios-2026-10-26.md` and `rust/agents/handoff-realism.md`.

1. **Measure what's held.** For each Tier 6 deck, count each fast-mana card held while castable, by turn: Chrome
   Mox, Mox Diamond, Mox Amber, Lotus Petal, Mana Vault, Sol Ring, Grim Monolith, Jeweled Lotus, Simian/Elvish Spirit
   Guide, Dark Ritual-style rituals and the like. Use the calibrate report and traces
   (`commander-sim --calibrate within --pool t6 --ai lookahead --jobs 10 --trace N`).
2. **Find why** for each card. Possible causes:
   - the AI's priority or utility for it (`ai/brain.rs`, `ai/pool.rs`, `ai/decks.rs`, the T6 plans in
     `impls/t6_plan_*.rs`);
   - an imprint or discard cost it won't pay (Chrome Mox's imprint, Mox Diamond's land discard);
   - a "keep it for later" rule that's too broad;
   - the lookahead scoring the deployed card as worth less.
   
   Check Oracle text (`data/scryfall_cache.json`) for each card. A real player holds fast mana only for a reason:
   playing around a wipe, or a Mox Diamond that would cost the only land drop.
3. **Fix** the causes that are wrong, with a test each. Keep each fix as narrow as possible.
4. **Measure, paired, on the same seeds:**
   - `commander-sim --realism --pool t6 --games 4000 --jobs 10 --quiet` before and after (seats 1-4, each deck's
     share, the median win round);
   - the calibrate per-deck ready rounds and fast-mana-held counts.
   
   If one deck's share passes 32% or falls below 18%, find out why.

## Rules

- **Commands.** Run every build, test and sim from `rust/`, under
  `systemd-run --user --scope -q -p MemoryMax=12G -p MemorySwapMax=0 <command>`. Another agent shares the machine.
- **Thread limits.** Pass `--jobs 10` to every `commander-sim` run and `-j 10` to cargo. Run one command at a time,
  in the background.
- **No Python**, anywhere in a command line.
- **Deck seats.** Don't seat the decks `omnath` or `godo` in new tests.
- **Shared files** (`engine/*`, `ai/*`, `state.rs`, `hooks.rs`, `combos.rs`): make the smallest change, mark it
  `// --- t6 fast mana`, and list each one in your report. The fix must not change the play of JD's own decks
  (`ai::is_main`) unless it's a plain rules bug: say which.
- **Formatting.** Format only your files (`rustfmt --edition 2024 <file>`), never `cargo fmt`. Lines go up to 120
  characters.
- **Commits.** Commit often, with no Co-Authored-By line, and never push.
- **Before you finish:** `cargo test --release --no-fail-fast` passes, and `cargo build --release --all-targets` has
  no warnings.
- **Scratch files.** Put them in `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/t6-fast-mana/` only.
- **Report** in your final reply: a summary first, then each cause found and fixed, the files you changed, the before
  and after numbers, and the risks.
- **Waiting.** Don't end your turn while your own background command is still running: wait for it.
