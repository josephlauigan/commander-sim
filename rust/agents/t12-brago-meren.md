# Brief: model the Tier 4/5 cards for `brago-azorius-blink-control` and `meren-golgari-recursion` (Tier 2)

You work in the git worktree `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/t12-brago-meren` on branch `t12-brago-meren`. It is your whole world: run every command from there.
Read `CLAUDE.md` first (memory cap, no Python, loose profile). Do not use `rust/agents/RULES.md`: it is from an older
port. These rules replace it.

## Goal

The pool deck(s) `brago-azorius-blink-control` and `meren-golgari-recursion` (Tier 2) were just replaced by EDHREC's average decks (Tier 1: bracket 2, "core"; Tier 2: bracket 3, "upgraded") (real lists). Some of
their cards are Partial or Unmodeled in the simulator, so the decks play differently from the real ones. Model those
cards faithfully to their Oracle text so they play as the real cards do.

- The cards and their Oracle text are in `rust/agents/t12-brago-meren.cards.md`. Cards marked "skip" belong to another agent.
- The audit shows each card's status: `rust/target/release/cardgen audit --deck <deck key>` (Partial / Unmodeled and
  why).

## How

1. **Prefer the ability language when the pattern is general.** That means compiling Oracle text in
   `rust/crates/cardgen/src/dsl_parse.rs` and running it in `rust/crates/sim-core/src/dsl/mod.rs`. A new pattern helps
   every card with that wording. Put your new patterns together under a comment `// --- Tiers 1/2 (brago-meren)`, so merging
   with other agents' work is easy. When you change the compiler, check that `cardgen export` changes no card you
   did not mean to change (`git diff data/cards.json`).
2. **Use hand-written card code for one-off cards**, in a new file `rust/crates/sim-core/src/impls/t12_brago_meren.rs`.
   - Register it in `impls/mod.rs` (at the end of `register`).
   - Follow the style of `impls/t3_korvold.rs`: a doc comment per card that says what is modelled and what is left
     out.
   - Look at how similar cards are already done (grep the impls) and reuse shared helpers (`impls/common.rs`,
     `engine/*`) instead of copying.
   - A card that is compiled from Oracle text and also hand-written must not run twice: check both paths (a compiled
     trigger that runs next to a hook is a bug that happened before: Contaminant Grafter).
3. **Keep shared-file changes small and additive** (`engine/*`, `ai/*`, `state.rs`, `hooks.rs`). List each one
   in your report.
4. **Status notes.** A card you model by hand that the audit still calls Partial or Unmodeled needs a
   `[card_notes]` entry in `data/audit.toml`: `"Name" = { status = "Full" | "Approximate", note = "..." }`. Use
   Approximate when something is left out, and say what. Put your entries under a comment `# --- Tiers 1/2 (brago-meren)`.
5. **Combos.** These decks win by combos as well as combat. For each of your decks, list the two- and three-card
   combos its new list holds (EDHREC's list; e.g. Thassa's Oracle with Demonic Consultation or Tainted Pact) and check
   each is in the combo catalogue (`rust/crates/sim-core/src/impls/combos.rs` and the files it chains). Add the
   missing ones in the same style, with a test. Remove nothing: other decks use the catalogue.
   **Tiers 1 and 2 allow no two-card infinite combos** (`decklists/pool/README.md`). If your new list holds one, put
   it at the top of your report (the two cards) and still add it to the catalogue; do not change the list.
6. **The deck file.** `decklists/pool/<tier folder>/<deck key>.md` still describes the old list above its decklist
   (the Game Changers line, Combos, Lands, Role, At the table, Strategy, Sim modeling notes). Rewrite those sections
   for the new list, in the same style. Do not touch the decklist or the import list below them. Run
   `rust/target/release/cardgen validate` after.
7. **Regenerate the card data** after compiler changes: `rust/target/release/cardgen export`. Commit
   `data/cards.json` and `data/decks.json` with your change.
8. **Tests.** Add one per card, or per pattern with two or three cards, in `rust/crates/sim-core/tests/cards_t12_brago_meren.rs`.
   Use `sim_core::testkit` (`table`, `perm`, `hand`, `lands`, `card`, `token`) as the other card tests do.
   Each test checks the card's effect on a small position. Don't rely on what a library holds: set up what you need.
9. **Priority:** do the cards that change how the deck plays first (engines, mana, combo pieces, lords, removal).
   Then the rest. A card that is truly unmodellable may stay as it is; say why in the report.

## Rules

- **Tests.** Every test must pass (`cargo test --release --no-fail-fast` shows every failure; plain `cargo test` stops at the first failing binary).
- **Commands.** Run every build, test and sim under the memory cap:
  `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 <command>`. Build from `rust/`.
- **No Python.** Use jq, shell tools, cardgen and Rust.
- **Formatting.** Format only the files you write (`rustfmt --edition 2024 <file>`), never `cargo fmt`. The
  maximum line length is 120.
- **Commits.** Commit often in your worktree (`git add -A && git commit -m "..."`). No Co-Authored-By line. Never
  push, and never touch other branches or worktrees.
- **Before you finish:** `cargo test --release` passes and `cargo build --release --all-targets` has no
  warnings. Then run your tier's scorecard
  (`rust/target/release/commander-sim --realism --pool t2 --ai adaptive --games 4000 --quiet`) and include
  its table in the report.
- **Report.** Don't write a report file (the harness refuses it). Put the full report in your final reply, after a
  short summary of 15 lines or fewer. The report lists:
  - each card's new status;
  - the combos found, and which you added;
  - the shared files changed;
  - the patterns added;
  - anything left Partial and why.
