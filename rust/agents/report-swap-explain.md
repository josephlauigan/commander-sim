# swap-explain report

Built: `--swap` now prints (1) a pre-flight model status per swapped-in card (cardgen audit verdict via the cardgen
library; WARNING lines for Partial/Unmodeled) and (2) a "Why" paired breakdown after the delta/VERDICT lines
(`--no-explain` skips both). New file `sim-cli/src/explain.rs`; `run_explained` in poolmode.rs plays the same games as
`run` and also builds the Analysis. `delta` lines unchanged (checked old vs new binary, same seed).

Shared sim-core changes (additive, stats only, no RNG): `state.rs` Player gets `cast_turns`, `cast_logged`;
`engine/turn.rs` records lands/hand at turns 4 and 8, mana at turn 8 too (`lands_T4/T8`, `hand_T4/T8`, `mana_T8`),
and logs first-cast turns at the end of each own turn (`step_end`).

Not done: per-card ability uses / triggers per game (the analyzer tracks no per-card counters; add `stat()` calls in
a card's code and a line in explain.rs if one is needed). "Rocks" is shown as total mana vs lands.
Also: sim-cli now depends on the cardgen crate (for the audit verdict).
