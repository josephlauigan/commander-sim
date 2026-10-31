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
- [x] Stage B, first run (binary B = cfbd9ec3; arms moved to `B1/`): t1 half A -3.2 [-8.5, +2.1], half B -3.6
  [-7.2, +0.0], steps a game 229k -> 181k / 172k. Both lean down, so the queue was stopped after t2-old (the
  `AI_SEARCH=0` arms are config-free: `B/t1-old`, `B/t2-old` are reused). Suspect: the clear-leader stop and the pick
  could overrule the top play on 2 shared copies (a 1-df t-test). Fix 3868adee (binary C): 6 shared copies against
  the top (`AI_SEARCH_PAIRS`). Ablations on t1 half B with C (`abl.sh`, `X/`): pairs 6; + gate off; + depth 1.
- Jobs back to 22 (coordinator, 2026-10-30 ~18:20, the other agent finished): `sim.sh` defaults to 22 again, cargo
  `-j 22`. Arms run so far at 12: `B/t1-old`, `B/t2-old`, `B1/*`, `X/t1-B-pairs6`; the output doesn't depend on
  `--jobs` (checked above), so paired and byte-for-byte comparisons across them stand; only games/s differ.
- The machine is shared (another agent's sims): games/s are noisy across arms; `OUT.cpu` (user+sys CPU seconds, from
  t2-old on) and the steps a game are the steadier cost numbers.
- [x] Stage C (below).
- [ ] Part (b): blocks, protection, wipe responses (not started).

## Gotchas

- `cargo build -p sim-cli --example X` / `--test X` builds only that target: build `-p sim-cli` alone for the binary
  and `cmp` it.
- `pgrep -f X` inside an `until` loop matches the loop itself: wait on output text.
- `gen` is a reserved word in edition 2024.
- The search cost goes to stderr so the report stays byte-identical; `OUT.err` has it.

## Stage B results (2000 games per arm, seeds 500000-501999, loose, look-ahead; all 15 arms rc=0)

