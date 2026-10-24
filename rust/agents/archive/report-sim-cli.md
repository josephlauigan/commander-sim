# Report: commander-sim (sim-cli)

Crate `rust/crates/sim-cli`, binary `commander-sim`; sim-core untouched. The previous agent's WIP already built clean
with no warnings; this session verified it against `python3 -m commander_sim ... --engine rust` (bridge rebuilt with
rust/build-py.sh).

Verified identical (apart from the "(Ns total)" timing line):
- Sauron vs t1, loose, 960 games, seed 500000, 22 jobs: all numbers equal. Only row order of tied counts can differ.
- --swap "Spell Pierce=>Negate" vs t3 (120 games, --jobs 1): identical. 48 games at --jobs 12 differed only in a tie.
- --calibrate within, --deck veyran --pool all: identical.
- --trace: identical to the Rust game (Python's own --trace calls the Python engine, ignoring --engine rust, so
  compare it to the bridge's trace).

Python quirks / limits (not bugs in Rust):
- Tie order (equal counts, e.g. "eliminated by", opponent tables) follows Counter insertion order, which in Python
  with --jobs > 1 depends on which chunk finishes first (imap_unordered). Rust merges in seed order = Python --jobs 1.
- `--analyze` crashes in Python with --engine rust (bridge players lack seen_names/cast_names/death_kind/first_bomb).
  Checked with a shim giving those empty: everything else identical; Rust fills them in properly.
- `--cards` reads Python's compiler output (scryfall/dsl sources, "not modeled" text); Rust describes the exported
  data/cards.json records, so source labels, "pain" tags and unparsed text differ.
- --dsl-all and --ai rigid are not available (Python card compiler / no rigid AI in Rust).
- Speed: single thread matches Python per game (~1.3 cpu-s/game lookahead); with 22 threads cpu-s/game rose to ~2.8
  while the machine was loaded by other agents (load avg 10-17). mimalloc made no difference (not kept). Worth a
  re-measure on a quiet machine.
