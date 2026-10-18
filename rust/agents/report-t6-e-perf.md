# Report t6-e-perf: engine speed after the Tier 6 pass

## Profile findings
- `perf` is unusable here (kernel.perf_event_paranoid=4, no sudo). Used a gdb stack sampler instead (200 samples, `--jobs 1 --games 40`
  `--calibrate within --pool t3`, both builds); inclusive share of samples per sim_core function, mainbase vs branch.
- Start: 40-game jobs-1 run 127-145 s (mainbase) vs 183-217 s (branch), about +45% (box shared with another session).
- Hot spots that grew: `combos::is_piece_name` (linear scan of ~100 combos x groups with string compares, called per permanent from
  `pval -> threat_value -> piece_threat`); `sba_toughness -> etgh -> dsl::pt` (new state-based action calls etgh for every creature
  at every SBA pass; `pt` scanned all permanents three times with Arc clones); `steelshaper_bonus` (subtype lookup per creature per pt call);
  `combo_options` (has_piece string scans over every combo and every card in hand/play).

## Optimisations (each its own commit, none changes a game result)
1. `is_piece_name` reads a HashSet built once (OnceLock).
2. `dsl::pt`: one pass over the statics (anthem, equip_bonus, bma_anthem) instead of three `statics()` iterators.
   After 1+2: t3 300 games jobs 12 CPU 1236 s vs mainbase 1094 s (+13%).
3. `cloud::steelshaper_bonus`: count equipped Steelshapers (name compare) before the subtype lookup.
4. `combos::combo_options`: held-name HashSet (hand + non-phased permanents) built once, replacing per-combo `has_piece`.
   After 3+4: CPU 1120 s (+2.4% vs the 1094 s mainbase run, taken under heavier load).
Not touched: HAND_MANA Spirit Guide scan, cast gates, would_be_cast_countered, each_combat did not show up in the samples.
sba_toughness still calls etgh per creature (no sound cheap bound found); it is now cheaper through pt.

## Byte-for-byte check
`--calibrate within --pool t3 --games 300 --seed 500000 --jobs 12`, output minus progress/timing lines, identical to the lock
(`rust/agents/perf-lock-t3.txt`) after every step. The lock was re-saved once, after merging t6-cedh (Karlach fix; a data-only change
in cards.json/card_overrides.toml that changes results); optimisations 3 and 4 were checked against the post-merge lock, 1 and 2
against the pre-merge one. `cargo test --release --no-fail-fast`: 2520 passed, 0 failed (one earlier run showed a
`practice --test http_play` failure that did not reproduce twice).

## Final timing (`--games 1500`, t3, seed 500000, jobs 12, back to back)
- mainbase: CPU 6031 s, wall 643 s (box load rose to ~12 from another session).
- branch (t6-e-perf with t6-cedh merged): CPU 5810 s, wall 488 s.
Under noisy load the branch is at parity with mainbase (CPU -4%, within noise); the quieter 300-game runs gave +2.4%. Goal (within ~10%) met.