Binary D (02b729d2). The half's summed share of the decisive games its decks sat in, `AI_SEARCH=0` against
`AI_SEARCH=<half's keys>` (one shared old arm per tier); Wald 95% CIs, the decks' variances summed. Level = the CI
includes 0. Cost per arm, the whole table (all seats; only the half's seats search by ISMCTS): playout steps a game,
look-ahead decisions a game, games/s and CPU seconds (user+sys) for the 2000 games at 22 jobs. The t1 and t2 old arms
first ran at 12 jobs on a shared machine; their speed is from a rerun at 22 (`Bs/`, output byte-identical).

| tier | half | old (95% CI) | ISMCTS (95% CI) | ISMCTS - old (95% CI) | steps/game old / ISMCTS | decisions/game | g/s old / ISMCTS | CPU s old / ISMCTS | verdict |
|---|---|---|---|---|---|---|---|---|---|
| t1 | A (Isshin, Light-Paws, Teysa) | 91.0% [87.2, 94.8] | 91.7% [87.9, 95.5] | +0.7 [-4.6, +6.0] | 228,753 / 208,913 (-9%) | 73.0 / 65.6 | 3.39 / 3.57 | 11,570 / 10,639 | level |
| t1 | B (Lathril, Tatyova) | 34.1% [31.5, 36.7] | 32.3% [29.7, 34.9] | -1.8 [-5.5, +1.9] | 228,753 / 204,871 (-10%) | 73.0 / 69.1 | 3.39 / 4.16 | 11,570 / 9,996 | level |
| t2 | A (Brago, Lord Windgrace, Sythis) | 84.3% [80.6, 88.0] | 81.6% [77.9, 85.3] | -2.7 [-7.9, +2.5] | 393,538 / 298,566 (-24%) | 104.4 / 95.4 | 1.95 / 2.61 | 21,686 / 15,998 | level |
| t2 | B (Kaalia, Meren) | 40.9% [38.1, 43.7] | 39.2% [36.4, 42.0] | -1.7 [-5.7, +2.3] | 393,538 / 371,881 (-6%) | 104.4 / 99.1 | 1.95 / 1.92 | 21,686 / 21,595 | level |
| t3 | A (Atraxa, Korvold, Tergrid) | 76.6% [73.0, 80.2] | 74.9% [71.3, 78.5] | -1.7 [-6.8, +3.4] | 163,089 / 144,888 (-11%) | 67.1 / 58.6 | 6.75 / 5.91 | 5,472 / 4,969 | level |
| t3 | B (Aurelia, Marwyn) | 48.4% [45.4, 51.4] | 50.8% [47.8, 53.8] | +2.4 [-1.8, +6.6] | 163,089 / 152,349 (-7%) | 67.1 / 63.2 | 6.75 / 5.87 | 5,472 / 5,146 | level |
| t4 | A (Chulane, Krenko, Yuriko) | 64.7% [61.2, 68.2] | 63.4% [60.0, 66.8] | -1.3 [-6.2, +3.6] | 202,728 / 181,984 (-10%) | 69.1 / 65.3 | 3.60 / 4.76 | 10,036 / 9,096 | level |
| t4 | B (Edgar Markov, Prosper) | 60.5% [57.5, 63.5] | 58.5% [55.5, 61.5] | -2.0 [-6.3, +2.3] | 202,728 / 191,018 (-6%) | 69.1 / 65.8 | 3.60 / 4.43 | 10,036 / 9,568 | level |
| t5 | A (Kinnan, Urza, Zur) | 79.8% [76.1, 83.5] | 78.4% [74.7, 82.1] | -1.4 [-6.6, +3.8] | 172,093 / 157,278 (-9%) | 68.2 / 66.8 | 6.19 / 6.50 | 6,995 / 6,623 | level |
| t5 | B (Miirym, Yawgmoth) | 45.0% [42.1, 47.9] | 46.0% [43.1, 48.9] | +1.0 [-3.1, +5.1] | 172,093 / 162,972 (-5%) | 68.2 / 66.7 | 6.19 / 6.43 | 6,995 / 6,615 | level |

- Every pair is level, but 7 of 10 deltas are below 0 (mean -0.85 points): ISMCTS is at best level, not better.
- Deck level (`pair.sh`, `stageB-decks.txt`): one ISMCTS deck's own CI excludes 0, Meren (t2 B) -3.7 [-6.3, -1.1];
  Korvold (t3 B, an old-arm tablemate) -3.8 [-6.3, -1.3] as Aurelia and Marwyn rose. No ISMCTS deck's CI is above 0.
- Cost: steps a game fall 5-24% on the table with 2-3 of the 5 decks on ISMCTS, CPU 0-26% (t2 A the most).
  Wall-clock games/s is noisy (the last slow games of a run set it; t3's ISMCTS arms read slower while their CPU
  time is 6-9% lower). The full-table cost (every seat on ISMCTS) is in the final checks below.

## Stage C

All ten pairs are level, so by the brief's rule ISMCTS is the default for every generic-policy deck (Stage C commit
4f749acf). `AI_SEARCH=0` keeps today's look-ahead (a
measurement arm); JD's decks and Tier 6 always keep it (`values::generic_deck`). For JD: the lean is down (7 of 10
deltas below 0, mean -0.85; Meren -3.7 [-6.3, -1.1]), so it is level at a lower cost, not better. If that lean is
judged real, `AI_SEARCH=0` (or flipping `parse_mode`'s default) restores today's search with no other change.

## Final checks (head)

- Defaults (no env, 300 games, the head binary) = the ISMCTS arm (binary D, `AI_SEARCH=1`) byte for byte on t1, t2,
  t3, t4, t5, all rc=0 (`F/head-tN.txt` vs `F/arm-tN.txt`). Later commits change only docs.
- Full-table cost with every seat on ISMCTS (head, 300 games) against today's (`M/`, design table): steps a game t1
  174,366 vs 226,711 (-23%), t2 269,998 vs 398,275 (-32%), t3 141,626 vs 165,907 (-15%), t4 171,661 vs 203,809
  (-16%), t5 156,125 vs 181,116 (-14%); decisions a game 59 / 88 / 57 / 62 / 69 (today 73 / 107 / 68 / 70 / 72), of
  which played from the policy 11.7 / 17.1 / 13.2 / 7.1 / 4.0; about 2,200-3,300 playouts a game.
- `cargo build --release --all-targets -j 22`: 0 warnings. `cargo test --release --no-fail-fast -j 22`: 3697
  passed, 0 failed. `cardgen validate`: all decks passed. `all_decks adaptive 3` rc=0; `lookahead 1` rc=0 (21 games in
  49.6 s; PR 4: 70.4 s).
- t5 skips 1-7 games an arm on an internal simulator error ("WARNING: N game(s) hit an internal simulator error"),
  in the old arms too and in PR 4's: not from this PR.

## Not done / next

- Part (b) (blocks, protection and wipe responses as searched decisions): **not started**.
- Ideas for the lean: the tree below the root explores the decider's later choices once a node has 5+ visits
  (widen 0.5); a slower widening, or opponent nodes at the first reply, are the obvious next ablations. The
  ablation table is in the design doc.
