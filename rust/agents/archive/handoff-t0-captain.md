# Handoff: t0-captain

Done (committed): all 24 cards coded in `impls/t0_captain.rs` (+ verdicts in data/card_overrides.toml, exported), 36 tests in
`tests/cards_t0_captain.rs` pass. Shared-file edits (additive): state.rs (`t0c` field), cardcode.rs (put_plus, evasion_blocked,
delayed_end_step, turn_start), engine/combat.rs (attack_triggers, attacker tap, pair_mark/pair_after, Heroic Sacrifice redirect),
engine/mana.rs + dsl/mod.rs (became_tapped), ai/brain.rs (as_flash in the end-of-turn window).

Next, in order: (1) trace the deck (`commander-sim --deck cloud --pool t0 --any-tier --trace N`) to see it cast Captain America early and
use Quinjet/Fury/etc; (2) full checks from T0-RULES (cargo test, warnings, all_decks adaptive 3 / lookahead 1, smoke run);
(3) report-t0-captain.md and the 15-line report.

Gotchas: the compiled (dsl) "that many"/"for each" effects use X = 3 by default (Ant-Man, Avenge were wrong: both now hand-coded with
`dsl = []`). Never run python (a hook blocks it).
