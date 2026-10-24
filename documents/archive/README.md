# Archived documents

Documents from the Python-to-Rust port and the first builds. They describe how the code was planned and built, not
always how it works now: for that, read the main documents in [`documents/`](..) and the top-level
[`README.md`](../../README.md). Git history has each file's earlier path.

| File | What it was | Written / last changed |
| --- | --- | --- |
| [architecture.md](architecture.md) | The full description of the Rust simulator after the port: engine, stack, combat, both AIs, card model, pools, statistics, seeding, guards, tests and simplifications. Many details still hold; deck lists, tier counts and test counts are from 10 October. | 2026-09-26 / 2026-10-10 |
| [rust-design.md](rust-design.md) | The port's data-model design (milestone M1): ids instead of pointers, arenas, interruptions as values. | 2026-10-10 / 2026-10-10 |
| [rust-rewrite-scope.md](rust-rewrite-scope.md) | The scope and milestones of the rewrite from Python to Rust, with the status of each phase. | 2026-10-10 / 2026-10-18 |
| [rust-practice-design.md](rust-practice-design.md) | Design record of practice mode's Rust port (phases 9-13). | 2026-10-10 / 2026-10-10 |
| [stack.md](stack.md) | Design and build log of the real stack and priority (phases 1-6, built 2026-10-02). | 2026-10-02 / 2026-10-10 |
| [practice-mode-progress.md](practice-mode-progress.md) | Working log of the first (Python) practice-mode build, 29 September to 2 October. | 2026-09-29 / 2026-10-10 |
| [PORT_STATUS.md](PORT_STATUS.md) | Was `rust/PORT_STATUS.md`: what had been ported while Python was the reference, and where each Python test went. | 2026-10-10 / 2026-10-10 |
| [PORTING_CARDS.md](PORTING_CARDS.md) | Was `rust/PORTING_CARDS.md`: how Python card code became Rust hook slots in `sim-core/src/impls/`. The hook-slot pattern still applies. | 2026-10-10 / 2026-10-10 |
