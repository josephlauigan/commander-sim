# Phase 2 doubles: handoff (done 2026-10-29; measured after merging origin/main)

Brief: `rust/agents/phase2-doubles.md`. Branch `phase2-doubles`, not pushed. Report: `rust/agents/report-phase2-doubles.md`.

## Done (committed)
- 34a92a6a: the doubles (cost hooks of the Medallions, Foundry Inspector, Etherium Sculptor, Starfield Mystic deleted;
  toxic hooks and `HOOK_TOXIC` deleted; Snap's hand resolve deleted; `dsl = []` for Laelia, Liliana Waker of the Dead,
  Oath of Teferi, Heliod's Pilgrim (replaces the load-time `data_fixes`); `paths_ok` for the false positives, the
  splits and the 18 shadowed spells; ai/pool.rs ranks a compiled cost static 68/52). Tests: `sim-core/tests/paths_doubles.rs`.
- be11c4f0: paths report (hard 21 -> 0, dsl-covers 28 -> 20).
- d6e8b034, 2afe37db: Treasure / floating-mana memory fix (also on main as cherry-picks; one copy after the merge).
- d9430727: merge of origin/main f6cb3ebc (no conflicts). 371d09db: `audit/phase2/paths-after-doubles.md` regenerated
  after the merge: still 0 hard doubles, dsl-covers 20.
- After the merge: `cardgen export` no change, validate OK; `cargo test --release --no-fail-fast` 3585 passed, 0 failed;
  `cargo build --release --all-targets` 0 warnings.

## Measured (2026-10-29; before = origin/main f6cb3ebc, after = branch head; all rc=0)
- Realism t0-t6, 4000 games, `--jobs 22`: every deck share inside noise; targets pass t0 4->4, t1 5->5, t2 1->2,
  t3 1->2, t4 1->2, t5 1->2, t6 3->3 (all flips are seat-share rows at a band edge).
- JD decks (`--deck K --pool all --games 2000`): cloud 15.5 -> 15.6 (t1-t3), seph 20.1 -> 20.5 (t3, t4),
  yshtola 24.0 -> 24.0 (t5), scion 27.4 -> 27.5 (t6). None plays a rule-changed card (paths_ok-only cards).
- Raw outputs: `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase2-doubles/{before,after}/`.

## Open (for JD)
- New on the branch: after-t4 seed 502910 panics `Option::unwrap()` on None (base t4: no skips). Not traced.
- Overflow panics still hit: t5 12 / 10 skips, yshtola 2 / 5 skips (before / after). Not fixed here.
- Nothing recalibrated (JD recalibrates after Phase 4).
