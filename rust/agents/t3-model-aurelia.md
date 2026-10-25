# Brief: model the Tier 3 cards for aurelia-boros-extra-combats

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/t3-model-aurelia` on branch `t3-model-aurelia`. It is your whole world: run every command
from there. Read `CLAUDE.md` first (memory cap, no Python, loose profile). Do not use `rust/agents/RULES.md`: it is
from an older port. These rules replace it.

## Goal

The Tier 3 pool deck(s) `aurelia-boros-extra-combats` were just replaced by EDHREC's bracket-3 average decks (real lists). Some of their
cards are Partial or Unmodeled in the simulator, so the decks play weaker than the real ones. Model those cards
faithfully to their Oracle text so they play as the real cards do.

- The cards and their Oracle text are in `rust/agents/t3-model-aurelia.cards.md`.
- The audit shows each card's status: `rust/target/release/cardgen audit --deck <deck key>` (Partial / Unmodeled and
  why).

## How

1. **Prefer the ability language when the pattern is general.** That means compiling Oracle text in
   `rust/crates/cardgen/src/dsl_parse.rs` and running it in `rust/crates/sim-core/src/dsl/mod.rs`. Examples: "other
   Elf creatures you control get +1/+1", "attacking creatures you control have double strike", "other permanents you
   control have indestructible", "whenever you sacrifice a permanent, put a +1/+1 counter on ~". A new pattern helps
   every card with that wording. Put your new patterns together under a comment `// --- Tier 3 (aurelia)`, so merging
   with other agents' work is easy.
2. **Use hand-written card code for one-off cards**, in a new file `rust/crates/sim-core/src/impls/t3_aurelia.rs`.
   - Register it in `impls/mod.rs`.
   - Follow the style of `impls/t6_green2.rs`: a doc comment per card that says what is modelled and what is left
     out.
   - Look at how similar cards are already done (grep the impls) and reuse shared helpers (`impls/common.rs`,
     `engine/*`) instead of copying.
3. **Keep shared-file changes small and additive** (`engine/*`, `ai/*`, `state.rs`, `hooks.rs`). List each one
   in your report.
4. **Status notes.** A card you model by hand that the audit still calls Partial or Unmodeled needs a
   `[card_notes]` entry in `data/audit.toml`: `"Name" = { status = "Full" | "Approximate", note = "..." }`. Use
   Approximate when something is left out, and say what.
5. **Regenerate the card data** after compiler changes: `rust/target/release/cardgen export`. Commit
   `data/cards.json` and `data/decks.json` with your change.
6. **Tests.** Add one per card, or per pattern with two or three cards, in `rust/crates/sim-core/tests/cards_t3_aurelia.rs`.
   Use `sim_core::testkit` (`table`, `perm`, `hand`, `lands`, `card`, `token`) as the other card tests do.
   Each test checks the card's effect on a small position.
7. **Priority:** do the cards that change how the deck plays first (lords, anthems, damage doublers, combat
   keywords, engines). Then the rest. A card that is truly unmodellable may stay as it is; say why in the report.

## Rules

- **Commands.** Run every build, test and sim under the memory cap:
  `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 <command>`. Build from `rust/`.
- **No Python.** Use jq, shell tools, cardgen and Rust.
- **Formatting.** Format only the files you write (`rustfmt --edition 2024 <file>`), never `cargo fmt`. The
  maximum line length is 120.
- **Commits.** Commit often in your worktree (`git add -A && git commit -m "..."`). No Co-Authored-By line. Never
  push, and never touch other branches or worktrees.
- **Before you finish:** `cargo test --release` passes and `cargo build --release --all-targets` has no
  warnings. Then run the Tier 3 scorecard
  (`rust/target/release/commander-sim --realism --pool t3 --ai adaptive --games 4000 --quiet`) and include its
  table in the report.
- **Report.** Write `rust/agents/report-t3-model-aurelia.md`, commit it, and end with a short reply (15 lines or fewer).
  The report lists:
  - each card's new status;
  - the shared files changed;
  - the patterns added;
  - anything left Partial and why.
