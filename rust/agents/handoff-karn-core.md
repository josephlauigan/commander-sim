# Handoff: karn-core
Done: key wiring (practice, sim-cli, ai, engine key lists) for `karn` (uncommitted until noted below).
Plan: (1) engine mechanism: Floating.art + Source::FloatArt (artifact-only colourless, whole turn; AI) and ManaPool.art (practice);
(2) impls/karn.rs: Karn card (self_pt, upkeep, milestone), Mightstone/Weakstone, Tron lands, Urza's Workshop, factories, karn_prio/special;
(3) printings/karn.txt; (4) practice check; (5) 100-game run, report.
Gotchas: engine floating mana only empties at turn start (turn.rs ~864); human gather_floating moves floating into the pool, which empties each step.
