# Brief: deck AI audit, phase 2 (fix every problem)

JD wants every card in his decks implemented correctly AND valued by the deck's AI for that deck's strategy. Phase 1
listed the problems in `audit/ai-audit/<key>.md` (section "Problems", plus any "generic-WRONG" or model gaps in the
"All cards" table). **Fix all of them for your decks.** Your task message names your worktree, your decks or items.

## Ground rules

- Work only in your own worktree (`/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/audit-<x>`), on
  its branch. `cd` into it for every command. It already has a build in `rust/target` (copied): build with
  `cargo build --release -j 2` from `rust/`.
- **CPU (strict, the machine is shared):** cargo with `-j 2`; tests with `RUST_TEST_THREADS=2`; sims with `--jobs 2`;
  everything under `nice -n 19` and `systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0`. One heavy
  command at a time.
- **No Python. No decklist changes** (no `cardgen deck`, no edits to decklists/ lists). **Never run `cardgen audit`**
  (it overwrites the deck docs; the core agent fixes that). No Co-Authored-By lines.
- **Pool decks must not change** (only the core agent's Teferi's Protection fix may change them). Scope fixes to your
  deck keys: their priority functions, plan config, tutor wish-list arms (`tutor_pick_named` in `ai/decks.rs`), their
  impls files, hand-tagged lines in `data/carddb.txt` for cards only your decks play. If a fix must touch shared code
  (ai/brain.rs, ai/pool.rs, engine/*, dsl/*, cardcode.rs, mine.rs shared fns), gate it on your deck keys, keep it
  small and additive, and list it in your report.
- **Do not touch these lines** (another session changed them on an unmerged branch): cloud.rs `steelshaper_bonus`;
  seph.rs around line 836 (`treasures -= 1`); mine.rs `ring_level`, every `g.hooks.is_empty()`, and Deflecting Swat
  inside `veyran_protect`; decks.rs `deck_prio`'s dispatch block, `protect_response`, `wipe_modes`. Leave anything
  Godo (`t6_godo*.rs`, Godo tests) alone.
- Card behaviour must match Oracle text (Scryfall cache: `data/scryfall_cache.json`, via `jq`).

## How to work

1. Read your decks' audit files and deck docs (`decklists/JD/<file>.md`, file per key in `data/my_decks.toml`).
2. Fix the problems in order of likely impact. For each fix: a test in `rust/crates/sim-core/tests/` (follow the
   existing `cards_<deck>.rs` files and `testkit`), then commit (standalone `git add -A && git commit -m "..."`).
3. Show each strategy fix works in play: a short `--trace N` excerpt or an adaptive card report
   (`--deck <key> --pool <its tier> --analyze --ai adaptive --games 300 --jobs 2`) before/after, in your report.
   You cannot change a fact of the decklist; if a "problem" is really a deckbuilding issue, say so instead.
4. Before you finish: `cargo build --release -j 2 --all-targets` with no warnings, and `cargo test --release -j 2`
   with `RUST_TEST_THREADS=2` passing (the practice-mode `http_play` image/reset race is a known flake: rerun it once).
   Pool check: `rust/target/release/commander-sim --calibrate within --pool t3 --games 40 --ai adaptive --jobs 2`
   output must be identical to the same command on the `ai-audit` worktree's binary
   (`/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/ai-audit/rust/target/release/commander-sim`, run
   from that worktree). Core agent: expected to differ only where Teferi's Protection is involved; explain.

## Handoff and report

- Keep `rust/agents/handoff-audit-<x>.md` current (done / next / gotchas); write it after your first fix.
- When your context gets long (about 60 tool calls), stop at a clean point (committed, handoff updated) and end with a
  report whose first line is `HANDOFF`.
- Final report: at most 15 lines (commits, fixed / not fixed per problem number, test and pool-check results, shared
  code touched). Full details in `rust/agents/report-audit-<x>.md`, committed.
