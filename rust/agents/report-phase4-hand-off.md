# Report: Phase 4 PR 7, each deck's hand AI behind a switch (generic by default)

Brief: `rust/agents/phase4-hand-off.md`. Design and per-deck survey: `audit/phase4/hand-off-design.md`. Handoff:
`audit/phase4/hand-off-handoff.md`. JD (2026-10-31): switch the hand AI off now, delete it after the recalibration's
on/off test. No sims; nothing deleted.

## What it does

- `AI_HAND`: unset or `0` = the generic policy for every switchable deck (**the new default**), `1` = every deck's
  hand AI as before, a key list = those decks keep theirs. `ai::hand_key` returns "" for a deck whose hand AI is off,
  so every deck-keyed `match` and `is_main` falls through to the generic policy.
- 14 switchable decks: veyran, sauron, yshtola, galadriel, alela, kefka, sheoldred, cloud, karn, thrasios (JD's);
  brago, aurelia, sisay, dack (pool). Others are unchanged.
- With the hand AI off, a JD deck is a generic-policy deck (`values::generic_deck`), so derived values, the learned
  evaluation and ISMCTS apply to it. Tier 6 stays out.
- 13 new plan files (`data/plans/`), written from each hand AI; each header says what the schema can't express.
  Read only while that deck's hand AI is off.

## Known gaps (for the recalibration to check first)

- Plan files can't express board-reading rules: Sauron's Army power, Kefka flipped and its loop-safety guard (the
  Crawler loop that would deck itself), Dack's commander-cost bar and Chalice's net, Alela's priority bonus, equip
  targets. Decks relying on these may play worse with the hand AI off.
- thanos, urdragon and ghave have no hand AI but stay outside `generic_deck` (as before this PR), so they don't get
  derived values, the learned evaluation or ISMCTS. Worth deciding at the recalibration.

## Checks (coordinator, 2026-10-31)

- Read the switch (`ai/mod.rs`) and the `generic_deck` change; read kefka's plan file against its header.
- `cargo build --release --all-targets` 0 warnings; `cargo test --release --no-fail-fast` 3750 passed, 0 failed
  (151 tests of hand AI now force it on; `tests/hand_off.rs` checks each deck on and off). Merges cleanly with #185.

## For the recalibration

Pair `AI_HAND=<key>` against the default, same seeds: veyran and cloud t1-t3; sauron and sheoldred t3-t4; yshtola and
thrasios t5; galadriel t1-t2; alela t1; kefka t3; karn t4-t5; brago t2 and aurelia t3 (half-tier); sisay and dack t6.
Delete a deck's hand AI where off is level or better; otherwise improve its plan file and retest.
