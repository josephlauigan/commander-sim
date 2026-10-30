# Handoff: Phase 4 PR 5, ISMCTS where it matters

Brief: `rust/agents/phase4-search.md`. Design: `audit/phase4/search-design.md`. Branch `phase4-search` (on
`phase4-eval`). Scratch: `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase4-search/`
(`sim.sh BIN POOL GAMES OUT AI_SEARCH|- [SEED] [K=V..]`: one capped sim, rc / seconds / games/s in `OUT.rc`, the
search cost line and diagnostics on `OUT.err`; `half.sh OLD NEW "Name;Name"`; `pair.sh`; `gate.awk`;
`bin/commander-sim-base` = the base, built from a clean `git archive f03f69bc`).

## State

- [x] Today's cost (`M/`, 300 games t1-t5; counter binary = base byte for byte on all five): design doc table.
- [x] Design (c08923d0).
- [x] Code: `ai/ismcts.rs`; dispatch in `search::choose_inner` / `choose_attack_inner` / `choose_counter_inner`;
  `brain::main` asks `ismcts::tree_pick` inside a playout; cost counter `state::SearchCost` (per seat) booked by
  `search::booked`, printed by `realism` on stderr. Tests `tests/ismcts.rs` (5).
- [ ] Stage A (`stageA.sh BIN`, outputs `A/`).
- [ ] Stage B (`stageB.sh`, binary `bin/commander-sim-B`, outputs `B/tN-{old,A,B}.txt`).
- [ ] Stage C.
- [ ] Part (b): blocks, protection, wipe responses (not started).

## Gotchas

- `cargo build -p sim-cli --example X` / `--test X` builds only that target: build `-p sim-cli` alone for the binary
  and `cmp` it.
- `pgrep -f X` inside an `until` loop matches the loop itself: wait on output text.
- `gen` is a reserved word in edition 2024.
- The search cost goes to stderr so the report stays byte-identical; `OUT.err` has it.
