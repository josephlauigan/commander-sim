# Handoff: Phase 4 PR 1, deck plans as data

Brief: `rust/agents/phase4-plans.md`. Design: `audit/phase4/plans-design.md`. Branch `phase4-plans` (on `phase3-exile`).
Scratch (sim outputs): `/tmp/claude-1000/-home-joseph-projects/61d4d38b-9b1e-4bc1-8845-ec4ae3a757af/scratchpad/phase4-plans/`.

## State

- [x] Design, loader (`ai/plan_data.rs`, `build.rs`), plan test (`tests/plans.rs`), A/B switch `AI_PLAN_DATA`.
- [x] Stage A: plan files for the 7 `DeckConfig` decks (knobs only). `--realism --games 300 --jobs 22 --quiet`, base
      binary (ad333eaf) vs branch with `AI_PLAN_DATA=1`: byte-identical on t1, t2, t4, t5 (all rc=0). Tests pass.
- [ ] Stage B, per deck (paired test `--realism --pool <tier> --games 2000 --jobs 22`, `AI_PLAN_DATA=0` vs
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
- [ ] Then the 8 cEDH plans (`impls/t6_plan_*.rs`), one at a time on t6.

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
