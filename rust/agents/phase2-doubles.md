# Brief: Phase 2, PR 2: fix the hard doubles, delete hand code the compiled version covers

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/phase2-doubles` on branch `phase2-doubles`, which is based
on `phase2-one-model` (PR #153). Run every command from there. Read `CLAUDE.md` first: memory cap, no Python,
loose profile. Do not use `rust/agents/RULES.md`.

## Background

`cardgen audit --paths` (PR #153; the report is `audit/phase2/paths-2026-10-26.md`) lists each card's paths into
the sim. It found **21 hard doubles**: the same action from two executors for one event. It also found **28 cards
whose hand-written rule code the compiled ability language (DSL) already covers** (the dsl-covers group).

JD's direction (2026-10-26): the goal is **one card model**, with the ability language preferred, and hand-written
**rule** code removed wherever the compiled version does the card correctly. Hand-written **AI** slots (`prio`,
`options`, `hand_options`, `land_options`, `gy_options`, `spell_imp`, `pval`) stay: they belong to a later
phase.

## Work

For every hard double, and for every dsl-covers card:

1. **Read the Oracle text** (`data/scryfall_cache.json`), the compiled DSL (`data/cards.json`), the hand code
   (find it with `cardgen audit --paths` and grep the `impls/`), and any executing tag. Decide which version
   does the card correctly.
2. **If the compiled version is correct and complete for that ability, delete the hand rule code for it.** That's
   the slot registration, plus the function if nothing else uses it. Keep its AI slots. Keep or rewrite the card's
   tests so they still pass, now exercising the compiled path.
3. **If the hand code is right and the compiled version is wrong or partial,** remove or reduce the compiled
   ability for that card with a `dsl = [...]` or `dsl = []` override in `data/card_overrides.toml`, giving the
   reason in a note. Then re-export with `cardgen export` and `cardgen validate`. If the ability language could
   express it with a small parser fix, note that as a follow-up rather than doing it here.
4. **If the two paths really do different parts of the card,** add `paths_ok = "why"` to its override.
5. **Every card you touch** gets a test that the effect happens exactly once (for example: a Medallion reduces a red
   spell's cost by exactly {1}; Laelia's attack gives the right result once; Squelcher's spells can't be countered
   and nothing runs twice).
6. **Check the report's own doubts by hand:**
   - Twilight Diviner and Daxos (events are matched per slot, so they may not be real doubles);
   - the toxic cards (does the `combat_damage` code give toxic?);
   - Tataru Taru (does the `draw` tag and the code etb draw twice?);
   - Allosaurus Shepherd;
   - Liliana, Waker of the Dead;
   - Oath of Teferi (compiled exile with no return).
   
   A false positive gets `paths_ok` with the reason, and your report says so.
7. **Heliod's Pilgrim:** move the load-time `data_fixes` in `sim-core/src/cards.rs` into an override, if that's a
   clean swap.
8. **The `HOOK_TOXIC` name list** (`impls/tymna_cards.rs`): replace it with reduced `dsl` overrides if that's
   clean. Otherwise leave it and say why.

## Checks

- `cardgen audit --paths` afterwards: **0 hard doubles not declared with `paths_ok`**, and the dsl-covers count
  down by the number of deletions. Put the new summary in your report, and regenerate
  `audit/phase2/paths-2026-10-26.md` as `paths-after-doubles.md`.
- `cargo test --release --no-fail-fast` passes, and `cargo build --release --all-targets` has no warnings.
- **Measure:** these fixes change behaviour. Run `commander-sim --realism --pool tN --games 4000 --jobs 20 --quiet`
  before (on the base branch's binary, built first) and after, for every tier whose pool plays a changed card (find
  them with `cardgen audit --paths --deck` or the decks in `data/decks.json`). Report each deck's share and the
  scored targets, before and after.
- **JD's decks:** if a changed card is in one of JD's decks (`data/my_decks.toml`), say which, and run a
  `commander-sim --deck KEY --games 2000 --jobs 20` before and after against its own tiers (the default).

## Rules

- **Commands.** Run every build, test and sim from `rust/`, under
  `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 <command>`. You're the only job on the
  machine: `-j 20` for cargo, `--jobs 20` for sims, one command at a time, in the background.
- **No Python**, anywhere in a command line.
- **Card data.** Change it only through `data/card_overrides.toml` and `cardgen export`. Don't change deck lists.
- **Deck seats.** Don't seat the decks `omnath` or `godo` in new tests.
- **Formatting.** Format only your files (`rustfmt --edition 2024 <file>`), never `cargo fmt`. Lines go up to 120
  characters.
- **Commits.** Commit often, with no Co-Authored-By line, and never push.
- **Scratch files.** Put them in `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase2-doubles/` only.
- **Report** in your final reply:
  - a table of every card handled: its double class, the side kept, what was deleted or overridden, and its test;
  - the false positives;
  - the follow-ups for the ability language;
  - the new paths summary;
  - the before and after scorecards;
  - the risks.
- **Waiting.** Don't end your turn while your own background command is still running: wait for it.
