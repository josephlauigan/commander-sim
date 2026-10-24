# Task: `commander-sim`, the run harness in Rust (no Python)

Worktree: `/home/joseph/projects/mtg_sims/commander-sim/.claude/worktrees/agent-a823efba96445de25`
(branch `worktree-agent-a823efba96445de25`). A previous agent was stopped while writing `main.rs`; its work is in
WIP commit eb30623 (new crate `rust/crates/sim-cli`).

Today: `python3 -m commander_sim --deck sauron --pool t1 --profile loose --games 960 --seed 500000 --jobs 22
--engine rust` — Python's compare.py / poolmode.py parse options, pick seats and seeds, dispatch games (each played
by Rust through crates/sim-py and sim-core/src/run.rs), tally and print reports. Build the same with no Python, in
the new crate `rust/crates/sim-cli` (binary `commander-sim`); touch sim-core only additively. clap/rayon/serde_json
are fine.

1. Port compare.py, poolmode.py, pools.py, decks.py, deck_files.py, __main__.py and the experiment tools
   (tools/swaptest.py, tools/searchtest.py if useful). Options: --deck, --swap (paired A/B, same seeds), --pool
   t1..t5|all, --all-decks, --calibrate within|ordering|all, --games/--n, --seed, --profiles/--profile, --analyze,
   --trace GAME, --cards, --brief, --ai lookahead|adaptive (rigid if Rust has it), --temp, --quiet, --jobs. Drop
   --engine. Say what you did with --dsl-all and anything needing the Python card compiler.
2. Same seat selection, seeds, profiles and statistics as poolmode.py, so numbers equal
   `python3 -m commander_sim ... --engine rust` exactly; same report layout and wording. Verify on: the Sauron vs t1
   960-game run, a --swap run, --calibrate within, --analyze, --trace (rebuild the Python bridge first with
   `rust/build-py.sh` so both use the same Rust code).
3. Parallel games on threads (--jobs, default like Python's) with the progress line.
4. `rust/crates/sim-cli/README.md`; update `rust/README.md`.

No difftest needed. Commit message: "Rust port: commander-sim command-line program (the run harness without Python)".
