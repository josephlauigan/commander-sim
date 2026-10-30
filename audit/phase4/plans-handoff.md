# Handoff: Phase 4 PR 1 and PR 2, deck plans as data

## PR 2 (branch phase4-cedh, brief rust/agents/phase4-cedh.md)

Scratch: `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase4-cedh/` (sim
outputs `<name>.txt` + `.rc`; `sim.sh BIN POOL GAMES OUT ENV`, `sim2.sh BIN OUT ENV ARGS...`, `pair.sh`). Schema
extensions: `audit/phase4/plans-design.md`, "PR 2". Each Tier 6 plan file is a translation of its hand plan file,
checked slot by slot on 600 dealt states by `tests/plans.rs::plan_matches_hand` (keep, every hand card's priority,
the wish list a tutor can use, hold-up, every catalogue combo's go-now); a mutation of one score fails it.

Paired tests: 2000 games per arm, same seeds, `AI_PLAN_DATA=0` vs `=<deck>` (t6 decks: one shared hand arm).

| deck | tier | hand share (95% CI) | plan share (95% CI) | plan - hand (95% CI) | verdict | hand code |
|---|---|---|---|---|---|---|
| yuriko-dimir-ninjas | t4 | 18.8% [16.9, 20.7] n=1599 | 18.1% [16.2, 20.0] n=1598 | -0.7 [-3.4, +2.0] | level | t4::yuriko_wish / yuriko_prio deleted, default plan |
| yawgmoth-mono-black-aristocrats | t5 | 17.6% [15.7, 19.5] n=1585 (PR 1 hand arm) | 17.6% [15.7, 19.5] n=1587 | +0.0 [-2.7, +2.7] (PR 1 plan: +1.9 over it) | the -1.9 closes (`counts`, `bf:<set>`) | deleted in PR 1, default plan |
| scion | t6 | pending | | | | |
| kinnan-bonder-mana-combo | t6 | 24.6% [22.3, 26.9] n=1307 | 24.6% [22.3, 26.9] n=1306 | +0.0 [-3.3, +3.3] | level: byte-identical output | t6_plan_kinnan.rs deleted, default plan |
| sisay-legends-toolbox | t6 | 31.1% [28.6, 33.6] n=1335 | 31.1% [28.6, 33.6] n=1335 | +0.0 [-3.5, +3.5] | level: byte-identical output | five slots deleted, default plan; `fetch` stays code (legends::sisay_pick) |
| rograkh-silas-turbo-breach | t6 | 18.6% [16.5, 20.7] n=1320 | 18.3% [16.2, 20.4] n=1319 | -0.3 [-3.3, +2.7] | level | t6_plan_rograkh_silas.rs deleted, default plan |
| kraum-tymna-blue-breach-midrange | t6 | 29.8% [27.3, 32.3] n=1292 | 29.8% [27.3, 32.3] n=1292 | +0.0 | level: byte-identical output | t6_plan_kraum.rs deleted, default plan |
| rograkh-thrasios-creature-combo | t6 | 21.1% [18.9, 23.3] n=1325 | 20.8% [18.6, 23.0] n=1326 | -0.3 [-3.4, +2.8] | level | t6_plan_rograkh_thrasios.rs deleted, default plan |
| dack-fayden-white-stax | t6 | - | - | - | no plan file | kept: see below |
| godo | - | - | - | - | skipped (not seated) | kept |

Dack is not expressible as plan data: its slots read its own commander's cost and tax against total mana
(`pre_cast`), a per-round risk bar over the table's stop chances (`dack_bar`, `table_risk`), Chalice's net over the
opponents' hands and libraries (`chalice_net`), the opponents' deck creature share (`creature_table`), spell locks
(`protected`) and counter likelihood products. Moving those would be code under quantity names.

## PR 1

Brief: `rust/agents/phase4-plans.md`. Design: `audit/phase4/plans-design.md`. Branch `phase4-plans` (on `phase3-exile`).
Scratch (sim outputs): `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase4-plans/`.

## State

- [x] Design, loader (`ai/plan_data.rs`, `build.rs`), plan test (`tests/plans.rs`), A/B switch `AI_PLAN_DATA`.
- [x] Stage A: plan files for the 7 `DeckConfig` decks (knobs only). `--realism --games 300 --jobs 22 --quiet`, base
      binary (ad333eaf) vs branch with `AI_PLAN_DATA=1`: byte-identical on t1, t2, t4, t5 (all rc=0). Tests pass.
- [x] Stage B, per deck (paired test `--realism --pool <tier> --games 2000 --jobs 22`, `AI_PLAN_DATA=0` vs
      `AI_PLAN_DATA=<deck>`):

| deck | tier | hand share (95% CI) | plan share (95% CI) | verdict | hand code |
|---|---|---|---|---|---|
| light-paws-aura-voltron | t1 | - | - | knobs only, identical (Stage A) | CONFIG entry deleted, default plan |
| chulane-bant-value-combo | t4 | 13.2% [11.6, 14.8] n=1622 | 15.7% [13.9, 17.5] n=1622 | plan better, +2.5 [+0.1, +4.9] | chulane_prio / chulane_wish deleted, default plan |
| yuriko-dimir-ninjas | t4 | 20.9% [18.9, 22.9] n=1599 | 18.1% [16.2, 20.0] n=1599 | plan worse, -2.8 [-5.5, -0.1] | kept (t4::yuriko_wish / yuriko_prio), default hand |
| yawgmoth-mono-black-aristocrats | t5 | 17.6% [15.7, 19.5] n=1585 | 15.7% [13.9, 17.5] n=1584 | level, -1.9 [-4.5, +0.7] (point estimate lower) | yawg_prio / yawg_wish deleted, default plan |
| kaalia-mardu-creature-cheat | t2 | 26.8% [24.6, 29.0] n=1556 | 26.8% [24.6, 29.0] n=1556 | level: byte-identical output | t2::kaalia_prio deleted, default plan |

- gaa-azorius-stax-control and heliod-mono-white-stax are retired from the pool (test seats only): no paired test is
  possible; their plan files carry the knobs, their hand code stays.
- [ ] Not started: the 8 cEDH plans (`impls/t6_plan_*.rs`), one at a time on t6. Their slots (`keep`, `hold_up`,
      `go_now`, f64 `prio`) need schema the files lack: a mana-aware keep rule (`t6_keep::mana`), hold-up utility,
      per-combo go-now gates. Start with scion (only `keep`) once `[mulligan]` can count fast mana and rituals.

## How to run a paired test

```
cd <worktree>; S=<scratch>
AI_PLAN_DATA=0 systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 \
  rust/target/release/commander-sim --realism --pool t4 --games 2000 --jobs 22 --quiet > $S/B-<deck>-hand.txt
AI_PLAN_DATA=<deck> ... > $S/B-<deck>-plan.txt
grep "Each deck's share" $S/B-<deck>-*.txt
```

`systemd-run --scope` passes the caller's environment through, so the env var reaches the binary.

## Gotchas

- Hand wish lists named cards no longer in the lists (Draco, Enter the Infinite, Avenger of Zendikar, Consecrated
  Sphinx, Kor Skyfisher, Man-o'-War ...): the plan test refuses those in plan files.
- Yuriko diagnostic (derived wish, hand `yuriko_prio`): 17.5% [15.6, 19.4], -3.4 [-6.1, -0.7] against hand: the
  wish list is the loss. Next step for yuriko: schema for "held only on the battlefield" groups and engines ranked
  ahead of a line, then re-run its pair.
- Paired CIs are Wald intervals on each share and on the difference (unpaired variance, conservative for same
  seeds): `audit/phase4/pair.sh HAND.txt PLAN.txt` (awk, no Python).
- Hand code left in `ai/plans.rs` CONFIG: yuriko (worse), gaa and heliod (retired, no paired test possible).

## Final checks (head of the branch)

- `cargo build --release --all-targets -j 22`: 0 warnings. `cargo test --release --no-fail-fast -j 22`: 3671 passed,
  0 failed (practice included); `cardgen validate`: all decks passed; `all_decks adaptive 3` and `lookahead 1`: rc=0.
- Defaults (no env) after the deletions reproduce the plan arms byte for byte: t4 2000 = the chulane arm, t5 2000 =
  the yawgmoth arm, t1 300 = the Stage A base (light-paws). All rc=0.
