# Handoff: Phase 4 PR 5, ISMCTS where it matters

Brief: `rust/agents/phase4-search.md`. Design: `audit/phase4/search-design.md`. Branch `phase4-search` (on
`phase4-eval`). Scratch: `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase4-search/`
(`sim.sh` (env JOBS, default 12) `BIN POOL GAMES OUT AI_SEARCH|- [SEED] [K=V..]`: one capped sim, rc / seconds / games/s in `OUT.rc`, the
search cost line and diagnostics on `OUT.err`; `half.sh OLD NEW "Name;Name"`; `pair.sh`; `gate.awk`;
`bin/commander-sim-base` = the base, built from a clean `git archive f03f69bc`).

## State

- [x] Today's cost (`M/`, 300 games t1-t5; counter binary = base byte for byte on all five): design doc table.
- [x] Design (c08923d0).
- [x] Code: `ai/ismcts.rs`; dispatch in `search::choose_inner` / `choose_attack_inner` / `choose_counter_inner`;
  `brain::main` asks `ismcts::tree_pick` inside a playout; cost counter `state::SearchCost` (per seat) booked by
  `search::booked`, printed by `realism` on stderr. Tests `tests/ismcts.rs` (5).
- [x] Stage A (binary `bin/commander-sim-A2`, `stageA.sh`, outputs `A/`, 22 jobs): `AI_SEARCH` unset = base byte for
  byte on t1, t2, t4, t5 and `=0` on t1 (300 games, all rc=0); `AI_SEARCH=1` twice on t1 = the same output and cost.
  Binary B (cfbd9ec3, attacks/counters capped at 6) unset = base on t1 too.
- Jobs: from Stage B on, sims run `--jobs 12` and cargo `-j 12` (coordinator, 2026-10-30: a second agent shares the
  machine). The output does not depend on `--jobs`: base and ISMCTS t1 300 games at 12 = at 22 byte for byte
  (`A/t1-base-j12.txt`, `A/t1-onB-j12.txt`), so the 22-job identity checks above stand. Every Stage B arm uses 12, so
  games/s compare like with like (they are not comparable with the 22-job numbers in the design's cost table).
- [ ] Stage B running (`stageB.sh`, binary `bin/commander-sim-B` = cfbd9ec3, 12 jobs, `stageB.log`).
- [ ] Stage C.
- [ ] Part (b): blocks, protection, wipe responses (not started).

## Gotchas

- `cargo build -p sim-cli --example X` / `--test X` builds only that target: build `-p sim-cli` alone for the binary
  and `cmp` it.
- `pgrep -f X` inside an `until` loop matches the loop itself: wait on output text.
- `gen` is a reserved word in edition 2024.
- The search cost goes to stderr so the report stays byte-identical; `OUT.err` has it.
